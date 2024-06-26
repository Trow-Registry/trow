#[cfg(test)]
mod common;

#[cfg(test)]
mod interface_tests {

    use std::fmt::Write;
    use std::io::BufReader;
    use std::path::Path;

    use axum::body::Body;
    use axum::Router;
    use hyper::Request;
    use reqwest::StatusCode;
    use test_temp_dir::test_temp_dir;
    use tower::ServiceExt;
    use tracing_test::traced_test;
    use trow::registry::digest;

    use crate::common::{self, response_body_vec};

    const TROW_ADDRESS: &str = "http://127.0.0.1:39368";

    async fn start_trow(data_dir: &Path) -> Router {
        let mut trow_builder = trow::TrowConfig::new();
        data_dir.clone_into(&mut trow_builder.data_dir);
        trow_builder.service_name = TROW_ADDRESS.to_string();
        trow_builder.build_app().await.unwrap()
    }

    async fn upload_config(trow: &Router) {
        let config = "{}\n".as_bytes();
        let digest = digest::Digest::digest_sha256(BufReader::new(config)).unwrap();
        let req = Request::post(format!("/v2/config/blobs/uploads/?digest={digest}"))
            .body(Body::from(config))
            .unwrap();
        let resp = trow.clone().oneshot(req).await.unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::CREATED,
            "request failed: {}",
            common::response_body_string(resp).await
        );
    }

    async fn push_random_foreign_manifest(trow: &Router, name: &str, tag: &str) -> String {
        //Note config was uploaded as blob earlier
        let config = "{}\n".as_bytes();
        let config_digest = digest::Digest::digest_sha256(BufReader::new(config)).unwrap();

        //To ensure each manifest is different, just use foreign content with random contents
        let ran_size = fastrand::u32(0..=u32::MAX);
        let ran_digest = (0..32).fold(String::new(), |mut output, _| {
            write!(output, "{:02x}", fastrand::u8(0..=u8::MAX)).unwrap();
            output
        });

        let config_len = config.len();
        let manifest = format!(
            r#"{{ "mediaType": "application/vnd.oci.image.manifest.v1+json",
                  "config": {{ "digest": "{config_digest}",
                             "mediaType": "application/vnd.oci.image.config.v1+json",
                             "size": {config_len} }},
                 "layers": [
                    {{
                              "mediaType": "application/vnd.docker.image.rootfs.foreign.diff.tar.gzip",
                              "size": {ran_size},
                              "digest": "sha256:{ran_digest}",
                              "urls": [
                                 "https://mcr.microsoft.com/v2/windows/servercore/blobs/sha256:9038b92872bc268d5c975e84dd94e69848564b222ad116ee652c62e0c2f894b2"
                              ]
                           }}
                 ], "schemaVersion": 2 }}"#,
        );
        let bytes = manifest.clone();
        let resp = trow
            .clone()
            .oneshot(
                Request::put(format!("/v2/{}/manifests/{}", name, tag))
                    .body(bytes)
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        digest::Digest::digest_sha256(BufReader::new(manifest.as_bytes()))
            .unwrap()
            .to_string()
    }

    async fn get_history(
        trow: &Router,
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
        let resp = trow
            .clone()
            .oneshot(
                Request::get(format!("/{}/manifest_history/{}{}", repo, tag, options))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = response_body_vec(resp).await;
        // type should be decided by caller, but this is just a test
        let x: serde_json::Value = serde_json::from_slice(&body).unwrap();

        x
    }

    /**
     * Tests of Trow's support for manifest history.
     *
     * Given a tag, we should be able to get the digest it currently points to and all previous digests, with dates.
     *
     */
    #[tokio::test]
    #[traced_test]
    async fn manifest_test() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let trow = start_trow(data_dir).await;
        upload_config(&trow).await;

        // Following is intentionally interleaved to add delays
        let mut history_one = Vec::new();
        history_one.push(push_random_foreign_manifest(&trow, "history", "one").await);
        let mut history_two = Vec::new();
        history_two.push(push_random_foreign_manifest(&trow, "history", "two").await);
        history_one.push(push_random_foreign_manifest(&trow, "history", "one").await);
        history_two.push(push_random_foreign_manifest(&trow, "history", "two").await);
        history_one.push(push_random_foreign_manifest(&trow, "history", "one").await);

        let json = get_history(&trow, "history", "one", None, None).await;
        assert_eq!(json["image"], "history:one");
        assert_eq!(json["history"].as_array().unwrap().len(), 3);

        let json = get_history(&trow, "history", "two", None, None).await;
        assert_eq!(json["image"], "history:two");
        assert_eq!(json["history"].as_array().unwrap().len(), 2);

        let json = get_history(&trow, "history", "one", Some(1), None).await;
        assert_eq!(json["history"].as_array().unwrap().len(), 1);

        let start = json["history"][0]["digest"].as_str().unwrap();
        let json = get_history(&trow, "history", "one", Some(20), Some(start.to_string())).await;
        assert_eq!(json["history"].as_array().unwrap().len(), 2);
    }
}
