#[cfg(test)]
mod common;

#[cfg(test)]
mod interface_tests {

    use std::io::BufReader;
    use std::path::Path;

    use axum::body::Body;
    use axum::Router;
    use hyper::Request;
    use reqwest::StatusCode;
    use test_temp_dir::test_temp_dir;
    use tower::ServiceExt;
    use trow::registry_interface::digest;
    use trow::trow_server::api_types::{HealthStatus, ReadyStatus};
    use trow::trow_server::manifest;
    use trow::types::{RepoCatalog, TagList};

    use crate::common;
    use crate::common::DIST_API_HEADER;

    async fn start_trow(data_dir: &Path) -> Router {
        let mut trow_builder = trow::TrowConfig::new();
        data_dir.clone_into(&mut trow_builder.data_dir);
        trow_builder.build_app().await.unwrap()
    }

    async fn get_main(cl: &Router) {
        let resp = cl
            .clone()
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get(DIST_API_HEADER).unwrap(), "registry/2.0");

        //All v2 registries should respond with a 200 to this
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
                Request::get("/v2/test/test/blobs/sha256:baadf00d")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    async fn get_manifest(cl: &Router, name: &str, tag: &str, size: Option<usize>) {
        //Might need accept headers here
        let resp = cl
            .clone()
            .oneshot(
                Request::get(&format!("/v2/{name}/manifests/{tag}"))
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
        let mani: manifest::OCIManifestV2 = common::response_body_json(resp).await;

        assert_eq!(mani.schema_version, 2);
    }

    async fn get_non_existent_manifest(cl: &Router, name: &str, tag: &str) {
        // Might need accept headers here
        let resp = cl
            .clone()
            .oneshot(
                Request::get(&format!("/v2/{name}/manifests/{tag}"))
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
                Request::get(&"/v2/_catalog".to_string())
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
                Request::get(&format!("/v2/{}/tags/list", tl.repo_name()))
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
                Request::get(&format!(
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

    async fn upload_with_put(cl: &Router, name: &str) {
        let resp = cl
            .clone()
            .oneshot(
                Request::post(&format!("/v2/{name}/blobs/uploads/"))
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
        let digest = digest::Digest::try_sha256(BufReader::new(config)).unwrap();
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

    async fn upload_with_post(cl: &Router, name: &str) {
        let config = "{ }\n".as_bytes();
        let digest = digest::Digest::try_sha256(BufReader::new(config)).unwrap();

        let resp = cl
            .clone()
            .oneshot(
                Request::post(&format!("/v2/{}/blobs/uploads/?digest={}", name, digest))
                    .body(Body::from(config))
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
        assert_eq!(range, format!("0-{}", (config.len() - 1))); //note first byte is 0, hence len - 1
    }

    async fn push_oci_manifest(cl: &Router, name: &str, tag: &str) -> String {
        //Note config was uploaded as blob in earlier test
        let config = "{}\n".as_bytes();
        let config_digest = digest::Digest::try_sha256(BufReader::new(config)).unwrap();

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
                Request::put(&format!("/v2/{}/manifests/{}", name, tag))
                    .body(Body::from(bytes))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        // Try pulling by digest
        let digest = digest::Digest::try_sha256(BufReader::new(manifest.as_bytes())).unwrap();
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
                Request::put(&format!("/v2/{}/manifests/{}", name, tag))
                    .body(Body::from(bytes))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        // Try pulling by digest
        let digest = digest::Digest::try_sha256(BufReader::new(manifest.as_bytes())).unwrap();
        digest.to_string()
    }

    async fn push_oci_manifest_with_foreign_blob(cl: &Router, name: &str, tag: &str) -> String {
        //Note config was uploaded as blob in earlier test
        let config = "{}\n".as_bytes();
        let config_digest = digest::Digest::try_sha256(BufReader::new(config)).unwrap();

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
                Request::put(&format!("/v2/{}/manifests/{}", name, tag))
                    .body(Body::from(bytes))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        // Try pulling by digest
        let digest = digest::Digest::try_sha256(BufReader::new(manifest.as_bytes())).unwrap();
        digest.to_string()
    }

    async fn delete_manifest(cl: &Router, name: &str, digest: &str) {
        let resp = cl
            .clone()
            .oneshot(
                Request::delete(&format!("/v2/{}/manifests/{}", name, digest))
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
                Request::delete(&format!(
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
    async fn attempt_delete_by_tag(cl: &Router, name: &str, tag: &str) {
        let resp = cl
            .clone()
            .oneshot(
                Request::delete(&format!("/v2/{}/manifests/{}", name, tag))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    async fn delete_config_blob(cl: &Router, name: &str) {
        //Deletes blob uploaded in config test
        let config = "{}\n".as_bytes();
        let config_digest = digest::Digest::try_sha256(BufReader::new(config)).unwrap();
        let resp = cl
            .clone()
            .oneshot(
                Request::delete(&format!("/v2/{name}/blobs/{config_digest}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
    }

    async fn get_health(cl: &Router) {
        let resp = cl
            .clone()
            .oneshot(Request::get("/healthz").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let hr: HealthStatus = common::response_body_json(resp).await;

        assert!(hr.is_healthy);
    }

    async fn get_readiness(cl: &Router) {
        let resp = cl
            .clone()
            .oneshot(Request::get("/readiness").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let rr: ReadyStatus = common::response_body_json(resp).await;

        assert!(rr.is_ready);
    }

    async fn get_metrics(cl: &Router) {
        let resp = cl
            .clone()
            .oneshot(Request::get("/metrics").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let body = String::from_utf8(common::response_body_vec(resp).await).unwrap();

        assert!(body.contains("available_space"));
        assert!(body.contains("free_space"));
        assert!(body.contains("total_space"));

        // assert!(body.contains("total_manifest_requests{type=\"manifests\"} 6"));
        // assert!(body.contains("total_blob_requests{type=\"blobs\"} 9"));

        // get_manifest(cl, "onename", "tag", None).await;
        // let manifest_response = cl
        //     .clone()
        //     .oneshot(
        //         Request::get(&format!("{}/metrics", ORIGIN))
        //             .body(Body::empty())
        //             .unwrap(),
        //     )
        //     .await
        //     .unwrap();

        // let manifest_body =
        //     String::from_utf8(common::response_body_vec(manifest_response).await).unwrap();

        // assert!(manifest_body.contains("total_manifest_requests{type=\"manifests\"} 7"));

        // get_non_existent_blob(cl).await;
        // let blob_response = cl
        //     .clone()
        //     .oneshot(
        //         Request::get(&format!("{}/metrics", ORIGIN))
        //             .body(Body::empty())
        //             .unwrap(),
        //     )
        //     .await
        //     .unwrap();

        // assert_eq!(blob_response.status(), StatusCode::OK);

        // let blob_body = common::response_body_string(blob_response).await;

        // assert!(blob_body.contains("total_blob_requests{type=\"blobs\"} 10"));
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_runner() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let trow = start_trow(data_dir).await;

        println!("Running get_main()");
        get_main(&trow).await;
        println!("Running get_blob()");
        get_non_existent_blob(&trow).await;

        println!("Running upload_layer(fifth/fourth/repo/image/test:tag)");
        common::upload_layer(&trow, "fifth/fourth/repo/image/test", "tag").await;
        println!("Running upload_layer(fourth/repo/image/test:tag)");
        common::upload_layer(&trow, "fourth/repo/image/test", "tag").await;
        println!("Running upload_layer(repo/image/test:tag)");
        common::upload_layer(&trow, "repo/image/test", "tag").await;
        println!("Running upload_layer(image/test:latest)");
        common::upload_layer(&trow, "image/test", "latest").await;
        println!("Running upload_layer(onename:tag)");
        common::upload_layer(&trow, "onename", "tag").await;
        println!("Running upload_layer(onename:latest)");
        common::upload_layer(&trow, "onename", "latest").await;
        println!("Running upload_with_put()");
        upload_with_put(&trow, "puttest").await;
        println!("Running upload_with_post");
        upload_with_post(&trow, "posttest").await;
        println!("Running push_oci_manifest()");
        let manifest_digest = push_oci_manifest(&trow, "puttest", "puttest1").await;
        println!("Running push_manifest_list()");
        let digest_manifest_list =
            push_manifest_list(&trow, &manifest_digest, "listtest", "listtest1").await;
        println!("Running get_manifest(puttest:puttest1)");
        get_manifest(&trow, "puttest", "puttest1", Some(354)).await;
        println!("Running get_manifest(puttest:digest)");
        get_manifest(&trow, "puttest", &manifest_digest, Some(354)).await;
        println!("Running delete_manifest(puttest:digest)");
        delete_manifest(&trow, "puttest", &manifest_digest).await;
        println!("Running delete_manifest(listtest)");
        delete_manifest(&trow, "listtest", &digest_manifest_list).await;
        println!("Running delete_non_existent_manifest(onename)");
        delete_non_existent_manifest(&trow, "onename").await;
        println!("Running attempt_delete_by_tag(onename:tag)");
        attempt_delete_by_tag(&trow, "onename", "tag").await;
        println!("Running get_non_existent_manifest(puttest:puttest1)");
        get_non_existent_manifest(&trow, "puttest", "puttest1").await;

        println!("Running push_oci_manifest_with_foreign_blob()");
        let digest = push_oci_manifest_with_foreign_blob(&trow, "foreigntest", "blobtest1").await;
        delete_manifest(&trow, "foreigntest", &digest).await;

        println!("Running delete_config_blob");
        delete_config_blob(&trow, "puttest").await;

        println!("Running get_manifest(onename:tag)");
        get_manifest(&trow, "onename", "tag", None).await;
        println!("Running get_manifest(image/test:latest)");
        get_manifest(&trow, "image/test", "latest", None).await;
        println!("Running get_manifest(repo/image/test:tag)");
        get_manifest(&trow, "repo/image/test", "tag", None).await;

        let mut rc = RepoCatalog::new();
        rc.insert("fifth/fourth/repo/image/test".to_string());
        rc.insert("fourth/repo/image/test".to_string());
        rc.insert("repo/image/test".to_string());
        rc.insert("image/test".to_string());
        rc.insert("onename".to_string());

        println!("Running check_repo_catalog");
        check_repo_catalog(&trow, &rc).await;

        let mut tl = TagList::new("repo/image/test".to_string());
        tl.insert("tag".to_string());
        println!("Running check_tag_list 1");
        check_tag_list(&trow, &tl).await;

        common::upload_layer(&trow, "onename", "three").await;
        common::upload_layer(&trow, "onename", "four").await;

        // list, in order should be [four, latest, tag, three]
        let mut tl2 = TagList::new("onename".to_string());
        tl2.insert("four".to_string());
        tl2.insert("latest".to_string());
        tl2.insert("tag".to_string());
        tl2.insert("three".to_string());
        println!("Running check_tag_list 2");
        check_tag_list(&trow, &tl2).await;

        let mut tl3 = TagList::new("onename".to_string());
        tl3.insert("four".to_string());
        tl3.insert("latest".to_string());
        println!("Running check_tag_list_n_last 3");
        check_tag_list_n_last(&trow, 2, "", &tl3).await;

        let mut tl4 = TagList::new("onename".to_string());
        tl4.insert("tag".to_string());
        tl4.insert("three".to_string());
        println!("Running check_tag_list_n_last 4");
        check_tag_list_n_last(&trow, 2, "latest", &tl4).await;

        println!("Running get_readiness");
        get_readiness(&trow).await;

        println!("Running get_health");
        get_health(&trow).await;

        println!("Running get_metrics");
        get_metrics(&trow).await;
        check_tag_list_n_last(&trow, 2, "latest", &tl4).await;
    }
}
