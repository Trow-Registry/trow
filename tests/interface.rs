/// Test API end-points
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
    use futures::Future;
    use hyper::{Client, StatusCode};
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

    #[test]
    fn get_main() {
        let _lyc = start_lycaon();

        let mut core = Core::new().expect("Failed to start hyper");
        let client = Client::new(&core.handle());

        let uri = "http://localhost:8000"
            .parse()
            .expect("Failure parsing URI");
        let work = client.get(uri).map(|res| {
            assert_eq!(StatusCode::Ok, res.status());
        });
        core.run(work).expect("Failed to run get");

    }

}
