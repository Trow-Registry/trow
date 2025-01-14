#![cfg(test)]

mod common;

mod interface_tests {
    use std::fs;
    use std::path::Path;
    use std::sync::Arc;

    use axum::body::Body;
    use axum::Router;
    use hyper::Request;
    use oci_spec::image::ImageManifest;
    use reqwest::StatusCode;
    use test_temp_dir::test_temp_dir;
    use tower::ServiceExt;
    use trow::registry::{manifest, RegistryProxiesConfig, SingleRegistryProxyConfig};
    use trow::TrowServerState;

    use crate::common;
    use crate::common::trow_router;

    async fn start_trow(data_dir: &Path) -> (Arc<TrowServerState>, Router) {
        let config_file = RegistryProxiesConfig {
            offline: false,
            registries: vec![
                SingleRegistryProxyConfig {
                    alias: "docker".to_string(),
                    host: "registry-1.docker.io".to_string(),
                    username: None,
                    password: None,
                    ignore_repos: vec![],
                },
                SingleRegistryProxyConfig {
                    alias: "nvcr".to_string(),
                    host: "nvcr.io".to_string(),
                    username: None,
                    password: None,
                    ignore_repos: vec![],
                },
                SingleRegistryProxyConfig {
                    alias: "quay".to_string(),
                    host: "quay.io".to_string(),
                    username: None,
                    password: None,
                    ignore_repos: vec![],
                },
            ],
        };

        trow_router(data_dir, |cfg| {
            cfg.proxy_registry_config = Some(config_file);
        })
        .await
    }

    async fn get_manifest(cl: &Router, name: &str, tag: &str) -> (manifest::OCIManifest, String) {
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
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "Could not get {}:{}",
            name,
            tag
        );
        let digest = resp
            .headers()
            .get("Docker-Content-Digest")
            .expect("No digest header")
            .to_str()
            .unwrap().to_owned();
        let manifest = common::response_body_json(resp).await;
        (manifest, digest)
    }

    async fn upload_to_nonwritable_repo(cl: &Router, name: &str) {
        let resp = cl
            .clone()
            .oneshot(
                Request::post(format!("/v2/{name}/blobs/uploads/"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("Error uploading layer");
        assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);

        //Try manifest
        let mani: ImageManifest = serde_json::from_str(
            r#"{
            "schemaVersion": 2,
            "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
            "config": {
                "mediaType": "application/vnd.docker.container.image.v1+json",
                "digest": "sha256:5f13f818131e80418214222144b621a5c663da7f898c3ff3434b424252b79dc0",
                "size": 0
            },
            "layers": [{
                "mediaType": "application/vnd.docker.image.rootfs.diff.tar.gzip",
                "digest": "sha256:997890bc85c5796408ceb20b0ca75dabe6fe868136e926d24ad0f36aa424f99d",
                "size": 0
            }]
        }"#,
        )
        .unwrap();
        let manifest_addr = format!("/v2/{}/manifests/{}", name, "tag");
        let resp = cl
            .clone()
            .oneshot(
                Request::put(&manifest_addr)
                    .body(Body::from(serde_json::to_vec(&mani).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_get_manifest_proxy_docker() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let trow = start_trow(data_dir).await.1;
        get_manifest(&trow, "f/docker/amouat/trow", "latest").await;
        get_manifest(&trow, "f/docker/amouat/trow", "latest").await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_get_manifest_proxy_nvcr() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let trow = start_trow(data_dir).await.1;
        get_manifest(&trow, "f/nvcr/nvidia/doca/doca_hbn", "5.1.0-doca1.3.0").await;
        get_manifest(&trow, "f/nvcr/nvidia/doca/doca_hbn", "5.1.0-doca1.3.0").await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_get_manifest_proxy_partial_cache() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let trow = start_trow(data_dir).await.1;
        // This should use same alpine image as base (so partially cached)
        get_manifest(&trow, "f/docker/library/alpine", "3.13.4").await;
        get_manifest(&trow, "f/docker/library/nginx", "1.20.0-alpine").await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_get_manifest_proxy_docker_library() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let (state, trow) = start_trow(data_dir).await;
        // Special case: docker/library
        // check that it works and that manifests are written in the correct location
        get_manifest(&trow, "f/docker/alpine", "3.13.4").await;
        let digest = sqlx::query_scalar!("SELECT manifest_digest FROM tag WHERE repo = 'f/docker/library/alpine' AND tag = '3.13.4'")
            .fetch_one(&mut *state.db.acquire().await.unwrap())
            .await
            .expect("Tag not found in database");
        let file = data_dir.join(format!("./blobs/{digest}"));
        assert!(file.exists());
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_get_manifest_proxy_multiplatform() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let trow = start_trow(data_dir).await.1;
        //Download an amd64 manifest, then the multi platform version of the same manifest
        get_manifest(
            &trow,
            "f/docker/hello-world",
            "sha256:f54a58bc1aac5ea1a25d796ae155dc228b3f0e11d046ae276b39c4bf2f13d8c4",
        )
        .await;
        get_manifest(&trow, "f/docker/hello-world", "linux").await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_get_manifest_proxy_no_auth() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let trow = start_trow(data_dir).await.1;
        // test a registry that doesn't require auth
        get_manifest(&trow, "f/quay/openshifttest/scratch", "latest").await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_get_manifest_proxy_update_latest_tag() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();
        let (state, trow) = start_trow(data_dir).await;

        // Check that tags get updated to point to latest digest
        let (man_3_13, digest_3_13) = get_manifest(&trow, "f/docker/alpine", "3.13.4").await;
        sqlx::query!("INSERT INTO tag (repo, tag, manifest_digest) VALUES ('f/docker/library/alpine', 'latest', $1)", digest_3_13)
            .execute(&mut *state.db.acquire().await.unwrap())
            .await
            .expect("Failed to insert tag");

        let (man_latest, digest_latest) = get_manifest(&trow, "f/docker/library/alpine", "latest").await;
        assert_ne!(digest_3_13, digest_latest, "Trow did not update digest of `latest` tag");
        assert_ne!(
            serde_json::to_string(&man_3_13).unwrap(),
            serde_json::to_string(&man_latest).unwrap(),
            "Trow did not update manifest of `latest` tag"
        );
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_upload_manifest_nonwritable_repo() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let trow = start_trow(data_dir).await.1;
        //test writing manifest to proxy dir isn't allowed
        upload_to_nonwritable_repo(&trow, "f/failthis").await;
    }
}
