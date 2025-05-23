use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::routing::get;
use digest::Digest;

use super::extracts::AlwaysHost;
use super::macros::endpoint_fn_7_levels;
use super::response::OciJson;
use crate::TrowServerState;
use crate::registry::digest;
use crate::registry::manifest::{
    ManifestReference, OCIManifest, REGEX_TAG, layer_is_distributable,
};
use crate::registry::server::PROXY_DIR;
use crate::routes::macros::route_7_levels;
use crate::routes::response::errors::Error;
use crate::routes::response::trow_token::TrowToken;
use crate::types::{ManifestDeleted, VerifiedManifest};

/*
---
Pulling an image
GET /v2/<name>/manifests/<reference>

# Parameters
name - The namespace of the repository
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
            .config
            .registry_proxies
            .get_proxy_config(&repo, &reference)
            .await
        {
            Some(cfg) => cfg,
            None => {
                return Err(Error::NameInvalid(format!(
                    "No registered proxy matches {repo}"
                )));
            }
        };

        proxy_cfg
            .download_remote_image(&image, &state)
            .await
            .map_err(|e| {
                tracing::error!("Error downloading image: {e}");
                Error::Internal
            })?
    } else {
        let digest = match &reference {
            ManifestReference::Tag(_) => {
                let tdigest = sqlx::query_scalar!(
                    "SELECT t.manifest_digest FROM tag t WHERE t.repo = $1 AND t.tag = $2",
                    repo,
                    raw_reference
                )
                .fetch_optional(&state.db_ro)
                .await?;
                match tdigest {
                    Some(d) => d,
                    None => {
                        return Err(Error::ManifestUnknown(format!(
                            "Unknown tag: {raw_reference}"
                        )));
                    }
                }
            }
            ManifestReference::Digest(_) => raw_reference,
        };
        let maybe_digest = sqlx::query_scalar!(
            r#"
            SELECT rba.manifest_digest
            FROM repo_blob_assoc rba
            WHERE rba.manifest_digest = $2 AND rba.repo_name = $1
            "#,
            repo,
            digest
        )
        .fetch_optional(&state.db_ro)
        .await?;

        match maybe_digest {
            Some(Some(d)) => d,
            _ => {
                return Err(Error::ManifestUnknown(format!("Unknown digest {digest}")));
            }
        }
    };

    let res = sqlx::query!(
        r#"
        SELECT m.json ->> 'mediaType' as "media_type: String", m.blob
        FROM manifest m
        WHERE m.digest = $1
        "#,
        digest
    )
    .fetch_one(&state.db_ro)
    .await?;

    Ok(OciJson::new_raw(res.blob.into())
        .set_digest(digest)
        .set_content_type(&res.media_type.unwrap_or("application/json".to_owned())))
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
        })?
        .to_vec();
    let manifest_parsed = serde_json::from_slice::<'_, OCIManifest>(&manifest_bytes)
        .map_err(|e| Error::ManifestInvalid(format!("{e}")))?;

    match &manifest_parsed {
        OCIManifest::List(m) => {
            let assets = m
                .manifests()
                .iter()
                .filter(|l| layer_is_distributable(l.media_type()))
                .map(|m| m.digest().as_ref());
            for digest in assets {
                let res = sqlx::query!(
                    r"SELECT rba.manifest_digest FROM repo_blob_assoc rba
                    WHERE rba.manifest_digest = $1 AND rba.repo_name = $2",
                    digest,
                    repo_name
                )
                .fetch_optional(&state.db_ro)
                .await?;
                if res.is_none() {
                    return Err(Error::ManifestInvalid(format!(
                        "Manifest asset not found: {digest}"
                    )));
                }
            }
        }
        OCIManifest::V2(m) => {
            let assets = m
                .layers()
                .iter()
                .filter(|l| layer_is_distributable(l.media_type()))
                .map(|l| l.digest().as_ref());
            for digest in assets {
                let res = sqlx::query!(
                    r"SELECT rba.blob_digest FROM repo_blob_assoc rba
                    WHERE rba.blob_digest = $1 AND rba.repo_name = $2",
                    digest,
                    repo_name
                )
                .fetch_optional(&state.db_ro)
                .await?;
                if res.is_none() {
                    return Err(Error::ManifestInvalid(format!(
                        "Blob asset not found: {digest}"
                    )));
                }
            }
        }
    }
    let computed_digest = Digest::digest_sha256_slice(&manifest_bytes);
    let computed_digest_str = computed_digest.as_str();
    if !is_tag && computed_digest_str != reference {
        return Err(Error::ManifestInvalid(
            "Given digest does not match".to_string(),
        ));
    }

    sqlx::query!(
        r#"
        INSERT INTO manifest (digest, json, blob)
        VALUES ($1, jsonb($2), $2)
        ON CONFLICT (digest) DO NOTHING
        "#,
        computed_digest_str,
        manifest_bytes
    )
    .execute(&state.db_rw)
    .await?;
    sqlx::query!(
        r#"
        INSERT INTO repo_blob_assoc
        VALUES ($1, NULL, $2)
        ON CONFLICT (repo_name, blob_digest, manifest_digest) DO NOTHING
        "#,
        repo_name,
        computed_digest_str
    )
    .execute(&state.db_rw)
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
        .execute(&state.db_rw)
        .await?;
    }

    // check if the manifest has a Subject field, if so return the header OCI-Subject
    let subject = manifest_parsed.subject().map(|s| s.digest().to_string());

    Ok(VerifiedManifest::new(
        Some(host),
        repo_name,
        computed_digest,
        reference,
        subject,
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
        .execute(&state.db_rw)
        .await?;
    } else {
        let digest = Digest::try_from_raw(&reference)?;
        let digest_str = digest.as_str();
        let res = sqlx::query!(
            "DELETE FROM repo_blob_assoc WHERE repo_name = $1 AND blob_digest = $2",
            repo,
            digest_str
        )
        .execute(&state.db_rw)
        .await?;

        if res.rows_affected() > 0 {
            let remaining_assoc = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM repo_blob_assoc WHERE manifest_digest = $1",
                digest_str
            )
            .fetch_one(&state.db_ro)
            .await?;

            if remaining_assoc == 0 {
                sqlx::query!("DELETE FROM manifest where digest = $1", digest_str)
                    .execute(&state.db_rw)
                    .await?;
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
