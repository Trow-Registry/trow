use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};

use axum::body::Body;
use axum::Router;
use http_body_util::BodyExt;
use hyper::body::Buf;
use hyper::{Request, Response};
use rand::Rng;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tower::ServiceExt;
use trow::registry::digest::Digest;
use trow::registry::manifest;

/* None of these are dead code, they are called from tests */
#[allow(dead_code)]
pub const DIST_API_HEADER: &str = "Docker-Distribution-API-Version";
#[allow(dead_code)]
pub const UPLOAD_HEADER: &str = "Docker-Upload-Uuid";
#[allow(dead_code)]
pub const LOCATION_HEADER: &str = "Location";
#[allow(dead_code)]
pub const RANGE_HEADER: &str = "Range";

#[cfg(test)]
#[allow(dead_code)]
pub fn gen_rand_blob(size: usize) -> Vec<u8> {
    let mut blob = Vec::with_capacity(size);
    for _ in 0..size {
        blob.push(fastrand::u8(0..=255));
    }
    blob
}

#[cfg(test)]
#[allow(dead_code)]
pub async fn response_body_vec(resp: Response<Body>) -> Vec<u8> {
    let mut buf = Vec::new();
    resp.into_body()
        .collect()
        .await
        .unwrap()
        .aggregate()
        .reader()
        .read_to_end(&mut buf)
        .unwrap();
    buf
}

#[cfg(test)]
#[allow(dead_code)]
pub async fn response_body_string(resp: Response<Body>) -> String {
    let vec = response_body_vec(resp).await;
    String::from_utf8(vec).unwrap()
}

#[cfg(test)]
#[allow(dead_code)]
pub async fn response_body_json<T: DeserializeOwned>(resp: Response<Body>) -> T {
    let reader = resp
        .into_body()
        .collect()
        .await
        .unwrap()
        .aggregate()
        .reader();
    serde_json::from_reader(reader).unwrap()
}

#[cfg(test)]
#[allow(dead_code)]
pub async fn upload_fake_image(cl: &Router, name: &str, tag: &str) {
    use crate::common;

    let resp = cl
        .clone()
        .oneshot(
            Request::post(&format!("/v2/{}/blobs/uploads/", name))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Error uploading layer");
    assert_eq!(resp.status(), StatusCode::ACCEPTED);
    let uuid = resp.headers().get(UPLOAD_HEADER).unwrap().to_str().unwrap();
    let location = resp
        .headers()
        .get(LOCATION_HEADER)
        .unwrap()
        .to_str()
        .unwrap();

    //Upload file. Start uploading blob with patch then digest with put
    let blob = gen_rand_blob(100);
    let resp = cl
        .clone()
        .oneshot(
            Request::patch(location)
                .body(Body::from(blob.clone()))
                .unwrap(),
        )
        .await
        .expect("Failed to send patch request");
    assert_eq!(resp.status(), StatusCode::ACCEPTED);
    let range = resp.headers().get(RANGE_HEADER).unwrap().to_str().unwrap();
    assert_eq!(range, format!("0-{}", (blob.len() - 1))); //note first byte is 0, hence len - 1

    let digest = Digest::digest_sha256(BufReader::new(blob.as_slice())).unwrap();
    let resp = cl
        .clone()
        .oneshot(
            Request::put(&format!(
                "/v2/{}/blobs/uploads/{}?digest={}",
                name, uuid, digest
            ))
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .expect("Failed to send put request");
    assert_eq!(resp.status(), StatusCode::CREATED);

    //Finally get it back again
    let resp = cl
        .clone()
        .oneshot(
            Request::get(&format!("/v2/{}/blobs/{}", name, digest))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let digest_header = resp
        .headers()
        .get("Docker-Content-Digest")
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(digest.to_string(), digest_header);
    let body = common::response_body_vec(resp).await;
    assert_eq!(blob, body);

    //Upload manifest
    //For time being use same blob for config and layer
    let config = manifest::Object {
        media_type: "application/vnd.docker.container.image.v1+json".to_owned(),
        size: Some(blob.len() as u64),
        digest: digest.to_string(),
    };
    let layer = manifest::Object {
        media_type: "application/vnd.docker.image.rootfs.diff.tar.gzip".to_owned(),
        size: Some(blob.len() as u64),
        digest: digest.to_string(),
    };
    let layers = vec![layer];
    let mani = manifest::OCIManifestV2 {
        schema_version: 2,
        media_type: Some("application/vnd.docker.distribution.manifest.v2+json".to_owned()),
        config,
        layers,
    };
    let manifest_addr = format!("/v2/{}/manifests/{}", name, tag);
    let resp = cl
        .clone()
        .oneshot(
            Request::put(&manifest_addr)
                .body(Body::from(serde_json::to_vec(&mani).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let location = resp.headers().get("Location").unwrap().to_str().unwrap();
    assert_eq!(&location, &manifest_addr);
}

#[cfg(test)]
#[allow(dead_code)]
/// Returns a temporary file filled with `contents`
pub fn get_file<T: Serialize>(dir: &Path, contents: T) -> PathBuf {
    let rnum: u16 = rand::thread_rng().gen();
    let path = dir.join(rnum.to_string());
    let mut file = File::create(&path).unwrap();
    file.write_all(serde_yaml::to_string(&contents).unwrap().as_bytes())
        .unwrap();
    file.flush().unwrap();

    path
}
