#[cfg(test)]
mod common;

#[cfg(test)]
mod interface_tests {
    use std::path::Path;
    use std::process::{Child, Command};
    use std::time::Duration;
    use std::{fs, thread};

    use environment::Environment;
    use reqwest::StatusCode;
    use trow_server::{manifest, RegistryProxiesConfig, SingleRegistryProxyConfig};

    use crate::common;

    const PORT: &str = "39369";
    const ORIGIN: &str = "http://127.0.0.1:39369";

    struct TrowInstance {
        pid: Child,
    }

    async fn start_trow() -> TrowInstance {
        let config_file = common::get_file(RegistryProxiesConfig {
            offline: false,
            registries: vec![
                SingleRegistryProxyConfig {
                    alias: "docker".to_string(),
                    host: "registry-1.docker.io".to_string(),
                    username: None,
                    password: None,
                },
                SingleRegistryProxyConfig {
                    alias: "nvcr".to_string(),
                    host: "nvcr.io".to_string(),
                    username: None,
                    password: None,
                },
                SingleRegistryProxyConfig {
                    alias: "quay".to_string(),
                    host: "quay.io".to_string(),
                    username: None,
                    password: None,
                },
            ],
        });

        let mut child = Command::new("cargo")
            .arg("run")
            .env_clear()
            .envs(Environment::inherit().compile())
            .env("RUST_LOG", "info")
            .arg("--")
            .arg("--proxy-registry-config-file")
            .arg(config_file.path())
            .arg("--port")
            .arg(PORT)
            .spawn()
            .expect("failed to start");

        let mut timeout = 600;

        let client = reqwest::Client::new();

        let mut response = client.get(ORIGIN).send().await;
        while timeout > 0 && (response.is_err() || (response.unwrap().status() != StatusCode::OK)) {
            thread::sleep(Duration::from_millis(100));
            response = client.get(ORIGIN).send().await;
            timeout -= 1;
        }
        if timeout == 0 {
            child.kill().unwrap();
            panic!("Failed to start Trow");
        }
        TrowInstance { pid: child }
    }

    impl Drop for TrowInstance {
        fn drop(&mut self) {
            common::kill_gracefully(&self.pid);
        }
    }

    async fn get_manifest(cl: &reqwest::Client, name: &str, tag: &str) -> manifest::Manifest {
        //Might need accept headers here
        let resp = cl
            .get(&format!("{}/v2/{}/manifests/{}", ORIGIN, name, tag))
            .send()
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "Could not get {}:{}",
            name,
            tag
        );
        resp.json().await.unwrap()
    }

    async fn upload_to_nonwritable_repo(cl: &reqwest::Client, name: &str) {
        let resp = cl
            .post(&format!("{ORIGIN}/v2/{name}/blobs/uploads/"))
            .send()
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
        let mani = manifest::ManifestV2 {
            schema_version: 2,
            media_type: Some("application/vnd.docker.distribution.manifest.v2+json".to_owned()),
            config,
            layers,
        };
        let manifest_addr = format!("{}/v2/{}/manifests/{}", ORIGIN, name, "tag");
        let resp = cl.put(&manifest_addr).json(&mani).send().await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_runner() {
        //Need to start with empty repo
        fs::remove_dir_all("./data").unwrap_or(());

        //Had issues with stopping and starting trow causing test fails.
        //It might be possible to improve things with a thread_local
        let _trow = start_trow().await;
        let client = reqwest::Client::new();

        //Using registry proxy should be able to download image even though it's not in registry
        //These tests are repeated to exercise caching logic
        get_manifest(&client, "f/docker/amouat/trow", "latest").await;
        get_manifest(&client, "f/docker/amouat/trow", "latest").await;

        get_manifest(&client, "f/nvcr/nvidia/doca/doca_hbn", "5.1.0-doca1.3.0").await;
        get_manifest(&client, "f/nvcr/nvidia/doca/doca_hbn", "5.1.0-doca1.3.0").await;

        //This should use same alpine image as base (so partially cached)
        get_manifest(&client, "f/docker/library/alpine", "3.13.4").await;
        get_manifest(&client, "f/docker/library/nginx", "1.20.0-alpine").await;

        // Special case: docker/library
        // check that it works and that manifests are written in the correct location
        get_manifest(&client, "f/docker/alpine", "3.13.4").await;
        assert!(!Path::new("./data/manifests/f/docker/alpine/3.13.4").exists());
        assert!(Path::new("./data/manifests/f/docker/library/alpine/3.13.4").exists());

        //Download an amd64 manifest, then the multi platform version of the same manifest
        get_manifest(
            &client,
            "f/docker/hello-world",
            "sha256:f54a58bc1aac5ea1a25d796ae155dc228b3f0e11d046ae276b39c4bf2f13d8c4",
        )
        .await;
        get_manifest(&client, "f/docker/hello-world", "linux").await;

        // test a registry that doesn't require auth
        get_manifest(&client, "f/quay/openshifttest/scratch", "latest").await;

        // Check that tags get updated to point to latest digest
        {
            let man_3_13 = get_manifest(&client, "f/docker/alpine", "3.13.4").await;
            fs::copy(
                "./data/manifests/f/docker/library/alpine/3.13.4",
                "./data/manifests/f/docker/library/alpine/latest",
            )
            .unwrap();
            let man_latest = get_manifest(&client, "f/docker/library/alpine", "latest").await;
            assert_ne!(
                serde_json::to_string(&man_3_13).unwrap(),
                serde_json::to_string(&man_latest).unwrap(),
                "Trow did not update digest of `latest` tag"
            );
        }

        //test writing manifest to proxy dir isn't allowed
        upload_to_nonwritable_repo(&client, "f/failthis").await;
    }
}
