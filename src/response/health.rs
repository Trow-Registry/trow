use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::trow_server::api_types::HealthStatus;

impl IntoResponse for HealthStatus {
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

    use crate::trow_server::api_types::HealthStatus;

    fn build_healthy_response() -> HealthStatus {
        HealthStatus {
            message: String::from("Healthy"),
            is_healthy: true,
        }
    }

    fn build_unhealthy_response() -> HealthStatus {
        HealthStatus {
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
