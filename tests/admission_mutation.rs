#[cfg(test)]
mod common;

#[cfg(test)]
mod admission_mutation_tests {
    use std::collections::HashMap;
    use std::process::{Child, Command};
    use std::thread;
    use std::time::Duration;

    use environment::Environment;
    use hyper::header;
    use json_patch::PatchOperation;
    use k8s_openapi::api::core::v1::Pod;
    use kube::core::admission::AdmissionReview;
    use reqwest::StatusCode;
    use trow_server::RegistryProxyConfig;

    use crate::common;

    const PORT: &str = "39365";
    const HOST: &str = "127.0.0.1:39365";
    const ORIGIN: &str = "http://127.0.0.1:39365";

    struct TrowInstance {
        pid: Child,
    }

    /// Call out to cargo to start trow.
    /// Seriously considering moving to docker run.
    async fn start_trow() -> TrowInstance {
        let config_file = common::get_file(vec![
            RegistryProxyConfig {
                alias: "docker".to_string(),
                host: "registry-1.docker.io".to_string(),
                username: None,
                password: None,
            },
            RegistryProxyConfig {
                alias: "ecr".to_string(),
                host: "1234.dkr.ecr.saturn-5.amazonaws.com".to_string(),
                username: Some("AWS".to_string()),
                password: None,
            },
        ]);

        let mut child = Command::new("cargo")
            .arg("run")
            .env_clear()
            .envs(Environment::inherit().compile())
            .arg("--")
            .arg("--name")
            .arg(HOST)
            .arg("--port")
            .arg(PORT)
            .arg("--proxy-registry-config-file")
            .arg(config_file.path())
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

    async fn test_request(cl: &reqwest::Client, image_string: &str, new_image_str: Option<&str>) {
        let review = serde_json::json!({
            "kind": "AdmissionReview",
            "apiVersion": "admission.k8s.io/v1",
            "request": {
                "uid": "0b4ab323-b607-11e8-a555-42010a8002b4",
                "kind": {
                    "group": "",
                    "version": "v1",
                    "kind": "Pod"
                },
                "resource": {
                    "group": "",
                    "version": "v1",
                    "resource": "pods"
                },
                "namespace": "default",
                "operation": "CREATE",
                "userInfo": {
                    "username": "system:serviceaccount:kube-system:replicaset-controller",
                    "uid": "fc3f24b4-b5e2-11e8-a555-42010a8002b4",
                    "groups": [
                        "system:serviceaccounts",
                        "system:serviceaccounts:kube-system",
                        "system:authenticated"
                    ]
                },
                "object": {
                    "metadata": {
                        "name": "test3-88c6d6597-rll2c",
                        "generateName": "test3-88c6d6597-",
                        "namespace": "default",
                        "uid": "0b4aae46-b607-11e8-a555-42010a8002b4",
                        "creationTimestamp": "2018-09-11T21:10:00Z",
                        "labels": {
                        "pod-template-hash": "447282153",
                        "run": "test3"
                        }
                    },
                    "spec": {
                        "initContainers": [{
                            "name": "init-test",
                            "image": image_string,
                        }],
                        "containers": [{
                            "name": "test3",
                            "image": image_string
                        }]
                    }
                }
            }
        });

        let resp = cl
            .post(&format!("{}/mutate-image", ORIGIN))
            .header(header::CONTENT_TYPE, "application/json")
            .body(review.to_string())
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let mut val = resp.json::<serde_json::Value>().await.unwrap();
        // Fixes "missing field" (which is bollocks)
        val["response"]["auditAnnotations"] =
            serde_json::to_value(HashMap::<String, String>::new()).unwrap();

        let review: AdmissionReview<Pod> = serde_json::from_value(val).unwrap();
        let response = review.response.unwrap();

        assert!(response.allowed);

        if let Some(new_img) = new_image_str {
            let patch = String::from_utf8_lossy(response.patch.as_ref().unwrap());

            let expected_raw_patch = json_patch::Patch(vec![
                PatchOperation::Replace(json_patch::ReplaceOperation {
                    path: "/spec/containers/0/image".to_string(),
                    value: serde_json::Value::String(new_img.to_string()),
                }),
                PatchOperation::Replace(json_patch::ReplaceOperation {
                    path: "/spec/initContainers/0/image".to_string(),
                    value: serde_json::Value::String(new_img.to_string()),
                }),
            ]);
            let expected_patch = serde_json::to_string(&expected_raw_patch).unwrap();

            assert_eq!(
                patch, expected_patch,
                "Mutation response patch is not correct"
            );
        } else if let Some(patch) = response.patch {
            let patch_str = String::from_utf8_lossy(&patch);
            assert_eq!(patch_str, "[]", "Unexpected patch");
        }
    }

    #[tokio::test]
    async fn test_runner() {
        let client = reqwest::Client::new();
        let _trow = start_trow().await;

        println!("Test explicit docker.io/library");
        test_request(
            &client,
            "docker.io/library/nginx:tag",
            Some(&format!("{HOST}/f/docker/library/nginx:tag")),
        )
        .await;

        println!("Test implicit docker.io/library");
        test_request(
            &client,
            "nginx:tag",
            Some(&format!("{HOST}/f/docker/library/nginx:tag")),
        )
        .await;

        println!("Test ecr");
        test_request(
            &client,
            "1234.dkr.ecr.saturn-5.amazonaws.com/spyops:secret",
            Some(&format!("{HOST}/f/ecr/spyops:secret")),
        )
        .await;

        println!("Test uknown registry");
        test_request(&client, "example.com/area51", None).await;

        println!("Test invalid image");
        test_request(&client, "http://invalid.com/DANCE", None).await;
    }
}
