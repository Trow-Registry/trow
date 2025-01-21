use axum::body;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use tokio::io::AsyncRead;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::registry::BlobReader;

impl<S: AsyncRead + Send + 'static> IntoResponse for BlobReader<S> {
    fn into_response(self) -> Response {
        let digest = self.digest().to_string();
        let size = self.blob_size();
        let reader = self.get_reader();
        let stream = FramedRead::new(*reader, BytesCodec::new());

        Response::builder()
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .header(header::CONTENT_LENGTH, size)
            .header("Docker-Content-Digest", digest)
            .body(body::Body::from_stream(stream))
            .unwrap()
            .into_response()
    }
}
