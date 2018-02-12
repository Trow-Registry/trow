extern crate crypto;
extern crate environment;
extern crate futures;
#[macro_use]
extern crate hyper;
extern crate rand;
extern crate tokio_core;

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
    use hyper::header::{ContentLength, ContentType, Location};
    use hyper::{Client, Error, Method, Request, Response, StatusCode};
    use tokio_core::reactor::Core;
    use rand;
    use rand::Rng;
    use futures::Future;
    use futures::Stream;

    const LYCAON_ADDRESS: &'static str = "http://localhost:8000";

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
        let mut response = get_sync(LYCAON_ADDRESS);
        while timeout > 0 && (response.is_err() || (response.unwrap().status() != StatusCode::Ok)) {
            thread::sleep(Duration::from_millis(100));
            response = get_sync(LYCAON_ADDRESS);
            timeout -= 1;
        }
        if timeout == 0 {
            child.kill().unwrap();
            panic!("Failed to start Lycaon");
        }
        LycaonInstance { pid: child }
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

    fn get_sync(url: &str) -> Result<Response, Error> {
        let mut core = Core::new().expect("Failed to start hyper");
        let client = Client::new(&core.handle());

        let uri = url.parse()?;
        let work = client.get(uri);
        core.run(work)
    }

    /*

    fn get_data_sync(url: &str, out: &mut Write) -> Result<Response, Error> {
        let mut core = Core::new().expect("Failed to start hyper");
        let client = Client::new(&core.handle());

        let uri = url.parse()?;
        let work = client.get(uri).and_then(|res| {
            res.body()
                .for_each(|chunk| out.write_all(&chunk).map(|_| ()).map_err(From::from));
            res
        });
        core.run(work)
    }

    */

    fn post_sync(url: &str) -> Result<Response, Error> {
        let mut core = Core::new().expect("Failed to start hyper");
        let client = Client::new(&core.handle());

        let uri = url.parse()?;
        let req = Request::new(Method::Post, uri);
        let work = client.request(req);
        core.run(work)
    }

    fn delete_sync(url: &str) -> Result<Response, Error> {
        let mut core = Core::new().expect("Failed to start hyper");
        let client = Client::new(&core.handle());

        let uri = url.parse()?;
        let req = Request::new(Method::Delete, uri);
        let work = client.request(req);
        core.run(work)
    }

    fn patch_sync(url: &str, data: &Vec<u8>) -> Result<Response, Error> {
        let mut core = Core::new().expect("Failed to start hyper");
        let client = Client::new(&core.handle());

        let uri = url.parse()?;
        let mut req = Request::new(Method::Patch, uri);

        req.headers_mut().set(ContentType::octet_stream());
        req.headers_mut().set(ContentLength(data.len() as u64));

        req.set_body(data.clone());
        let work = client.request(req);
        core.run(work)
    }

    fn put_sync(url: &str) -> Result<Response, Error> {
        let mut core = Core::new().expect("Failed to start hyper");
        let client = Client::new(&core.handle());

        let uri = url.parse()?;
        let mut req = Request::new(Method::Put, uri);

        req.headers_mut().set(ContentType::octet_stream());
        req.headers_mut().set(ContentLength(0));

        let work = client.request(req);
        core.run(work)
    }

    fn get_main() {
        let resp = get_sync(LYCAON_ADDRESS).unwrap();
        assert_eq!(resp.status(), StatusCode::Ok);
        assert_eq!(
            resp.headers().get::<DistributionApi>().unwrap().0,
            "registry/2.0"
        );

        //All v2 registries should respond with a 200 to this
        let resp = get_sync(&(LYCAON_ADDRESS.to_owned() + "/v2/")).unwrap();
        assert_eq!(resp.status(), StatusCode::Ok);
        assert_eq!(
            resp.headers().get::<DistributionApi>().unwrap().0,
            "registry/2.0"
        );
    }

    fn get_blob() {
        //Currently have stub value in lycaon
        let resp =
            get_sync(&(LYCAON_ADDRESS.to_owned() + "/v2/test/test/blobs/test_digest")).unwrap();
        assert_eq!(resp.status(), StatusCode::Ok);
        //Add test that we get file

        //Try getting something on another instance should redirect

        //Try getting something that doesn't exist
        let resp =
            get_sync(&(LYCAON_ADDRESS.to_owned() + "/v2/test/test/blobs/not-an-entry")).unwrap();
        assert_eq!(resp.status(), StatusCode::NotFound);
    }

    fn unsupported() {
        //Delete currently unimplemented
        let resp =
            delete_sync(&(LYCAON_ADDRESS.to_owned() + "/v2/name/repo/manifests/ref")).unwrap();
        assert_eq!(resp.status(), StatusCode::MethodNotAllowed);
    }

    fn upload_layer() {
        //Should support both image/test and imagetest, only former working currently
        let resp =
            post_sync(&(LYCAON_ADDRESS.to_owned() + "/v2/image/test/blobs/uploads/")).unwrap();
        assert_eq!(resp.status(), StatusCode::Accepted);
        let uuid = resp.headers().get::<UploadUuid>().unwrap();
        let location = resp.headers().get::<Location>().unwrap();
        //PATCH for chunked, PUT for monolithic
        //start with PATCH as don't need digest
        let blob = gen_rand_blob(100);
        let resp = patch_sync(location, &blob).unwrap();
        assert_eq!(resp.status(), StatusCode::Accepted);

        // TODO: digest handling
        let mut hasher = Sha256::new();
        hasher.input(&blob);
        let digest = hasher.result_str();
        let resp = put_sync(&format!(
            "{}/v2/image/test/blobs/uploads/{}?digest={}",
            LYCAON_ADDRESS, uuid, digest
        )).unwrap();
        assert_eq!(resp.status(), StatusCode::Created);

        //Finally get it back again
        let resp = get_sync(&format!(
            "{}/v2/image/test/blobs/{}",
            LYCAON_ADDRESS, digest
        )).unwrap();
        assert_eq!(resp.status(), StatusCode::Ok);
        let mut buf = Vec::new();
        resp.body()
            .for_each(|chunk| buf.write_all(&chunk).map(|_| ()).map_err(From::from)).wait().unwrap();

        assert_eq!(blob, buf);
    }

    #[test]
    fn test_runner() {
        //Had issues with stopping and starting lycaon causing test fails.
        //It might be possible to improve things with a thread_local
        let _lyc = start_lycaon();
        println!("Running get_main()");
        get_main();
        println!("Running get_blob()");
        get_blob();
        println!("Running unsupported()");
        unsupported();
        println!("Running upload_layer()");
        upload_layer();
    }

}
