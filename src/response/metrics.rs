use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::registry_interface::MetricsResponse;

impl IntoResponse for MetricsResponse {
    fn into_response(self) -> Response {
        Response::builder()
            .header(header::CONTENT_TYPE, "text/plain")
            .header(header::CONTENT_LENGTH, self.metrics.len())
            .status(StatusCode::OK)
            .body(Body::from(self.metrics))
            .unwrap()
            .into_response()
    }
}

#[cfg(test)]
mod test {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    use crate::registry_interface::MetricsResponse;

    fn build_metrics_response() -> MetricsResponse {
        MetricsResponse {
            metrics: String::from("# HELP available_space ...."),
        }
    }

    #[test]
    fn test_metrics_resp() {
        let response = build_metrics_response().into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
