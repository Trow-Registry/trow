#[cfg(test)]
mod common;

#[cfg(test)]
mod validation_tests {
    use std::collections::HashMap;
    use std::process::Child;
    use std::process::Command;
    use std::thread;
    use std::time::Duration;

    use environment::Environment;
    use k8s_openapi::api::core::v1::Pod;
    use kube::core::admission::AdmissionReview;
    use reqwest::StatusCode;

    use crate::common;
    use trow_server::ImageValidationConfig;

    const PORT: &str = "39366";
    const HOST: &str = "127.0.0.1:39366";
    const ORIGIN: &str = "http://127.0.0.1:39366";

    struct TrowInstance {
        pid: Child,
    }

    /// Call out to cargo to start trow.
    /// Seriously considering moving to docker run.
    async fn start_trow() -> TrowInstance {
        let config_file = common::get_file(ImageValidationConfig {
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
        });

        let mut child = Command::new("cargo")
            .arg("run")
            .env_clear()
            .envs(Environment::inherit().compile())
            .arg("--")
            .arg("--no-tls")
            .arg("--name")
            .arg(HOST)
            .arg("--port")
            .arg(PORT)
            .arg("--image-validation-config-file")
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
            panic!("Failed to start Trow",);
        }
        TrowInstance { pid: child }
    }

    impl Drop for TrowInstance {
        fn drop(&mut self) {
            common::kill_gracefully(&self.pid);
        }
    }

    /* Uses a copy of an actual AdmissionReview to test. */
    async fn validate_example(cl: &reqwest::Client) {
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
            .post(&format!("{}/validate-image", ORIGIN))
            .body(review)
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

        assert!(!response.allowed);

        assert_eq!(response.result.message, "unknown_registry.io/nginx: Image is neither explicitely allowed nor denied (using default behavior)");
    }

    async fn test_image(cl: &reqwest::Client, image_string: &str, is_allowed: bool) {
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
            .post(&format!("{}/validate-image", ORIGIN))
            .body(review)
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

        assert_eq!(
            response.allowed, is_allowed,
            "Wrong response for `{}` ({:?})",
            image_string, response.result.message
        );
    }

    #[tokio::test]
    async fn test_runner() {
        let client = reqwest::Client::new();

        //Had issues with stopping and starting trow causing test fails.
        //It might be possible to improve things with a thread_local
        let _trow = start_trow().await;
        validate_example(&client).await;

        // explicitely allowed
        test_image(&client, "trow.test/am/test:tag", true).await;
        test_image(&client, "k8s.gcr.io/metrics-server-amd64:v0.2.1", true).await;
        test_image(&client, "docker.io/amouat/myimage:test", true).await;
        test_image(&client, "localhost:8000/hello/world", true).await;

        // explicitely denied
        test_image(&client, "localhost:8000/secret/shine-box", false).await;

        // default denied
        test_image(&client, "virus.land.cc/not/suspect", false).await;

        // invalid image ref
        // Some very weird refs are actually technically valid, like `example.com` !
        test_image(&client, "hello human", false).await;
        test_image(&client, "docker.io/voyager@jasper-byrne", false).await;
    }
}
