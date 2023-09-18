use axum::body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

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
        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header(
                "WWW-Authenticate",
                format!(
                    "Bearer realm=\"{}/login\",service=\"trow_registry\",scope=\"push/pull\"",
                    realm
                ),
            )
            .header("Content-Type", "application/json")
            .body(body::Empty::new())
            .unwrap()
            .into_response()
    }
}
