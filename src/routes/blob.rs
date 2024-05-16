use std::sync::Arc;

use anyhow::Result;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::Router;
use digest::Digest;
use tracing::{event, Level};

use super::macros::endpoint_fn_7_levels;
use crate::registry::{digest, BlobReader};
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
307 - redirect to another service for downloading[1]
 */
async fn get_blob(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((one, digest)): Path<(String, String)>,
) -> Result<BlobReader<impl futures::AsyncRead>, Error> {
    let digest = match Digest::try_from_raw(&digest) {
        Ok(d) => d,
        Err(e) => {
            event!(Level::ERROR, "Error parsing digest: {}", e);
            return Err(Error::DigestInvalid);
        }
    };

    match state.registry.get_blob(&one, &digest).await {
        Ok(r) => Ok(r),
        Err(e) => {
            event!(Level::ERROR, "Error getting blob: {}", e);
            Err(Error::NotFound)
        }
    }
}

endpoint_fn_7_levels!(
    get_blob(
        auth_user: TrowToken,
        state: State<Arc<TrowServerState>>;
        path: [image_name, digest]
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
    Path((one, digest)): Path<(String, String)>,
) -> Result<BlobDeleted, Error> {
    let digest = Digest::try_from_raw(&digest).map_err(|_| Error::DigestInvalid)?;
    state
        .registry
        .delete_blob(&one, &digest)
        .await
        .map_err(|_| Error::NotFound)?;
    Ok(BlobDeleted {})
}

endpoint_fn_7_levels!(
    delete_blob(
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>;
    path: [image_name, digest]
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
