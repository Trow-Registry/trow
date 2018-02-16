extern crate crypto;
extern crate environment;
extern crate futures;
#[macro_use]
extern crate hyper;
extern crate jwt;
extern crate rustc_serialize;
extern crate rand;
extern crate hypersync;

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

    // fn generate_manifest(digest: &str) -> Result<Value, ()> {
    //     use std::fs::File;
    //     use std::io::Read;
    //     let mut template = File::open(MANIFEST_TEMPLATE).unwrap();
    //     let mut contents = String::new();
    //     template.read_to_string(&mut contents).unwrap();
    //     let mut v: Value = serde_json::from_str(&contents).unwrap();
    //     v["fsLayers"][0]["blobSum"] = Value::String(digest.to_string());
    //     Ok(v)
    // }

    #[derive(Clone, Debug, Default, RustcDecodable, RustcEncodable)]
    struct EmptyStruct {}

    #[derive(Clone, Debug, Default, RustcDecodable, RustcEncodable)]
    #[allow(non_snake_case)]
    struct BlobSummary {
        blobSum: String
    }

    #[derive(Debug, RustcDecodable, RustcEncodable)]
    #[allow(non_snake_case)]
    struct SigningManifest {
        schemaVersion: u8,
        name: String,
        tag: String,
        architecture: String,
        fsLayers: Vec<BlobSummary>,
        history: Vec<EmptyStruct>
    }

    impl Default for SigningManifest {
        fn default() -> Self {
            SigningManifest {
                schemaVersion: 1,
                name: "test/test".to_owned(),
                tag: "latest".to_owned(),
                architecture: "amd64".to_owned(),
                fsLayers: Default::default(),
                history: Default::default(),
            }
        }
    }

    #[derive(Debug, Default)]
    struct SignatureJWK {
        crv: String,
        kty: String,
        x: String,
        y: String,
    }

    #[derive(Debug, Default)]
    struct SignatureHeader {
        alg: String,
        jwk: SignatureJWK
    }

    #[derive(Debug, Default)]
    struct Signature {
        header: SignatureHeader,
        payload: String,
        protected: String,
        signature: String,
    }

    #[derive(Debug, Default)]
    #[allow(non_snake_case)]
    struct Manifest {
        schemaVersion: u8,
        name: String,
        tag: String,
        architecture: String,
        fsLayers: Vec<BlobSummary>,
        history: Vec<EmptyStruct>,
        signatures: Vec<Signature>,

    }

    impl Manifest {
        /* transformation function from SigningManifest -> Manifest
         * Possibly would be worth it to think about being able to
         * statically catch incorrect transformations when structs
         * change shape. ie. missed values in the transformation...
         */
        fn from_signing_manifest(sig: &SigningManifest) -> Self {
            Manifest {
                schemaVersion: sig.schemaVersion.to_owned(),
                name: sig.name.to_owned(),
                tag: sig.tag.to_owned(),
                architecture: sig.architecture.to_owned(),
                fsLayers: sig.fsLayers.clone(),
                history: sig.history.clone(),

                signatures: Default::default(),
            }
        }

    }

    fn sign_manifest(digest: &str) {
        // copying from https://github.com/ContainerSolutions/manifest-sample-python/blob/master/construct-manifest.py
        use jwt::{Header, Token};
        // let format_length = manifest_str.len() - 1;
        // let format_tail = '}';
        let header: Header = Default::default();
        let claims = SigningManifest {
            fsLayers: vec!(BlobSummary { blobSum: digest.to_owned() }),
            ..Default::default()
        };

        let manifest = Manifest::from_signing_manifest(&claims);
        let token = Token::new(header, claims);


        println!("{:?}", manifest);

        let signed = token.signed(b"secret_key", Sha256::new()).ok();

        println!("{:?}", signed);
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
        let resp = hypersync::post(
            &(LYCAON_ADDRESS.to_owned() + "/v2/image/test/blobs/uploads/"),
        ).unwrap();
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
            LYCAON_ADDRESS,
            uuid,
            digest
        )).unwrap();
        assert_eq!(resp.status(), StatusCode::Created);

        //Finally get it back again
        let resp = hypersync::get(&format!(
            "{}/v2/image/test/blobs/{}",
            LYCAON_ADDRESS,
            digest
        )).unwrap();
        assert_eq!(resp.status(), StatusCode::Ok);
        let mut buf = Vec::new();
        resp.body()
            .for_each(|chunk| {
                buf.write_all(&chunk).map(|_| ()).map_err(From::from)
            })
            .wait()
            .unwrap();

        assert_eq!(blob, buf);
    }

    fn upload_manifest() {
        // manifest with invalid layer
        sign_manifest("invalid_digest");
        // manifest with test/test/test_digest layer
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
        get_blob();
        println!("Running unsupported()");
        unsupported();
        println!("Running upload_layer()");
        upload_layer();
        println!("Running upload_manifest()");
        upload_manifest();
    }

}
