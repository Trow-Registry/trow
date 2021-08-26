extern crate environment;
extern crate hyper;
extern crate rand;
extern crate reqwest;
extern crate serde_json;

#[cfg(test)]
mod common;

#[cfg(test)]
mod interface_tests {
    use crate::common;
    use environment::Environment;
    use reqwest::StatusCode;
    use std::fs::File;
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
            .arg("--max-manifest-size")
            .arg("1")
            .arg("--max-blob-size")
            .arg("3")
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

    #[cfg(test)]
    pub async fn put_sized_blob(cl: &reqwest::Client, size: usize) -> StatusCode {
        let resp = cl
            .post(&format!("{}/v2/{}/blobs/uploads/", TROW_ADDRESS, "sized"))
            .send()
            .await
            .expect("Error uploading layer");
        assert_eq!(resp.status(), StatusCode::ACCEPTED);
        let location = resp
            .headers()
            .get(common::LOCATION_HEADER)
            .unwrap()
            .to_str()
            .unwrap();

        let blob = common::gen_rand_blob(size);
        let resp = cl
            .patch(location)
            .body(blob)
            .send()
            .await
            .expect("Failed to send patch request");

        resp.status()
    }

    #[tokio::test]
    async fn test_runner() {
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

        //put_sized_blob(&client, 100).await;

        assert_eq!(
            StatusCode::ACCEPTED,
            put_sized_blob(&client, 3 * 1024 * 1024 - 1).await
        );
        assert_eq!(
            StatusCode::RANGE_NOT_SATISFIABLE,
            put_sized_blob(&client, 3 * 1024 * 1024 + 1).await
        );
    }
}
