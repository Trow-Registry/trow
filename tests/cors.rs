#[cfg(test)]
mod common;

#[cfg(test)]
mod cors_tests {
    use std::path::Path;

    use axum::body::Body;
    use axum::Router;
    use base64::engine::general_purpose as base64_engine;
    use base64::Engine as _;
    use hyper::Request;
    use reqwest::header::HeaderMap;
    use reqwest::{header, StatusCode};
    use test_temp_dir::test_temp_dir;
    use tower::ServiceExt;

    use crate::common::trow_router;

    const HOST: &str = "127.0.0.1:39368";

    async fn start_trow(data_dir: &Path) -> Router {
        trow_router(data_dir, |cfg| {
            cfg.service_name = HOST.to_string();
            cfg.with_user("authtest".to_owned(), "authpass");
            cfg.cors = Some(vec![
                "http://extrality.ai:8973".to_string(),
                "https://example.com".to_string(),
            ]);
        })
        .await
    }

    #[tokio::test]
    async fn test_cors_preflight() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let trow = start_trow(data_dir).await;

        let mut headers = HeaderMap::new();

        headers.insert(header::ORIGIN, "https://example.com".parse().unwrap());
        headers.insert(
            header::ACCESS_CONTROL_REQUEST_METHOD,
            "OPTIONS".parse().unwrap(),
        );

        let resp = trow
            .clone()
            .oneshot(
                Request::options("/")
                    .header(header::ORIGIN, "https://example.com")
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "OPTIONS")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .unwrap(),
            "https://example.com"
        );
        assert_eq!(
            resp.headers()
                .get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
                .unwrap(),
            "true"
        );
        let res_cors_methods = resp
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_METHODS)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(res_cors_methods.contains("GET"));
        assert!(res_cors_methods.contains("OPTIONS"));
        assert!(res_cors_methods.contains("POST"));
    }

    #[tokio::test]
    async fn test_cors_method_get() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let trow = start_trow(data_dir).await;

        let resp = trow
            .clone()
            .oneshot(
                Request::get("/")
                    .header(header::ORIGIN, "https://example.com")
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .unwrap(),
            "https://example.com"
        );
        assert_eq!(
            resp.headers()
                .get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
                .unwrap(),
            "true"
        );
    }

    #[tokio::test]
    async fn test_cors_headers_authorization() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let trow = start_trow(data_dir).await;

        let bytes = base64_engine::STANDARD.encode(b"authtest:authpass");
        let resp = trow
            .clone()
            .oneshot(
                Request::get("/login")
                    .header(header::ORIGIN, "https://example.com")
                    .header(header::AUTHORIZATION, format!("Basic {}", bytes))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers()
                .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
                .unwrap(),
            "https://example.com"
        );
        assert_eq!(
            resp.headers()
                .get(header::ACCESS_CONTROL_ALLOW_CREDENTIALS)
                .unwrap(),
            "true"
        );
    }
}
