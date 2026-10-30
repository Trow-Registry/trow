use axum::body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::services::manifest_results::ManifestDeleted;

impl IntoResponse for ManifestDeleted {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::ACCEPTED)
            .body(body::Body::empty())
            .unwrap()
            .into_response()
    }
}
