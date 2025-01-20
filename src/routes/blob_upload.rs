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
use crate::registry::server::PROXY_DIR;
use crate::registry::{digest, ContentInfo, TrowServer};
use crate::routes::macros::route_7_levels;
use crate::routes::response::errors::Error;
use crate::routes::response::trow_token::TrowToken;
use crate::routes::response::upload_info::UploadInfo;
use crate::types::{AcceptedUpload, DigestQuery, OptionalDigestQuery, Upload};
use crate::TrowServerState;

mod utils {
    use std::ops::RangeInclusive;

    use sqlx::SqlitePool;
    use uuid::Uuid;

    use super::*;

    pub async fn complete_upload(
        db: &SqlitePool,
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
        .fetch_one(&mut *db.acquire().await?)
        .await?;
        let upload_id_bin = Uuid::parse_str(upload_id).unwrap();

        let size = registry
            .storage
            .write_blob_part_stream(&upload_id_bin, data.into_data_stream(), range)
            .await?;

        registry
            .storage
            .complete_blob_write(&upload_id_bin, digest)
            .await?;

        sqlx::query!(
            r#"
            DELETE FROM blob_upload
            WHERE uuid=$1
            "#,
            upload.uuid
        )
        .execute(&mut *db.acquire().await?)
        .await?;

        let digest_str = digest.as_str();
        let size_i64 = size.total_stored as i64;
        sqlx::query!(
            r#"
            INSERT INTO blob (digest, size)
            VALUES ($1, $2) ON CONFLICT (digest) DO NOTHING
            "#,
            digest_str,
            size_i64
        )
        .execute(&mut *db.acquire().await?)
        .await?;

        sqlx::query!(
            "INSERT INTO repo_blob_association VALUES ($1, $2) ON CONFLICT DO NOTHING",
            upload.repo,
            digest_str,
        )
        .execute(&mut *db.acquire().await?)
        .await?;

        Ok(AcceptedUpload::new(
            digest.clone(),
            upload.repo,
            upload_id_bin,
            (0, size.total_stored.saturating_sub(1)), // Note first byte is 0
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
    if repo.starts_with(PROXY_DIR) {
        return Err(Error::UnsupportedForProxiedRepo);
    }
    let uuid_str = uuid.to_string();
    let upload = sqlx::query!(
        r#"
        SELECT * FROM blob_upload
        WHERE uuid=$1
        "#,
        uuid_str,
    )
    .fetch_one(&mut *state.db.acquire().await?)
    .await?;
    assert_eq!(upload.repo, repo);

    let accepted_upload = utils::complete_upload(
        &state.db,
        &state.registry,
        &uuid_str,
        &digest.digest,
        chunk,
        None,
    )
    .await?;

    // missing location header
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
    let uuid_str = uuid.to_string();
    sqlx::query!(
        r#"
        SELECT * FROM blob_upload
        WHERE uuid=$1
        "#,
        uuid_str,
    )
    .fetch_one(&mut *state.db.acquire().await?)
    .await?;

    let content_range = content_info.map(|ci| ci.range.0..=ci.range.1);
    let size = state
        .registry
        .storage
        .write_blob_part_stream(&uuid, chunk.into_data_stream(), content_range)
        .await?;
    let total_stored = size.total_stored as i64;
    sqlx::query!(
        "UPDATE blob_upload SET offset=$2 WHERE uuid=$1",
        uuid_str,
        total_stored,
    )
    .execute(&mut *state.db.acquire().await?)
    .await?;

    Ok(UploadInfo::new(
        uuid_str,
        repo,
        (0, (size.total_stored).saturating_sub(1)), // Note first byte is 0
    ))
}

endpoint_fn_7_levels!(
    patch_blob_upload(
        auth_user: TrowToken,
        content_info: Option<ContentInfo>,
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
    if repo_name.starts_with(PROXY_DIR) {
        return Err(Error::UnsupportedForProxiedRepo);
    }

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
    .execute(&mut *state.db.acquire().await?)
    .await?;

    if let Some(digest) = digest.digest {
        // Have a monolithic upload with data
        return match utils::complete_upload(
            &state.db,
            &state.registry,
            &upload_uuid,
            &digest,
            data,
            None,
        )
        .await
        {
            Ok(accepted_upload) => Ok(Upload::Accepted(accepted_upload)),
            Err(e) => Err(e),
        };
    }

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
        .header("Range", format!("0-{}", (offset as u64).saturating_sub(1)))
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
        "/v2" "/blobs/uploads/{uuid}",
        put(put_blob_upload, put_blob_upload_2level, put_blob_upload_3level, put_blob_upload_4level, put_blob_upload_5level, put_blob_upload_6level, put_blob_upload_7level),
        patch(patch_blob_upload, patch_blob_upload_2level, patch_blob_upload_3level, patch_blob_upload_4level, patch_blob_upload_5level, patch_blob_upload_6level, patch_blob_upload_7level),
        get(get_blob_upload, get_blob_upload_2level, get_blob_upload_3level, get_blob_upload_4level, get_blob_upload_5level, get_blob_upload_6level, get_blob_upload_7level)
    );
    app
}

#[cfg(test)]
mod tests {

    use axum::body::Body;
    use http_body_util::BodyExt;
    use hyper::Request;
    use reqwest::StatusCode;
    use test_temp_dir::test_temp_dir;
    use tower::{Service, ServiceExt};
    use uuid::Uuid;

    use super::*;
    use crate::registry::Digest;
    use crate::test_utilities::{self, resp_header};

    // POST blob upload
    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_post_blob_upload_create_new_upload() {
        let tmp_dir = test_temp_dir!();
        let (state, _) = test_utilities::trow_router(|_| {}, &tmp_dir).await;
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

    // POST followed by a single PUT
    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_put_blob_upload() {
        let tmp_dir = test_temp_dir!();
        let (_, mut router) = test_utilities::trow_router(|_| {}, &tmp_dir).await;
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

        let uuid = resp_header!(resp, test_utilities::UPLOAD_HEADER);
        let range = resp_header!(resp, test_utilities::RANGE_HEADER);
        let location = resp_header!(resp, test_utilities::LOCATION_HEADER);
        assert_eq!(range, "0-0"); // Haven't uploaded anything yet
        assert_eq!(
            location,
            format!("/v2/{}/blobs/uploads/{}", repo_name, uuid)
        );

        let blob = "super secret blob".as_bytes();
        let digest = Digest::digest_sha256_slice(blob);
        let loc = &format!("/v2/{}/blobs/uploads/{}?digest={}", repo_name, uuid, digest);

        let resp = router
            .call(Request::put(loc).body(Body::from(blob)).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let range = resp
            .headers()
            .get(test_utilities::RANGE_HEADER)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(range, format!("0-{}", (blob.len() - 1))); //note first byte is 0, hence len - 1
    }

    /// A single POST
    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_post_blob_upload_complete_upload() {
        let tmp_dir = test_temp_dir!();
        let (_, router) = test_utilities::trow_router(|_| {}, &tmp_dir).await;
        let repo_name = "test";

        let config = "{ }\n".as_bytes();
        let digest = Digest::digest_sha256_slice(config);

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

    // POST (skipped) then PATCH
    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_patch_blob_upload() {
        let tmp_dir = test_temp_dir!();
        let (state, _) = test_utilities::trow_router(|_| {}, &tmp_dir).await;
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
