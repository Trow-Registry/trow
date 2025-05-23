use axum::body;
use axum::http::header;
use axum::response::{IntoResponse, Response};

pub struct Html<'a>(pub &'a str);

impl IntoResponse for Html<'_> {
    fn into_response(self) -> Response {
        Response::builder()
            .header(header::CONTENT_TYPE, "text/html")
            .header(header::CONTENT_LENGTH, self.0.len())
            .body(body::Body::from(self.0.to_owned()))
            .unwrap()
            .into_response()
    }
}
