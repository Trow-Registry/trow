#[cfg(test)]
mod common;

#[cfg(test)]
mod interface_tests {

    use environment::Environment;

    use crate::common;

    use reqwest::StatusCode;
    use std::fs::{self, File};
    use std::io::Read;
    use std::process::Child;
    use std::process::Command;
    use std::thread;
    use std::time::Duration;
    use trow_server::manifest;

    const TROW_ADDRESS: &str = "https://trow.test:8443";

    struct TrowInstance {
        pid: Child,
    }
    /// Call out to cargo to start trow.
    /// Seriously considering moving to docker run.

    async fn start_trow() -> TrowInstance {
        let mut child = Command::new("cargo")
            .arg("run")
            .env_clear()
            .envs(Environment::inherit().compile())
            .arg("--")
            .arg("--proxy-docker-hub")
            .spawn()
            .expect("failed to start");

        let mut timeout = 100;

        let mut buf = Vec::new();
        File::open("./certs/domain.crt")
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
        let cert = reqwest::Certificate::from_pem(&buf).unwrap();
        // get a client builder
        let client = reqwest::Client::builder()
            .add_root_certificate(cert)
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        let mut response = client.get(TROW_ADDRESS).send().await;
        while timeout > 0 && (response.is_err() || (response.unwrap().status() != StatusCode::OK)) {
            thread::sleep(Duration::from_millis(100));
            response = client.get(TROW_ADDRESS).send().await;
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

    async fn get_manifest(cl: &reqwest::Client, name: &str, tag: &str) {
        //Might need accept headers here
        let resp = cl
            .get(&format!("{}/v2/{}/manifests/{}", TROW_ADDRESS, name, tag))
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
        let _: manifest::Manifest = resp.json().await.unwrap();
    }

    async fn upload_to_nonwritable_repo(cl: &reqwest::Client, name: &str) {
        let resp = cl
            .post(&format!("{}/v2/{}/blobs/uploads/", TROW_ADDRESS, name))
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
        let manifest_addr = format!("{}/v2/{}/manifests/{}", TROW_ADDRESS, name, "tag");
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

        let mut buf = Vec::new();
        File::open("./certs/domain.crt")
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
        let cert = reqwest::Certificate::from_pem(&buf).unwrap();
        // get a client builder
        let client = reqwest::Client::builder()
            .add_root_certificate(cert)
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        //Using docker proxy should be able to download image even though it's not in registry
        //These tests are repeated to exercise caching logic
        get_manifest(&client, "f/docker/amouat/trow", "latest").await;
        get_manifest(&client, "f/docker/amouat/trow", "latest").await;

        //NOTE: if tag is updated also update nginx tag
        get_manifest(&client, "f/docker/library/alpine", "3.13").await;
        get_manifest(&client, "f/docker/library/alpine", "3.13").await;

        //This should use same alpine image as base (so partially cached)
        get_manifest(&client, "f/docker/library/nginx", "1.21.0-alpine").await;

        //Need to special case single name repos
        get_manifest(&client, "f/docker/alpine", "latest").await;

        //Download an amd64 manifest, then the multi platform version of the same manifest
        get_manifest(
            &client,
            "f/docker/hello-world",
            "sha256:f54a58bc1aac5ea1a25d796ae155dc228b3f0e11d046ae276b39c4bf2f13d8c4",
        )
        .await;
        get_manifest(&client, "f/docker/hello-world", "linux").await;

        //test writing manifest to proxy dir isn't allowed
        upload_to_nonwritable_repo(&client, "f/failthis").await;
    }
}
