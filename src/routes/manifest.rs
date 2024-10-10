use std::collections::HashSet;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::Router;
use bytes::Buf;
use digest::Digest;
use sea_orm::sea_query::{Expr, OnConflict};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QuerySelect, Set,
    TransactionTrait,
};

use super::extracts::AlwaysHost;
use super::macros::endpoint_fn_7_levels;
use super::response::OciJson;
use crate::registry::digest;
use crate::registry::manifest::{OCIManifest, REGEX_TAG};
use crate::registry::server::PROXY_DIR;
use crate::routes::macros::route_7_levels;
use crate::routes::response::errors::Error;
use crate::routes::response::trow_token::TrowToken;
use crate::types::{ManifestDeleted, VerifiedManifest};
use crate::{entity, TrowServerState};

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
    Path((repo, reference)): Path<(String, String)>,
) -> Result<OciJson<OCIManifest>, Error> {
    println!("-- HELLO --");
    let digest = if REGEX_TAG.is_match(&reference) {
        let tag = entity::tag::Entity::find()
            .filter(entity::tag::Column::Repo.eq(&repo))
            .filter(entity::tag::Column::Tag.eq(&reference))
            .one(&state.db)
            .await?
            .ok_or(Error::NotFound)?;
        tag.manifest_digest.clone()
    } else {
        Digest::try_from_raw(&reference)?
    };
    // let txn = state
    //     .db
    //     .begin_with_config(Some(IsolationLevel::RepeatableRead), None)
    //     .await?;
    let manifest = entity::blob::Entity::find_by_id(digest.clone())
        .filter(entity::blob::Column::IsManifest.eq(true))
        .inner_join(entity::repo_blob_association::Entity)
        .filter(entity::repo_blob_association::Column::RepoName.eq(repo.clone()))
        .one(&state.db)
        .await?
        .ok_or(Error::NotFound)?;

    let manifest_raw = state
        .registry
        .storage
        .get_manifest(&repo, &manifest.digest)
        .await?;
    // txn.commit().await?;
    let manifest_parsed: OCIManifest = serde_json::from_slice(&manifest_raw)
        .map_err(|e| Error::ManifestInvalid(format!("serialization error: {e}")))?;
    let content_type = manifest_parsed
        .media_type()
        .as_ref()
        .map(|mt| mt.to_string())
        .unwrap_or("application/json".to_string());
    Ok(OciJson::new_raw(manifest_raw)
        .set_digest(&manifest.digest)
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

    let existing_assets = entity::repo_blob_association::Entity::find()
        .select_only()
        .column(entity::repo_blob_association::Column::BlobDigest)
        .filter(Expr::col(entity::repo_blob_association::Column::BlobDigest).is_in(&assets))
        .filter(entity::repo_blob_association::Column::RepoName.eq(repo_name.clone()))
        .into_tuple()
        .all(&state.db)
        .await?;

    let assets_set = HashSet::<String>::from_iter(assets.into_iter());
    let existing_assets_set = HashSet::from_iter(existing_assets.into_iter());
    let assets_diff: Vec<&String> = assets_set.difference(&existing_assets_set).collect();
    if !assets_diff.is_empty() {
        return Err(Error::ManifestInvalid(format!(
            "Missing manifest assets: {:?}",
            assets_diff
        )));
    }

    let size = manifest_bytes.len();
    let computed_digest = Digest::digest_sha256(manifest_bytes.clone().reader()).unwrap();
    if !is_tag && computed_digest.as_str() != &reference {
        return Err(Error::ManifestInvalid("Digest does not match".to_string()));
    }
    let txn = state.db.begin().await?;
    entity::repo::insert_if_not_exists(&txn, repo_name.clone()).await?;
    entity::blob::Entity::insert(entity::blob::ActiveModel {
        digest: Set(computed_digest.clone()),
        is_manifest: Set(true),
        size: Set(size as i32),
        ..Default::default()
    })
    .on_conflict(
        OnConflict::column(entity::blob::Column::Digest)
            .update_column(entity::blob::Column::LastAccessed)
            .to_owned(),
    )
    .exec(&txn)
    .await?;
    entity::repo_blob_association::Entity::insert(entity::repo_blob_association::ActiveModel {
        repo_name: Set(repo_name.clone()),
        blob_digest: Set(computed_digest.clone()),
    })
    .on_conflict(
        OnConflict::columns([
            entity::repo_blob_association::Column::RepoName,
            entity::repo_blob_association::Column::BlobDigest,
        ])
        .do_nothing()
        .to_owned(),
    )
    .exec(&txn)
    .await?;
    if is_tag {
        entity::tag::insert_or_update(
            reference.clone(),
            repo_name.clone(),
            computed_digest.clone(),
            &txn,
        )
        .await?;
    }

    state
        .registry
        .storage
        .write_image_manifest(manifest_bytes, &repo_name, &computed_digest)
        .await?;
    txn.commit().await?;

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
    Path((repo, digest)): Path<(String, Digest)>,
) -> Result<ManifestDeleted, Error> {
    if repo.starts_with(PROXY_DIR) {
        return Err(Error::UnsupportedForProxiedRepo);
    }
    // irh, Digest is not doing validation it seems ?
    if REGEX_TAG.is_match(digest.as_str()) {
        return Err(Error::Unsupported);
    }
    entity::repo_blob_association::Entity::delete_by_id((repo.clone(), digest.clone()))
        .exec(&state.db)
        .await?;

    Ok(ManifestDeleted {})
}
endpoint_fn_7_levels!(
    delete_image_manifest(
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>;
    path: [image_name, digest: Digest]
    ) -> Result<ManifestDeleted, Error>
);

pub fn route(mut app: Router<Arc<TrowServerState>>) -> Router<Arc<TrowServerState>> {
    #[rustfmt::skip]
    route_7_levels!(
        app,
        "/v2" "/manifests/:reference",
        get(get_manifest, get_manifest_2level, get_manifest_3level, get_manifest_4level, get_manifest_5level, get_manifest_6level, get_manifest_7level),
        put(put_image_manifest, put_image_manifest_2level, put_image_manifest_3level, put_image_manifest_4level, put_image_manifest_5level, put_image_manifest_6level, put_image_manifest_7level),
        delete(delete_image_manifest, delete_image_manifest_2level, delete_image_manifest_3level, delete_image_manifest_4level, delete_image_manifest_5level, delete_image_manifest_6level, delete_image_manifest_7level)
    );
    app
}
