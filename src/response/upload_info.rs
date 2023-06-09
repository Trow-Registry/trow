use axum::body::Body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use log::debug;

pub use crate::types::UploadInfo;

impl IntoResponse for UploadInfo {
    fn into_response(self) -> Response {
        let location_url = format!(
            "{}/v2/{}/blobs/uploads/{}",
            self.base_url(),
            self.repo_name(),
            self.uuid()
        );
        let (left, right) = self.range();
        debug!("Range: {}-{}, Length: {}", left, right, right - left);

        Response::builder()
            .header("Docker-Upload-UUID", self.uuid().0.clone())
            .header("Range", format!("{}-{}", left, right))
            .header("X-Content-Length", format!("{}", right - left))
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

    use crate::response::upload_info::UploadInfo;
    use crate::types::{RepoName, Uuid};
    fn build_response() -> UploadInfo {
        UploadInfo::new(
            "ftp://darpa.org".to_string(),
            Uuid("whatever".to_owned()),
            RepoName("moredhel/test".to_owned()),
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
