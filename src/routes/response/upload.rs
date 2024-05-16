use axum::response::{IntoResponse, Response};

pub use crate::types::Upload;

impl IntoResponse for Upload {
    fn into_response(self) -> Response {
        match self {
            Upload::Info(info) => info.into_response(),
            Upload::Accepted(accepted) => accepted.into_response(),
        }
    }
}
