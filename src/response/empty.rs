use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Empty;

impl IntoResponse for Empty {
    fn into_response(self) -> Response {
        ().into_response()
    }
}

#[cfg(test)]
mod test {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    use crate::response::empty::Empty;

    #[test]
    fn empty_ok() {
        let response = Empty {}.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
