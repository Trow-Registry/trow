use rocket::http::{Header, Status};
use rocket::response::{Responder, Response};
use rocket::request::Request;

use response::RegistryTrait;

// TODO refactor this out in place of dynamic updates
const BASE_URL: &str = "http://localhost:8000";

#[derive(Debug, Serialize)]
pub enum UuidResponse {
    Uuid {
        uuid: String,
        name: String,
        repo: String,
        left: u32,
        right: u32,
    },
    Empty,
}

impl RegistryTrait for UuidResponse {
    fn ok<'r>(self, _req: &Request) -> Result<Response<'r>, Status> {
        debug!("Uuid Ok");

        if let UuidResponse::Uuid {uuid, name, repo, left, right} = self {
            let location_url = format!("{}/v2/{}/{}/blobs/uploads/{}?query=true", BASE_URL, name, repo, uuid);
            let upload_uuid = Header::new("Docker-Upload-UUID", uuid);
            let range = Header::new("Range", format!("{}-{}", left, right));
            debug!("Range: {}-{}, Length: {}", left, right, right-left);
            let length = Header::new("Content-Length", format!("{}", right - left));
            let location = Header::new("Location", location_url);
            Response::build()
                .header(upload_uuid)
                .header(location)
                .header(range)
                .header(length)
                // TODO: move into the type so it is better encoded?...
                .status(Status::Accepted)
                .ok()
        } else { panic!("oh noes!") }
    }

    fn err<'r>(self, _req: &Request) -> Result<Response<'r>, Status> {
        debug!("Uuid Error");
        Response::build()
            .status(Status::NotFound)
            .ok()
    }
}

impl<'r> Responder<'r> for UuidResponse {
    fn respond_to(self, _: &Request) -> Result<Response<'r>, Status> {
        panic!("please use the RegistryTrait::{ok,err} functions")
    }
}

