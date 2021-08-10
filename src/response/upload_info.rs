use crate::response::get_base_url;
pub use crate::types::{create_upload_info, UploadInfo};
use rocket::http::{Header, Status};
use rocket::request::Request;
use rocket::response::{Responder, Response};

impl<'r> Responder<'r, 'static> for UploadInfo {
    fn respond_to(self, req: &Request) -> Result<Response<'static>, Status> {
        let location_url = format!(
            "{}/v2/{}/blobs/uploads/{}",
            get_base_url(req),
            self.repo_name(),
            self.uuid()
        );
        let (left, right) = self.range();
        let upload_uuid = Header::new("Docker-Upload-UUID", self.uuid().0.clone());
        let range = Header::new("Range", format!("{}-{}", left, right));
        let length = Header::new("X-Content-Length", format!("{}", right - left));
        let location = Header::new("Location", location_url);

        debug!("Range: {}-{}, Length: {}", left, right, right - left);

        Response::build()
            .header(upload_uuid)
            .header(location)
            .header(range)
            .header(length)
            // TODO: move into the type so it is better encoded?...
            .status(Status::Accepted)
            .ok()
    }
}

#[cfg(test)]
mod test {
    use crate::response::upload_info::{create_upload_info, UploadInfo};
    use crate::types::{RepoName, Uuid};
    use rocket::http::Status;
    use rocket::response::Responder;

    use crate::response::test_helper::test_client;
    fn build_response() -> UploadInfo {
        create_upload_info(
            Uuid("whatever".to_owned()),
            RepoName("moredhel/test".to_owned()),
            (0, 0),
        )
    }

    #[test]
    fn uuid_uuid() {
        let cl = test_client();
        let req = cl.get("/");
        let response = build_response().respond_to(req.inner()).unwrap();
        let headers = response.headers();
        assert_eq!(response.status(), Status::Accepted);
        assert!(headers.contains("Docker-Upload-UUID"));
        assert!(headers.contains("Location"));
        assert!(headers.contains("Range"));
    }
}
