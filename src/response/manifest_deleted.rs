use axum::body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::types::ManifestDeleted;

impl IntoResponse for ManifestDeleted {
    fn into_response(self) -> Response {
        Response::builder()
            .status(StatusCode::ACCEPTED)
            .body(body::Empty::new())
            .unwrap()
            .into_response()
    }
}
