#![cfg(test)]
#![allow(dead_code)] // Rustup thinks everything in here is dead code

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

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
use trow::{routes, TrowConfig, TrowServerState};

pub const DIST_API_HEADER: &str = "Docker-Distribution-API-Version";
pub const UPLOAD_HEADER: &str = "Docker-Upload-Uuid";
pub const LOCATION_HEADER: &str = "Location";
pub const RANGE_HEADER: &str = "Range";

pub async fn trow_router<F: FnOnce(&mut TrowConfig)>(
    temp_dir: &Path,
    custom_cfg: F,
) -> (Arc<TrowServerState>, Router) {
    let mut trow_builder = TrowConfig::new();
    trow_builder.data_dir = temp_dir.to_owned();
    custom_cfg(&mut trow_builder);
    let state = trow_builder.build_server_state().await.unwrap();
    let router = routes::create_app(state.clone());

    (state, router)
}

pub fn gen_rand_blob(size: usize) -> Vec<u8> {
    let mut blob = Vec::with_capacity(size);
    for _ in 0..size {
        blob.push(fastrand::u8(0..=255));
    }
    blob
}

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

pub async fn response_body_string(resp: Response<Body>) -> String {
    let vec = response_body_vec(resp).await;
    String::from_utf8(vec).unwrap()
}

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

/// Returns (blob-digest, manifest-digest)
pub async fn upload_fake_image(cl: &Router, name: &str, tag: &str) -> (Digest, Digest) {
    let resp = cl
        .clone()
        .oneshot(
            Request::post(format!("/v2/{}/blobs/uploads/", name))
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

    let blob_digest = Digest::digest_sha256_slice(blob.as_slice());
    let resp = cl
        .clone()
        .oneshot(
            Request::put(format!(
                "/v2/{}/blobs/uploads/{}?digest={}",
                name, uuid, blob_digest
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
            Request::get(format!("/v2/{}/blobs/{}", name, blob_digest))
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
    assert_eq!(blob_digest.to_string(), digest_header);
    let body = response_body_vec(resp).await;
    assert_eq!(blob, body);

    //Upload manifest
    //For time being use same blob for config and layer
    let blob_size = blob.len();
    let raw_manifest = format!(
        r#"{{
        "schemaVersion": 2,
        "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
        "config": {{
            "mediaType": "application/vnd.docker.container.image.v1+json",
            "size": {blob_size},
            "digest": "{blob_digest}"
        }},
        "layers": [{{
            "mediaType": "application/vnd.docker.image.rootfs.diff.tar.gzip",
            "size": {blob_size},
            "digest": "{blob_digest}"
        }}]
    }}"#
    );
    let manifest_digest = Digest::digest_sha256_slice(raw_manifest.as_bytes());
    let _: manifest::OCIManifest = serde_json::from_str(&raw_manifest).unwrap();

    let manifest_addr = format!("/v2/{}/manifests/{}", name, tag);
    let resp = cl
        .clone()
        .oneshot(
            Request::put(&manifest_addr)
                .body(Body::from(raw_manifest))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        StatusCode::CREATED,
        "response: {}",
        response_body_string(resp).await
    );
    let location = resp.headers().get("Location").unwrap().to_str().unwrap();
    assert_eq!(&location, &manifest_addr);
    (blob_digest, manifest_digest)
}

/// Returns a temporary file filled with `contents`
pub fn get_file<T: Serialize>(dir: &Path, contents: T) -> PathBuf {
    let rnum: u16 = rand::thread_rng().gen();
    let path = dir.join(rnum.to_string());
    let mut file = File::create(&path).unwrap();
    file.write_all(serde_yaml_ng::to_string(&contents).unwrap().as_bytes())
        .unwrap();
    file.flush().unwrap();

    path
}
