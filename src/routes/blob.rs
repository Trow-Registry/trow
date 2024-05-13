use std::sync::Arc;

use anyhow::Result;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use digest::Digest;
use tracing::{event, Level};

use super::extracts::AlwaysHost;
use super::macros::endpoint_fn_7_levels;
use crate::registry::{digest, BlobReader, ContentInfo, StorageDriverError};
use crate::response::errors::Error;
use crate::response::trow_token::TrowToken;
use crate::response::upload_info::UploadInfo;
use crate::registry::storage::StorageBackendError;
use crate::types::{AcceptedUpload, BlobDeleted, DigestQuery, Upload, Uuid};
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

    match state.client.get_blob(&one, &digest).await {
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

/*
---
Monolithic Upload
PUT /v2/<name>/blobs/uploads/<uuid>?digest=<digest>
Content-Length: <size of layer>
Content-Type: application/octet-stream

<Layer Binary Data>
---
Completes the upload.
*/
pub async fn put_blob(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((repo, uuid)): Path<(String, String)>,
    AlwaysHost(host): AlwaysHost,
    Query(digest): Query<DigestQuery>,
    chunk: Body,
) -> Result<AcceptedUpload, Error> {
    let digest = match digest.digest {
        Some(d) => d,
        None => return Err(Error::DigestInvalid),
    };

    let size = match state
        .client
        .store_blob_chunk(&repo, &uuid, None, chunk)
        .await
    {
        Ok(stored) => stored.total_stored,
        Err(StorageDriverError::InvalidName(name)) => return Err(Error::NameInvalid(name)),
        Err(StorageDriverError::InvalidContentRange) => {
            return Err(Error::BlobUploadInvalid(
                "Invalid Content Range".to_string(),
            ))
        }
        Err(e) => {
            event!(Level::ERROR, "Error storing blob chunk: {}", e);
            return Err(Error::InternalError);
        }
    };

    let digest_obj = Digest::try_from_raw(&digest).map_err(|_| Error::DigestInvalid)?;
    state
        .client
        .complete_and_verify_blob_upload(&repo, &uuid, &digest_obj)
        .await
        .map_err(|e| match e {
            StorageDriverError::InvalidDigest => Error::DigestInvalid,
            e => {
                event!(Level::ERROR, "Error completing blob upload: {}", e);
                Error::InternalError
            }
        })?;

    Ok(AcceptedUpload::new(
        host,
        digest_obj,
        repo,
        Uuid(uuid),
        (0, (size as u32).saturating_sub(1)), // Note first byte is 0
    ))
}

endpoint_fn_7_levels!(
    put_blob(
        auth_user: TrowToken,
        state: State<Arc<TrowServerState>>;
        path: [image_name, uuid],
        host: AlwaysHost,
        digest: Query<DigestQuery>,
        chunk: Body
    ) -> Result<AcceptedUpload, Error>
);

/*

---
Chunked Upload

PATCH /v2/<name>/blobs/uploads/<uuid>
Content-Length: <size of chunk>
Content-Range: <start of range>-<end of range>
Content-Type: application/octet-stream

<Layer Chunk Binary Data>
---

Uploads a blob or chunk of a blob.

Checks UUID. Returns UploadInfo with range set to correct position.

*/
pub async fn patch_blob(
    _auth_user: TrowToken,
    info: Option<ContentInfo>,
    State(state): State<Arc<TrowServerState>>,
    Path((repo, uuid)): Path<(String, String)>,
    AlwaysHost(host): AlwaysHost,
    chunk: Body,
) -> Result<UploadInfo, Error> {
    match state
        .client
        .store_blob_chunk(&repo, &uuid, info, chunk)
        .await
    {
        Ok(stored) => {
            let repo_name = repo;
            let uuid = Uuid(uuid);
            Ok(UploadInfo::new(
                host,
                uuid,
                repo_name,
                (0, (stored.total_stored as u32).saturating_sub(1)), // First byte is 0
            ))
        }
        Err(StorageDriverError::InvalidName(name)) => Err(Error::NameInvalid(name)),
        Err(StorageDriverError::InvalidContentRange) => Err(Error::BlobUploadInvalid(
            "Invalid Content Range".to_string(),
        )),
        Err(_) => Err(Error::InternalError),
    }
}

endpoint_fn_7_levels!(
    patch_blob(
        auth_user: TrowToken,
        info: Option<ContentInfo>,
        state: State<Arc<TrowServerState>>;
        path: [image_name, uuid],
        host: AlwaysHost,
        chunk: Body
    ) -> Result<UploadInfo, Error>
);

/*
 Starting point for an uploading a new image or new version of an image.

 We respond with details of location and UUID to upload to with patch/put.

 No data is being transferred _unless_ the request ends with "?digest".
 In this case the whole blob is attached.
*/
pub async fn post_blob_upload(
    auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    host: AlwaysHost,
    Query(digest): Query<DigestQuery>,
    Path(repo_name): Path<String>,
    data: Body,
) -> Result<Upload, Error> {
    let uuid = state
        .client
        .storage
        .request_blob_upload(&repo_name)
        .await
        .map_err(|e| match e {
            StorageBackendError::InvalidName(n) => Error::NameInvalid(n),
            _ => Error::InternalError,
        })?;

    if digest.digest.is_some() {
        // Have a monolithic upload with data
        return put_blob(
            auth_user,
            State(state),
            Path((repo_name, uuid)),
            host,
            Query(digest),
            data,
        )
        .await
        .map(Upload::Accepted);
    }

    Ok(Upload::Info(UploadInfo::new(
        host.0,
        Uuid(uuid),
        repo_name.clone(),
        (0, 0),
    )))
}

endpoint_fn_7_levels!(
    post_blob_upload(
        auth_user: TrowToken,
        state: State<Arc<TrowServerState>>,
        host: AlwaysHost,
        digest: Query<DigestQuery>;
        path: [image_name],
        data: Body
    ) -> Result<Upload, Error>
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
        .client
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
