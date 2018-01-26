extern crate environment;
extern crate futures;
extern crate hyper;
extern crate tokio_core;

#[cfg(test)]
mod interface_tests {

    use environment::Environment;

    use std::process::Command;
    use std::process::Child;
    use std::time::Duration;
    use std::thread;
    use hyper::{Client, Error, Method, Request, Response, StatusCode};
    use tokio_core::reactor::Core;

    const LYCAON_ADDRESS: &'static str = "http://localhost:8000";

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

    fn get_sync(url: &str) -> Result<Response, Error> {
        let mut core = Core::new().expect("Failed to start hyper");
        let client = Client::new(&core.handle());

        let uri = url.parse()?;
        let work = client.get(uri);
        core.run(work)
    }

    fn post_sync(url: &str) -> Result<Response, Error> {
        let mut core = Core::new().expect("Failed to start hyper");
        let client = Client::new(&core.handle());

        let uri = url.parse()?;
        let req = Request::new(Method::Post, uri);
        let work = client.request(req);
        core.run(work)
    }

    //#[test]
    fn get_main() {
        //let _lyc = start_lycaon();

        let resp = get_sync(LYCAON_ADDRESS).unwrap();
        assert_eq!(resp.status(), StatusCode::Ok);
        assert_eq!(
            resp.headers()
                .get_raw("Docker-Distribution-API-Version")
                .unwrap(),
            "registry/2.0"
        );

        //All v2 registries should respond with a 200 to this
        let resp = get_sync(&(LYCAON_ADDRESS.to_owned() + "/v2/")).unwrap();
        assert_eq!(resp.status(), StatusCode::Ok);
        assert_eq!(
            resp.headers()
                .get_raw("Docker-Distribution-API-Version")
                .unwrap(),
            "registry/2.0"
        );
    }

    //#[test]
    fn get_blob() {
        //let _lyc = start_lycaon();

        //Currently have stub value in lycaon
        let resp =
            get_sync(&(LYCAON_ADDRESS.to_owned() + "/v2/test/test/blobs/test_digest")).unwrap();
        assert_eq!(resp.status(), StatusCode::Ok);
        assert_eq!(
            resp.headers()
                .get_raw("Docker-Distribution-API-Version")
                .unwrap(),
            "registry/2.0"
        );

        //Try getting something that doesn't exist
        let resp =
            get_sync(&(LYCAON_ADDRESS.to_owned() + "/v2/test/test/blobs/not-an-entry")).unwrap();
        assert_eq!(resp.status(), StatusCode::Ok);
        assert_eq!(
            resp.headers()
                .get_raw("Docker-Distribution-API-Version")
                .unwrap(),
            "registry/2.0"
        );
    }

    #[test]
    #[ignore]
    fn upload_layer() {
        

        let resp =
            post_sync(&(LYCAON_ADDRESS.to_owned() + "/v2/imagetest/blobs/uploads/")).unwrap();

        //should give 202 accepted

        let _uuid = resp.headers().get_raw("Docker-Upload-Uuid").unwrap();
        //assert uuid in request.headers.get("Location")

        //return request.headers.get("Location")
    }

    #[test]
    fn test_runner() {
        //Had issues with stopping and starting lycaon causing test fails.
        //It might be able to improve things with a thread_local variable.
        let _lyc = start_lycaon();
        get_main();
        get_blob();

    }

}
