use axum::body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::types::AcceptedUpload;

impl IntoResponse for AcceptedUpload {
    fn into_response(self) -> Response {
        let location = format!("/v2/{}/blobs/{}", self.repo_name(), self.digest());
        tracing::debug!("accepted upload response");
        let (left, right) = self.range();
        Response::builder()
            .status(StatusCode::CREATED)
            .header("Location", location)
            .header("Docker-Content-Digest", self.digest().to_string())
            .header("Range", format!("{}-{}", left, right))
            .header("Content-Length", "0")
            .body(body::Body::empty())
            .unwrap()
            .into_response()
    }
}

#[cfg(test)]
mod test {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    use crate::registry::Digest;
    use crate::types::AcceptedUpload;

    #[tokio::test]
    async fn test_resp() {
        let accepted_upload = AcceptedUpload::new(
            Digest::try_from_raw(
                "sha256:05c6e08f1d9fdafa03147fcb8f82f124c76d2f70e3d989dc8aadb5e7d7450bec",
            )
            .unwrap(),
            "moredhel/test".to_owned(),
            uuid::Uuid::new_v4(),
            (0, 0),
        );

        let response = accepted_upload.into_response();

        let headers = response.headers();
        assert_eq!(response.status(), StatusCode::CREATED);
        assert!(headers.contains_key("Location"));
        assert!(headers.contains_key("Range"));
        assert!(headers.contains_key("Docker-Content-Digest"));
        assert!(headers.contains_key("Content-Length"));
    }
}
