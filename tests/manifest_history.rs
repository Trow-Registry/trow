#[cfg(test)]
mod common;

#[cfg(test)]
mod interface_tests {

    use environment::Environment;

    use crate::common;
    use rand::Rng;

    use reqwest::StatusCode;
    use std::fs;
    use std::io::BufReader;
    use std::process::Child;
    use std::process::Command;
    use std::thread;
    use std::time::Duration;
    use trow_server::digest;

    const PORT: &str = "39368";
    const HOST: &str = "127.0.0.1:39368";
    const TROW_ADDRESS: &str = "http://127.0.0.1:39368";

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
            .arg("--name")
            .arg(HOST)
            .arg("--port")
            .arg(PORT)
            .spawn()
            .expect("failed to start");

        let mut timeout = 100;
        let client = reqwest::Client::new();
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

    async fn upload_config(cl: &reqwest::Client) {
        let config = "{}\n".as_bytes();
        let digest = digest::sha256_tag_digest(BufReader::new(config)).unwrap();
        let resp = cl
            .post(&format!(
                "{}/v2/{}/blobs/uploads/?digest={}",
                TROW_ADDRESS, "config", digest
            ))
            .body(config)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    async fn push_random_foreign_manifest(cl: &reqwest::Client, name: &str, tag: &str) -> String {
        //Note config was uploaded as blob earlier
        let config = "{}\n".as_bytes();
        let config_digest = digest::sha256_tag_digest(BufReader::new(config)).unwrap();

        //To ensure each manifest is different, just use foreign content with random contents
        let mut rng = rand::thread_rng();
        let ran_size: u32 = rng.gen();
        let mut digest = [0u8; 32];
        rng.fill(&mut digest[..]);
        let mut ran_digest = "".to_string();
        for b in &digest {
            ran_digest.push_str(&format!("{:x}", b).to_string());
        }

        let manifest = format!(
            r#"{{ "mediaType": "application/vnd.oci.image.manifest.v1+json",
                 "config": {{ "digest": "{}",
                             "mediaType": "application/vnd.oci.image.config.v1+json",
                             "size": {} }},
                 "layers": [
                    {{
                              "mediaType": "application/vnd.docker.image.rootfs.foreign.diff.tar.gzip",
                              "size": {},
                              "digest": "sha256:{}",
                              "urls": [
                                 "https://mcr.microsoft.com/v2/windows/servercore/blobs/sha256:9038b92872bc268d5c975e84dd94e69848564b222ad116ee652c62e0c2f894b2"
                              ]
                           }}
                 ], "schemaVersion": 2 }}"#,
            config_digest,
            config.len(),
            ran_size,
            ran_digest
        );
        let bytes = manifest.clone();
        let resp = cl
            .put(&format!("{}/v2/{}/manifests/{}", TROW_ADDRESS, name, tag))
            .body(bytes)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let digest = digest::sha256_tag_digest(BufReader::new(manifest.as_bytes())).unwrap();
        digest
    }

    async fn get_history(
        cl: &reqwest::Client,
        repo: &str,
        tag: &str,
        limit: Option<u32>,
        digest: Option<String>,
    ) -> serde_json::Value {
        let mut options = "".to_owned();
        if let Some(val) = digest {
            options = format!("?last={}", val);

            if let Some(val) = limit {
                options = format!("{}&{}", options, val);
            }
        } else if let Some(val) = limit {
            options = format!("?n={}", val);
        }

        let resp = cl
            .get(&format!(
                "{}/{}/manifest_history/{}{}",
                TROW_ADDRESS, repo, tag, options
            ))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // type should be decided by caller, but this is just a test
        let x: serde_json::Value = resp.json().await.unwrap();

        x
    }

    /**
     * Tests of Trow's support for manifest history.
     *
     * Given a tag, we should be able to get the digest it currently points to and all previous digests, with dates.
     *
     */
    #[tokio::test]
    async fn manifest_test() {
        //Need to start with empty repo
        fs::remove_dir_all("./data").unwrap_or(());
        let _trow = start_trow().await;
        let client = reqwest::Client::new();

        upload_config(&client).await;

        // Following is intentionally interleaved to add delays
        let mut history_one = Vec::new();
        history_one.push(push_random_foreign_manifest(&client, "history", "one").await);
        let mut history_two = Vec::new();
        history_two.push(push_random_foreign_manifest(&client, "history", "two").await);
        history_one.push(push_random_foreign_manifest(&client, "history", "one").await);
        history_two.push(push_random_foreign_manifest(&client, "history", "two").await);
        history_one.push(push_random_foreign_manifest(&client, "history", "one").await);

        let json = get_history(&client, "history", "one", None, None).await;
        assert_eq!(json["image"], "history:one");
        assert_eq!(json["history"].as_array().unwrap().len(), 3);

        let json = get_history(&client, "history", "two", None, None).await;
        assert_eq!(json["image"], "history:two");
        assert_eq!(json["history"].as_array().unwrap().len(), 2);

        let json = get_history(&client, "history", "one", Some(1), None).await;
        assert_eq!(json["history"].as_array().unwrap().len(), 1);

        let start = json["history"][0]["digest"].as_str().unwrap();
        let json = get_history(&client, "history", "one", Some(20), Some(start.to_string())).await;
        assert_eq!(json["history"].as_array().unwrap().len(), 2);
    }
}
