use axum::http::HeaderValue;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::routes::response::errors::Error;
/*
 * Generate a WWW-Authenticate header
 */
#[derive(Debug, Serialize)]
pub struct Authenticate {
    base_url: String,
}

impl Authenticate {
    pub fn new(base_url: String) -> Self {
        Authenticate { base_url }
    }
}

impl IntoResponse for Authenticate {
    fn into_response(self) -> Response {
        let realm = self.base_url;

        let mut response = Error::Unauthorized.into_response();
        response.headers_mut().insert(
            "WWW-Authenticate",
            HeaderValue::from_str(&format!(
                "Bearer realm=\"{realm}/login\",service=\"trow_registry\",scope=\"push/pull\""
            ))
            .unwrap(),
        );

        response
    }
}
