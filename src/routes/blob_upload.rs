use std::sync::Arc;

use anyhow::Result;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::response::Response;
use axum::routing::{post, put};
use axum::Router;
use digest::Digest;
use hyper::StatusCode;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DbErr, EntityTrait, IntoActiveModel, ModelTrait, RuntimeErr,
    Set, TransactionTrait,
};
use tracing::{event, Level};

use super::extracts::AlwaysHost;
use super::macros::endpoint_fn_7_levels;
use crate::registry::{digest, ContentInfo, RegistryError, TrowServer};
use crate::routes::macros::route_7_levels;
use crate::routes::response::errors::Error;
use crate::routes::response::trow_token::TrowToken;
use crate::routes::response::upload_info::UploadInfo;
use crate::types::{AcceptedUpload, DigestQuery, OptionalDigestQuery, Upload};
use crate::{entity, TrowServerState};

async fn _complete_upload(
    txn: impl TransactionTrait,
    registry: TrowServer,
    upload: entity::blob_upload::Model,
    digest: &Digest,
    repo: &str,
    chunk: Body,
) -> Result<AcceptedUpload, Error> {
    let size = registry
        .storage
        .write_blob_part_stream(
            &upload.uuid,
            chunk.into_data_stream(),
            None, // range: ???
        )
        .await?;
    registry
        .storage
        .complete_blob_write(&upload.uuid, &digest)
        .await?;

    upload.delete(&txn).await?;

    let digest_obj = Digest::try_from_raw(&digest).map_err(|_| Error::DigestInvalid)?;
    state
        .registry
        .complete_and_verify_blob_upload(&txn, &repo, &uuid, &digest)
        .await
        .map_err(|e| match e {
            RegistryError::InvalidDigest => Error::DigestInvalid,
            e => {
                event!(Level::ERROR, "Error completing blob upload: {}", e);
                Error::InternalError
            }
        })?;

    let blob = entity::blob::ActiveModel {
        digest: Set(digest.clone()),
        repo: Set(repo.clone()),
        size: Set(size as i32),
        ..Default::default()
    };
    blob.insert(&txn).await?;
    // upload.delete(&txn).await?;

    txn.commit().await?;

    Ok(AcceptedUpload::new(
        host,
        digest_obj,
        repo,
        uuid,
        (0, (size as u32).saturating_sub(1)), // Note first byte is 0
    ))
}

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
async fn put_blob_upload(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((repo, uuid)): Path<(String, uuid::Uuid)>,
    AlwaysHost(host): AlwaysHost,
    Query(digest): Query<DigestQuery>,
    chunk: Body,
) -> Result<AcceptedUpload, Error> {
    let txn = state.db.begin().await?;
    let upload = match entity::blob_upload::Entity::find_by_id(uuid.clone())
        .one(&txn)
        .await?
    {
        Some(u) => u,
        None => return Err(Error::NotFound),
    };

    let size = match state
        .registry
        .store_blob_chunk(&repo, &uuid, None, chunk)
        .await
    {
        Ok(stored) => stored.total_stored,
        Err(RegistryError::InvalidName(name)) => return Err(Error::NameInvalid(name)),
        Err(RegistryError::InvalidContentRange) => {
            return Err(Error::BlobUploadInvalid(
                "Invalid Content Range".to_string(),
            ))
        }
        Err(e) => {
            event!(Level::ERROR, "Error storing blob chunk: {}", e);
            return Err(Error::InternalError);
        }
    };
    state
        .registry
        .storage
        .complete_blob_write(&uuid, &digest.digest)
        .await?;
    upload.delete(&txn).await?;

    let digest_obj = digest.digest;
    state
        .registry
        .complete_and_verify_blob_upload(&txn, &repo, &uuid, &digest_obj)
        .await
        .map_err(|e| match e {
            RegistryError::InvalidDigest => Error::DigestInvalid,
            e => {
                event!(Level::ERROR, "Error completing blob upload: {}", e);
                Error::InternalError
            }
        })?;

    let blob = entity::blob::ActiveModel {
        digest: Set(digest.clone()),
        repo: Set(repo.clone()),
        size: Set(size as i32),
        ..Default::default()
    };
    blob.insert(&txn).await?;
    // upload.delete(&txn).await?;

    txn.commit().await?;

    Ok(AcceptedUpload::new(
        host,
        digest_obj,
        repo,
        uuid,
        (0, (size as u32).saturating_sub(1)), // Note first byte is 0
    ))
}

endpoint_fn_7_levels!(
    put_blob_upload(
        auth_user: TrowToken,
        state: State<Arc<TrowServerState>>;
        path: [image_name, uuid: uuid::Uuid],
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
async fn patch_blob_upload(
    _auth_user: TrowToken,
    content_info: Option<ContentInfo>,
    State(state): State<Arc<TrowServerState>>,
    Path((repo, uuid)): Path<(String, uuid::Uuid)>,
    AlwaysHost(host): AlwaysHost,
    chunk: Body,
) -> Result<UploadInfo, Error> {
    let txn = state.db.begin().await?;
    let mut upload = match entity::blob_upload::Entity::find_by_id(uuid)
        .one(&txn)
        .await?
    {
        Some(u) => u.into_active_model(),
        None => return Err(Error::NotFound),
    };

    let size = match state
        .registry
        .store_blob_chunk(&repo, &uuid, content_info, chunk)
        .await
    {
        Ok(stored) => stored.total_stored,
        Err(RegistryError::InvalidName(name)) => return Err(Error::NameInvalid(name)),
        Err(RegistryError::InvalidContentRange) => {
            return Err(Error::BlobUploadInvalid(
                "Invalid Content Range".to_string(),
            ))
        }
        Err(e) => {
            event!(Level::ERROR, "Error storing blob chunk: {}", e);
            return Err(Error::InternalError);
        }
    };
    // let mut upload = upload.into_active_model();
    upload.offset = Set(size as i32);

    txn.commit().await?;

    Ok(UploadInfo::new(
        host,
        uuid,
        repo,
        (0, (size as u32).saturating_sub(1)), // Note first byte is 0
    ))
}

endpoint_fn_7_levels!(
    patch_blob_upload(
        auth_user: TrowToken,
        info: Option<ContentInfo>,
        state: State<Arc<TrowServerState>>;
        path: [image_name, uuid: uuid::Uuid],
        host: AlwaysHost,
        chunk: Body
    ) -> Result<UploadInfo, Error>
);

/*
POST /v2/<name>/blobs/uploads/?digest=<digest>

Starting point for an uploading a new image or new version of an image.

We respond with details of location and UUID to upload to with patch/put.

No data is being transferred _unless_ the request ends with "?digest".
In this case the whole blob is attached.
*/
async fn post_blob_upload(
    auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    host: AlwaysHost,
    Query(digest): Query<OptionalDigestQuery>,
    Path(repo_name): Path<String>,
    data: Body,
) -> Result<Upload, Error> {
    let txn = state.db.begin().await?;

    entity::repo::insert_if_not_exists(&txn, repo_name.clone()).await?;
    let upload = entity::blob_upload::ActiveModel {
        repo: Set(repo_name.clone()),
        ..Default::default()
    };
    let upload = upload.insert(&txn).await?;

    if digest.digest.is_some() {
        // Have a monolithic upload with data
        return _complete_upload(txn, upload, data)
            .await
            .map(Upload::Accepted);
        // return put_blob_upload(
        //     auth_user,
        //     State(state),
        //     Path((repo_name, upload.uuid)),
        //     host,
        //     Query(digest),
        //     data,
        // )
        // .await
        // .map(Upload::Accepted);
    }
    println!("hello 3");
    txn.commit().await?;
    println!("hello 4");
    Ok(Upload::Info(UploadInfo::new(
        host.0,
        upload.uuid,
        repo_name,
        (0, 0),
    )))
}

endpoint_fn_7_levels!(
    post_blob_upload(
        auth_user: TrowToken,
        state: State<Arc<TrowServerState>>,
        host: AlwaysHost,
        digest: Query<OptionalDigestQuery>;
        path: [image_name],
        data: Body
    ) -> Result<Upload, Error>
);

/*
GET /v2/<name>/blobs/uploads/<upload_id>
*/
async fn get_blob_upload(
    _auth: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    AlwaysHost(host): AlwaysHost,
    Path((repo_name, upload_id)): Path<(String, uuid::Uuid)>,
) -> Result<Response, Error> {
    // let offset = state
    //     .registry
    //     .storage
    //     .get_upload_status(&repo_name, &upload_id)
    //     .await
    //     .map_err(|e| match e {
    //         StorageBackendError::InvalidName(n) => Error::NameInvalid(n),
    //         _ => Error::InternalError,
    //     })?;
    let offset = 1;

    Ok(Response::builder()
        .header("Docker-Upload-UUID", upload_id.to_string())
        .header("Range", format!("0-{offset}"))
        .header("Content-Length", "0")
        .header("Location", host)
        .status(StatusCode::NO_CONTENT)
        .body(Body::empty())
        .unwrap())
}

endpoint_fn_7_levels!(
    get_blob_upload(
        auth: TrowToken,
        state: State<Arc<TrowServerState>>,
        host: AlwaysHost;
        path: [image_name, upload_id: uuid::Uuid]
    ) -> Result<Response, Error>
);

pub fn route(mut app: Router<Arc<TrowServerState>>) -> Router<Arc<TrowServerState>> {
    #[rustfmt::skip]
    route_7_levels!(
        app,
        "/v2" "/blobs/uploads/",
        post(post_blob_upload, post_blob_upload_2level, post_blob_upload_3level, post_blob_upload_4level, post_blob_upload_5level, post_blob_upload_6level, post_blob_upload_7level)
    );
    #[rustfmt::skip]
    route_7_levels!(
        app,
        "/v2" "/blobs/uploads/:uuid",
        put(put_blob_upload, put_blob_upload_2level, put_blob_upload_3level, put_blob_upload_4level, put_blob_upload_5level, put_blob_upload_6level, put_blob_upload_7level),
        patch(patch_blob_upload, patch_blob_upload_2level, patch_blob_upload_3level, patch_blob_upload_4level, patch_blob_upload_5level, patch_blob_upload_6level, patch_blob_upload_7level),
        get(get_blob_upload, get_blob_upload_2level, get_blob_upload_3level, get_blob_upload_4level, get_blob_upload_5level, get_blob_upload_6level, get_blob_upload_7level)
    );
    app
}
