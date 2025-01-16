pub mod accepted_upload;
pub mod authenticate;
pub mod blob_deleted;
pub mod blob_reader;
pub mod content_info;
pub mod errors;
pub mod health;
pub mod html;
pub mod manifest_deleted;
pub mod manifest_reader;
pub mod metrics;
pub mod readiness;
pub mod tag_list;
pub mod trow_token;
pub mod upload;
pub mod upload_info;
pub mod verified_manifest;

use std::marker::PhantomData;

use axum::body::Body;
use axum::http::{header, HeaderValue};
use axum::response::{IntoResponse, Response};
use bytes::Bytes;

use crate::registry::Digest;

#[derive(Debug, Default)]
pub struct OciJson<T> {
    response: Response<Body>,
    content_length: usize,
    content_type: std::marker::PhantomData<T>,
}

impl<T> OciJson<T>
where
    T: serde::Serialize,
{
    pub fn new(content: &T) -> Self {
        let body_bytes = serde_json::to_vec(content).unwrap();
        let content_length = body_bytes.len();
        let response = Response::new(Body::from(body_bytes));

        Self {
            response,
            content_length,
            content_type: PhantomData,
        }
    }

    /// To work around the fact that manifests cannot be serialized/deserialized
    /// or their digest might not match
    pub fn new_raw(content: Bytes) -> Self {
        let content_length = content.len();
        let response = Response::new(Body::from(content));
        Self {
            response,
            content_length,
            content_type: PhantomData,
        }
    }

    pub fn set_digest(mut self, digest: &Digest) -> Self {
        self.response.headers_mut().insert(
            "Docker-Content-Digest",
            HeaderValue::from_str(digest.as_str()).unwrap(),
        );
        self
    }

    /// Overwrites the default content type of `application/json`
    pub fn set_content_type(mut self, content_type: &str) -> Self {
        self.response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(content_type).unwrap(),
        );
        self
    }
}

impl<T> IntoResponse for OciJson<T> {
    fn into_response(mut self) -> Response {
        let headers = self.response.headers_mut();
        headers
            .entry(header::CONTENT_TYPE)
            .or_insert(HeaderValue::from_str("application/json").unwrap());
        headers
            .entry(header::CONTENT_LENGTH)
            .or_insert(HeaderValue::from(self.content_length));
        println!("resp: {:?}\n\n---", self.response.headers());
        println!();
        self.response
    }
}
