#[cfg(test)]
mod common;

#[cfg(test)]
mod interface_tests {
    use std::fs;
    use std::path::Path;

    use axum::body::Body;
    use axum::Router;
    use hyper::Request;
    use reqwest::StatusCode;
    use test_temp_dir::test_temp_dir;
    use tower::ServiceExt;
    use trow::trow_server::{manifest, RegistryProxiesConfig, SingleRegistryProxyConfig};

    use crate::common;

    async fn start_trow(data_dir: &Path) -> Router {
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

        let mut trow_builder = trow::TrowConfig::new();
        trow_builder.proxy_registry_config = Some(config_file);
        data_dir.clone_into(&mut trow_builder.data_dir);
        trow_builder.build_app().await.unwrap()
    }

    async fn get_manifest(cl: &Router, name: &str, tag: &str) -> manifest::OCIManifest {
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
        common::response_body_json(resp).await
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
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        //Try manifest
        let config = manifest::Object {
            media_type: "application/vnd.docker.container.image.v1+json".to_owned(),
            digest: "fake".to_string(),
            size: None,
        };
        let layer = manifest::Object {
            media_type: "application/vnd.docker.image.rootfs.diff.tar.gzip".to_owned(),
            size: None,
            digest: "fake".to_string(),
        };

        let layers = vec![layer];
        let mani = manifest::OCIManifestV2 {
            schema_version: 2,
            media_type: Some("application/vnd.docker.distribution.manifest.v2+json".to_owned()),
            config,
            layers,
        };
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
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_get_manifest_proxy_docker() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let trow = start_trow(data_dir).await;
        get_manifest(&trow, "f/docker/amouat/trow", "latest").await;
        get_manifest(&trow, "f/docker/amouat/trow", "latest").await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_get_manifest_proxy_nvcr() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let trow = start_trow(data_dir).await;
        get_manifest(&trow, "f/nvcr/nvidia/doca/doca_hbn", "5.1.0-doca1.3.0").await;
        get_manifest(&trow, "f/nvcr/nvidia/doca/doca_hbn", "5.1.0-doca1.3.0").await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_get_manifest_proxy_partial_cache() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let trow = start_trow(data_dir).await;
        // This should use same alpine image as base (so partially cached)
        get_manifest(&trow, "f/docker/library/alpine", "3.13.4").await;
        get_manifest(&trow, "f/docker/library/nginx", "1.20.0-alpine").await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_get_manifest_proxy_docker_library() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let trow = start_trow(data_dir).await;
        // Special case: docker/library
        // check that it works and that manifests are written in the correct location
        get_manifest(&trow, "f/docker/alpine", "3.13.4").await;
        assert!(!data_dir.join("./manifests/f/docker/alpine/3.13.4").exists());
        assert!(data_dir
            .join("./manifests/f/docker/library/alpine/3.13.4")
            .exists());
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_get_manifest_proxy_multiplatform() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let trow = start_trow(data_dir).await;
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

        let trow = start_trow(data_dir).await;
        // test a registry that doesn't require auth
        get_manifest(&trow, "f/quay/openshifttest/scratch", "latest").await;
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_get_manifest_proxy_update_latest_tag() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let trow = start_trow(data_dir).await;
        // Check that tags get updated to point to latest digest
        {
            let man_3_13 = get_manifest(&trow, "f/docker/alpine", "3.13.4").await;
            fs::copy(
                data_dir.join("./manifests/f/docker/library/alpine/3.13.4"),
                data_dir.join("./manifests/f/docker/library/alpine/latest"),
            )
            .unwrap();
            let man_latest = get_manifest(&trow, "f/docker/library/alpine", "latest").await;
            assert_ne!(
                serde_json::to_string(&man_3_13).unwrap(),
                serde_json::to_string(&man_latest).unwrap(),
                "Trow did not update digest of `latest` tag"
            );
        }
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_upload_manifest_nonwritable_repo() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let trow = start_trow(data_dir).await;
        //test writing manifest to proxy dir isn't allowed
        upload_to_nonwritable_repo(&trow, "f/failthis").await;
    }
}
