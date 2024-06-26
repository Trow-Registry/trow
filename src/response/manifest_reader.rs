use axum::body;
use axum::http::header;
use axum::response::{IntoResponse, Response};

use crate::registry::ManifestReader;

impl IntoResponse for ManifestReader {
    fn into_response(self) -> Response {
        let content_type = self.content_type().to_string();
        let digest = self.digest().to_string();
        let size = self.size();
        let json = self.get_contents();
        Response::builder()
            .header(header::CONTENT_TYPE, content_type)
            .header(header::CONTENT_LENGTH, size)
            .header("Docker-Content-Digest", digest)
            .body(body::Body::from(json))
            .unwrap()
            .into_response()
    }
}
