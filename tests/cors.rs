#[cfg(test)]
mod common;

#[cfg(test)]
mod cors_tests {
    use std::process::{Child, Command};
    use std::time::Duration;
    use std::{fs, thread};

    use base64::engine::general_purpose as base64_engine;
    use base64::Engine as _;
    use environment::Environment;
    use reqwest::header::HeaderMap;
    use reqwest::{header, StatusCode};

    use crate::common;

    const PORT: &str = "39368";
    const HOST: &str = "127.0.0.1:39368";
    const ORIGIN: &str = "http://127.0.0.1:39368";

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
            .arg("--user")
            .arg("authtest")
            .arg("--password")
            .arg("authpass")
            .arg("--name")
            .arg(HOST)
            .arg("--port")
            .arg(PORT)
            .arg("--cors")
            .arg("http://extrality.ai:8973,https://example.com")
            .spawn()
            .expect("failed to start");

        let mut timeout = 100;
        // get a client builder
        let client = reqwest::Client::new();
        let mut response = client.get(ORIGIN).send().await;

        while timeout > 0 && (response.is_err() || (response.unwrap().status() != StatusCode::OK)) {
            thread::sleep(Duration::from_millis(100));
            response = client.get(ORIGIN).send().await;
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

    async fn test_cors_preflight(cl: &reqwest::Client) {
        let mut headers = HeaderMap::new();

        headers.insert(header::ORIGIN, "https://example.com".parse().unwrap());
        headers.insert(
            header::ACCESS_CONTROL_REQUEST_METHOD,
            "OPTIONS".parse().unwrap(),
        );

        let resp = cl
            .request(hyper::Method::OPTIONS, &(ORIGIN.to_owned()))
            .headers(headers)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .unwrap(),
            "https://example.com"
        );
        assert_eq!(
            resp.headers()
                .get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
                .unwrap(),
            "true"
        );
        let res_cors_methods = resp
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_METHODS)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(res_cors_methods.contains("GET"));
        assert!(res_cors_methods.contains("OPTIONS"));
        assert!(res_cors_methods.contains("POST"));
    }

    async fn test_cors_method_get(cl: &reqwest::Client) {
        let resp = cl
            .get(&(ORIGIN.to_owned()))
            .header(header::ORIGIN, "https://example.com")
            .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .unwrap(),
            "https://example.com"
        );
        assert_eq!(
            resp.headers()
                .get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
                .unwrap(),
            "true"
        );
    }

    async fn test_cors_headers_authorization(cl: &reqwest::Client) {
        let bytes = base64_engine::STANDARD.encode(b"authtest:authpass");
        let resp = cl
            .get(&(ORIGIN.to_owned() + "/login"))
            .header(header::ORIGIN, "https://example.com")
            .header(header::AUTHORIZATION, format!("Basic {}", bytes))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .unwrap(),
            "https://example.com"
        );
        assert_eq!(
            resp.headers()
                .get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
                .unwrap(),
            "true"
        );
    }

    #[tokio::test]
    async fn test_runner() {
        //Need to start with empty repo
        fs::remove_dir_all("./data").unwrap_or(());

        //Had issues with stopping and starting trow causing test fails.
        //It might be possible to improve things with a thread_local
        let _trow = start_trow().await;

        // get a client builder
        let client = reqwest::Client::new();

        println!("Running test_cors_preflight()");
        test_cors_preflight(&client).await;
        println!("Running test_cors_method_get()");
        test_cors_method_get(&client).await;
        println!("Running test_cors_headers_authorization()");
        test_cors_headers_authorization(&client).await;
    }
}
