use rocket::http::Status;
use rocket::response::{Responder, Response};
use rocket::request::Request;
use response::{RegistryTrait};

#[derive(Debug, Serialize)]
pub struct Empty;

impl RegistryTrait for Empty {
    fn ok<'r>(self, _req: &Request) -> Result<Response<'r>, Status> {
        debug!("Empty Ok");
        Response::build()
            .ok()
    }
    fn err<'r>(self, _req: &Request) -> Result<Response<'r>, Status> {
        debug!("Empty Error");
        Response::build()
            .ok()
    }
}

impl<'r> Responder<'r> for Empty {
    fn respond_to(self, _: &Request) -> Result<Response<'r>, Status> {
        panic!("please use the RegistryTrait::{ok,err} functions")
    }
}
