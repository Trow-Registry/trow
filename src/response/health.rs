use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::types::HealthResponse;

impl IntoResponse for HealthResponse {
    fn into_response(self) -> Response {
        let json = serde_json::to_string(&self).unwrap_or_else(|_| "{}".to_string());
        let resp = Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::CONTENT_LENGTH, json.len());

        match self.is_healthy {
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

    use crate::types::HealthResponse;

    fn build_healthy_response() -> HealthResponse {
        HealthResponse {
            message: String::from("Healthy"),
            is_healthy: true,
        }
    }

    fn build_unhealthy_response() -> HealthResponse {
        HealthResponse {
            message: String::from("Healthy"),
            is_healthy: false,
        }
    }

    #[test]
    fn test_healthy_resp() {
        let response = build_healthy_response().into_response();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_unhealthy_response() {
        let response = build_unhealthy_response().into_response();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
