use crate::response::get_base_url;
use crate::types::VerifiedManifest;
use rocket::http::Header;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Responder, Response};

impl<'r> Responder<'r, 'static> for VerifiedManifest {
    fn respond_to(self, req: &Request) -> Result<Response<'static>, Status> {
        //The front end is responsible for assembling URLs, backend should deal in arguments
        let location = format!(
            "{}/v2/{}/manifests/{}",
            get_base_url(req),
            self.repo_name(),
            self.tag()
        );
        let location_header = Header::new("Location", location);
        let digest = Header::new("Docker-Content-Digest", self.digest().to_string());
        Response::build()
            .status(Status::Created)
            .header(location_header)
            .header(digest)
            .ok()
    }
}

#[cfg(test)]
mod test {
    use crate::registry_interface::{Digest, DigestAlgorithm};
    use crate::response::test_helper::test_client;
    use crate::types::{create_verified_manifest, RepoName};
    use rocket::http::Status;
    use rocket::response::Responder;

    #[test]
    fn accepted_ok() {
        let cl = test_client();
        let req = cl.get("/");
        let response = create_verified_manifest(
            RepoName("repo_name".to_string()),
            Digest {
                algo: DigestAlgorithm::Sha256,
                hash: "05c6e08f1d9fdafa03147fcb8f82f124c76d2f70e3d989dc8aadb5e7d7450bec"
                    .to_string(),
            },
            "ref".to_string(),
        )
        .respond_to(req.inner())
        .unwrap();
        assert_eq!(response.status(), Status::Created);
    }
}
