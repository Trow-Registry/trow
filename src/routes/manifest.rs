use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::routing::get;

use super::extracts::AlwaysHost;
use super::macros::endpoint_fn_7_levels;
use super::response::OciJson;
use crate::TrowServerState;
use crate::routes::extracts::ImageNamespace;
use crate::routes::macros::route_7_levels;
use crate::routes::response::errors::Error;
use crate::routes::response::trow_token::TrowToken;
use crate::types::{ManifestDeleted, VerifiedManifest};
use crate::utils::manifest::OCIManifest;

async fn get_manifest(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((repo, raw_reference)): Path<(String, String)>,
    Query(query): Query<ImageNamespace>,
) -> Result<OciJson<OCIManifest>, Error> {
    let payload = state
        .services
        .manifest
        .get_manifest(repo, raw_reference, query.ns.as_deref())
        .await?;
    Ok(OciJson::new_raw(payload.bytes)
        .set_digest(payload.digest)
        .set_content_type(&payload.content_type))
}

endpoint_fn_7_levels!(
    get_manifest(
        auth_user: TrowToken,
        state: State<Arc<TrowServerState>>;
        path: [image_name, reference: String],
        ns: Query<ImageNamespace>
    ) -> Result<OciJson<OCIManifest>, Error>
);

async fn put_image_manifest(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    AlwaysHost(host): AlwaysHost,
    Path((repo_name, reference)): Path<(String, String)>,
    body: Body,
) -> Result<VerifiedManifest, Error> {
    Ok(state
        .services
        .manifest
        .put_manifest(repo_name, reference, host, body)
        .await?)
}
endpoint_fn_7_levels!(
    put_image_manifest(
        auth_user: TrowToken,
        state: State<Arc<TrowServerState>>,
        host: AlwaysHost;
        path: [image_name, reference: String],
        chunk: Body
    ) -> Result<VerifiedManifest, Error>
);

async fn delete_image_manifest(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((repo, reference)): Path<(String, String)>,
) -> Result<ManifestDeleted, Error> {
    Ok(state
        .services
        .manifest
        .delete_manifest(repo, reference)
        .await?)
}
endpoint_fn_7_levels!(
    delete_image_manifest(
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>;
    path: [image_name, digest: String]
    ) -> Result<ManifestDeleted, Error>
);

pub fn route(mut app: Router<Arc<TrowServerState>>) -> Router<Arc<TrowServerState>> {
    #[rustfmt::skip]
    route_7_levels!(
        app,
        "/v2" "/manifests/{reference}",
        get(get_manifest, get_manifest_2level, get_manifest_3level, get_manifest_4level, get_manifest_5level, get_manifest_6level, get_manifest_7level),
        put(put_image_manifest, put_image_manifest_2level, put_image_manifest_3level, put_image_manifest_4level, put_image_manifest_5level, put_image_manifest_6level, put_image_manifest_7level),
        delete(delete_image_manifest, delete_image_manifest_2level, delete_image_manifest_3level, delete_image_manifest_4level, delete_image_manifest_5level, delete_image_manifest_6level, delete_image_manifest_7level)
    );
    app
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use axum::body::Body;
    use hyper::{Request, StatusCode};
    use oci_spec::image::{Descriptor, ImageManifestBuilder, MediaType};
    use test_temp_dir::test_temp_dir;
    use tower::ServiceExt;

    use crate::test_utilities;
    use crate::utils::digest::Digest;

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn test_delete_manifest() {
        let tmp_dir = test_temp_dir!();
        let (state, router) = test_utilities::trow_router(|_| {}, &tmp_dir).await;

        let dummy_blob_digest =
            "sha256:44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a";
        let manifest = serde_json::to_vec(
            &ImageManifestBuilder::default()
                .schema_version(2u32)
                .layers([])
                .media_type(MediaType::ImageManifest)
                .config(Descriptor::new(
                    MediaType::EmptyJSON,
                    2,
                    oci_spec::image::Digest::from_str(dummy_blob_digest).unwrap(),
                ))
                .build()
                .unwrap(),
        )
        .unwrap();
        let manifest_digest = Digest::digest_sha256_slice(&manifest);
        let manifest_digest_str = manifest_digest.as_str();
        sqlx::query!(
            "INSERT INTO manifest (digest, json, blob) VALUES ($1, jsonb($2), $2)",
            manifest_digest_str,
            manifest,
        )
        .execute(state.services.repos().db_rw())
        .await
        .unwrap();
        for repo in ["test1", "test2"] {
            sqlx::query!(
                r#"INSERT INTO repo_blob_assoc (repo_name, manifest_digest) VALUES ($1, $2)"#,
                repo,
                manifest_digest_str
            )
            .execute(state.services.repos().db_rw())
            .await
            .unwrap();
        }

        let resp = router
            .clone()
            .oneshot(
                Request::delete(format!("/v2/test1/manifests/{manifest_digest_str}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::ACCEPTED,
            "unexpected status code: {resp:?}"
        );
        let res = sqlx::query!(
            r#"SELECT COUNT(*) as "count!" FROM manifest WHERE digest = $1"#,
            manifest_digest_str
        )
        .fetch_one(state.services.repos().db_ro())
        .await
        .unwrap();
        assert_eq!(res.count, 1, "manifest was deleted unexpectedly");

        let resp = router
            .oneshot(
                Request::delete(format!("/v2/test2/manifests/{manifest_digest_str}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::ACCEPTED,
            "unexpected status code: {resp:?}"
        );
        let res = sqlx::query!(
            r#"SELECT COUNT(*) as "count!" FROM manifest WHERE digest = $1"#,
            manifest_digest_str
        )
        .fetch_one(state.services.repos().db_ro())
        .await
        .unwrap();
        assert_eq!(res.count, 0, "manifest was not deleted");
    }

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn test_get_manifest_no_mediatype() {
        let tmp_dir = test_temp_dir!();
        let (state, router) = test_utilities::trow_router(|_| {}, &tmp_dir).await;

        let manifest_json = r#"{"schemaVersion":2,"manifests":[{"mediaType":"application/vnd.oci.image.manifest.v1+json","digest":"sha256:9606ce5d502876100782b78351910efb2008d738e438ebc708fa4fabf5c01e9b","size":3317,"platform":{"architecture":"amd64","os":"linux"}},{"mediaType":"application/vnd.oci.image.manifest.v1+json","digest":"sha256:97530f8d15b87fa5e5a42687371de32b3919c49c53e16366428972eeb64735f6","size":3317,"platform":{"architecture":"arm64","os":"linux"}}]}"#;
        let manifest_bytes = manifest_json.as_bytes();
        let manifest_digest = Digest::digest_sha256_slice(manifest_bytes);
        let manifest_digest_str = manifest_digest.as_str();

        sqlx::query!(
            "INSERT INTO manifest (digest, json, blob) VALUES ($1, jsonb($2), $2)",
            manifest_digest_str,
            manifest_bytes,
        )
        .execute(state.services.repos().db_rw())
        .await
        .unwrap();

        let repo_name = "test-repo";
        sqlx::query!(
            r#"INSERT INTO repo_blob_assoc (repo_name, manifest_digest) VALUES ($1, $2)"#,
            repo_name,
            manifest_digest_str
        )
        .execute(state.services.repos().db_rw())
        .await
        .unwrap();

        let resp = router
            .oneshot(
                Request::get(format!("/v2/{repo_name}/manifests/{manifest_digest_str}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "unexpected status code: {resp:?}"
        );

        let content_type = resp
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(
            content_type, "application/vnd.oci.image.index.v1+json",
            "unexpected content-type"
        );
    }

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn test_head_nonexistent_manifest() {
        let tmp_dir = test_temp_dir!();
        let (_, router) = test_utilities::trow_router(|_| {}, &tmp_dir).await;

        let repo_name = "test-repo";
        let non_existent_digest = "sha256:nonexistentdigest";

        let resp = router
            .oneshot(
                Request::head(format!("/v2/{repo_name}/manifests/{non_existent_digest}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            resp.status(),
            StatusCode::NOT_FOUND,
            "unexpected status code: {resp:?}"
        );
    }
}
