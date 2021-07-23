extern crate base64;
extern crate environment;
extern crate hyper;
extern crate rand;
extern crate reqwest;
extern crate rocket_contrib;
extern crate serde_json;

mod common;

#[cfg(test)]
mod no_cors_tests {

    use crate::common;
    use environment::Environment;

    use reqwest::StatusCode;
    use std::fs::{self, File};
    use std::io::Read;
    use std::process::Child;
    use std::process::Command;
    use std::thread;
    use std::time::Duration;
    use reqwest::header::{ORIGIN, ACCESS_CONTROL_REQUEST_METHOD, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN};
    use reqwest::header::HeaderMap;

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

    async fn test_preflight(cl: &reqwest::Client) {
        let mut headers = HeaderMap::new();

        headers.insert(ORIGIN, "https://example.com".parse().unwrap());
        headers.insert(ACCESS_CONTROL_REQUEST_METHOD, "OPTIONS".parse().unwrap());

        let resp = cl
            .request(hyper::Method::OPTIONS,&(TROW_ADDRESS.to_owned()))
            .headers(headers)
            .send()
            .await
            .unwrap();
        
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);  
        assert_eq!(resp.headers().get(ACCESS_CONTROL_ALLOW_METHODS), None);
        assert_eq!(resp.headers().get(ACCESS_CONTROL_ALLOW_ORIGIN), None); 

        
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

        println!("Running test_preflight()");
        test_preflight(&client).await;
    }
}