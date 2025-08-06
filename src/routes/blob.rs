use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::get;

use super::macros::endpoint_fn_7_levels;
use crate::TrowServerState;
use crate::registry::manifest::ManifestReference;
use crate::registry::{BlobReader, Digest};
use crate::routes::extracts::ImageNamespace;
use crate::routes::macros::route_7_levels;
use crate::routes::response::errors::Error;
use crate::routes::response::trow_token::TrowToken;

/*
---
Pulling a Layer
GET /v2/<name>/blobs/<digest>
name - name of the repository
digest - unique identifier for the blob to be downloaded

# Responses
200 - blob is downloaded
307 - redirect to another service for downloading (docker API, not OCI)
 */
async fn get_blob(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((mut repo, digest)): Path<(String, Digest)>,
    Query(query): Query<ImageNamespace>,
) -> Result<BlobReader<impl tokio::io::AsyncRead>, Error> {
    let digest_str = digest.as_str();
    if let Some(image) = state
        .registry
        .config
        .registry_proxies
        .get_proxied_image(&repo, &ManifestReference::Digest(digest.clone()), query.ns)
        .await
    {
        repo = format!("f/{}/{}", image.get_host(), image.get_repo())
    }

    let rowid = sqlx::query_scalar!(
        r#"
        SELECT b.rowid as "rowid!" FROM blob b
        JOIN repo_blob_assoc rba ON b.digest = rba.blob_digest
        WHERE b.digest = $1 AND rba.repo_name = $2
        "#,
        digest_str,
        repo
    )
    .fetch_one(&state.db_ro)
    .await?;
    sqlx::query!(
        "UPDATE blob SET last_accessed=unixepoch() WHERE rowid=$1",
        rowid
    )
    .execute(&state.db_rw)
    .await?;

    let stream = match state
        .registry
        .storage
        .get_blob_stream(&repo, digest.as_str())
        .await
    {
        Ok(stream) => stream,
        Err(_) => return Err(Error::Internal),
    };
    Ok(BlobReader::new(digest, stream).await)
}

endpoint_fn_7_levels!(
    get_blob(
        auth_user: TrowToken,
        state: State<Arc<TrowServerState>>;
        path: [image_name, digest: Digest],
        query: Query<ImageNamespace>
    ) -> Result<BlobReader<impl tokio::io::AsyncRead>, Error>
);

pub fn route(mut app: Router<Arc<TrowServerState>>) -> Router<Arc<TrowServerState>> {
    #[rustfmt::skip]
    route_7_levels!(
        app,
        "/v2" "/blobs/{digest}",
        get(get_blob, get_blob_2level, get_blob_3level, get_blob_4level, get_blob_5level, get_blob_6level, get_blob_7level)
    );
    app
}
