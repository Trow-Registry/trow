#[cfg(test)]
mod common;

#[cfg(test)]
mod interface_tests {

    use std::path::Path;
    use std::process::{Child, Command};
    use std::thread;
    use std::time::Duration;

    use environment::Environment;
    use reqwest::StatusCode;
    use test_temp_dir::test_temp_dir;

    const PORT: &str = "39376";
    const HOST: &str = "127.0.0.1:39376";
    const TROW_ADDRESS: &str = "http://127.0.0.1:39376";

    struct TrowInstance {
        pid: Child,
    }
    /// Call out to cargo to start trow.
    /// Seriously considering moving to docker run.
    async fn start_trow(temp_dir: &Path) -> TrowInstance {
        let mut child = Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg("--port")
            .arg(PORT)
            .arg("--data-dir")
            .arg(temp_dir)
            .env_clear()
            .envs(Environment::inherit().compile())
            .spawn()
            .expect("failed to start");

        let mut timeout = 100;
        let client = reqwest::Client::new();
        let mut response = client.get(TROW_ADDRESS).send().await;
        while timeout > 0 && (response.is_err() || (response.unwrap().status() != StatusCode::OK)) {
            thread::sleep(Duration::from_millis(100));
            response = client.get(TROW_ADDRESS).send().await;
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
            unsafe {
                libc::kill(self.pid.id() as i32, libc::SIGTERM);
            }
        }
    }

    /**
     * Run a simple docker push/pull against the registry.
     *
     * This assumes Docker is installed and has a cert for the registry.
     * For that reason, it's set to ignored by default and has to be manually enabled.
     *
     */
    #[tokio::test]
    #[tracing_test::traced_test]
    #[ignore]
    async fn smoke_test() {
        let temp_dir = test_temp_dir!();

        //Had issues with stopping and starting trow causing test fails.
        //It might be possible to improve things with a thread_local
        let _trow = start_trow(temp_dir.as_path_untracked()).await;

        println!("Running docker pull alpine:latest");
        let mut status = Command::new("docker")
            .args(["pull", "alpine:latest"])
            .status()
            .expect("Failed to call docker pull - prereq for smoke test");
        assert!(status.success());

        println!("Running docker tag alpine:latest {HOST}/alpine:trow");
        let image_name = format!("{HOST}/alpine:trow");
        status = Command::new("docker")
            .args(["tag", "alpine:latest", &image_name])
            .status()
            .expect("Failed to call docker");
        assert!(status.success());

        println!("Running docker push {HOST}/alpine:trow");
        status = Command::new("docker")
            .args(["push", &image_name])
            .status()
            .expect("Failed to call docker");
        assert!(status.success());

        println!("Running docker rmi {HOST}/alpine:trow");
        status = Command::new("docker")
            .args(["rmi", &image_name])
            .status()
            .expect("Failed to call docker");
        assert!(status.success());

        println!("Running docker pull {HOST}/alpine:trow");
        status = Command::new("docker")
            .args(["pull", &image_name])
            .status()
            .expect("Failed to call docker");

        assert!(status.success());
    }
}
