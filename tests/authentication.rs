#[cfg(test)]
mod common;

#[cfg(test)]
mod authentication_tests {

    use std::process::{Child, Command};
    use std::time::Duration;
    use std::{fs, thread};

    use axum::http::header;
    use base64::engine::general_purpose as base64_engine;
    use base64::Engine as _;
    use environment::Environment;
    use reqwest::StatusCode;

    use crate::common;

    const PORT: &str = "39367";
    const HOST: &str = "127.0.0.1:39367";
    const TROW_ADDRESS: &str = "http://127.0.0.1:39367";

    struct TrowInstance {
        pid: Child,
    }
    /// Call out to cargo to start trow.
    /// Seriously considering moving to docker run.

    async fn start_trow() -> TrowInstance {
        let mut child = Command::new("cargo")
            .arg("run")
            .env_clear()
            .envs(Environment::inherit().compile())
            .arg("--")
            .arg("--name")
            .arg(HOST)
            .arg("--port")
            .arg(PORT)
            .arg("--user")
            .arg("authtest")
            .arg("--password")
            .arg("authpass")
            .spawn()
            .expect("failed to start");

        let mut timeout = 600; // This should be a full minute

        // get a client builder
        let client = reqwest::Client::builder().build().unwrap();

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
            common::kill_gracefully(&self.pid);
        }
    }

    async fn test_auth_redir(cl: &reqwest::Client) {
        let resp = cl
            .get(&(TROW_ADDRESS.to_owned() + "/v2/"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        //Test get redir header
        assert_eq!(
            resp.headers().get(header::WWW_AUTHENTICATE).unwrap(),
            &format!(
                "Bearer realm=\"{}/login\",service=\"trow_registry\",scope=\"push/pull\"",
                TROW_ADDRESS
            )
        );
    }

    async fn test_login(cl: &reqwest::Client) {
        let bytes = base64_engine::STANDARD.encode(b"authtest:authpass");
        let resp = cl
            .get(&(TROW_ADDRESS.to_owned() + "/login"))
            .header(header::AUTHORIZATION, format!("Basic {}", bytes))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        // Uncomment if you want to inspect the token
        // let _token: JsonValue = resp.json().unwrap();
        let resp = cl
            .get(&format!(
                "{}/v2/{}/manifests/{}",
                TROW_ADDRESS, "name", "tag"
            ))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    async fn test_login_fail(cl: &reqwest::Client) {
        let resp = cl
            .get(&(TROW_ADDRESS.to_owned() + "/login"))
            .header(header::AUTHORIZATION, "Basic thisstringwillfail")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_runner() {
        //Need to start with empty repo
        fs::remove_dir_all("./data").unwrap_or(());

        //Had issues with stopping and starting trow causing test fails.
        //It might be possible to improve things with a thread_local
        let _trow = start_trow().await;

        // get a client builder
        let client = reqwest::Client::builder().build().unwrap();

        println!("Running test_auth_redir()");
        test_auth_redir(&client).await;
        println!("Running test_login()");
        test_login(&client).await;
        println!("Running test_login_fail()");
        test_login_fail(&client).await;
    }
}
