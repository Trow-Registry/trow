pub mod accepted_upload;
pub mod authenticate;
pub mod blob_deleted;
pub mod blob_reader;
pub mod content_info;
pub mod errors;
pub mod health;
pub mod html;
pub mod manifest_deleted;
pub mod manifest_history;
pub mod manifest_reader;
pub mod metrics;
pub mod readiness;
pub mod repo_catalog;
pub mod tag_list;
pub mod trow_token;
pub mod upload;
pub mod upload_info;
pub mod verified_manifest;

use axum::body::Body;
use axum::http::header;
use axum::response::{IntoResponse, Response};

#[derive(Debug, Clone, Copy, Default)]
pub struct OciJson<T> (pub T);

impl<T> IntoResponse for OciJson<T>
where
    T: serde::Serialize,
{
    fn into_response(self) -> Response {
        let json = serde_json::to_vec(&self.0).unwrap();

        Response::builder()
        .header(header::CONTENT_TYPE,  "application/json")
        .header(header::CONTENT_LENGTH, json.len())
        .body(Body::from(json))
        .unwrap()
        .into_response()
    }
}
