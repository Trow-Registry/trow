use std::sync::Arc;

use anyhow::Result;
use axum::extract::{Path, State};
use digest::Digest;
use tracing::{event, Level};

use super::macros::endpoint_fn_7_levels;
use crate::registry::{digest, BlobReader};
use crate::response::errors::Error;
use crate::response::trow_token::TrowToken;
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
pub async fn get_blob(
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
pub async fn delete_blob(
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
