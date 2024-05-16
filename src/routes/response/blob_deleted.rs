use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::types::BlobDeleted;

impl IntoResponse for BlobDeleted {
    fn into_response(self) -> Response {
        StatusCode::ACCEPTED.into_response()
    }
}
