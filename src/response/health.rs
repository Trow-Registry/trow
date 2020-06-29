use rocket::http::ContentType;
use rocket::http::{Header, Status};
use rocket::request::Request;
use rocket::response::{Responder, Response};
use crate::types::HealthResponse;

impl<'r> Responder<'r> for HealthResponse {
    fn respond_to(self, req: &Request) -> Result<Response<'r>, Status> {
        if self.is_healthy {
            Response::build()
                .status(Status::Ok)
                .header(ContentType::JSON)
                .ok()

        } else {
            Response::build()
                .status(Status::InternalServerError)
                .header(ContentType::JSON)
                .ok()
        }
    }
}
    