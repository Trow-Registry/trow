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
    ActiveModelTrait, DatabaseTransaction, EntityTrait, IntoActiveModel, ModelTrait, Set,
    TransactionTrait,
};

use super::extracts::AlwaysHost;
use super::macros::endpoint_fn_7_levels;
use crate::registry::{digest, ContentInfo, TrowServer};
use crate::routes::macros::route_7_levels;
use crate::routes::response::errors::Error;
use crate::routes::response::trow_token::TrowToken;
use crate::routes::response::upload_info::UploadInfo;
use crate::types::{AcceptedUpload, DigestQuery, OptionalDigestQuery, Upload};
use crate::{entity, TrowServerState};

mod utils {
    use std::ops::RangeInclusive;

    use super::*;

    pub async fn complete_upload(
        txn: &DatabaseTransaction,
        host: &str,
        registry: &TrowServer,
        upload: entity::blob_upload::Model,
        digest: &Digest,
        data: Body,
        range: Option<RangeInclusive<u64>>,
    ) -> Result<AcceptedUpload, Error> {
        let repo = upload.repo.clone();
        let uuid = upload.uuid;
        let size = registry
            .storage
            .write_blob_part_stream(&uuid, data.into_data_stream(), range)
            .await?;
        let mut active_upload = upload.clone().into_active_model();
        active_upload.offset = Set(size.total_stored as i32);
        active_upload.update(txn).await?;
        registry.storage.complete_blob_write(&uuid, digest).await?;

        upload.delete(txn).await?;
        entity::blob::ActiveModel {
            digest: Set(digest.clone()),
            size: Set(size.total_stored as i32),
            is_manifest: Set(false),
            ..Default::default()
        }
        .insert(txn)
        .await?;
        entity::repo_blob_association::ActiveModel {
            repo_name: Set(repo.clone()),
            blob_digest: Set(digest.clone()),
        }
        .insert(txn)
        .await?;

        Ok(AcceptedUpload::new(
            host.to_string(),
            digest.clone(),
            repo,
            uuid,
            (0, (size.total_stored as u32).saturating_sub(1)), // Note first byte is 0
        ))
    }
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
    let upload = match entity::blob_upload::Entity::find_by_id(uuid)
        .one(&txn)
        .await?
    {
        Some(u) => u,
        None => return Err(Error::NotFound),
    };
    assert_eq!(upload.repo, repo);

    let accepted_upload = utils::complete_upload(
        &txn,
        &host,
        &state.registry,
        upload,
        &digest.digest,
        chunk,
        None,
    )
    .await?;
    txn.commit().await?;

    Ok(accepted_upload)
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
    let mut upload = entity::blob_upload::Entity::find_by_id(uuid)
        .one(&txn)
        .await?
        .ok_or(Error::NotFound)?
        .into_active_model();

    let size = state
        .registry
        .storage
        .write_blob_part_stream(
            &uuid,
            chunk.into_data_stream(),
            content_info.map(|d| d.range.0..=d.range.1),
        )
        .await?;

    upload.offset = Set(size.total_stored as i32);
    txn.commit().await?;

    Ok(UploadInfo::new(
        host,
        uuid,
        repo,
        (0, (size.total_stored as u32).saturating_sub(1)), // Note first byte is 0
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
    _auth_user: TrowToken,
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
        offset: Set(0),
        ..Default::default()
    };
    let upload = upload.insert(&txn).await?;

    if let Some(digest) = digest.digest {
        // Have a monolithic upload with data
        return match utils::complete_upload(
            &txn,
            &host.0,
            &state.registry,
            upload,
            &digest,
            data,
            None,
        )
        .await
        {
            Ok(accepted_upload) => {
                txn.commit().await?;
                Ok(Upload::Accepted(accepted_upload))
            }
            Err(e) => return Err(e),
        };
    }
    println!("aaaaa");
    txn.commit().await?;
    println!("bbbb");
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

#[cfg(test)]
mod tests {

    use std::io::BufReader;

    use axum::body::Body;
    use http_body_util::BodyExt;
    use hyper::Request;
    use reqwest::StatusCode;
    use sea_orm::EntityTrait;
    use tower::{Service, ServiceExt};
    use uuid::Uuid;

    use crate::registry::Digest;
    use crate::{entity, test_utilities};

    use super::*;

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_post_blob_upload_create_new_upload() {
        let (db, state, _) = test_utilities::trow_router(|_| {}, None).await;
        let resp = post_blob_upload(
            TrowToken::default(),
            State(state),
            AlwaysHost("trow.io".to_owned()),
            Query(OptionalDigestQuery::default()),
            Path("test/blobs".to_owned()),
            Body::empty()
        ).await;

        let upload = match resp {
            Ok(Upload::Accepted(upload)) => upload,
            _ => panic!("Invalid value: {resp:?}")
        };
        assert_eq!(upload.range(), (0, 0)); // Haven't uploaded anything yet
        let upload = entity::blob_upload::Entity::find_by_id(*upload.uuid())
            .one(&db)
            .await
            .unwrap();
        assert!(upload.is_some());
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_put_blob_upload() {
        let (_, _, mut router) = test_utilities::trow_router(|_| {}, None).await;
        let repo_name = "test";
        let resp = router
            .call(
                Request::post(&format!("/v2/{repo_name}/blobs/uploads/"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::ACCEPTED,
            "resp: {:?}",
            resp.into_body().collect().await.unwrap().to_bytes()
        );
        let resp_headers = resp.headers();
        let uuid = resp_headers
            .get(test_utilities::UPLOAD_HEADER)
            .unwrap()
            .to_str()
            .unwrap();

        let range = resp_headers
            .get(test_utilities::RANGE_HEADER)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(range, "0-0"); // Haven't uploaded anything yet

        //used by oci_manifest_test
        let config = "{}\n".as_bytes();
        let digest = Digest::digest_sha256(BufReader::new(config)).unwrap();
        let loc = &format!("/v2/{}/blobs/uploads/{}?digest={}", repo_name, uuid, digest);

        let resp = router
            .call(Request::put(loc).body(Body::from(config)).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let range = resp
            .headers()
            .get(test_utilities::RANGE_HEADER)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(range, format!("0-{}", (config.len() - 1))); //note first byte is 0, hence len - 1
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_post_blob_upload_complete_upload() {
        let (_, _, router) = test_utilities::trow_router(|_| {}, None).await;
        let repo_name = "test";

        let config = "{ }\n".as_bytes();
        let digest = Digest::digest_sha256(BufReader::new(config)).unwrap();

        let resp = router
            .clone()
            .oneshot(
                Request::post(&format!(
                    "/v2/{}/blobs/uploads/?digest={}",
                    repo_name, digest
                ))
                .body(Body::from(config))
                .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let range = resp
            .headers()
            .get(test_utilities::RANGE_HEADER)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(range, format!("0-{}", (config.len() - 1))); //note first byte is 0, hence len - 1
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_patch_blob_upload() {
        let (db, state, _) = test_utilities::trow_router(|_| {}, None).await;
        entity::repo::insert_if_not_exists(&db, "germany".to_owned()).await.unwrap();
        let upload = entity::blob_upload::ActiveModel {
            offset: Set(7),
            repo: Set("germany".to_owned()),
            ..Default::default()
        };
        let upload = upload.insert(&db).await.unwrap();
        state.registry.storage.write_blob_part_stream(&upload.uuid, Body::from("whazaaa").into_data_stream(), None).await.unwrap();


        let resp = patch_blob_upload(
            TrowToken::default(),
            None,
            State(state),
            Path((upload.repo.clone(), upload.uuid.clone())),
            AlwaysHost("trow.io".to_owned()),
            Body::from("whaaa so much dataaa")
        ).await;

        let uploadinfo = match resp {
            Ok(ui) => ui,
            _ => panic!("Invalid response: {resp:?}")
        };

        assert_eq!(
            uploadinfo.base_url(),
            "trow.io".to_owned()
        );
        assert_eq!(uploadinfo.range(), (0, 7 + 20));
        assert_eq!(uploadinfo.repo_name(), &upload.repo);
    }
}
