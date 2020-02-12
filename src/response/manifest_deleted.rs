use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use rocket::http::Status;
use crate::types::ManifestDeleted;

impl<'r> Responder<'r> for ManifestDeleted {
    fn respond_to(self, _req: &Request) -> response::Result<'r> {

        Response::build().status(Status::Accepted).ok()
    }
}