#[cfg(test)]
mod common;

#[cfg(test)]
mod admission_mutation_tests {
    use std::collections::HashMap;

    use axum::body::Body;
    use axum::Router;
    use hyper::Request;
    use json_patch::PatchOperation;
    use k8s_openapi::api::core::v1::Pod;
    use kube::core::admission::AdmissionReview;
    use reqwest::StatusCode;
    use tower::ServiceExt;
    use trow::registry::{RegistryProxiesConfig, SingleRegistryProxyConfig};

    use crate::common;

    const HOST: &str = "127.0.0.1:39365";

    async fn start_trow() -> Router {
        let config_file = RegistryProxiesConfig {
            offline: false,
            registries: vec![
                SingleRegistryProxyConfig {
                    alias: "docker".to_string(),
                    host: "registry-1.docker.io".to_string(),
                    username: None,
                    password: None,
                    ignore_repos: vec!["library/milk".to_string()],
                },
                SingleRegistryProxyConfig {
                    alias: "ecr".to_string(),
                    host: "1234.dkr.ecr.saturn-5.amazonaws.com".to_string(),
                    username: Some("AWS".to_string()),
                    password: None,
                    ignore_repos: vec![],
                },
            ],
        };

        let mut trow_builder = trow::TrowConfig::new();
        trow_builder.service_name = HOST.to_string();
        trow_builder.proxy_registry_config = Some(config_file);
        trow_builder.build_app().await.unwrap()
    }

    async fn test_request(trow: &Router, image_string: &str, new_image_str: Option<&str>) {
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

        let resp = trow
            .clone()
            .oneshot(
                Request::post("/mutate-image")
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(review.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = common::response_body_vec(resp).await;
        let mut val = serde_json::from_slice::<serde_json::Value>(&body).unwrap();
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
                    path: jsonptr::PointerBuf::parse("/spec/containers/0/image").unwrap(),
                    value: serde_json::Value::String(new_img.to_string()),
                }),
                PatchOperation::Replace(json_patch::ReplaceOperation {
                    path: jsonptr::PointerBuf::parse("/spec/initContainers/0/image").unwrap(),
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
    async fn test_explicit_docker_io_library() {
        let trow = start_trow().await;
        println!("Test explicit docker.io/library");
        test_request(
            &trow,
            "docker.io/library/nginx:tag",
            Some(&format!("{HOST}/f/docker/library/nginx:tag")),
        )
        .await;
    }
    #[tokio::test]
    async fn test_implicit_docker_io_library() {
        let trow = start_trow().await;
        test_request(
            &trow,
            "nginx:tag",
            Some(&format!("{HOST}/f/docker/library/nginx:tag")),
        )
        .await;
    }
    #[tokio::test]
    async fn test_ignore_docker_io_library_milk() {
        let trow = start_trow().await;

        println!("Test ignore docker.io/library/milk");
        test_request(&trow, "docker.io/library/milk:tag", None).await;

        println!("Test ignore docker.io/library/milk");
        test_request(&trow, "milk:tagggged", None).await;
    }
    #[tokio::test]
    async fn test_ecr() {
        let trow = start_trow().await;

        println!("Test ecr");
        test_request(
            &trow,
            "1234.dkr.ecr.saturn-5.amazonaws.com/spyops:secret",
            Some(&format!("{HOST}/f/ecr/spyops:secret")),
        )
        .await;
    }
    #[tokio::test]
    async fn test_unknown_registry() {
        let trow = start_trow().await;

        println!("Test unknown registry");
        test_request(&trow, "example.com/area51", None).await;
    }
    #[tokio::test]
    async fn test_invalid_image() {
        let trow = start_trow().await;

        println!("Test invalid image");
        test_request(&trow, "http://invalid.com/DANCE", None).await;
    }
}
