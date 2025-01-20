#![cfg(test)]
mod common;

mod registry_interface {

    use axum::body::Body;
    use axum::http::HeaderValue;
    use axum::Router;
    use hyper::Request;
    use oci_spec::image::ImageManifest;
    use reqwest::StatusCode;
    use test_temp_dir::{test_temp_dir, TestTempDir};
    use tower::ServiceExt;
    use trow::registry::api_types::{HealthStatus, ReadyStatus};
    use trow::registry::digest;
    use trow::types::{RepoCatalog, TagList};

    use crate::common::{self, response_body_string, trow_router, DIST_API_HEADER};

    async fn start_trow(data_dir: &TestTempDir) -> Router {
        trow_router(data_dir.as_path_untracked(), |_| {}).await.1
    }

    async fn get_main(cl: &Router) {
        let resp = cl
            .clone()
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(DIST_API_HEADER),
            Some(&HeaderValue::from_static("registry/2.0"))
        );

        // All v2 registries should respond with a 200 to this
        let resp = cl
            .clone()
            .oneshot(Request::get("/v2/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get(DIST_API_HEADER).unwrap(), "registry/2.0");
    }

    async fn get_non_existent_blob(cl: &Router) {
        let resp = cl
            .clone()
            .oneshot(
                Request::get("/v2/test/test/blobs/sha256:baadf00dbaadf00dbaadf00dbaadf00d")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::NOT_FOUND,
            "resp: {}",
            response_body_string(resp).await
        );
    }

    async fn get_manifest(cl: &Router, name: &str, tag: &str, size: Option<usize>) {
        //Might need accept headers here
        let resp = cl
            .clone()
            .oneshot(
                Request::get(format!("/v2/{name}/manifests/{tag}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        //Just check header exists for minute
        resp.headers()
            .get("Docker-Content-Digest")
            .unwrap()
            .to_str()
            .unwrap();

        if let Some(s) = size {
            let actual_size = resp
                .headers()
                .get("Content-Length")
                .unwrap()
                .to_str()
                .unwrap();
            assert_eq!(actual_size, format!("{}", s));
        }
        let mani: ImageManifest = common::response_body_json(resp).await;

        assert_eq!(mani.schema_version(), 2);
    }

    async fn get_non_existent_manifest(cl: &Router, name: &str, tag: &str) {
        // Might need accept headers here
        let resp = cl
            .clone()
            .oneshot(
                Request::get(format!("/v2/{name}/manifests/{tag}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    async fn check_repo_catalog(cl: &Router, rc: &RepoCatalog) {
        let resp = cl
            .clone()
            .oneshot(
                Request::get("/v2/_catalog".to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = common::response_body_vec(resp).await;
        let rc_resp: RepoCatalog = serde_json::from_slice(&body).unwrap();
        assert_eq!(rc, &rc_resp);
    }

    async fn check_tag_list(cl: &Router, tl: &TagList) {
        let resp = cl
            .clone()
            .oneshot(
                Request::get(format!("/v2/{}/tags/list", tl.repo_name()))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body = common::response_body_vec(resp).await;
        let tl_resp: TagList = serde_json::from_slice(&body).unwrap();
        assert_eq!(tl, &tl_resp);
    }

    async fn check_tag_list_n_last(cl: &Router, n: u32, last: &str, tl: &TagList) {
        let resp = cl
            .clone()
            .oneshot(
                Request::get(format!(
                    "/v2/{}/tags/list?last={}&n={}",
                    tl.repo_name(),
                    last,
                    n
                ))
                .body(Body::empty())
                .unwrap(),
            )
            .await
            .unwrap();
        let tl_resp: TagList = common::response_body_json(resp).await;
        assert_eq!(tl, &tl_resp);
    }

    async fn upload_blob_with_put(cl: &Router, name: &str) {
        let resp = cl
            .clone()
            .oneshot(
                Request::post(format!("/v2/{name}/blobs/uploads/"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
        let uuid = resp
            .headers()
            .get(common::UPLOAD_HEADER)
            .unwrap()
            .to_str()
            .unwrap();

        let range = resp
            .headers()
            .get(common::RANGE_HEADER)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(range, "0-0"); // Haven't uploaded anything yet

        //used by oci_manifest_test
        let config = "{}\n".as_bytes();
        let digest = digest::Digest::digest_sha256_slice(config);
        let loc = &format!("/v2/{}/blobs/uploads/{}?digest={}", name, uuid, digest);

        let resp = cl
            .clone()
            .oneshot(Request::put(loc).body(Body::from(config)).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let range = resp
            .headers()
            .get(common::RANGE_HEADER)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(range, format!("0-{}", (config.len() - 1))); //note first byte is 0, hence len - 1
    }

    async fn upload_blob_with_post(cl: &Router, repo_name: &str) {
        let blob_content = "{ }\n".as_bytes();
        let digest = digest::Digest::digest_sha256_slice(blob_content);

        let resp = cl
            .clone()
            .oneshot(
                Request::post(format!(
                    "/v2/{}/blobs/uploads/?digest={}",
                    repo_name, digest
                ))
                .body(Body::from(blob_content))
                .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let range = resp
            .headers()
            .get(common::RANGE_HEADER)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(range, format!("0-{}", (blob_content.len() - 1))); //note first byte is 0, hence len - 1
    }

    async fn push_oci_manifest(cl: &Router, name: &str, tag: &str) -> String {
        //Note config was uploaded as blob in earlier test
        let config = "{ }\n".as_bytes();
        let config_digest = digest::Digest::digest_sha256_slice(config);

        let manifest = format!(
            r#"{{ "mediaType": "application/vnd.oci.image.manifest.v1+json",
                 "config": {{ "digest": "{}",
                             "mediaType": "application/vnd.oci.image.config.v1+json",
                             "size": {} }},
                 "layers": [], "schemaVersion": 2 }}"#,
            config_digest,
            config.len()
        );
        let bytes = manifest.clone();
        let resp = cl
            .clone()
            .oneshot(
                Request::put(format!("/v2/{}/manifests/{}", name, tag))
                    .body(Body::from(bytes))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        // Try pulling by digest
        let digest = digest::Digest::digest_sha256_slice(manifest.as_bytes());
        digest.to_string()
    }

    async fn push_manifest_list(cl: &Router, digest: &str, name: &str, tag: &str) -> String {
        let manifest = format!(
            r#"{{
                "schemaVersion": 2,
                "mediaType": "application/vnd.docker.distribution.manifest.list.v2+json",
                "manifests": [
                  {{
                    "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
                    "size": 7143,
                    "digest": "{}",
                    "platform": {{
                      "architecture": "ppc64le",
                      "os": "linux"
                    }}
                  }}
                ]
              }}
              "#,
            digest
        );
        let bytes = manifest.clone();
        let resp = cl
            .clone()
            .oneshot(
                Request::put(format!("/v2/{}/manifests/{}", name, tag))
                    .body(Body::from(bytes))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        // Try pulling by digest
        let digest = digest::Digest::digest_sha256_slice(manifest.as_bytes());
        digest.to_string()
    }

    async fn push_oci_manifest_with_foreign_blob(
        cl: &Router,
        repo_name: &str,
        tag: &str,
    ) -> String {
        //Note config was uploaded as blob in earlier test
        let config = "{ }\n".as_bytes();
        let config_digest = digest::Digest::digest_sha256_slice(config);

        upload_blob_with_post(cl, repo_name).await;

        let manifest = format!(
            r#"{{ "mediaType": "application/vnd.oci.image.manifest.v1+json",
                 "config": {{ "digest": "{}",
                             "mediaType": "application/vnd.oci.image.config.v1+json",
                             "size": {} }},
                 "layers": [
                    {{
                              "mediaType": "application/vnd.docker.image.rootfs.foreign.diff.tar.gzip",
                              "size": 1612893008,
                              "digest": "sha256:9038b92872bc268d5c975e84dd94e69848564b222ad116ee652c62e0c2f894b2",
                              "urls": [
                                 "https://mcr.microsoft.com/v2/windows/servercore/blobs/sha256:9038b92872bc268d5c975e84dd94e69848564b222ad116ee652c62e0c2f894b2"
                              ]
                           }}
                 ], "schemaVersion": 2 }}"#,
            config_digest,
            config.len()
        );
        let bytes = manifest.clone();
        let resp = cl
            .clone()
            .oneshot(
                Request::put(format!("/v2/{}/manifests/{}", repo_name, tag))
                    .body(Body::from(bytes))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        // Try pulling by digest
        let digest = digest::Digest::digest_sha256_slice(manifest.as_bytes());
        digest.to_string()
    }

    async fn delete_manifest(cl: &Router, repo: &str, reference: &str) {
        let resp = cl
            .clone()
            .oneshot(
                Request::delete(format!("/v2/{}/manifests/{}", repo, reference))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
    }

    async fn delete_blob(cl: &Router, repo: &str, digest: &str) {
        let resp = cl
            .clone()
            .oneshot(
                Request::delete(format!("/v2/{}/blobs/{}", repo, digest))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
    }

    async fn delete_non_existent_manifest(cl: &Router, name: &str) {
        let resp = cl
            .clone()
            .oneshot(
                Request::delete(format!(
                    "/v2/{}/manifests/{}",
                    name, "sha256:9038b92872bc268d5c975e84dd94e69848564b222ad116ee652c62e0c2f894b2"
                ))
                .body(Body::empty())
                .unwrap(),
            )
            .await
            .unwrap();
        // If it doesn't exist, that's kinda the same as deleted, right?
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_get_non_existent_blob() {
        let tmp_dir = test_temp_dir!();
        let trow = start_trow(&tmp_dir).await;
        get_non_existent_blob(&trow).await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn upload_image() {
        let tmp_dir = test_temp_dir!();
        let trow = start_trow(&tmp_dir).await;
        common::upload_fake_image(&trow, "fifth/fourth/repo/image/test", "tag").await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn blob_upload_with_put() {
        let tmp_dir = test_temp_dir!();
        let trow = start_trow(&tmp_dir).await;
        upload_blob_with_put(&trow, "puttest").await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn blob_upload_with_post() {
        let tmp_dir = test_temp_dir!();
        let trow = start_trow(&tmp_dir).await;
        upload_blob_with_post(&trow, "posttest").await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_push_oci_manifest() {
        let tmp_dir = test_temp_dir!();
        let trow = start_trow(&tmp_dir).await;
        upload_blob_with_post(&trow, "puttest").await;
        push_oci_manifest(&trow, "puttest", "puttest1").await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_push_manifest_list() {
        let tmp_dir = test_temp_dir!();
        let trow = start_trow(&tmp_dir).await;
        upload_blob_with_post(&trow, "listtest").await;
        let digest = push_oci_manifest(&trow, "listtest", "noooo").await;
        push_manifest_list(&trow, &digest, "listtest", "listtest1").await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_get_manifest() {
        let tmp_dir = test_temp_dir!();
        let trow = start_trow(&tmp_dir).await;
        common::upload_fake_image(&trow, "get/manifest", "1").await;
        get_manifest(&trow, "get/manifest", "1", Some(570)).await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_delete_manifest_digest() {
        let tmp_dir = test_temp_dir!();
        let trow = start_trow(&tmp_dir).await;
        let (_, man_digest) = common::upload_fake_image(&trow, "delete/manifest", "1").await;
        delete_manifest(&trow, "delete/manifest", man_digest.as_str()).await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_delete_non_existent_manifest() {
        let tmp_dir = test_temp_dir!();
        let trow = start_trow(&tmp_dir).await;
        delete_non_existent_manifest(&trow, "delete/nonexistent").await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_delete_by_tag() {
        let tmp_dir = test_temp_dir!();
        let trow = start_trow(&tmp_dir).await;
        common::upload_fake_image(&trow, "delete/tag", "tag").await;
        delete_manifest(&trow, "delete/tag", "tag").await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_get_non_existent_manifest() {
        let tmp_dir = test_temp_dir!();
        let trow = start_trow(&tmp_dir).await;
        get_non_existent_manifest(&trow, "nonexistent", "nonexistent").await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_push_oci_manifest_with_foreign_blob() {
        let tmp_dir = test_temp_dir!();
        let trow = start_trow(&tmp_dir).await;
        let digest = push_oci_manifest_with_foreign_blob(&trow, "foreigntest", "blobtest1").await;
        delete_manifest(&trow, "foreigntest", &digest).await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_root_routes() {
        let tmp_dir = test_temp_dir!();
        let trow = start_trow(&tmp_dir).await;
        get_main(&trow).await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_catalog() {
        let tmp_dir = test_temp_dir!();
        let trow = start_trow(&tmp_dir).await;
        let repos = [
            "fifth/fourth/repo/image/test",
            "fourth/repo/image/test",
            "repo/image/test",
            "image/test",
            "onename",
        ];
        let mut rc = RepoCatalog::new();
        for r in repos.iter() {
            common::upload_fake_image(&trow, r, "tag").await;
            rc.insert(r.to_string());
        }

        common::upload_fake_image(&trow, "todeletetag", "1").await;
        delete_manifest(&trow, "todeletetag", "1").await;
        rc.insert("todeletetag".to_string());

        let (blob_digest, man_digest) =
            common::upload_fake_image(&trow, "todeletedigest", "1").await;
        delete_manifest(&trow, "todeletedigest", man_digest.as_str()).await;
        delete_blob(&trow, "todeletedigest", blob_digest.as_str()).await;

        check_repo_catalog(&trow, &rc).await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_tag_list() {
        let tmp_dir = test_temp_dir!();
        let trow = start_trow(&tmp_dir).await;

        common::upload_fake_image(&trow, "taglist", "tag").await;
        common::upload_fake_image(&trow, "taglist", "latest").await;
        common::upload_fake_image(&trow, "taglist", "tag").await;
        common::upload_fake_image(&trow, "taglist", "three").await;

        let mut tl = TagList::new("taglist".to_string());
        tl.insert("latest".to_string());
        tl.insert("tag".to_string());
        tl.insert("three".to_string());
        check_tag_list(&trow, &tl).await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_tag_list_n_last() {
        let tmp_dir = test_temp_dir!();
        let trow = start_trow(&tmp_dir).await;

        common::upload_fake_image(&trow, "taglistnlast", "tag").await;
        common::upload_fake_image(&trow, "taglistnlast", "latest").await;
        common::upload_fake_image(&trow, "taglistnlast", "tag").await;
        common::upload_fake_image(&trow, "taglistnlast", "three").await;

        let mut tl = TagList::new("taglistnlast".to_string());
        tl.insert("latest".to_string());
        tl.insert("tag".to_string());
        check_tag_list_n_last(&trow, 2, "", &tl).await;

        let mut tl2 = TagList::new("taglistnlast".to_string());
        tl2.insert("tag".to_string());
        tl2.insert("three".to_string());
        check_tag_list_n_last(&trow, 2, "latest", &tl2).await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_get_readiness() {
        let tmp_dir = test_temp_dir!();
        let trow = start_trow(&tmp_dir).await;

        let resp = trow
            .clone()
            .oneshot(Request::get("/readiness").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let rr: ReadyStatus = common::response_body_json(resp).await;

        assert!(rr.is_ready);
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_get_health() {
        let tmp_dir = test_temp_dir!();
        let trow = start_trow(&tmp_dir).await;

        let resp = trow
            .clone()
            .oneshot(Request::get("/healthz").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let hr: HealthStatus = common::response_body_json(resp).await;

        assert!(hr.is_healthy);
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_head_manifest_tag() {
        let tmp_dir = test_temp_dir!();
        let trow = start_trow(&tmp_dir).await;

        common::upload_fake_image(&trow, "headtest", "headtest1").await;
        let resp = trow
            .clone()
            .oneshot(
                Request::head("/v2/headtest/manifests/headtest1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let digest1 = resp
            .headers()
            .get("Docker-Content-Digest")
            .unwrap()
            .to_str()
            .unwrap();
        common::upload_fake_image(&trow, "headtest", "headtest2").await;
        let resp = trow
            .clone()
            .oneshot(
                Request::head("/v2/headtest/manifests/headtest1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let digest1_bis = resp
            .headers()
            .get("Docker-Content-Digest")
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(digest1, digest1_bis);
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_patch_with_first_chunk_should_return_202() {
        let tmp_dir = test_temp_dir!();
        let trow = start_trow(&tmp_dir).await;

        let test_blob_chunk = "chunk1".as_bytes();
        let resp = trow
            .clone()
            .oneshot(
                Request::post("/v2/patchtest/blobs/uploads/")
                    .header("Content-Length", "0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let loc = resp.headers().get("Location").unwrap().to_str().unwrap();
        let resp = trow
            .clone()
            .oneshot(
                Request::patch(loc)
                    .header("Content-Type", "application/octet-stream")
                    .header("Content-Length", "6")
                    .header("Content-Range", "0-5")
                    .body(Body::from(test_blob_chunk))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
        assert_eq!(
            resp.headers().get("Range").unwrap().to_str().unwrap(),
            "0-5"
        );
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_get_blob_upload() {
        let tmp_dir = test_temp_dir!();
        let trow = start_trow(&tmp_dir).await;

        let resp = trow
            .clone()
            .oneshot(
                Request::post("/v2/patchtest/blobs/uploads/")
                    .header("Content-Length", "0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let loc = resp.headers().get("Location").unwrap().to_str().unwrap();

        let resp = trow
            .clone()
            .oneshot(Request::get(loc).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        // ???
        // assert_eq!(
        //     resp.headers().get("Range").unwrap().to_str().unwrap(),
        //     "0-5"
        // );
    }
}
