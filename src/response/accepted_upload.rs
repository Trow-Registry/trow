use rocket::http::{Header, Status};
use rocket::request::Request;
use rocket::response::{Responder, Response};
use response::get_base_url;
use types::AcceptedUpload;


impl<'r> Responder<'r> for AcceptedUpload {
    fn respond_to(self, req: &Request) -> Result<Response<'r>, Status> {

        let location = format!("{}/v2/{}/blobs/{}", get_base_url(req), self.repo_name(), self.digest());
        debug!("accepted upload response");
        let location_header = Header::new("Location", location);
        let digest_header = Header::new("Docker-Content-Digest", self.digest().0.clone());
        Response::build()
            .status(Status::Created)
            .header(location_header)
            .header(digest_header)
            .ok()
    }
}

#[cfg(test)]
mod test {
    use response::upload_info::{create_upload_info, UploadInfo};
    use rocket::http::Status;
    use types::{Uuid, RepoName};

    use response::test_helper::test_route;
    fn build_response() -> UploadInfo {
        create_upload_info(
            Uuid("whatever".to_owned()),
            RepoName("moredhel/test".to_owned()),
            (0, 0),
        )
    }

    #[test]
    fn uuid_uuid() {
        let response = test_route(build_response());
        let headers = response.headers();
        assert_eq!(response.status(), Status::Accepted);
        assert!(headers.contains("Docker-Upload-UUID"));
        assert!(headers.contains("Location"));
        assert!(headers.contains("Range"));
    }
}
