use std::io::Cursor;

use rocket::http::ContentType;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{Responder, Response};


use crate::types::MetricsResponse;

impl<'r> Responder<'r> for MetricsResponse {
    fn respond_to(self, _req: &Request) -> Result<Response<'r>, Status> {
        let text = self.metrics;

        Response::build()
        .header(ContentType::Plain)
        .sized_body(Cursor::new(text))
        .status(Status::Ok)
        .ok()
    }
}
