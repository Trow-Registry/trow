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
DummyResponder!(UuidResponse);

impl RegistryTrait for UuidResponse {
    fn ok<'r>(&self, _req: &Request) -> Result<Response<'r>, Status> {
        debug!("Uuid Ok");

        if let &UuidResponse::Uuid {ref uuid, ref name, ref repo, ref left, ref right} = self {
            let location_url = format!("{}/v2/{}/{}/blobs/uploads/{}?q=t",
                                       BASE_URL,
                                       name,
                                       repo,
                                       uuid);
            let upload_uuid = Header::new("Docker-Upload-UUID", uuid.clone());
            let range = Header::new("Range", format!("{}-{}", left, right));
            let length = Header::new("Content-Length", format!("{}", right - left));
            let location = Header::new("Location", location_url);

            debug!("Range: {}-{}, Length: {}", left, right, right-left);
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

    fn err<'r>(&self, _req: &Request) -> Result<Response<'r>, Status> {
        debug!("Uuid Error");
        Response::build()
            .status(Status::NotFound)
            .ok()
    }
}

