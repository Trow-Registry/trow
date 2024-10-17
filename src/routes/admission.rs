use std::sync::Arc;

use anyhow::Result;
use axum::extract::{Json, State};
use axum::routing::post;
use axum::Router;
use k8s_openapi::api::core::v1::Pod;
use kube::core::admission::{AdmissionRequest, AdmissionResponse, AdmissionReview};
use kube::core::DynamicObject;

use crate::TrowServerState;

async fn validate_image(
    State(state): State<Arc<TrowServerState>>,
    Json(image_data): Json<AdmissionReview<Pod>>,
) -> Json<AdmissionReview<DynamicObject>> {
    let req: Result<AdmissionRequest<_>, _> = image_data.try_into();

    Json::from(match req {
        Err(e) => {
            AdmissionResponse::invalid(format!("Invalid admission request: {:#}", e)).into_review()
        }
        Ok(req) => state.registry.validate_admission(&req).await.into_review(),
    })
}

async fn mutate_image(
    State(state): State<Arc<TrowServerState>>,
    Json(image_data): Json<AdmissionReview<Pod>>,
) -> Json<AdmissionReview<DynamicObject>> {
    let req: Result<AdmissionRequest<_>, _> = image_data.try_into();

    let res = match req {
        Err(e) => {
            AdmissionResponse::invalid(format!("Invalid admission request: {:#}", e)).into_review()
        }
        Ok(req) => state
            .registry
            .mutate_admission(&req, &state.config.service_name)
            .await
            .into_review(),
    };

    Json::from(res)
}

pub fn route(app: Router<Arc<TrowServerState>>) -> Router<Arc<TrowServerState>> {
    app.route("/validate-image", post(validate_image))
        .route("/mutate-image", post(mutate_image))
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use axum::body::Body;
    use hyper::Request;
    use k8s_openapi::api::core::v1::Pod;
    use kube::core::admission::AdmissionReview;
    use reqwest::{header, StatusCode};
    use tower::ServiceExt;

    use crate::registry::{
        ImageValidationConfig, RegistryProxiesConfig, SingleRegistryProxyConfig,
    };
    use crate::test_utilities;

    #[rstest::rstest]
    #[case::explicit_allow("trow.test/am/test:tag", true, "")]
    #[case::explicit_allow("k8s.gcr.io/metrics-server-amd64:v0.2.1", true, "")]
    #[case::explicit_allow("docker.io/amouat/myimage:test", true, "")]
    #[case::explicit_allow("localhost:8000/hello/world", true, "")]
    #[case::explicit_deny("localhost:8000/secret/shiny-box", false, "explicitly denied")]
    #[case::default_false("virus.land.cc/not/suspect", false, "using default behavior")]
    #[case::invalid_image("hello human", false, "Invalid image reference")]
    #[case::invalid_image("docker.io/voyager@jasper-byrne", false, "Invalid image reference")]
    #[tokio::test]
    async fn validate_image(
        #[case] image: &str,
        #[case] expected_allow: bool,
        #[case] response_contains: &str,
    ) {
        let (_, _, router) = test_utilities::trow_router(
            |builder| {
                builder.image_validation_config = Some(ImageValidationConfig {
                    default: "Deny".to_string(),
                    allow: vec![
                        "registry-1.docker.io".to_string(),
                        "localhost:8000".to_string(),
                        "trow.test".to_string(),
                        "k8s.gcr.io".to_string(),
                    ],
                    deny: vec!["localhost:8000/secret/".to_string()],
                })
            },
            None,
        )
        .await;

        let review = serde_json::json!({
            "kind": "AdmissionReview",
            "apiVersion": "admission.k8s.io/v1beta1",
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
                        "containers": [{
                            "name": "test3",
                            "image": image,
                        }]
                    }
                }
            }
        })
        .to_string();

        let resp = router
            .clone()
            .oneshot(
                Request::post("/validate-image")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(review))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let mut val: serde_json::Value = test_utilities::response_body_json(resp).await;
        // Fixes "missing field" (which is bollocks)
        val["response"]["auditAnnotations"] =
            serde_json::to_value(HashMap::<String, String>::new()).unwrap();
        let review: AdmissionReview<Pod> = serde_json::from_value(val).unwrap();
        let response = review.response.unwrap();

        assert_eq!(
            response.allowed, expected_allow,
            "Wrong response for `{}` ({:?})",
            image, response.result.message
        );
        assert!(
            response.result.message.contains(response_contains),
            "Response message `{}` does not contain `{}`",
            response.result.message,
            response_contains
        );
    }

    #[rstest::rstest]
    #[case::default("docker.io/library/nginx:tag", Some("/f/docker/library/nginx:tag"))]
    #[case::implicit_repo("nginx:tag", Some("/f/docker/library/nginx:tag"))]
    #[case::explicit_ignore("docker.io/library/milk:tag", None)]
    #[case::explicit_ignore_implicit_repo("milk:tagggged", None)]
    #[case::ecr(
        "1234.dkr.ecr.saturn-5.amazonaws.com/spyops:secret",
        Some("/f/ecr/spyops:secret")
    )]
    #[case::unknown_registry("example.com/area51", None)]
    #[case::invalid_image("http://invalid.com/DANCE", None)]
    #[tokio::test]
    async fn mutate_image(#[case] original_image: &str, #[case] new_image_str: Option<&str>) {
        let host = "ftp://trow";
        let (_, _, router) = test_utilities::trow_router(
            |builder| {
                builder.proxy_registry_config = Some(RegistryProxiesConfig {
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
                });
                builder.service_name = host.to_string();
            },
            None,
        )
        .await;

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
                            "image": original_image,
                        }],
                        "containers": [{
                            "name": "test3",
                            "image": original_image
                        }]
                    }
                }
            }
        });

        let resp = router
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
        let mut val: serde_json::Value = test_utilities::response_body_json(resp).await;
        // Fixes "missing field" (which is bollocks)
        val["response"]["auditAnnotations"] =
            serde_json::to_value(HashMap::<String, String>::new()).unwrap();
        let review: AdmissionReview<Pod> = serde_json::from_value(val).unwrap();
        let response = review.response.unwrap();

        assert!(response.allowed);
        if let Some(new_img) = new_image_str {
            let patch = String::from_utf8_lossy(response.patch.as_ref().unwrap());
            let expected_raw_patch = format!(
                r#"[
                    {{"op": "replace", "path": "/spec/containers/0/image", "value": "{newimg}" }},
                    {{"op": "replace", "path": "/spec/initContainers/0/image", "value": "{newimg}" }}
                ]"#,
                newimg = format!("{host}{new_img}")
            );
            let expected_patch = serde_json::to_string(
                &serde_json::from_str::<json_patch::Patch>(&expected_raw_patch).unwrap(),
            )
            .unwrap();

            assert_eq!(
                patch, expected_patch,
                "Mutation response patch is not correct"
            );
        } else if let Some(patch) = response.patch {
            let patch_str = String::from_utf8_lossy(&patch);
            assert_eq!(patch_str, "[]", "Unexpected patch");
        }
    }
}
