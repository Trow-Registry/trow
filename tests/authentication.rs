extern crate base64;
extern crate environment;
extern crate hyper;
extern crate rand;
extern crate reqwest;
extern crate serde_json;

mod common;

#[cfg(test)]
mod authentication_tests {

    use crate::common;
    use environment::Environment;

    use base64::encode;
    use reqwest::StatusCode;
    use std::fs::{self, File};
    use std::io::Read;
    use std::process::Child;
    use std::process::Command;
    use std::thread;
    use std::time::Duration;

    const TROW_ADDRESS: &str = "https://trow.test:8443";

    const AUTHN_HEADER: &str = "www-authenticate";
    const AUTHZ_HEADER: &str = "Authorization";

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
            .spawn()
            .expect("failed to start");

        let mut timeout = 600; // This should be a full minute

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

    async fn test_auth_redir(cl: &reqwest::Client) {
        let resp = cl
            .get(&(TROW_ADDRESS.to_owned() + "/v2"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        //Test get redir header
        assert_eq!(
            resp.headers().get(AUTHN_HEADER).unwrap(),
            "Bearer realm=\"https://trow.test:8443/login\",service=\"trow_registry\",scope=\"push/pull\""
        );
    }

    async fn test_login(cl: &reqwest::Client) {
        let bytes = encode(b"authtest:authpass");
        let resp = cl
            .get(&(TROW_ADDRESS.to_owned() + "/login"))
            .header(AUTHZ_HEADER, format!("Basic {}", bytes))
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
            .header(AUTHZ_HEADER, "Basic thisstringwillfail")
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

        println!("Running test_auth_redir()");
        test_auth_redir(&client).await;
        println!("Running test_login()");
        test_login(&client).await;
        println!("Running test_login_fail()");
        test_login_fail(&client).await;
    }
}
