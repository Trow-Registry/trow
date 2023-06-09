use axum::body;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::types::VerifiedManifest;

impl IntoResponse for VerifiedManifest {
    fn into_response(self) -> Response {
        //The front end is responsible for assembling URLs, backend should deal in arguments
        let location = format!(
            "{}/v2/{}/manifests/{}",
            self.base_url().unwrap(),
            self.repo_name(),
            self.tag()
        );
        Response::builder()
            .header("Location", location)
            .header("Docker-Content-Digest", self.digest().to_string())
            .status(StatusCode::CREATED)
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
    use crate::types::{RepoName, VerifiedManifest};

    #[test]
    fn accepted_ok() {
        let response = VerifiedManifest::new(
            Some("https://extrality.ai".to_string()),
            RepoName("repo_name".to_string()),
            Digest {
                algo: DigestAlgorithm::Sha256,
                hash: "05c6e08f1d9fdafa03147fcb8f82f124c76d2f70e3d989dc8aadb5e7d7450bec"
                    .to_string(),
            },
            "ref".to_string(),
        )
        .into_response();
        assert_eq!(response.status(), StatusCode::CREATED);
    }
}
