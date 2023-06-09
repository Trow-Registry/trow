use axum::body;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::registry_interface::ManifestReader;

impl IntoResponse for ManifestReader {
    fn into_response(self) -> Response {
        let content_type = self.content_type().to_string();
        let digest = self.digest().to_string();
        let size = self.size();
        let stream = FramedRead::new(self.get_reader(), BytesCodec::new());
        Response::builder()
            .header(header::CONTENT_TYPE, content_type)
            .header(header::CONTENT_LENGTH, size)
            .header("Docker-Content-Digest", digest)
            .body(body::StreamBody::from(stream))
            .unwrap()
            .into_response()
    }
}
