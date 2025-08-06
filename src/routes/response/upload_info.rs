use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub use crate::types::UploadInfo;

impl IntoResponse for UploadInfo {
    fn into_response(self) -> Response {
        let location_url = format!("/v2/{}/blobs/uploads/{}", self.repo_name(), self.uuid());
        let (left, right) = self.range();
        tracing::debug!("Range: {}-{}, Length: {}", left, right, right - left);

        Response::builder()
            .header("Docker-Upload-UUID", self.uuid().to_string())
            .header("Range", format!("{left}-{right}"))
            .header("Content-Length", format!("{}", right - left))
            .header("Location", location_url)
            .status(StatusCode::ACCEPTED)
            .body(Body::empty())
            .unwrap()
            .into_response()
    }
}

#[cfg(test)]
mod test {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    use crate::routes::response::upload_info::UploadInfo;

    fn build_response() -> UploadInfo {
        UploadInfo::new(
            uuid::Uuid::new_v4().to_string(),
            "moredhel/test".to_owned(),
            (0, 0),
        )
    }

    #[test]
    fn uuid_uuid() {
        let response = build_response().into_response();
        let headers = response.headers();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
        assert!(headers.contains_key("Docker-Upload-UUID"));
        assert!(headers.contains_key("Location"));
        assert!(headers.contains_key("Range"));
    }
}
