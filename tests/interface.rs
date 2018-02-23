extern crate crypto;
extern crate environment;
extern crate futures;
#[macro_use]
extern crate hyper;
extern crate hypersync;
extern crate lycaon;
extern crate rand;
extern crate serde_json;

#[cfg(test)]
mod interface_tests {

    use crypto::digest::Digest;
    use crypto::sha2::Sha256;

    use environment::Environment;

    use std::process::Command;
    use std::process::Child;
    use std::time::Duration;
    use std::thread;
    use std::io::Write;
    use hyper::header::Location;
    use hyper::StatusCode;
    use hypersync::hypersync;
    use rand;
    use rand::Rng;
    use futures::Future;
    use futures::Stream;
    use lycaon::manifest;
    use serde_json;

    const LYCAON_ADDRESS: &'static str = "http://localhost:8000";
    // const MANIFEST_TEMPLATE: &'static str = "./tests/manifest-template.json";

    header! { (DistributionApi, "Docker-Distribution-API-Version") => [String] }
    header! { (UploadUuid, "Docker-Upload-Uuid") => [String] }

    struct LycaonInstance {
        pid: Child,
    }
    /// Call out to cargo to start lycaon.
    /// Seriously considering moving to docker run.

    fn start_lycaon() -> LycaonInstance {
        let mut child = Command::new("cargo")
            //.current_dir("../../")
            .arg("run")
            .env_clear()
            .envs(Environment::inherit().compile())
            .spawn()
            .expect("failed to start");

        let mut timeout = 20;
        let mut response = hypersync::get(LYCAON_ADDRESS);
        while timeout > 0 && (response.is_err() || (response.unwrap().status() != StatusCode::Ok)) {
            thread::sleep(Duration::from_millis(100));
            response = hypersync::get(LYCAON_ADDRESS);
            timeout -= 1;
        }
        if timeout == 0 {
            child.kill().unwrap();
            panic!("Failed to start Lycaon");
        }
        LycaonInstance { pid: child }
    }

    fn setup() {
        // create dummy layer
        use std::fs;
        fs::create_dir_all("./data/layers/test/test").unwrap();
        fs::File::create("./data/layers/test/test/test_digest").unwrap();
    }

    impl Drop for LycaonInstance {
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

    fn get_main() {
        let resp = hypersync::get(LYCAON_ADDRESS).unwrap();
        assert_eq!(resp.status(), StatusCode::Ok);
        assert_eq!(
            resp.headers().get::<DistributionApi>().unwrap().0,
            "registry/2.0"
        );

        //All v2 registries should respond with a 200 to this
        let resp = hypersync::get(&(LYCAON_ADDRESS.to_owned() + "/v2/")).unwrap();
        assert_eq!(resp.status(), StatusCode::Ok);
        assert_eq!(
            resp.headers().get::<DistributionApi>().unwrap().0,
            "registry/2.0"
        );
    }

    fn get_non_existent_blob() {
        let resp = hypersync::get(
            &(LYCAON_ADDRESS.to_owned() + "/v2/test/test/blobs/not-an-entry"),
        ).unwrap();
        assert_eq!(resp.status(), StatusCode::NotFound);
    }

    fn unsupported() {
        //Delete currently unimplemented
        let resp = hypersync::delete(&(LYCAON_ADDRESS.to_owned() + "/v2/name/repo/manifests/ref"))
            .unwrap();
        assert_eq!(resp.status(), StatusCode::MethodNotAllowed);
    }

    fn upload_layer() {
        //Should support both image/test and imagetest, only former working currently
        let resp = hypersync::post(&(LYCAON_ADDRESS.to_owned() + "/v2/image/test/blobs/uploads/"))
            .unwrap();
        assert_eq!(resp.status(), StatusCode::Accepted);
        let uuid = resp.headers().get::<UploadUuid>().unwrap();
        let location = resp.headers().get::<Location>().unwrap();

        //Upload file. Start uploading blob with patch then digest with put
        let blob = gen_rand_blob(100);
        let resp = hypersync::patch(location, &blob).unwrap();
        assert_eq!(resp.status(), StatusCode::Accepted);

        let mut hasher = Sha256::new();
        hasher.input(&blob);
        let digest = hasher.result_str();
        let resp = hypersync::put(&format!(
            "{}/v2/image/test/blobs/uploads/{}?digest={}",
            LYCAON_ADDRESS, uuid, digest), &Vec::new()).unwrap();
        assert_eq!(resp.status(), StatusCode::Created);

        //Finally get it back again
        let resp = hypersync::get(&format!(
            "{}/v2/image/test/blobs/{}",
            LYCAON_ADDRESS, digest
        )).unwrap();
        assert_eq!(resp.status(), StatusCode::Ok);
        let mut buf = Vec::new();
        resp.body()
            .for_each(|chunk| buf.write_all(&chunk).map(|_| ()).map_err(From::from))
            .wait()
            .unwrap();

        assert_eq!(blob, buf);

        //Upload manifest
        //For time being use same blog for config and layer
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
        let mut layer_vec = Vec::new();
        layer_vec.push(layer);
        let layers = Box::new(layer_vec);
        let mani = manifest::ManifestV2 {
            schema_version: 2,
            media_type: "application/vnd.docker.distribution.manifest.v2+json".to_owned(),
            config,
            layers,
        };
        let resp = hypersync::put(&format!(
            "{}/v2/image/test/manifests/test", LYCAON_ADDRESS), &serde_json::to_vec(&mani).unwrap()
        ).unwrap();
        assert_eq!(resp.status(), StatusCode::Created);

    }

    #[test]
    fn test_runner() {
        //Had issues with stopping and starting lycaon causing test fails.
        //It might be possible to improve things with a thread_local
        let _lyc = start_lycaon();
        setup();
        println!("Running get_main()");
        get_main();
        println!("Running get_blob()");
        get_non_existent_blob();
        println!("Running unsupported()");
        unsupported();
        println!("Running upload_layer()");
        upload_layer();
        println!("Running upload_manifest()");
    }

}
