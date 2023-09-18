use std::sync::Arc;

use anyhow::Result;
use axum::extract::{BodyStream, Path, Query, State};
use axum::http::header::HeaderMap;
use tracing::{event, Level};

use crate::registry_interface::{digest, BlobReader, BlobStorage, ContentInfo, StorageDriverError};
use crate::response::errors::Error;
use crate::response::get_base_url;
use crate::response::trow_token::TrowToken;
use crate::response::upload_info::UploadInfo;
use crate::types::{AcceptedUpload, BlobDeleted, DigestQuery, RepoName, Upload, Uuid};
use crate::TrowServerState;

/*
---
Pulling a Layer
GET /v2/<name>/blobs/<digest>
name - name of the repository
digest - unique identifier for the blob to be downoaded

# Responses
200 - blob is downloaded
307 - redirect to another service for downloading[1]
 */
pub async fn get_blob(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((one, digest)): Path<(String, String)>,
) -> Result<BlobReader, Error> {
    let digest = match digest::parse(&digest) {
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
pub async fn get_blob_2level(
    auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((one, two, digest)): Path<(String, String, String)>,
) -> Result<BlobReader, Error> {
    get_blob(
        auth_user,
        State(state),
        Path((format!("{one}/{two}"), digest)),
    )
    .await
}
pub async fn get_blob_3level(
    auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((one, two, three, digest)): Path<(String, String, String, String)>,
) -> Result<BlobReader, Error> {
    get_blob(
        auth_user,
        State(state),
        Path((format!("{one}/{two}/{three}"), digest)),
    )
    .await
}
pub async fn get_blob_4level(
    auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((one, two, three, four, digest)): Path<(String, String, String, String, String)>,
) -> Result<BlobReader, Error> {
    get_blob(
        auth_user,
        State(state),
        Path((format!("{one}/{two}/{three}/{four}"), digest)),
    )
    .await
}
pub async fn get_blob_5level(
    auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((one, two, three, four, five, digest)): Path<(
        String,
        String,
        String,
        String,
        String,
        String,
    )>,
) -> Result<BlobReader, Error> {
    get_blob(
        auth_user,
        State(state),
        Path((format!("{one}/{two}/{three}/{four}/{five}"), digest)),
    )
    .await
}

/*
---
Monolithic Upload
PUT /v2/<name>/blobs/uploads/<uuid>?digest=<digest>
Content-Length: <size of layer>
Content-Type: application/octet-stream

<Layer Binary Data>
 */

/**
 * Completes the upload.
 */
pub async fn put_blob(
    headers: HeaderMap,
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((repo, uuid)): Path<(String, String)>,
    Query(digest): Query<DigestQuery>,
    chunk: BodyStream,
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

    let digest_obj = digest::parse(&digest).map_err(|_| Error::DigestInvalid)?;
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
        get_base_url(&headers, &state.config),
        digest_obj,
        RepoName(repo),
        Uuid(uuid),
        (0, (size as u32).saturating_sub(1)), // Note first byte is 0
    ))
}
pub async fn put_blob_2level(
    headers: HeaderMap,
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    Path((one, two, uuid)): Path<(String, String, String)>,
    digest: Query<DigestQuery>,
    chunk: BodyStream,
) -> Result<AcceptedUpload, Error> {
    put_blob(
        headers,
        auth_user,
        state,
        Path((format!("{one}/{two}"), uuid)),
        digest,
        chunk,
    )
    .await
}
pub async fn put_blob_3level(
    headers: HeaderMap,
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    Path((one, two, three, uuid)): Path<(String, String, String, String)>,
    digest: Query<DigestQuery>,
    chunk: BodyStream,
) -> Result<AcceptedUpload, Error> {
    put_blob(
        headers,
        auth_user,
        state,
        Path((format!("{one}/{two}/{three}"), uuid)),
        digest,
        chunk,
    )
    .await
}
pub async fn put_blob_4level(
    headers: HeaderMap,
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    Path((one, two, three, four, uuid)): Path<(String, String, String, String, String)>,
    digest: Query<DigestQuery>,
    chunk: BodyStream,
) -> Result<AcceptedUpload, Error> {
    put_blob(
        headers,
        auth_user,
        state,
        Path((format!("{one}/{two}/{three}/{four}"), uuid)),
        digest,
        chunk,
    )
    .await
}
pub async fn put_blob_5level(
    headers: HeaderMap,
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    Path((one, two, three, four, five, uuid)): Path<(
        String,
        String,
        String,
        String,
        String,
        String,
    )>,
    digest: Query<DigestQuery>,
    chunk: BodyStream,
) -> Result<AcceptedUpload, Error> {
    put_blob(
        headers,
        auth_user,
        state,
        Path((format!("{one}/{two}/{three}/{four}/{five}"), uuid)),
        digest,
        chunk,
    )
    .await
}

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
    headers: HeaderMap,
    _auth_user: TrowToken,
    info: Option<ContentInfo>,
    State(state): State<Arc<TrowServerState>>,
    Path((repo, uuid)): Path<(String, String)>,
    chunk: BodyStream,
) -> Result<UploadInfo, Error> {
    match state
        .client
        .store_blob_chunk(&repo, &uuid, info, chunk)
        .await
    {
        Ok(stored) => {
            let repo_name = RepoName(repo);
            let uuid = Uuid(uuid);
            Ok(UploadInfo::new(
                get_base_url(&headers, &state.config),
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

pub async fn patch_blob_2level(
    headers: HeaderMap,
    auth_user: TrowToken,
    info: Option<ContentInfo>,
    State(state): State<Arc<TrowServerState>>,
    Path((one, two, uuid)): Path<(String, String, String)>,
    chunk: BodyStream,
) -> Result<UploadInfo, Error> {
    patch_blob(
        headers,
        auth_user,
        info,
        State(state),
        Path((format!("{one}/{two}"), uuid)),
        chunk,
    )
    .await
}

pub async fn patch_blob_3level(
    headers: HeaderMap,
    auth_user: TrowToken,
    info: Option<ContentInfo>,
    State(state): State<Arc<TrowServerState>>,
    Path((one, two, three, uuid)): Path<(String, String, String, String)>,
    chunk: BodyStream,
) -> Result<UploadInfo, Error> {
    patch_blob(
        headers,
        auth_user,
        info,
        State(state),
        Path((format!("{one}/{two}/{three}"), uuid)),
        chunk,
    )
    .await
}

pub async fn patch_blob_4level(
    headers: HeaderMap,
    auth_user: TrowToken,
    info: Option<ContentInfo>,
    state: State<Arc<TrowServerState>>,
    Path((one, two, three, four, uuid)): Path<(String, String, String, String, String)>,
    chunk: BodyStream,
) -> Result<UploadInfo, Error> {
    patch_blob(
        headers,
        auth_user,
        info,
        state,
        Path((format!("{one}/{two}/{three}/{four}"), uuid)),
        chunk,
    )
    .await
}

pub async fn patch_blob_5level(
    headers: HeaderMap,
    auth_user: TrowToken,
    info: Option<ContentInfo>,
    state: State<Arc<TrowServerState>>,
    Path((one, two, three, four, five, uuid)): Path<(
        String,
        String,
        String,
        String,
        String,
        String,
    )>,
    chunk: BodyStream,
) -> Result<UploadInfo, Error> {
    patch_blob(
        headers,
        auth_user,
        info,
        state,
        Path((format!("{one}/{two}/{three}/{four}/{five}"), uuid)),
        chunk,
    )
    .await
}

/*
 Starting point for an uploading a new image or new version of an image.

 We respond with details of location and UUID to upload to with patch/put.

 No data is being transferred _unless_ the request ends with "?digest".
 In this case the whole blob is attached.
*/
pub async fn post_blob_upload(
    auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    headers: HeaderMap,
    Query(digest): Query<DigestQuery>,
    Path(repo_name): Path<String>,
    data: BodyStream,
) -> Result<Upload, Error> {
    /*
        Ask the backend for a UUID.

        We should also need to do some checking that the user is allowed
        to upload first.

        If using a true UUID it is possible for the frontend to generate
        and tell the backend what the UUID is. This is a potential
        optimisation, but is arguably less flexible.
    */
    let uuid = state
        .client
        .start_blob_upload(&repo_name)
        .await
        .map_err(|e| match e {
            StorageDriverError::InvalidName(n) => Error::NameInvalid(n),
            _ => Error::InternalError,
        })?;

    if digest.digest.is_some() {
        // Have a monolithic upload with data
        return put_blob(
            headers,
            auth_user,
            State(state),
            Path((repo_name, uuid)),
            Query(digest),
            data,
        )
        .await
        .map(Upload::Accepted);
    }

    Ok(Upload::Info(UploadInfo::new(
        get_base_url(&headers, &state.config),
        Uuid(uuid),
        RepoName(repo_name.clone()),
        (0, 0),
    )))
}
pub async fn post_blob_upload_2level(
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    headers: HeaderMap,
    digest: Query<DigestQuery>,
    Path((one, two)): Path<(String, String)>,
    data: BodyStream,
) -> Result<Upload, Error> {
    post_blob_upload(
        auth_user,
        state,
        headers,
        digest,
        Path(format!("{one}/{two}")),
        data,
    )
    .await
}
pub async fn post_blob_upload_3level(
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    headers: HeaderMap,
    digest: Query<DigestQuery>,
    Path((one, two, three)): Path<(String, String, String)>,
    data: BodyStream,
) -> Result<Upload, Error> {
    post_blob_upload(
        auth_user,
        state,
        headers,
        digest,
        Path(format!("{one}/{two}/{three}")),
        data,
    )
    .await
}
pub async fn post_blob_upload_4level(
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    headers: HeaderMap,
    digest: Query<DigestQuery>,
    Path((one, two, three, four)): Path<(String, String, String, String)>,
    data: BodyStream,
) -> Result<Upload, Error> {
    post_blob_upload(
        auth_user,
        state,
        headers,
        digest,
        Path(format!("{one}/{two}/{three}/{four}")),
        data,
    )
    .await
}
pub async fn post_blob_upload_5level(
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    headers: HeaderMap,
    digest: Query<DigestQuery>,
    Path((one, two, three, four, five)): Path<(String, String, String, String, String)>,
    data: BodyStream,
) -> Result<Upload, Error> {
    post_blob_upload(
        auth_user,
        state,
        headers,
        digest,
        Path(format!("{one}/{two}/{three}/{four}/{five}")),
        data,
    )
    .await
}

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
    let digest = digest::parse(&digest).map_err(|_| Error::DigestInvalid)?;
    state
        .client
        .delete_blob(&one, &digest)
        .await
        .map_err(|_| Error::NotFound)?;
    Ok(BlobDeleted {})
}
pub async fn delete_blob_2level(
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    Path((one, two, digest)): Path<(String, String, String)>,
) -> Result<BlobDeleted, Error> {
    delete_blob(auth_user, state, Path((format!("{one}/{two}"), digest))).await
}
pub async fn delete_blob_3level(
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    Path((one, two, three, digest)): Path<(String, String, String, String)>,
) -> Result<BlobDeleted, Error> {
    delete_blob(
        auth_user,
        state,
        Path((format!("{one}/{two}/{three}"), digest)),
    )
    .await
}
pub async fn delete_blob_4level(
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    Path((one, two, three, four, digest)): Path<(String, String, String, String, String)>,
) -> Result<BlobDeleted, Error> {
    delete_blob(
        auth_user,
        state,
        Path((format!("{one}/{two}/{three}/{four}"), digest)),
    )
    .await
}
pub async fn delete_blob_5level(
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    Path((one, two, three, four, five, digest)): Path<(
        String,
        String,
        String,
        String,
        String,
        String,
    )>,
) -> Result<BlobDeleted, Error> {
    delete_blob(
        auth_user,
        state,
        Path((format!("{one}/{two}/{three}/{four}/{five}"), digest)),
    )
    .await
}
