use std::sync::Arc;

use anyhow::Result;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::Router;

use super::macros::endpoint_fn_7_levels;
use crate::registry::{BlobReader, Digest};
use crate::routes::macros::route_7_levels;
use crate::routes::response::errors::Error;
use crate::routes::response::trow_token::TrowToken;
use crate::types::BlobDeleted;
use crate::TrowServerState;

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
    Path((repo, digest)): Path<(String, Digest)>,
) -> Result<BlobReader<impl futures::AsyncRead>, Error> {
    let mut conn = state.db.acquire().await?;
    let digest_str = digest.as_str();
    sqlx::query!(
        r#"
        SELECT * FROM blob
        WHERE digest = $1
        "#,
        digest_str
    )
    .fetch_one(&mut *conn)
    .await?;

    let stream = match state.registry.storage.get_blob_stream(&repo, &digest).await {
        Ok(stream) => stream,
        Err(_) => return Err(Error::InternalError),
    };
    Ok(BlobReader::new(digest, stream).await)
}

endpoint_fn_7_levels!(
    get_blob(
        auth_user: TrowToken,
        state: State<Arc<TrowServerState>>;
        path: [image_name, digest: Digest]
    ) -> Result<BlobReader<impl futures::AsyncRead>, Error>
);

/**
 * Deletes the given blob.
 *
 * Really unsure about this method - why should the user delete a blob?
 * TODO: This should probably be denied if the blob is referenced by any manifests
 * (manifest should be deleted first)
 */
async fn delete_blob(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((repo, digest)): Path<(String, Digest)>,
) -> Result<BlobDeleted, Error> {
    let mut conn = state.db.acquire().await?;
    let digest_str = digest.as_str();

    sqlx::query!(
        r#"
            DELETE FROM repo_blob_association
            WHERE repo_name = $1
                AND blob_digest = $2
            "#,
        repo,
        digest_str
    )
    .execute(&mut *conn)
    .await?;
    state.registry.storage.delete_blob(&repo, &digest).await?;

    Ok(BlobDeleted {})
}

endpoint_fn_7_levels!(
    delete_blob(
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>;
    path: [image_name, digest: Digest]
    ) -> Result<BlobDeleted, Error>
);

pub fn route(mut app: Router<Arc<TrowServerState>>) -> Router<Arc<TrowServerState>> {
    #[rustfmt::skip]
    route_7_levels!(
        app,
        "/v2" "/blobs/:digest",
        get(get_blob, get_blob_2level, get_blob_3level, get_blob_4level, get_blob_5level, get_blob_6level, get_blob_7level),
        delete(delete_blob, delete_blob_2level, delete_blob_3level, delete_blob_4level, delete_blob_5level, delete_blob_6level, delete_blob_7level)
    );
    app
}
