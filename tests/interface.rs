/// Test API end-points
extern crate environment;
extern crate futures;
extern crate curl;

#[cfg(test)]
mod interface_tests {

    use environment::Environment;

    use std::process::Command;
    use std::process::Child;
    use std::time::Duration;
    use std::thread;
    use curl::easy::Easy;

    struct LycaonInstance {
        pid: Child,
    }
    /// Call out to cargo to start lycaon.
    /// Seriously considering moving to docker run.

    fn start_lycaon() -> LycaonInstance {
        let child = Command::new("cargo")
            .arg("run")
            .env_clear()
            .envs(Environment::inherit().compile())
            .spawn()
            .expect("failed to start");

        //FIXME: change to poll for start-up
        thread::sleep(Duration::from_millis(500));
        LycaonInstance { pid: child }
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
        let mut easy = Easy::new();
        easy.url("http://localhost:8000").unwrap();
        easy.perform().unwrap();

        assert_eq!(easy.response_code().unwrap(), 200);
    }

}
