#[cfg(test)]
mod common;

#[cfg(test)]
mod cors_tests {

    use crate::common;
    use environment::Environment;

    use base64::{engine::general_purpose as base64_engine, Engine as _};
    use reqwest::header::HeaderMap;
    use reqwest::header::{
        ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_METHODS,
        ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_REQUEST_METHOD, AUTHORIZATION, ORIGIN,
    };
    use reqwest::StatusCode;
    use std::fs::{self, File};
    use std::io::Read;
    use std::process::Child;
    use std::process::Command;
    use std::thread;
    use std::time::Duration;

    const TROW_ADDRESS: &str = "https://trow.test:8443";

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
            .arg("-u")
            .arg("authtest")
            .arg("-p")
            .arg("authpass")
            .arg("--enable-cors")
            .spawn()
            .expect("failed to start");

        let mut timeout = 100;

        let mut buf = Vec::new();
        File::open("./certs/domain.crt")
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
        let cert = reqwest::Certificate::from_pem(&buf).unwrap();
        // get a client builder
        let client = reqwest::Client::builder()
            .add_root_certificate(cert)
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

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

    async fn test_cors_preflight(cl: &reqwest::Client) {
        let mut headers = HeaderMap::new();

        headers.insert(ORIGIN, "https://example.com".parse().unwrap());
        headers.insert(ACCESS_CONTROL_REQUEST_METHOD, "OPTIONS".parse().unwrap());

        let resp = cl
            .request(hyper::Method::OPTIONS, &(TROW_ADDRESS.to_owned()))
            .headers(headers)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        assert_eq!(
            resp.headers().get(ACCESS_CONTROL_ALLOW_ORIGIN).unwrap(),
            "https://example.com"
        );
        assert_eq!(
            resp.headers()
                .get(ACCESS_CONTROL_ALLOW_CREDENTIALS)
                .unwrap(),
            "true"
        );
        let res_cors_methods = resp
            .headers()
            .get(ACCESS_CONTROL_ALLOW_METHODS)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(res_cors_methods.contains("GET"));
        assert!(res_cors_methods.contains("OPTIONS"));
        assert!(res_cors_methods.contains("POST"));
    }

    async fn test_cors_method_get(cl: &reqwest::Client) {
        let resp = cl
            .get(&(TROW_ADDRESS.to_owned()))
            .header(ORIGIN, "https://example.com")
            .header(ACCESS_CONTROL_REQUEST_METHOD, "GET")
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(ACCESS_CONTROL_ALLOW_ORIGIN).unwrap(),
            "https://example.com"
        );
        assert_eq!(
            resp.headers()
                .get(ACCESS_CONTROL_ALLOW_CREDENTIALS)
                .unwrap(),
            "true"
        );
    }

    async fn test_cors_headers_authorization(cl: &reqwest::Client) {
        let bytes = base64_engine::STANDARD.encode(b"authtest:authpass");
        let resp = cl
            .get(&(TROW_ADDRESS.to_owned() + "/login"))
            .header(ORIGIN, "https://example.com")
            .header(AUTHORIZATION, format!("Basic {}", bytes))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get(ACCESS_CONTROL_ALLOW_ORIGIN).unwrap(),
            "https://example.com"
        );
        assert_eq!(
            resp.headers()
                .get(ACCESS_CONTROL_ALLOW_CREDENTIALS)
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

        let mut buf = Vec::new();
        File::open("./certs/domain.crt")
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
        let cert = reqwest::Certificate::from_pem(&buf).unwrap();
        // get a client builder
        let client = reqwest::Client::builder()
            .add_root_certificate(cert)
            .danger_accept_invalid_certs(true)
            .build()
            .unwrap();

        println!("Running test_cors_preflight()");
        test_cors_preflight(&client).await;
        println!("Running test_cors_method_get()");
        test_cors_method_get(&client).await;
        println!("Running test_cors_headers_authorization()");
        test_cors_headers_authorization(&client).await;
    }
}
