#[cfg(test)]
mod common;

#[cfg(test)]
mod interface_tests {

    use environment::Environment;

    use crate::common;

    use reqwest::StatusCode;

    use std::fs::{self, File};
    use std::io::{BufReader, Read};
    use std::process::Child;
    use std::process::Command;
    use std::thread;
    use std::time::Duration;
    use trow::types::{HealthResponse, ReadinessResponse, RepoCatalog, TagList};
    use trow_server::{digest, manifest};

    const TROW_ADDRESS: &str = "https://trow.test:8443";
    const DIST_API_HEADER: &str = "Docker-Distribution-API-Version";

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

    async fn get_main(cl: &reqwest::Client) {
        let resp = cl.get(TROW_ADDRESS).send().await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get(DIST_API_HEADER).unwrap(), "registry/2.0");

        //All v2 registries should respond with a 200 to this
        let resp = cl
            .get(&(TROW_ADDRESS.to_owned() + "/v2/"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get(DIST_API_HEADER).unwrap(), "registry/2.0");
    }

    async fn get_non_existent_blob(cl: &reqwest::Client) {
        let resp = cl
            .get(&(TROW_ADDRESS.to_owned() + "/v2/test/test/blobs/sha256:baadf00d"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    async fn get_manifest(cl: &reqwest::Client, name: &str, tag: &str, size: Option<usize>) {
        //Might need accept headers here
        let resp = cl
            .get(&format!("{}/v2/{}/manifests/{}", TROW_ADDRESS, name, tag))
            .send()
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
        let mani: manifest::ManifestV2 = resp.json().await.unwrap();

        assert_eq!(mani.schema_version, 2);
    }

    async fn get_non_existent_manifest(cl: &reqwest::Client, name: &str, tag: &str) {
        //Might need accept headers here
        let resp = cl
            .get(&format!("{}/v2/{}/manifests/{}", TROW_ADDRESS, name, tag))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    async fn check_repo_catalog(cl: &reqwest::Client, rc: &RepoCatalog) {
        let resp = cl
            .get(&format!("{}/v2/_catalog", TROW_ADDRESS))
            .send()
            .await
            .unwrap();
        let rc_resp: RepoCatalog = serde_json::from_str(&resp.text().await.unwrap()).unwrap();
        assert_eq!(rc, &rc_resp);
    }

    async fn check_tag_list(cl: &reqwest::Client, tl: &TagList) {
        let resp = cl
            .get(&format!("{}/v2/{}/tags/list", TROW_ADDRESS, tl.repo_name()))
            .send()
            .await
            .unwrap();
        let tl_resp: TagList = serde_json::from_str(&resp.text().await.unwrap()).unwrap();
        assert_eq!(tl, &tl_resp);
    }

    async fn check_tag_list_n_last(cl: &reqwest::Client, n: u32, last: &str, tl: &TagList) {
        let resp = cl
            .get(&format!(
                "{}/v2/{}/tags/list?last={}&n={}",
                TROW_ADDRESS,
                tl.repo_name(),
                last,
                n
            ))
            .send()
            .await
            .unwrap();
        let tl_resp: TagList = serde_json::from_str(&resp.text().await.unwrap()).unwrap();
        assert_eq!(tl, &tl_resp);
    }

    async fn upload_with_put(cl: &reqwest::Client, name: &str) {
        let resp = cl
            .post(&format!("{}/v2/{}/blobs/uploads/", TROW_ADDRESS, name))
            .send()
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
        let digest = digest::sha256_tag_digest(BufReader::new(config)).unwrap();
        let loc = &format!(
            "{}/v2/{}/blobs/uploads/{}?digest={}",
            TROW_ADDRESS, name, uuid, digest
        );

        let resp = cl.put(loc).body(config).send().await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let range = resp
            .headers()
            .get(common::RANGE_HEADER)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(range, format!("0-{}", (config.len() - 1))); //note first byte is 0, hence len - 1
    }

    async fn upload_with_post(cl: &reqwest::Client, name: &str) {
        let config = "{ }\n".as_bytes();
        let digest = digest::sha256_tag_digest(BufReader::new(config)).unwrap();
        let resp = cl
            .post(&format!(
                "{}/v2/{}/blobs/uploads/?digest={}",
                TROW_ADDRESS, name, digest
            ))
            .body(config)
            .send()
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

    async fn push_oci_manifest(cl: &reqwest::Client, name: &str, tag: &str) -> String {
        //Note config was uploaded as blob in earlier test
        let config = "{}\n".as_bytes();
        let config_digest = digest::sha256_tag_digest(BufReader::new(config)).unwrap();

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
            .put(&format!("{}/v2/{}/manifests/{}", TROW_ADDRESS, name, tag))
            .body(bytes)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        // Try pulling by digest
        let digest = digest::sha256_tag_digest(BufReader::new(manifest.as_bytes())).unwrap();
        digest
    }

    async fn push_manifest_list(
        cl: &reqwest::Client,
        digest: &str,
        name: &str,
        tag: &str,
    ) -> String {
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
            .put(&format!("{}/v2/{}/manifests/{}", TROW_ADDRESS, name, tag))
            .body(bytes)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        // Try pulling by digest
        let digest = digest::sha256_tag_digest(BufReader::new(manifest.as_bytes())).unwrap();
        digest
    }

    async fn push_oci_manifest_with_foreign_blob(
        cl: &reqwest::Client,
        name: &str,
        tag: &str,
    ) -> String {
        //Note config was uploaded as blob in earlier test
        let config = "{}\n".as_bytes();
        let config_digest = digest::sha256_tag_digest(BufReader::new(config)).unwrap();

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
            .put(&format!("{}/v2/{}/manifests/{}", TROW_ADDRESS, name, tag))
            .body(bytes)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        // Try pulling by digest
        let digest = digest::sha256_tag_digest(BufReader::new(manifest.as_bytes())).unwrap();
        digest
    }

    async fn delete_manifest(cl: &reqwest::Client, name: &str, digest: &str) {
        let resp = cl
            .delete(&format!(
                "{}/v2/{}/manifests/{}",
                TROW_ADDRESS, name, digest
            ))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
    }

    async fn delete_non_existent_manifest(cl: &reqwest::Client, name: &str) {
        let resp = cl
            .delete(&format!(
                "{}/v2/{}/manifests/{}",
                TROW_ADDRESS,
                name,
                "sha256:9038b92872bc268d5c975e84dd94e69848564b222ad116ee652c62e0c2f894b2"
            ))
            .send()
            .await
            .unwrap();
        // If it doesn't exist, that's kinda the same as deleted, right?
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
    }
    async fn attempt_delete_by_tag(cl: &reqwest::Client, name: &str, tag: &str) {
        let resp = cl
            .delete(&format!("{}/v2/{}/manifests/{}", TROW_ADDRESS, name, tag))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    async fn delete_config_blob(cl: &reqwest::Client, name: &str) {
        //Deletes blob uploaded in config test
        let config = "{}\n".as_bytes();
        let config_digest = digest::sha256_tag_digest(BufReader::new(config)).unwrap();
        let resp = cl
            .delete(&format!(
                "{}/v2/{}/blobs/{}",
                TROW_ADDRESS, name, config_digest
            ))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
    }

    async fn test_6level_error(cl: &reqwest::Client) {
        let name = "one/two/three/four/five/six";
        let resp = cl
            .post(&format!("{}/v2/{}/blobs/uploads/", TROW_ADDRESS, name))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    async fn get_health(cl: &reqwest::Client) {
        let resp = cl
            .get(&format!("{}/healthz", TROW_ADDRESS))
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let hr: HealthResponse = resp.json().await.unwrap();

        assert!(hr.is_healthy);
    }

    async fn get_readiness(cl: &reqwest::Client) {
        let resp = cl
            .get(&format!("{}/readiness", TROW_ADDRESS))
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let rr: ReadinessResponse = resp.json().await.unwrap();

        assert!(rr.is_ready);
    }

    async fn get_metrics(cl: &reqwest::Client) {
        let resp = cl
            .get(&format!("{}/metrics", TROW_ADDRESS))
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);

        let body = resp.text().await.unwrap();

        println!("testout {}", body);

        assert!(body.contains("available_space"));
        assert!(body.contains("free_space"));
        assert!(body.contains("total_space"));

        assert!(body.contains("total_manifest_requests{type=\"manifests\"} 6"));
        assert!(body.contains("total_blob_requests{type=\"blobs\"} 9"));

        get_manifest(cl, "onename", "tag", None).await;
        let manifest_response = cl
            .get(&format!("{}/metrics", TROW_ADDRESS))
            .send()
            .await
            .unwrap();

        let manifest_body = manifest_response.text().await.unwrap();

        assert!(manifest_body.contains("total_manifest_requests{type=\"manifests\"} 7"));

        get_non_existent_blob(cl).await;
        let blob_response = cl
            .get(&format!("{}/metrics", TROW_ADDRESS))
            .send()
            .await
            .unwrap();

        assert_eq!(blob_response.status(), StatusCode::OK);

        let blob_body = blob_response.text().await.unwrap();

        assert!(blob_body.contains("total_blob_requests{type=\"blobs\"} 10"));
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

        println!("Running get_main()");
        get_main(&client).await;
        println!("Running get_blob()");
        get_non_existent_blob(&client).await;

        println!("Running upload_layer(fifth/fourth/repo/image/test:tag)");
        common::upload_layer(&client, "fifth/fourth/repo/image/test", "tag").await;
        println!("Running upload_layer(fourth/repo/image/test:tag)");
        common::upload_layer(&client, "fourth/repo/image/test", "tag").await;
        println!("Running upload_layer(repo/image/test:tag)");
        common::upload_layer(&client, "repo/image/test", "tag").await;
        println!("Running upload_layer(image/test:latest)");
        common::upload_layer(&client, "image/test", "latest").await;
        println!("Running upload_layer(onename:tag)");
        common::upload_layer(&client, "onename", "tag").await;
        println!("Running upload_layer(onename:latest)");
        common::upload_layer(&client, "onename", "latest").await;
        println!("Running upload_with_put()");
        upload_with_put(&client, "puttest").await;
        println!("Running upload_with_post");
        upload_with_post(&client, "posttest").await;

        println!("Running test_5level_error()");
        test_6level_error(&client).await;

        println!("Running push_oci_manifest()");
        let digest = push_oci_manifest(&client, "puttest", "puttest1").await;
        println!("Running push_manifest_list()");
        let digest_list = push_manifest_list(&client, &digest, "listtest", "listtest1").await;
        println!("Running get_manifest(puttest:puttest1)");
        get_manifest(&client, "puttest", "puttest1", Some(354)).await;
        println!("Running delete_manifest(puttest:digest)");
        delete_manifest(&client, "puttest", &digest).await;
        println!("Running delete_manifest(listtest)");
        delete_manifest(&client, "listtest", &digest_list).await;
        println!("Running delete_non_existent_manifest(onename)");
        delete_non_existent_manifest(&client, "onename").await;
        println!("Running attempt_delete_by_tag(onename:tag)");
        attempt_delete_by_tag(&client, "onename", "tag").await;
        println!("Running get_non_existent_manifest(puttest:puttest1)");
        get_non_existent_manifest(&client, "puttest", "puttest1").await;
        println!("Running get_non_existent_manifest(puttest:digest)");
        get_non_existent_manifest(&client, "puttest", &digest).await;

        println!("Running push_oci_manifest_with_foreign_blob()");
        let digest = push_oci_manifest_with_foreign_blob(&client, "foreigntest", "blobtest1").await;
        delete_manifest(&client, "foreigntest", &digest).await;

        println!("Running delete_config_blob");
        delete_config_blob(&client, "puttest").await;

        println!("Running get_manifest(onename:tag)");
        get_manifest(&client, "onename", "tag", None).await;
        println!("Running get_manifest(image/test:latest)");
        get_manifest(&client, "image/test", "latest", None).await;
        println!("Running get_manifest(repo/image/test:tag)");
        get_manifest(&client, "repo/image/test", "tag", None).await;

        let mut rc = RepoCatalog::new();
        rc.insert("fifth/fourth/repo/image/test".to_string());
        rc.insert("fourth/repo/image/test".to_string());
        rc.insert("repo/image/test".to_string());
        rc.insert("image/test".to_string());
        rc.insert("onename".to_string());

        println!("Running check_repo_catalog");
        check_repo_catalog(&client, &rc).await;

        let mut tl = TagList::new("repo/image/test".to_string());
        tl.insert("tag".to_string());
        println!("Running check_tag_list 1");
        check_tag_list(&client, &tl).await;

        common::upload_layer(&client, "onename", "three").await;
        common::upload_layer(&client, "onename", "four").await;

        // list, in order should be [four, latest, tag, three]
        let mut tl2 = TagList::new("onename".to_string());
        tl2.insert("four".to_string());
        tl2.insert("latest".to_string());
        tl2.insert("tag".to_string());
        tl2.insert("three".to_string());

        println!("Running check_tag_list 2");
        check_tag_list(&client, &tl2).await;
        let mut tl3 = TagList::new("onename".to_string());
        tl3.insert("four".to_string());
        tl3.insert("latest".to_string());

        println!("Running check_tag_list_n_last 3");
        check_tag_list_n_last(&client, 2, "", &tl3).await;
        let mut tl4 = TagList::new("onename".to_string());
        tl4.insert("tag".to_string());
        tl4.insert("three".to_string());
        println!("Running check_tag_list_n_last 4");
        check_tag_list_n_last(&client, 2, "latest", &tl4).await;

        println!("Running get_readiness");
        get_readiness(&client).await;

        println!("Running get_health");
        get_health(&client).await;

        println!("Running get_metrics");
        get_metrics(&client).await;
        check_tag_list_n_last(&client, 2, "latest", &tl4).await;
    }
}
