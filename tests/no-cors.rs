#![cfg(test)]
mod common;

mod no_cors_tests {

    use std::path::Path;

    use axum::body::Body;
    use axum::Router;
    use hyper::Request;
    use reqwest::header::{
        HeaderMap, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN,
        ACCESS_CONTROL_REQUEST_METHOD, ORIGIN,
    };
    use reqwest::StatusCode;
    use test_temp_dir::test_temp_dir;
    use tower::ServiceExt;

    use crate::common::trow_router;

    const TROW_ADDRESS: &str = "http://127.0.0.1:39368";

    async fn start_trow(data_dir: &Path) -> Router {
        trow_router(data_dir, |cfg| {
            cfg.service_name = TROW_ADDRESS.to_string();
        })
        .await
        .1
    }

    async fn test_preflight(cl: &Router) {
        let mut headers = HeaderMap::new();

        headers.insert(ORIGIN, "https://example.com".parse().unwrap());
        headers.insert(ACCESS_CONTROL_REQUEST_METHOD, "OPTIONS".parse().unwrap());

        let resp = cl
            .clone()
            .oneshot(
                Request::options("/")
                    .header(ORIGIN, "https://example.com")
                    .header(ACCESS_CONTROL_REQUEST_METHOD, "OPTIONS")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(resp.headers().get(ACCESS_CONTROL_ALLOW_METHODS), None);
        assert_eq!(resp.headers().get(ACCESS_CONTROL_ALLOW_ORIGIN), None);
    }

    #[tokio::test]
    async fn test_runner() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let trow = start_trow(data_dir).await;

        println!("Running test_preflight()");
        test_preflight(&trow).await;
    }
}
