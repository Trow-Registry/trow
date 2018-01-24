extern crate tokio_core;
extern crate hyper;
extern crate futures;
extern crate environment;

#[cfg(test)]
mod interface_tests {

    use environment::Environment;

    use std::process::Command;
    use std::process::Child;
    use std::time::Duration;
    use std::thread;
    use hyper::{Client, StatusCode, Error, Response};
    use tokio_core::reactor::Core;

    struct LycaonInstance {
        pid: Child
    }
    /// Call out to cargo to start lycaon.
    /// Seriously considering moving to docker run.

    fn start_lycaon() -> LycaonInstance {
        let child = Command::new("cargo")
            //.current_dir("../../")
            .arg("run")
            .env_clear()
            .envs(Environment::inherit().compile())
            .spawn()
            .expect("failed to start");

        //FIXME: change to poll for start-up
        thread::sleep(Duration::from_millis(500));
        LycaonInstance{pid: child}
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

    #[test]
    fn get_main() {
        let _lyc = start_lycaon();

        let i = get_sync("http://localhost:8000").unwrap();
        assert_eq!(i.status(), StatusCode::Ok);
        assert_eq!(i.headers().get_raw("Docker-Distribution-API-Version").unwrap(), "registry/2.0");

    }

}
