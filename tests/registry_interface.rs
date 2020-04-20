extern crate crypto;
extern crate environment;
extern crate hyper;
extern crate rand;
extern crate reqwest;
extern crate serde_json;

#[cfg(test)]
mod common;

#[cfg(test)]
mod interface_tests {

    use environment::Environment;

    use crate::common;
    use crypto::digest::Digest;
    use crypto::sha2::Sha256;
    use reqwest;
    use reqwest::StatusCode;
    use serde_json;
    use std::fs::{self, File};
    use std::io::Read;
    use std::process::Child;
    use std::process::Command;
    use std::thread;
    use std::time::Duration;
    use trow::types::{RepoCatalog, RepoName, TagList};
    use trow_server::manifest;

    const TROW_ADDRESS: &str = "https://trow.test:8443";
    const DIST_API_HEADER: &str = "Docker-Distribution-API-Version";

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
            .spawn()
            .expect("failed to start");

        let mut timeout = 20;

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

        let mut response = client.get(TROW_ADDRESS).send();
        while timeout > 0 && (response.is_err() || (response.unwrap().status() != StatusCode::OK)) {
            thread::sleep(Duration::from_millis(100));
            response = client.get(TROW_ADDRESS).send();
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

    fn get_main(cl: &reqwest::Client) {
        let resp = cl.get(TROW_ADDRESS).send().unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get(DIST_API_HEADER).unwrap(), "registry/2.0");

        //All v2 registries should respond with a 200 to this
        let resp = cl.get(&(TROW_ADDRESS.to_owned() + "/v2/")).send().unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.headers().get(DIST_API_HEADER).unwrap(), "registry/2.0");
    }

    fn get_non_existent_blob(cl: &reqwest::Client) {
        let resp = cl
            .get(&(TROW_ADDRESS.to_owned() + "/v2/test/test/blobs/not-an-entry"))
            .send()
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    fn get_manifest(cl: &reqwest::Client, name: &str, tag: &str) {
        //Might need accept headers here
        let mut resp = cl
            .get(&format!("{}/v2/{}/manifests/{}", TROW_ADDRESS, name, tag))
            .send()
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let mani: manifest::ManifestV2 = resp.json().unwrap();
        assert_eq!(mani.schema_version, 2);
    }

    fn get_non_existent_manifest(cl: &reqwest::Client, name: &str, tag: &str) {
        //Might need accept headers here
        let resp = cl
            .get(&format!("{}/v2/{}/manifests/{}", TROW_ADDRESS, name, tag))
            .send()
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    fn check_repo_catalog(cl: &reqwest::Client, rc: &RepoCatalog) {
        let mut resp = cl
            .get(&format!("{}/v2/_catalog", TROW_ADDRESS))
            .send()
            .unwrap();
        let rc_resp: RepoCatalog = serde_json::from_str(&resp.text().unwrap()).unwrap();
        assert_eq!(rc, &rc_resp);
    }

    fn check_tag_list(cl: &reqwest::Client, tl: &TagList) {
        let mut resp = cl
            .get(&format!("{}/v2/{}/tags/list", TROW_ADDRESS, tl.repo_name()))
            .send()
            .unwrap();
        let tl_resp: TagList = serde_json::from_str(&resp.text().unwrap()).unwrap();
        assert_eq!(tl, &tl_resp);
    }

    fn check_tag_list_n_last(cl: &reqwest::Client, n: u32, last: &str, tl: &TagList) {
        let mut resp = cl
            .get(&format!("{}/v2/{}/tags/list?last={}&n={}", TROW_ADDRESS, tl.repo_name(), last, n))
            .send()
            .unwrap();
        let tl_resp: TagList = serde_json::from_str(&resp.text().unwrap()).unwrap();
        assert_eq!(tl, &tl_resp);
    }

    fn upload_with_put(cl: &reqwest::Client, name: &str) {
        let resp = cl
            .post(&format!("{}/v2/{}/blobs/uploads/", TROW_ADDRESS, name))
            .send()
            .unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
        let uuid = resp
            .headers()
            .get(common::UPLOAD_HEADER)
            .unwrap()
            .to_str()
            .unwrap();

        //used by oci_manifest_test
        let config = "{}\n".as_bytes();
        let mut hasher = Sha256::new();
        hasher.input(&config);
        let digest = format!("sha256:{}", hasher.result_str());
        let loc = &format!(
            "{}/v2/{}/blobs/uploads/{}?digest={}",
            TROW_ADDRESS, name, uuid, digest
        );

        let resp = cl.put(loc).body(config.clone()).send().unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    fn upload_with_post(cl: &reqwest::Client, name: &str) {

        let config = "{ }\n".as_bytes();
        let mut hasher = Sha256::new();
        hasher.input(&config);
        let digest = format!("sha256:{}", hasher.result_str());
        let resp = cl
            .post(&format!("{}/v2/{}/blobs/uploads/?digest={}", TROW_ADDRESS, name, digest))
            .body(config.clone())
            .send()
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    fn push_oci_manifest(cl: &reqwest::Client, name: &str, tag: &str) -> String {
        //Note config was uploaded as blob in earlier test
        let config = "{}\n".as_bytes();
        let mut hasher = Sha256::new();
        hasher.input(&config);
        let config_digest = format!("sha256:{}", hasher.result_str());

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
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        // Try pulling by digest
        hasher.reset();
        hasher.input(manifest.as_bytes());

        let digest = format!("sha256:{}", hasher.result_str());
        digest
    }

    fn push_manifest_list(cl: &reqwest::Client, digest: &str, name: &str, tag: &str) -> String {
     
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
            digest);
        let bytes = manifest.clone();
        let resp = cl
            .put(&format!("{}/v2/{}/manifests/{}", TROW_ADDRESS, name, tag))
            .body(bytes)
            .send()
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        // Try pulling by digest
        let mut hasher = Sha256::new();
        hasher.input(manifest.as_bytes());

        let digest = format!("sha256:{}", hasher.result_str());
        digest

    }

    fn push_oci_manifest_with_foreign_blob(cl: &reqwest::Client, name: &str, tag: &str) -> String {
        //Note config was uploaded as blob in earlier test
        let config = "{}\n".as_bytes();
        let mut hasher = Sha256::new();
        hasher.input(&config);
        let config_digest = format!("sha256:{}", hasher.result_str());

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
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        // Try pulling by digest
        hasher.reset();
        hasher.input(manifest.as_bytes());

        let digest = format!("sha256:{}", hasher.result_str());
        digest

    }

    fn delete_manifest(cl: &reqwest::Client, name: &str, digest: &str) {
        let resp = cl
            .delete(&format!(
                "{}/v2/{}/manifests/{}",
                TROW_ADDRESS, name, digest
            ))
            .send()
            .unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
    }

    fn delete_non_existent_manifest(cl: &reqwest::Client, name: &str) {
        let resp = cl
            .delete(&format!(
                "{}/v2/{}/manifests/{}",
                TROW_ADDRESS, name, "sha256:9038b92872bc268d5c975e84dd94e69848564b222ad116ee652c62e0c2f894b2"
            ))
            .send()
            .unwrap();
        // If it doesn't exist, that's kinda the same as deleted, right?
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
    }
    
    fn attempt_delete_by_tag(cl: &reqwest::Client, name: &str, tag: &str) {
        let resp = cl
            .delete(&format!(
                "{}/v2/{}/manifests/{}",
                TROW_ADDRESS, name, tag
            ))
            .send()
            .unwrap();
        assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    fn delete_config_blob(cl: &reqwest::Client, name: &str) {

        //Deletes blob uploaded in config test
        let config = "{}\n".as_bytes();
        let mut hasher = Sha256::new();
        hasher.input(&config);
        let config_digest = format!("sha256:{}", hasher.result_str());
        
        let resp = cl
            .delete(&format!("{}/v2/{}/blobs/{}", TROW_ADDRESS, name, config_digest))
            .send()
            .unwrap();
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
        
    }

    #[test]
    fn test_runner() {
        //Need to start with empty repo
        fs::remove_dir_all("./data").unwrap_or(());

        //Had issues with stopping and starting trow causing test fails.
        //It might be possible to improve things with a thread_local
        let _trow = start_trow();

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
        get_main(&client);
        println!("Running get_blob()");
        get_non_existent_blob(&client);

        println!("Running upload_layer(fourth/repo/image/test:tag)");
        common::upload_layer(&client, "fourth/repo/image/test", "tag");
        println!("Running upload_layer(repo/image/test:tag)");
        common::upload_layer(&client, "repo/image/test", "tag");
        println!("Running upload_layer(image/test:latest)");
        common::upload_layer(&client, "image/test", "latest");
        println!("Running upload_layer(onename:tag)");
        common::upload_layer(&client, "onename", "tag");
        println!("Running upload_layer(onename:latest)");
        common::upload_layer(&client, "onename", "latest");
        println!("Running upload_with_put()");
        upload_with_put(&client, "puttest");
        println!("Running upload_with_post");
        upload_with_post(&client, "posttest");

        println!("Running push_oci_manifest()");
        let digest = push_oci_manifest(&client, "puttest", "puttest1");
        println!("Running push_manifest_list()");
        let digest_list = push_manifest_list(&client, &digest, "listtest", "listtest1");
        println!("Running get_manifest(puttest:puttest1)");
        get_manifest(&client, "puttest", "puttest1");
        println!("Running delete_manifest(puttest:digest)");
        delete_manifest(&client, "puttest", &digest);
        println!("Running delete_manifest(listtest)");
        delete_manifest(&client, "listtest", &digest_list);
        println!("Running delete_non_existent_manifest(onename)");
        delete_non_existent_manifest(&client, "onename");
        println!("Running attempt_delete_by_tag(onename:tag)");
        attempt_delete_by_tag(&client, "onename", "tag");
        println!("Running get_non_existent_manifest(puttest:puttest1)");
        get_non_existent_manifest(&client, "puttest", "puttest1");
        println!("Running get_non_existent_manifest(puttest:digest)");
        get_non_existent_manifest(&client, "puttest", &digest);

        println!("Running push_oci_manifest_with_foreign_blob()");
        let digest = push_oci_manifest_with_foreign_blob(&client, "foreigntest", "blobtest1");
        delete_manifest(&client, "foreigntest", &digest);

        println!("Running delete_config_blob");
        delete_config_blob(&client, "puttest");

        println!("Running get_manifest(onename:tag)");
        get_manifest(&client, "onename", "tag");
        println!("Running get_manifest(image/test:latest)");
        get_manifest(&client, "image/test", "latest");
        println!("Running get_manifest(repo/image/test:tag)");
        get_manifest(&client, "repo/image/test", "tag");

        let mut rc = RepoCatalog::new();
        rc.insert(RepoName("fourth/repo/image/test".to_string()));
        rc.insert(RepoName("repo/image/test".to_string()));
        rc.insert(RepoName("image/test".to_string()));
        rc.insert(RepoName("onename".to_string()));


        println!("Running check_repo_catalog");
        check_repo_catalog(&client, &rc);

        let mut tl = TagList::new(RepoName("repo/image/test".to_string()));
        tl.insert("tag".to_string());
        println!("Running check_tag_list 1");
        check_tag_list(&client, &tl);

        
        common::upload_layer(&client, "onename", "three");
        common::upload_layer(&client, "onename", "four");

        // list, in order should be [four, latest, tag, three]
        let mut tl2 = TagList::new(RepoName("onename".to_string()));
        tl2.insert("four".to_string());
        tl2.insert("latest".to_string());
        tl2.insert("tag".to_string());
        tl2.insert("three".to_string());

        println!("Running check_tag_list 2");
        check_tag_list(&client, &tl2);
        
        let mut tl3 = TagList::new(RepoName("onename".to_string()));
        tl3.insert("four".to_string());
        tl3.insert("latest".to_string());
        

        println!("Running check_tag_list_n_last 3");
        check_tag_list_n_last(&client, 2, "", &tl3);
        let mut tl4 = TagList::new(RepoName("onename".to_string()));
        tl4.insert("tag".to_string());
        tl4.insert("three".to_string());
        
        println!("Running check_tag_list_n_last 4");
        check_tag_list_n_last(&client, 2, "latest", &tl4)
    }
}
