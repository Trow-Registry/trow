use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::Router;
use bytes::Buf;
use digest::Digest;

use super::extracts::AlwaysHost;
use super::macros::endpoint_fn_7_levels;
use super::response::OciJson;
use crate::registry::digest;
use crate::registry::manifest::{ManifestReference, OCIManifest, REGEX_TAG};
use crate::registry::server::PROXY_DIR;
use crate::routes::macros::route_7_levels;
use crate::routes::response::errors::Error;
use crate::routes::response::trow_token::TrowToken;
use crate::types::{ManifestDeleted, VerifiedManifest};
use crate::TrowServerState;

/*
---
Pulling an image
GET /v2/<name>/manifests/<reference>

# Parameters
name - The name of the image
reference - either a tag or a digest

# Client Headers
Accept: manifest-version

# Headers
Accept: manifest-version
?Docker-Content-Digest: digest of manifest file

# Returns
200 - return the manifest
404 - manifest not known to the registry
 */
async fn get_manifest(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((repo, raw_reference)): Path<(String, String)>,
) -> Result<OciJson<OCIManifest>, Error> {
    let reference = ManifestReference::try_from_str(&raw_reference).map_err(|e| {
        // Error::ManifestInvalid(format!("Invalid reference: {raw_reference} ({e:?})"))
        Error::ManifestUnknown(format!("Invalid reference: {raw_reference} ({e:?})"))
    })?;

    let digest = if repo.starts_with(PROXY_DIR) {
        let (proxy_cfg, image) = match state
            .registry
            .proxy_registry_config
            .get_proxy_config(&repo, &reference)
            .await
        {
            Some(cfg) => cfg,
            None => {
                return Err(Error::NameInvalid(format!(
                    "No registered proxy matches {repo}"
                )))
            }
        };

        let new_digest = proxy_cfg
            .download_remote_image(&image, &state.registry, &state.db)
            .await
            .map_err(|e| {
                tracing::error!("Error downloading image: {e}");
                Error::InternalError
            })?;
        new_digest.to_string()
    } else {
        let digest = match &reference {
            ManifestReference::Tag(_) => {
                let tdigest = sqlx::query_scalar!(
                    "SELECT t.manifest_digest FROM tag t WHERE t.repo = $1 AND t.tag = $2",
                    repo,
                    raw_reference
                )
                .fetch_optional(&mut *state.db.acquire().await?)
                .await?;
                match tdigest {
                    Some(d) => d,
                    None => {
                        return Err(Error::ManifestUnknown(format!(
                            "Unknown tag: {raw_reference}"
                        )))
                    }
                }
            }
            ManifestReference::Digest(_) => raw_reference,
        };
        let maybe_manifest = sqlx::query!(
            r#"
            SELECT * FROM blob b
            INNER JOIN repo_blob_association rba ON rba.blob_digest = b.digest
            WHERE b.digest = $2
                AND b.is_manifest is true
                AND rba.repo_name = $1
            "#,
            repo,
            digest
        )
        .fetch_optional(&mut *state.db.acquire().await?)
        .await?;

        if maybe_manifest.is_none() {
            return Err(Error::ManifestUnknown(format!("Unknown digest {digest}")));
        }
        digest
    };

    let digest_parsed = Digest::try_from_raw(&digest).unwrap();
    let manifest_raw = state
        .registry
        .storage
        .get_manifest(&repo, &digest_parsed)
        .await?;

    let manifest_parsed: OCIManifest = serde_json::from_slice(&manifest_raw)
        .map_err(|e| Error::ManifestInvalid(format!("serialization error: {e}")))?;
    let content_type = manifest_parsed
        .media_type()
        .as_ref()
        .map(|mt| mt.to_string())
        .unwrap_or("application/json".to_string());
    Ok(OciJson::new_raw(manifest_raw)
        .set_digest(&digest_parsed)
        .set_content_type(&content_type))
}

endpoint_fn_7_levels!(
    get_manifest(
        auth_user: TrowToken,
        state: State<Arc<TrowServerState>>;
        path: [image_name, reference: String]
    ) -> Result<OciJson<OCIManifest>, Error>
);

/*

---
Pushing an image manifest
PUT /v2/<name>/manifests/<reference>
Content-Type: <manifest media type>

TODO: return 413 payload too large
 */
async fn put_image_manifest(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    AlwaysHost(host): AlwaysHost,
    Path((repo_name, reference)): Path<(String, String)>,
    body: Body,
) -> Result<VerifiedManifest, Error> {
    if repo_name.starts_with(PROXY_DIR) {
        return Err(Error::UnsupportedForProxiedRepo);
    }
    let is_tag = REGEX_TAG.is_match(&reference);
    const MANIFEST_BODY_SIZE_LIMIT_MB: usize = 4;
    let manifest_bytes = axum::body::to_bytes(body, MANIFEST_BODY_SIZE_LIMIT_MB * 1024 * 1024)
        .await
        .map_err(|_| {
            Error::ManifestInvalid(format!(
                "Manifest is bigger than limit of {MANIFEST_BODY_SIZE_LIMIT_MB}MiB"
            ))
        })?;
    let manifest_parsed = serde_json::from_slice::<'_, OCIManifest>(&manifest_bytes)
        .map_err(|e| Error::ManifestInvalid(format!("{e}")))?;
    let assets = manifest_parsed.get_local_asset_digests();

    for digest in assets {
        tracing::debug!("Checking asset: {repo_name} {digest}");
        let res = sqlx::query!(
            r#"
            SELECT b.digest FROM blob b
            INNER JOIN repo_blob_association rba ON rba.blob_digest = b.digest
            WHERE b.digest = $1 AND rba.repo_name = $2"#,
            digest,
            repo_name
        )
        .fetch_optional(&mut *state.db.acquire().await?)
        .await?;
        if res.is_none() {
            return Err(Error::ManifestInvalid(format!("Asset not found: {digest}")));
        }
    }
    let size = manifest_bytes.len() as i64;
    let computed_digest = Digest::digest_sha256(manifest_bytes.clone().reader()).unwrap();
    let computed_digest_str = computed_digest.as_str();
    if !is_tag && computed_digest_str != reference {
        return Err(Error::ManifestInvalid("Digest does not match".to_string()));
    }

    state
        .registry
        .storage
        .write_image_manifest(manifest_bytes, &repo_name, &computed_digest)
        .await?;

    sqlx::query!(
        r#"
        INSERT INTO blob (digest, size, is_manifest)
        VALUES ($1, $2, true)
        ON CONFLICT (digest) DO NOTHING
        "#,
        computed_digest_str,
        size
    )
    .execute(&mut *state.db.acquire().await?)
    .await?;
    sqlx::query!(
        r#"
        INSERT INTO repo_blob_association
        VALUES ($1, $2)
        ON CONFLICT (repo_name, blob_digest) DO NOTHING
        "#,
        repo_name,
        computed_digest_str
    )
    .execute(&mut *state.db.acquire().await?)
    .await?;

    if is_tag {
        sqlx::query!(
            r#"
            INSERT INTO tag
            VALUES ($1, $2, $3)
            ON CONFLICT (repo, tag) DO UPDATE
                SET manifest_digest = EXCLUDED.manifest_digest
            "#,
            reference,
            repo_name,
            computed_digest_str,
        )
        .execute(&mut *state.db.acquire().await?)
        .await?;
    }

    Ok(VerifiedManifest::new(
        Some(host),
        repo_name,
        computed_digest,
        reference,
    ))
}
endpoint_fn_7_levels!(
    put_image_manifest(
        auth_user: TrowToken,
        state: State<Arc<TrowServerState>>,
        host: AlwaysHost;
        path: [image_name, reference: String],
        chunk: Body
    ) -> Result<VerifiedManifest, Error>
);

/*
---
Deleting an Image
DELETE /v2/<name>/manifests/<reference>
*/
async fn delete_image_manifest(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((repo, reference)): Path<(String, String)>,
) -> Result<ManifestDeleted, Error> {
    if repo.starts_with(PROXY_DIR) {
        return Err(Error::UnsupportedForProxiedRepo);
    }
    // irh, Digest is not doing validation it seems ?
    if REGEX_TAG.is_match(&reference) {
        sqlx::query!(
            r#"DELETE FROM tag WHERE repo = $1 AND tag = $2"#,
            repo,
            reference
        )
        .execute(&mut *state.db.acquire().await?)
        .await?;
    } else {
        let digest = Digest::try_from_raw(&reference)?;
        let res = sqlx::query!(
            "DELETE FROM repo_blob_association WHERE repo_name = $1 AND blob_digest = $2",
            repo,
            reference
        )
        .execute(&mut *state.db.acquire().await?)
        .await?;

        if res.rows_affected() > 0 {
            let remaining_assoc = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM repo_blob_association WHERE blob_digest = $1",
                reference
            )
            .fetch_one(&mut *state.db.acquire().await?)
            .await?;

            if remaining_assoc == 0 {
                state.registry.storage.delete_blob(&repo, &digest).await?;
            }
        }
    }

    Ok(ManifestDeleted {})
}
endpoint_fn_7_levels!(
    delete_image_manifest(
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>;
    path: [image_name, digest: String]
    ) -> Result<ManifestDeleted, Error>
);

pub fn route(mut app: Router<Arc<TrowServerState>>) -> Router<Arc<TrowServerState>> {
    #[rustfmt::skip]
    route_7_levels!(
        app,
        "/v2" "/manifests/{reference}",
        get(get_manifest, get_manifest_2level, get_manifest_3level, get_manifest_4level, get_manifest_5level, get_manifest_6level, get_manifest_7level),
        put(put_image_manifest, put_image_manifest_2level, put_image_manifest_3level, put_image_manifest_4level, put_image_manifest_5level, put_image_manifest_6level, put_image_manifest_7level),
        delete(delete_image_manifest, delete_image_manifest_2level, delete_image_manifest_3level, delete_image_manifest_4level, delete_image_manifest_5level, delete_image_manifest_6level, delete_image_manifest_7level)
    );
    app
}
