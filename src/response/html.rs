use std::io::Cursor;

use rocket::request::Request;
use rocket::http::{ContentType, Status};
use rocket::response::{Responder, Response};

pub struct HTML<'a>(pub &'a str);

impl<'a> Responder<'a> for HTML<'a> {
    fn respond_to(self, _: &Request) -> Result<Response<'a>, Status> {
        Response::build()
            .header(ContentType::HTML)
            .sized_body(Cursor::new(self.0))
            .ok()
    }
}
