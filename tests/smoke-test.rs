#![cfg(test)]

mod common;

mod smoke_test {

    use std::net::TcpListener;
    use std::path::Path;
    use std::process::{Child, Command};
    use std::thread;
    use std::time::Duration;

    use environment::Environment;
    use reqwest::StatusCode;
    use test_temp_dir::test_temp_dir;

    struct TrowInstance {
        pid: Child,
        port: u16,
    }

    /// Call out to cargo to start trow.
    async fn start_trow(temp_dir: &Path) -> TrowInstance {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let mut child = Command::new("./target/debug/trow")
            .arg("--port")
            .arg(port.to_string())
            .arg("--data-dir")
            .arg(temp_dir)
            .env_clear()
            .envs(Environment::inherit().compile())
            .spawn()
            .expect("failed to start");

        let uri = format!("http://127.0.0.1:{}", port);
        let mut timeout = 50;
        let client = reqwest::Client::new();
        let mut response = client.get(&uri).send().await;
        while timeout > 0 && (response.is_err() || (response.unwrap().status() != StatusCode::OK)) {
            thread::sleep(Duration::from_millis(500));
            response = client.get(&uri).send().await;
            timeout -= 1;
        }
        if timeout == 0 {
            child.kill().unwrap();
            panic!(
                "Failed to start Trow:\n{:?}\n---\n{:?}",
                child.stdout.unwrap(),
                child.stderr.unwrap()
            );
        }
        TrowInstance { pid: child, port }
    }

    impl Drop for TrowInstance {
        fn drop(&mut self) {
            unsafe {
                libc::kill(self.pid.id() as i32, libc::SIGTERM);
            }
        }
    }

    /**
     * Run a simple podman push/pull against the registry.
     *
     * This assumes podman is installed.
     */
    #[tokio::test]
    #[tracing_test::traced_test]
    async fn smoke_test() {
        let temp_dir = test_temp_dir!();

        //Had issues with stopping and starting trow causing test fails.
        //It might be possible to improve things with a thread_local
        let trow = start_trow(temp_dir.as_path_untracked()).await;

        let remote_image = "public.ecr.aws/docker/library/alpine:latest";
        let local_image = format!("127.0.0.1:{}/alpine:trow", trow.port);

        println!("Running podman pull alpine:latest");
        let mut status = Command::new("podman")
            .args(["pull", remote_image])
            .status()
            .expect("Failed to call podman pull - prereq for smoke test");
        assert!(status.success());

        println!("Running podman tag {remote_image} {local_image}");
        status = Command::new("podman")
            .args(["tag", remote_image, &local_image])
            .status()
            .expect("Failed to call podman");
        assert!(status.success());

        println!("Running podman push {local_image}");
        status = Command::new("podman")
            .args(["push", &local_image, "--tls-verify=false"])
            .status()
            .expect("Failed to call podman");
        assert!(status.success());

        println!("Running podman rmi {local_image}");
        status = Command::new("podman")
            .args(["rmi", &local_image])
            .status()
            .expect("Failed to call podman");
        assert!(status.success());

        println!("Running podman pull {local_image}");
        status = Command::new("podman")
            .args(["pull", &local_image, "--tls-verify=false"])
            .status()
            .expect("Failed to call podman");

        assert!(status.success());
    }
}
