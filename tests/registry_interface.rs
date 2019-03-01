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
mod interface_tests {

    use environment::Environment;

    use common;
    use reqwest::StatusCode;
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

    const TROW_ADDRESS: &str = "https://trow.test:8443";

    /*
    header! { (DistributionApi, "Docker-Distribution-API-Version") => [String] }
    header! { (UploadUuid, "Docker-Upload-Uuid") => [String] }
    */
    const DIST_API_HEADER: &str = "Docker-Distribution-API-Version";
    const UPLOAD_HEADER: &str = "Docker-Upload-Uuid";

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
            //Y U NO HV STOP?
            self.pid.kill().unwrap();
        }
    }

    fn get_main(cl: &reqwest::Client) {
        let resp = cl.get(TROW_ADDRESS).send().unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(DIST_API_HEADER).unwrap(),
            "registry/2.0"
        );

        //All v2 registries should respond with a 200 to this
        let resp = cl
            .get(&(TROW_ADDRESS.to_owned() + "/v2/"))
            .send()
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(DIST_API_HEADER).unwrap(),
            "registry/2.0"
        );
    }

    fn get_non_existent_blob(cl: &reqwest::Client) {
        let resp = cl
            .get(&(TROW_ADDRESS.to_owned() + "/v2/test/test/blobs/not-an-entry"))
            .send()
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    fn unsupported(cl: &reqwest::Client) {
        //Delete currently unimplemented
        let resp = cl
            .delete(&(TROW_ADDRESS.to_owned() + "/v2/name/repo/manifests/ref"))
            .send()
            .unwrap();
        assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
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
            .get(&format!(
                "{}/v2/{}/tags/list",
                TROW_ADDRESS,
                tl.repo_name()
            ))
            .send()
            .unwrap();
        let tl_resp: TagList = serde_json::from_str(&resp.text().unwrap()).unwrap();
        assert_eq!(tl, &tl_resp);
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
        common::upload_layer(&client, "repo/image/test", "tag");
        println!("Running upload_layer(image/test:latest)");
        common::upload_layer(&client, "image/test", "latest");
        println!("Running upload_layer(onename:tag)");
        common::upload_layer(&client, "onename", "tag");
        println!("Running upload_layer(onename:latest)");
        common::upload_layer(&client, "onename", "latest");
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
