extern crate crypto;
extern crate environment;
extern crate hyper;
extern crate rand;
extern crate reqwest;
extern crate serde_json;
extern crate trow;
extern crate trow_server;

mod common;

#[cfg(test)]
mod validation_tests {

  use common;
  use environment::Environment;
  use reqwest::StatusCode;
  use std::fs::{self, File};
  use std::io::Read;
  use std::process::Child;
  use std::process::Command;
  use std::thread;
  use std::time::Duration;

  const LYCAON_ADDRESS: &str = "https://trow.test:8443";

  struct TrowInstance {
    pid: Child,
  }
  /// Call out to cargo to start trow.
  /// Seriously considering moving to docker run.

  fn start_trow() -> TrowInstance {
    let mut child = Command::new("cargo")
      .arg("run")
      .env_clear()
      .envs(Environment::inherit().compile())
      .arg("--")
      .arg("--names")
      .arg("trow.test")
      .arg("--allow-prefixes")
      .arg("docker.io/amouat/")
      .arg("--allow-images")
      .arg("docker.io/debian:latest,quay.io/coreos/etcd:3.4")
      .arg("--disallow-local-prefixes")
      .arg("bla/, testy")
      .arg("--disallow-local-images")
      .arg("not:me, or/this:one")
      .spawn()
      .expect("failed to start");

    let mut timeout = 20;

    let mut buf = Vec::new();
    File::open("./certs/ca.crt")
      .unwrap()
      .read_to_end(&mut buf)
      .unwrap();
    let cert = reqwest::Certificate::from_pem(&buf).unwrap();
    // get a client builder
    let client = reqwest::Client::builder()
      .add_root_certificate(cert)
      .build()
      .unwrap();

    let mut response = client.get(LYCAON_ADDRESS).send();
    while timeout > 0 && (response.is_err() || (response.unwrap().status() != StatusCode::OK)) {
      thread::sleep(Duration::from_millis(100));
      response = client.get(LYCAON_ADDRESS).send();
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
      //Y U NO HV STOP?
      self.pid.kill().unwrap();
    }
  }

  /* Uses a copy of an actual AdmissionReview to test. */
  fn validate_example(cl: &reqwest::Client) {
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
            "apiVersion": "extensions/v1beta1",
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
            "image": "nginx",
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

    let mut resp = cl
      .post(&format!("{}/validate-image", LYCAON_ADDRESS))
      .body(review)
      .send()
      .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    //should deny by default
    let txt = resp.text().unwrap();
    assert!(txt.contains("\"allowed\":false"));
    assert!(txt.contains(
      "Remote image nginx disallowed as not contained in this registry and not in allow list"
    ));
  }

  fn test_image(cl: &reqwest::Client, image_string: &str, is_allowed: bool) {
    let start = r#"{
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
        "containers": [
          {
            "name": "test3",
            "image": ""#;
    let end = r#""
          }
        ]
      }
    }
  }
}"#;
    let review = format!("{}{}{}", start, image_string, end);

    let mut resp = cl
      .post(&format!("{}/validate-image", LYCAON_ADDRESS))
      .body(review)
      .send()
      .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    //should deny by default
    let txt = resp.text().unwrap();
    if is_allowed {
      println!("out {}", txt);
      assert!(txt.contains("\"allowed\":true"));
    } else {
      assert!(txt.contains("\"allowed\":false"));
    }
  }

  #[test]
  fn test_runner() {
    //Need to start with empty repo
    fs::remove_dir_all("./data").unwrap_or(());

    let mut buf = Vec::new();
    File::open("./certs/ca.crt")
      .unwrap()
      .read_to_end(&mut buf)
      .unwrap();
    let cert = reqwest::Certificate::from_pem(&buf).unwrap();
    // get a client builder
    let client = reqwest::Client::builder()
      .add_root_certificate(cert)
      .build()
      .unwrap();

    //Had issues with stopping and starting trow causing test fails.
    //It might be possible to improve things with a thread_local
    let _trow = start_trow();
    validate_example(&client);
    test_image(&client, "trow.test/am/test:tag", false);
    //push image and test again
    common::upload_layer(&client, "am/test", "tag");
    test_image(&client, "trow.test/am/test:tag", true);

    //k8s images should be allowed by default
    test_image(&client, "k8s.gcr.io/metrics-server-amd64:v0.2.1", true);

    //allow prefix example
    test_image(&client, "docker.io/amouat/myimage:test", true);
    test_image(&client, "amouat/myimage:test", true);
    test_image(&client, "docker.io/someone/myimage:test", false);

    //allow image example
    test_image(&client, "docker.io/debian:latest", true);
    test_image(&client, "debian", true);
    test_image(&client, "quay.io/coreos/etcd:3.4", true);
    test_image(&client, "quay.io/coreos/etcd:3.4.2", false);

    //deny prefix example
    common::upload_layer(&client, "bla/test", "tag");
    common::upload_layer(&client, "blatest", "tag");
    test_image(&client, "trow.test/bla/test:tag", false);
    test_image(&client, "trow.test/blatest:tag", true);
    common::upload_layer(&client, "testy/test", "tag");
    common::upload_layer(&client, "testytest", "tag");
    test_image(&client, "trow.test/testy/test:tag", false);
    test_image(&client, "trow.test/testytest:tag", false);

    //deny image example
    common::upload_layer(&client, "not", "me");
    common::upload_layer(&client, "or/this", "one");
    test_image(&client, "trow.test/not:me", false);
    test_image(&client, "trow.test/or/this:one", false);
  }
}
