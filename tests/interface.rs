extern crate crypto;
extern crate environment;
#[macro_use]
extern crate hyper;
extern crate rand;
extern crate reqwest;
extern crate serde_json;
extern crate trow;
extern crate trow_server;

#[cfg(test)]
mod interface_tests {

    use crypto::digest::Digest;
    use crypto::sha2::Sha256;

    use environment::Environment;

    use hyper::header::Location;
    use hyper::StatusCode;
    use rand;
    use rand::Rng;
    use reqwest;
    use serde_json;
    use std::fs::{self, File};
    use std::io::Read;
    use std::process::Child;
    use std::process::Command;
    use std::thread;
    use std::time::Duration;
    use trow::types::{RepoCatalog, RepoName, TagList};
    use trow_server::manifest;

    const LYCAON_ADDRESS: &str = "https://trow.test:8443";

    header! { (DistributionApi, "Docker-Distribution-API-Version") => [String] }
    header! { (UploadUuid, "Docker-Upload-Uuid") => [String] }

    struct TrowInstance {
        pid: Child,
    }
    /// Call out to cargo to start trow.
    /// Seriously considering moving to docker run.

    fn start_trow() -> TrowInstance {
        let mut child = Command::new("cargo")
            //.current_dir("../../")
            .arg("run")
            .env_clear()
            .envs(Environment::inherit().compile())
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
        while timeout > 0 && (response.is_err() || (response.unwrap().status() != StatusCode::Ok)) {
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

    fn gen_rand_blob(size: usize) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let mut blob = Vec::with_capacity(size);
        for _ in 0..size {
            blob.push(rng.gen::<u8>());
        }
        blob
    }

    fn get_main(cl: &reqwest::Client) {
        let resp = cl.get(LYCAON_ADDRESS).send().unwrap();
        assert_eq!(resp.status(), StatusCode::Ok);
        assert_eq!(
            resp.headers().get::<DistributionApi>().unwrap().0,
            "registry/2.0"
        );

        //All v2 registries should respond with a 200 to this
        let resp = cl
            .get(&(LYCAON_ADDRESS.to_owned() + "/v2/"))
            .send()
            .unwrap();
        assert_eq!(resp.status(), StatusCode::Ok);
        assert_eq!(
            resp.headers().get::<DistributionApi>().unwrap().0,
            "registry/2.0"
        );
    }

    fn get_non_existent_blob(cl: &reqwest::Client) {
        let resp = cl
            .get(&(LYCAON_ADDRESS.to_owned() + "/v2/test/test/blobs/not-an-entry"))
            .send()
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NotFound);
    }

    fn unsupported(cl: &reqwest::Client) {
        //Delete currently unimplemented
        let resp = cl
            .delete(&(LYCAON_ADDRESS.to_owned() + "/v2/name/repo/manifests/ref"))
            .send()
            .unwrap();
        assert_eq!(resp.status(), StatusCode::MethodNotAllowed);
    }

    fn upload_layer(cl: &reqwest::Client, name: &str, tag: &str) {
        //Should support both image/test and imagetest, only former working currently
        let resp = cl
            .post(&format!("{}/v2/{}/blobs/uploads/", LYCAON_ADDRESS, name))
            .send()
            .unwrap();
        assert_eq!(resp.status(), StatusCode::Accepted);
        let uuid = resp.headers().get::<UploadUuid>().unwrap().to_string();
        let location = resp.headers().get::<Location>().unwrap().to_string();

        //Upload file. Start uploading blob with patch then digest with put
        let blob = gen_rand_blob(100);
        let resp = cl
            .patch(location.as_str())
            .body(blob.clone())
            .send()
            .unwrap();
        assert_eq!(resp.status(), StatusCode::Accepted);

        let mut hasher = Sha256::new();
        hasher.input(&blob);
        let digest = hasher.result_str();
        let resp = cl
            .put(&format!(
                "{}/v2/{}/blobs/uploads/{}?digest={}",
                LYCAON_ADDRESS, name, uuid, digest
            )).send()
            .unwrap();
        assert_eq!(resp.status(), StatusCode::Created);

        //Finally get it back again
        let mut resp = cl
            .get(&format!("{}/v2/{}/blobs/{}", LYCAON_ADDRESS, name, digest))
            .send()
            .unwrap();
        assert_eq!(resp.status(), StatusCode::Ok);

        let mut buf = Vec::new();
        resp.copy_to(&mut buf).unwrap();

        assert_eq!(blob, buf);

        //Upload manifest
        //For time being use same blob for config and layer
        let config = manifest::Object {
            media_type: "application/vnd.docker.container.image.v1+json".to_owned(),
            size: blob.len() as u64,
            digest: digest.clone(),
        };
        let layer = manifest::Object {
            media_type: "application/vnd.docker.image.rootfs.diff.tar.gzip".to_owned(),
            size: blob.len() as u64,
            digest: digest.clone(),
        };
        let mut layers = Vec::new();
        layers.push(layer);
        let mani = manifest::ManifestV2 {
            schema_version: 2,
            media_type: "application/vnd.docker.distribution.manifest.v2+json".to_owned(),
            config,
            layers,
        };
        let manifest_addr = format!("{}/v2/{}/manifests/{}", LYCAON_ADDRESS, name, tag);
        let resp = cl.put(&manifest_addr).json(&mani).send().unwrap();
        assert_eq!(resp.status(), StatusCode::Created);
        let location = resp.headers().get::<Location>().unwrap().to_string();
        assert_eq!(&location, &manifest_addr);
    }

    fn get_manifest(cl: &reqwest::Client, name: &str, tag: &str) {
        //Might need accept headers here
        let mut resp = cl
            .get(&format!("{}/v2/{}/manifests/{}", LYCAON_ADDRESS, name, tag))
            .send()
            .unwrap();
        assert_eq!(resp.status(), StatusCode::Ok);
        let mani: manifest::ManifestV2 = resp.json().unwrap();
        assert_eq!(mani.schema_version, 2);
    }

    fn check_repo_catalog(cl: &reqwest::Client, rc: &RepoCatalog) {
        let mut resp = cl
            .get(&format!("{}/v2/_catalog", LYCAON_ADDRESS))
            .send()
            .unwrap();
        let rc_resp: RepoCatalog = serde_json::from_str(&resp.text().unwrap()).unwrap();
        assert_eq!(rc, &rc_resp);
    }

    fn check_tag_list(cl: &reqwest::Client, tl: &TagList) {
        let mut resp = cl
            .get(&format!(
                "{}/v2/{}/tags/list",
                LYCAON_ADDRESS,
                tl.repo_name()
            )).send()
            .unwrap();
        let tl_resp: TagList = serde_json::from_str(&resp.text().unwrap()).unwrap();
        assert_eq!(tl, &tl_resp);
    }

    fn validate_image(cl: &reqwest::Client) {
        
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
    }

    #[test]
    fn test_runner() {
        //Need to start with empty repo
        fs::remove_dir_all("./data").unwrap_or(());

        //Had issues with stopping and starting trow causing test fails.
        //It might be possible to improve things with a thread_local
        let _trow = start_trow();

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

        println!("Running get_main()");
        get_main(&client);
        println!("Running get_blob()");
        get_non_existent_blob(&client);
        println!("Running unsupported()");
        unsupported(&client);
        println!("Running upload_layer(repo/image/test:tag)");
        upload_layer(&client, "repo/image/test", "tag");
        println!("Running upload_layer(image/test:latest)");
        upload_layer(&client, "image/test", "latest");
        println!("Running upload_layer(onename:tag)");
        upload_layer(&client, "onename", "tag");
        println!("Running upload_layer(onename:latest)");
        upload_layer(&client, "onename", "latest");
        println!("Running get_manifest(onename:tag)");
        get_manifest(&client, "onename", "tag");
        println!("Running get_manifest(image/test:latest)");
        get_manifest(&client, "image/test", "latest");
        println!("Running get_manifest(repo/image/test:tag)");
        get_manifest(&client, "repo/image/test", "tag");

        let mut rc = RepoCatalog::new();
        rc.insert(RepoName("repo/image/test".to_string()));
        rc.insert(RepoName("image/test".to_string()));
        rc.insert(RepoName("onename".to_string()));

        check_repo_catalog(&client, &rc);

        let mut tl = TagList::new(RepoName("repo/image/test".to_string()));
        tl.insert("tag".to_string());
        check_tag_list(&client, &tl);

        let mut tl2 = TagList::new(RepoName("onename".to_string()));
        tl2.insert("tag".to_string());
        tl2.insert("latest".to_string());
        check_tag_list(&client, &tl2);
    }

}
