use crate::types::ManifestDeleted;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};

impl<'r> Responder<'r, 'static> for ManifestDeleted {
    fn respond_to(self, _req: &Request) -> response::Result<'static> {
        Response::build().status(Status::Accepted).ok()
    }
}
