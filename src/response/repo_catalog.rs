use axum::body::Body;
use axum::http::header;
use axum::response::{IntoResponse, Response};

use crate::types::RepoCatalog;

impl IntoResponse for RepoCatalog {
    fn into_response(self) -> Response {
        let json = serde_json::to_string(&self).unwrap();

        Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::CONTENT_LENGTH, json.len())
            .body(Body::from(json))
            .unwrap()
            .into_response()
    }
}
