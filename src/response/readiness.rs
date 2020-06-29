use rocket::http::ContentType;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Responder, Response};
use crate::types::ReadinessResponse;


impl<'r> Responder<'r> for ReadinessResponse {
    fn respond_to(self, req: &Request) -> Result<Response<'r>, Status> {
        if self.is_ready {
            Response::build()
                .status(Status::Ok)
                .header(ContentType::JSON)
                .ok()
        } else {
            Response::build()
            .status(Status::BadRequest)
            .header(ContentType::JSON)
            .ok()
        }
    }
}
