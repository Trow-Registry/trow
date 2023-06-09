use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::types::ReadinessResponse;

impl IntoResponse for ReadinessResponse {
    fn into_response(self) -> Response {
        let json = serde_json::to_string(&self).unwrap_or_default();
        let resp = Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::CONTENT_LENGTH, json.len());

        match self.is_ready {
            true => resp
                .status(StatusCode::OK)
                .body(json)
                .unwrap()
                .into_response(),
            false => resp
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .body(json)
                .unwrap()
                .into_response(),
        }
    }
}
#[cfg(test)]
mod test {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    use crate::types::ReadinessResponse;

    fn build_ready_response() -> ReadinessResponse {
        ReadinessResponse {
            message: String::from("Ready"),
            is_ready: true,
        }
    }

    fn build_not_ready_response() -> ReadinessResponse {
        ReadinessResponse {
            message: String::from("Not Ready"),
            is_ready: false,
        }
    }

    #[test]
    fn test_ready_resp() {
        let response = build_ready_response().into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_not_ready_resp() {
        let response = build_not_ready_response().into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
