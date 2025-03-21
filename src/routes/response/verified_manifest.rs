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
        let mut rbuilder = Response::builder()
            .header("Location", location)
            .header("Docker-Content-Digest", self.digest().to_string())
            .status(StatusCode::CREATED);
        if let Some(subject) = self.subject() {
            rbuilder = rbuilder.header("OCI-Subject", subject);
        }
        rbuilder.body(body::Body::empty()).unwrap().into_response()
    }
}

#[cfg(test)]
mod test {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    use crate::registry::Digest;
    use crate::types::VerifiedManifest;

    #[test]
    fn accepted_ok() {
        let response = VerifiedManifest::new(
            Some("https://extrality.ai".to_string()),
            "repo_name".to_string(),
            Digest::try_from_raw(
                "sha256:05c6e08f1d9fdafa03147fcb8f82f124c76d2f70e3d989dc8aadb5e7d7450bec",
            )
            .unwrap(),
            "ref".to_string(),
            None,
        )
        .into_response();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[test]
    fn with_subject() {
        let response = VerifiedManifest::new(
            Some("https://extrality.ai".to_string()),
            "repo_name".to_string(),
            Digest::try_from_raw(
                "sha256:05c6e08f1d9fdafa03147fcb8f82f124c76d2f70e3d989dc8aadb5e7d7450bec",
            )
            .unwrap(),
            "ref".to_string(),
            Some(
                "sha256:05c6e08f1d9fdafa03147fcb8f82f124c76d2f70e3d989dc8aadb5e7d7450bec"
                    .to_string(),
            ),
        )
        .into_response();
        assert_eq!(response.status(), StatusCode::CREATED);
    }
}
