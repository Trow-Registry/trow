#![cfg(test)]

mod common;

mod authentication_tests {

    use std::path::Path;

    use axum::body::Body;
    use axum::Router;
    use base64::engine::general_purpose as base64_engine;
    use base64::Engine as _;
    use hyper::Request;
    use reqwest::{header, StatusCode};
    use test_temp_dir::test_temp_dir;
    use tower::ServiceExt;

    use crate::common::trow_router;

    async fn start_trow(data_dir: &Path) -> Router {
        trow_router(data_dir, |cfg| {
            cfg.with_user("authtest".to_owned(), "authpass");
        })
        .await
        .1
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_auth_redir() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let trow = start_trow(data_dir).await;
        let fake_trow_address = "example.com";

        let resp = trow
            .clone()
            .oneshot(
                Request::get("/v2/")
                    .header(header::HOST, fake_trow_address)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        //Test get redir header
        assert_eq!(
            resp.headers()
                .get(reqwest::header::WWW_AUTHENTICATE)
                .unwrap(),
            &format!(
                "Bearer realm=\"http://{fake_trow_address}/login\",service=\"trow_registry\",scope=\"push/pull\"",
            )
        );
    }
    #[tokio::test]
    async fn test_login() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let trow = start_trow(data_dir).await;

        let bytes = base64_engine::STANDARD.encode(b"authtest:authpass");
        let resp = trow
            .clone()
            .oneshot(
                Request::get("/login")
                    .header(header::AUTHORIZATION, format!("Basic {}", bytes))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        // Uncomment if you want to inspect the token
        // let _token: JsonValue = resp.json().unwrap();
        let resp = trow
            .clone()
            .oneshot(
                Request::get(format!("/v2/{}/manifests/{}", "name", "tag"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_login_fail() {
        let tmp_dir = test_temp_dir!();
        let data_dir = tmp_dir.as_path_untracked();

        let trow = start_trow(data_dir).await;

        let resp = trow
            .clone()
            .oneshot(
                Request::get("/login")
                    .header(header::AUTHORIZATION, "Basic thisstringwillfail")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
