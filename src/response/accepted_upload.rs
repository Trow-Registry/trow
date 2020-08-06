use crate::response::get_base_url;
use crate::types::AcceptedUpload;
use rocket::http::{Header, Status};
use rocket::request::Request;
use rocket::response::{Responder, Response};

impl<'r> Responder<'r> for AcceptedUpload {
    fn respond_to(self, req: &Request) -> Result<Response<'r>, Status> {
        let location = format!(
            "{}/v2/{}/blobs/{}",
            get_base_url(req),
            self.repo_name(),
            self.digest()
        );
        debug!("accepted upload response");
        let location_header = Header::new("Location", location);
        let digest_header = Header::new("Docker-Content-Digest", self.digest().0.clone());
        let (left, right) = self.range();
        let range_header = Header::new("Range", format!("{}-{}", left, right));
        let length_header = Header::new("Content-Length", "0");

        Response::build()
            .status(Status::Created)
            .header(location_header)
            .header(digest_header)
            .header(range_header)
            .header(length_header)
            .ok()
    }
}

#[cfg(test)]
mod test {
    use crate::response::test_helper::test_route;
    use crate::types::{create_accepted_upload, AcceptedUpload, Digest};
    use crate::types::{RepoName, Uuid};
    use rocket::http::Status;

    fn build_response() -> AcceptedUpload {
        create_accepted_upload(
            Digest("123".to_string()),
            RepoName("moredhel/test".to_owned()),
            Uuid("whatever".to_owned()),
            (0, 0),
        )
    }

    #[test]
    fn test_resp() {
        let response = test_route(build_response());
        let headers = response.headers();
        assert_eq!(response.status(), Status::Created);
        assert!(headers.contains("Location"));
        assert!(headers.contains("Range"));
        assert!(headers.contains("Docker-Content-Digest"));
        assert!(headers.contains("Content-Length"));
    }
}
