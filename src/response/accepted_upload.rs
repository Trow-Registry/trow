use axum::body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use log::debug;

use crate::types::AcceptedUpload;

impl IntoResponse for AcceptedUpload {
    fn into_response(self) -> Response {
        let location = format!(
            "{}/v2/{}/blobs/{}",
            self.base_url(),
            self.repo_name(),
            self.digest()
        );
        debug!("accepted upload response");
        let (left, right) = self.range();
        Response::builder()
            .status(StatusCode::CREATED)
            .header("Location", location)
            .header("Docker-Content-Digest", self.digest().to_string())
            .header("Range", format!("{}-{}", left, right))
            .header("Content-Length", "0")
            .body(body::Empty::new())
            .unwrap()
            .into_response()
    }
}

#[cfg(test)]
mod test {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    use crate::registry_interface::{Digest, DigestAlgorithm};
    use crate::types::{AcceptedUpload, RepoName, Uuid};

    #[tokio::test]
    async fn test_resp() {
        let accepted_upload = AcceptedUpload::new(
            "http://trowuw".to_string(),
            Digest {
                algo: DigestAlgorithm::Sha256,
                hash: "05c6e08f1d9fdafa03147fcb8f82f124c76d2f70e3d989dc8aadb5e7d7450bec"
                    .to_string(),
            },
            RepoName("moredhel/test".to_owned()),
            Uuid("whatever".to_owned()),
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
