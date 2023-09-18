use axum::http::header;
use axum::response::{IntoResponse, Response};

use crate::types::TagList;

impl IntoResponse for TagList {
    fn into_response(self) -> Response {
        let json = serde_json::to_string(&self).unwrap();

        Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::CONTENT_LENGTH, json.len())
            .body(json)
            .unwrap()
            .into_response()
    }
}
