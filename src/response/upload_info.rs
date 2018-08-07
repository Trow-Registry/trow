use response::get_base_url;
use rocket::http::{Header, Status};
use rocket::request::Request;
use rocket::response::{Responder, Response};
pub use types::{create_upload_info, UploadInfo};

impl<'r> Responder<'r> for UploadInfo {
    fn respond_to(self, req: &Request) -> Result<Response<'r>, Status> {
        let location_url = format!(
            "{}/v2/{}/blobs/uploads/{}",
            get_base_url(req),
            self.repo_name(),
            self.uuid()
        );
        let (left, right) = self.range();
        let upload_uuid = Header::new("Docker-Upload-UUID", self.uuid().to_owned());
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
    use response::upload_info::{UploadInfo, create_upload_info};
    use rocket::http::Status;

    use response::test_helper::test_route;
    fn build_response() -> UploadInfo {
        create_upload_info(String::from("whatever"), String::from("moredhel/test"), (0, 0))
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
