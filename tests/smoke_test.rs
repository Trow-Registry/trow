#![cfg(test)]

mod common;

mod smoke_test {

    use std::net::TcpListener;
    use std::path::Path;
    use std::process::{Child, Command};
    use std::thread;
    use std::time::Duration;

    use axum::body::Body;
    use environment::Environment;
    use hyper::Request;
    use reqwest::StatusCode;
    use test_temp_dir::test_temp_dir;
    use tower::ServiceExt;
    use trow::registry::{ConfigFile, RegistryProxiesConfig, SingleRegistryProxyConfig};

    use crate::common::trow_router;

    struct TrowInstance {
        pid: Child,
        port: u16,
    }

    /// Call out to cargo to start trow.
    async fn start_trow(temp_dir: &Path, ipv6: bool) -> TrowInstance {
        let localhost = if ipv6 { "[::1]" } else { "127.0.0.1" };
        let listener = TcpListener::bind(format!("{localhost}:0")).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let mut child = Command::new("./target/debug/trow")
            .arg(format!(
                "--bind={ip}:{port}",
                ip = if ipv6 { "[::]" } else { "0.0.0.0" }
            ))
            .arg(format!("--data-dir={}", temp_dir.display()))
            .env_clear()
            .envs(Environment::inherit().compile())
            .spawn()
            .expect("failed to start");

        let uri = format!("http://{localhost}:{port}");
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
    async fn registry_smoke_test() {
        let temp_dir = test_temp_dir!();

        //Had issues with stopping and starting trow causing test fails.
        //It might be possible to improve things with a thread_local
        let trow = start_trow(temp_dir.as_path_untracked(), false).await;

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

    fn new_command(cmd: &str) -> Command {
        println!("Running: {cmd}");
        let mut cmd_it = cmd.split(' ');
        let mut cmd = Command::new(cmd_it.next().unwrap());
        cmd.args(cmd_it);
        cmd
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn pulls_from_trow_ipv4() {
        let temp_dir = test_temp_dir!();
        let data_trow0 = temp_dir.subdir_untracked("0");
        let data_trow1 = temp_dir.subdir_untracked("1");

        let trow0 = start_trow(&data_trow0, false).await;
        let trow1 = trow_router(&data_trow1, |cfg| {
            cfg.config_file = Some(ConfigFile {
                registry_proxies: RegistryProxiesConfig {
                    registries: vec![SingleRegistryProxyConfig {
                        alias: "trow".to_string(),
                        host: format!("http://127.0.0.1:{}", trow0.port),
                        ignore_repos: vec![],
                        password: None,
                        username: None,
                    }],
                    ..Default::default()
                },
                ..Default::default()
            });
        })
        .await;

        let remote_image = "public.ecr.aws/docker/library/alpine:latest";
        println!("Running podman pull alpine:latest");
        new_command(&format!("podman pull {remote_image}"))
            .status()
            .expect("Failed to call podman pull alpine:latest - prereq for test");
        new_command(&format!(
            "podman tag {remote_image} 127.0.0.1:{}/alpine:latest",
            trow0.port
        ))
        .status()
        .expect("Failed to call podman tag - prereq for test");
        new_command(&format!(
            "podman push 127.0.0.1:{}/alpine:latest --tls-verify=false",
            trow0.port
        ))
        .status()
        .expect("Failed to call podman push - prereq for test");
        let resp = trow1
            .1
            .clone()
            .oneshot(
                Request::get("/v2/f/trow/alpine/manifests/latest")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn pulls_from_trow_ipv6() {
        let temp_dir = test_temp_dir!();
        let data_trow0 = temp_dir.subdir_untracked("0");
        let data_trow1 = temp_dir.subdir_untracked("1");

        let trow0 = start_trow(&data_trow0, true).await;
        let trow1 = trow_router(&data_trow1, |cfg| {
            cfg.config_file = Some(ConfigFile {
                registry_proxies: RegistryProxiesConfig {
                    registries: vec![SingleRegistryProxyConfig {
                        alias: "trow".to_string(),
                        host: format!("http://[::1]:{}", trow0.port),
                        ignore_repos: vec![],
                        password: None,
                        username: None,
                    }],
                    ..Default::default()
                },
                ..Default::default()
            });
        })
        .await;

        let remote_image = "public.ecr.aws/docker/library/alpine:latest";
        println!("Running podman pull alpine:latest");
        new_command(&format!("podman pull {remote_image}"))
            .status()
            .unwrap();
        new_command(&format!(
            "podman tag {remote_image} ipv6-localhost:{}/alpine:latest",
            trow0.port
        ))
        .status()
        .unwrap();
        new_command(&format!(
            "podman push ipv6-localhost:{}/alpine:latest --tls-verify=false",
            trow0.port
        ))
        .status()
        .unwrap();
        let resp = trow1
            .1
            .clone()
            .oneshot(
                Request::get("/v2/f/trow/alpine/manifests/latest")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
