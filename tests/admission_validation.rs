#[cfg(test)]
mod common;

#[cfg(test)]
mod validation_tests {
    use std::collections::HashMap;

    use axum::body::Body;
    use axum::Router;
    use hyper::Request;
    use k8s_openapi::api::core::v1::Pod;
    use kube::core::admission::AdmissionReview;
    use reqwest::{header, StatusCode};
    use tower::ServiceExt;
    use trow::trow_server::ImageValidationConfig;

    use crate::common;

    const HOST: &str = "127.0.0.1:39366";

    /// Call out to cargo to start trow.
    /// Seriously considering moving to docker run.
    async fn start_trow() -> Router {
        let config_file = ImageValidationConfig {
            default: "Deny".to_string(),
            allow: vec![
                "registry-1.docker.io".to_string(),
                "nvcr.io".to_string(),
                "quay.io".to_string(),
                "localhost:8000".to_string(),
                "trow.test".to_string(),
                "k8s.gcr.io".to_string(),
            ],
            deny: vec!["localhost:8000/secret/".to_string()],
        };

        let mut trow_builder = trow::TrowConfig::new();
        trow_builder.service_name = HOST.to_string();
        trow_builder.image_validation_config = Some(config_file);
        trow_builder.build_app().await.unwrap()
    }

    /* Uses a copy of an actual AdmissionReview to test. */
    async fn validate_example(cl: &Router) {
        let review = r#"{
  "kind": "AdmissionReview",
  "apiVersion": "admission.k8s.io/v1beta1",
  "request": {
    "uid": "0b4ab323-b607-11e8-a555-42010a8002a3",
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
      "uid": "fc3f24b4-b5e2-11e8-a555-42010a8002a3",
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
        "uid": "0b4aae46-b607-11e8-a555-42010a8002a3",
        "creationTimestamp": "2018-09-11T21:10:00Z",
        "labels": {
          "pod-template-hash": "447282153",
          "run": "test3"
        },
        "annotations": {
          "kubernetes.io/limit-ranger": "LimitRanger plugin set: cpu request for container test3"
        },
        "ownerReferences": [
          {
            "apiVersion": "networking.k8s.io/v1",
            "kind": "ReplicaSet",
            "name": "test3-88c6d6597",
            "uid": "0b4790c2-b607-11e8-a555-42010a8002a3",
            "controller": true,
            "blockOwnerDeletion": true
          }
        ]
      },
      "spec": {
        "volumes": [
          {
            "name": "default-token-6swbv",
            "secret": {
              "secretName": "default-token-6swbv"
            }
          }
        ],
        "containers": [
          {
            "name": "test3",
            "image": "unknown_registry.io/nginx",
            "resources": {
              "requests": {
                "cpu": "100m"
              }
            },
            "volumeMounts": [
              {
                "name": "default-token-6swbv",
                "readOnly": true,
                "mountPath": "/var/run/secrets/kubernetes.io/serviceaccount"
              }
            ],
            "terminationMessagePath": "/dev/termination-log",
            "terminationMessagePolicy": "File",
            "imagePullPolicy": "Always"
          }
        ],
        "restartPolicy": "Always",
        "terminationGracePeriodSeconds": 30,
        "dnsPolicy": "ClusterFirst",
        "serviceAccountName": "default",
        "serviceAccount": "default",
        "securityContext": {},
        "schedulerName": "default-scheduler",
        "tolerations": [
          {
            "key": "node.kubernetes.io/not-ready",
            "operator": "Exists",
            "effect": "NoExecute",
            "tolerationSeconds": 300
          },
          {
            "key": "node.kubernetes.io/unreachable",
            "operator": "Exists",
            "effect": "NoExecute",
            "tolerationSeconds": 300
          }
        ]
      },
      "status": {
        "phase": "Pending",
        "qosClass": "Burstable"
      }
    },
    "oldObject": null
  }
}"#;

        let resp = cl
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
        let mut val: serde_json::Value = common::response_body_json(resp).await;
        // Fixes "missing field" (which is bollocks)
        val["response"]["auditAnnotations"] =
            serde_json::to_value(HashMap::<String, String>::new()).unwrap();
        let review: AdmissionReview<Pod> = serde_json::from_value(val).unwrap();
        let response = review.response.unwrap();

        assert!(!response.allowed);

        assert_eq!(response.result.message, "unknown_registry.io/nginx: Image is neither explicitly allowed nor denied (using default behavior)");
    }

    async fn test_image(cl: &Router, image_string: &str, is_allowed: bool) {
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
                            "image": image_string,
                        }]
                    }
                }
            }
        })
        .to_string();

        let resp = cl
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
        let mut val: serde_json::Value = common::response_body_json(resp).await;
        // Fixes "missing field" (which is bollocks)
        val["response"]["auditAnnotations"] =
            serde_json::to_value(HashMap::<String, String>::new()).unwrap();
        let review: AdmissionReview<Pod> = serde_json::from_value(val).unwrap();
        let response = review.response.unwrap();

        assert_eq!(
            response.allowed, is_allowed,
            "Wrong response for `{}` ({:?})",
            image_string, response.result.message
        );
    }

    #[tokio::test]
    async fn test_runner() {
        let trow = start_trow().await;
        validate_example(&trow).await;

        // explicitly allowed
        test_image(&trow, "trow.test/am/test:tag", true).await;
        test_image(&trow, "k8s.gcr.io/metrics-server-amd64:v0.2.1", true).await;
        test_image(&trow, "docker.io/amouat/myimage:test", true).await;
        test_image(&trow, "localhost:8000/hello/world", true).await;

        // explicitly denied
        test_image(&trow, "localhost:8000/secret/shine-box", false).await;

        // default denied
        test_image(&trow, "virus.land.cc/not/suspect", false).await;

        // invalid image ref
        // Some very weird refs are actually technically valid, like `example.com` !
        test_image(&trow, "hello human", false).await;
        test_image(&trow, "docker.io/voyager@jasper-byrne", false).await;
    }
}
