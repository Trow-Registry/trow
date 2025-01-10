use std::sync::Arc;

use anyhow::Result;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::response::Response;
use axum::routing::{post, put};
use axum::Router;
use digest::Digest;
use hyper::StatusCode;

use super::macros::endpoint_fn_7_levels;
use crate::registry::{digest, ContentInfo, TrowServer};
use crate::routes::macros::route_7_levels;
use crate::routes::response::errors::Error;
use crate::routes::response::trow_token::TrowToken;
use crate::routes::response::upload_info::UploadInfo;
use crate::types::{AcceptedUpload, DigestQuery, OptionalDigestQuery, Upload};
use crate::TrowServerState;

mod utils {
    use std::ops::RangeInclusive;

    use sqlx::{Sqlite, Transaction};
    use uuid::Uuid;

    use super::*;

    pub async fn complete_upload(
        txn: &mut Transaction<'static, Sqlite>,

        registry: &TrowServer,
        upload_id: &str,
        digest: &Digest,
        data: Body,
        range: Option<RangeInclusive<u64>>,
    ) -> Result<AcceptedUpload, Error> {
        let upload = sqlx::query!(
            r#"
            SELECT * FROM blob_upload
            WHERE uuid=$1
            "#,
            upload_id,
        )
        .fetch_one(&mut **txn)
        .await?;
        let upload_id_bin = Uuid::parse_str(upload_id).unwrap();

        let size = registry
            .storage
            .write_blob_part_stream(&upload_id_bin, data.into_data_stream(), range)
            .await?;

        sqlx::query!(
            r#"
            DELETE FROM blob_upload
            WHERE uuid=$1
            "#,
            upload.uuid
        )
        .execute(&mut **txn)
        .await?;

        let digest_str = digest.as_str();
        let size_i64 = size.total_stored as i64;
        sqlx::query!(
            r#"
            INSERT INTO blob (digest, size, is_manifest)
            VALUES ($1, $2, false)
            "#,
            digest_str,
            size_i64
        )
        .execute(&mut **txn)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO repo_blob_association
            VALUES ($1, $2)
            "#,
            upload.repo,
            digest_str,
        )
        .execute(&mut **txn)
        .await?;

        Ok(AcceptedUpload::new(
            digest.clone(),
            upload.repo,
            upload_id_bin,
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

    Query(digest): Query<DigestQuery>,
    chunk: Body,
) -> Result<AcceptedUpload, Error> {
    let mut txn = state.db.begin().await?;
    let uuid_str = uuid.to_string();
    let upload = sqlx::query!(
        r#"
        SELECT * FROM blob_upload
        WHERE uuid=$1
        "#,
        uuid_str,
    )
    .fetch_one(&mut *txn)
    .await?;
    assert_eq!(upload.repo, repo);

    let accepted_upload = utils::complete_upload(
        &mut txn,
        &state.registry,
        &uuid_str,
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
    chunk: Body,
) -> Result<UploadInfo, Error> {
    let mut txn = state.db.begin().await?;
    let uuid_str = uuid.to_string();
    sqlx::query!(
        r#"
        SELECT * FROM blob_upload
        WHERE uuid=$1
        "#,
        uuid_str,
    )
    .fetch_one(&mut *txn)
    .await?;

    let size = state
        .registry
        .storage
        .write_blob_part_stream(
            &uuid,
            chunk.into_data_stream(),
            content_info.map(|d| d.range.0..=d.range.1),
        )
        .await?;
    let total_stored = size.total_stored as u32;

    sqlx::query!(
        r#"
        UPDATE blob_upload
        SET offset=$1
        WHERE uuid=$1
        "#,
        total_stored,
    )
    .execute(&mut *txn)
    .await?;

    Ok(UploadInfo::new(
        uuid_str,
        repo,
        (0, (total_stored).saturating_sub(1)), // Note first byte is 0
    ))
}

endpoint_fn_7_levels!(
    patch_blob_upload(
        auth_user: TrowToken,
        info: Option<ContentInfo>,
        state: State<Arc<TrowServerState>>;
        path: [image_name, uuid: uuid::Uuid],
        chunk: Body
    ) -> Result<UploadInfo, Error>
);

/*
POST /v2/<name>/blobs/uploads/?digest=<digest>

Starting point for uploading a new image or new version of an image.

We respond with details of location and UUID to upload to with patch/put.

No data is being transferred _unless_ the request ends with "?digest".
In this case the whole blob is attached.
*/
async fn post_blob_upload(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,

    Query(digest): Query<OptionalDigestQuery>,
    Path(repo_name): Path<String>,
    data: Body,
) -> Result<Upload, Error> {
    let mut txn = state.db.begin().await?;

    // Create new blob upload
    let upload_uuid = uuid::Uuid::new_v4().to_string();
    sqlx::query!(
        r#"
        INSERT INTO blob_upload (uuid, repo, offset)
        VALUES ($1, $2, $3)
        "#,
        upload_uuid,
        repo_name,
        0_i64
    )
    .execute(&mut *txn)
    .await?;

    if let Some(digest) = digest.digest {
        // Have a monolithic upload with data
        return match utils::complete_upload(
            &mut txn,
            &state.registry,
            &upload_uuid,
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
            Err(e) => Err(e),
        };
    }

    txn.commit().await?;

    Ok(Upload::Info(UploadInfo::new(
        upload_uuid,
        repo_name,
        (0, 0),
    )))
}

endpoint_fn_7_levels!(
    post_blob_upload(
        auth_user: TrowToken,
        state: State<Arc<TrowServerState>>,
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
    Path((repo_name, upload_id)): Path<(String, uuid::Uuid)>,
) -> Result<Response, Error> {
    let upload_id_str = upload_id.to_string();
    let offset: i64 = sqlx::query_scalar!(
        "SELECT offset FROM blob_upload WHERE uuid = $1 AND repo = $2",
        upload_id_str,
        repo_name
    )
    .fetch_one(&state.db)
    .await?;
    let location = format!("/v2/{}/blobs/uploads/{}", repo_name, upload_id);

    Ok(Response::builder()
        .header("Docker-Upload-UUID", upload_id.to_string())
        .header("Range", format!("0-{}", offset - 1)) // Offset is 0-based
        .header("Content-Length", "0")
        .header("Location", location)
        .status(StatusCode::NO_CONTENT)
        .body(Body::empty())
        .unwrap())
}

endpoint_fn_7_levels!(
    get_blob_upload(
        auth: TrowToken,
        state: State<Arc<TrowServerState>>;
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
    use tower::{Service, ServiceExt};
    use uuid::Uuid;

    use super::*;
    use crate::registry::Digest;
    use crate::test_utilities;

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_post_blob_upload_create_new_upload() {
        let (state, _) = test_utilities::trow_router(|_| {}, None).await;
        let resp = post_blob_upload(
            TrowToken::default(),
            State(state.clone()),
            Query(OptionalDigestQuery::default()),
            Path("test/blobs".to_owned()),
            Body::empty(),
        )
        .await;

        let upload = match resp {
            Ok(Upload::Info(upload)) => upload,
            _ => panic!("Invalid value: {resp:?}"),
        };
        assert_eq!(upload.range(), (0, 0)); // Haven't uploaded anything yet
        let mut conn = state.db.acquire().await.unwrap();
        let upload_uuid = upload.uuid().to_string();
        sqlx::query!(
            r#"
            SELECT * FROM blob_upload
            WHERE uuid = $1
            "#,
            upload_uuid
        )
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_put_blob_upload() {
        let (_, mut router) = test_utilities::trow_router(|_| {}, None).await;
        let repo_name = "test";
        let resp = router
            .call(
                Request::post(format!("/v2/{repo_name}/blobs/uploads/"))
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
        let (_, router) = test_utilities::trow_router(|_| {}, None).await;
        let repo_name = "test";

        let config = "{ }\n".as_bytes();
        let digest = Digest::digest_sha256(BufReader::new(config)).unwrap();

        let resp = router
            .clone()
            .oneshot(
                Request::post(format!(
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
        let (state, _) = test_utilities::trow_router(|_| {}, None).await;
        let upload_uuid = Uuid::new_v4();
        let upload_uuid_str = upload_uuid.to_string();
        sqlx::query!(
            r#"
            INSERT INTO blob_upload (uuid, offset, repo)
            VALUES ($1, 7, 'germany')
            "#,
            upload_uuid_str
        )
        .execute(&mut *state.db.acquire().await.unwrap())
        .await
        .unwrap();
        state
            .registry
            .storage
            .write_blob_part_stream(&upload_uuid, Body::from("whazaaa").into_data_stream(), None)
            .await
            .unwrap();

        let resp = patch_blob_upload(
            TrowToken::default(),
            None,
            State(state),
            Path(("germany".to_string(), upload_uuid)),
            Body::from("whaaa so much dataaa"),
        )
        .await;

        let uploadinfo = match resp {
            Ok(ui) => ui,
            _ => panic!("Invalid response: {resp:?}"),
        };

        assert_eq!(uploadinfo.range(), (0, 20 + 7 - 1));
        assert_eq!(uploadinfo.repo_name(), "germany");
    }
}
