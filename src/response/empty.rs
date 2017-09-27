use rocket::http::Status;
use rocket::response::{Responder, Response};
use rocket::request::Request;
use response::RegistryTrait;

#[derive(Debug, Serialize)]
pub struct Empty;
DummyResponder!(Empty);

impl RegistryTrait for Empty {
    fn ok<'r>(&self, _req: &Request) -> Result<Response<'r>, Status> {
        Response::build()
            .ok()
    }
}
