use std::io::Cursor;

use crate::types::TagList;
use rocket::http::ContentType;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};

impl<'r> Responder<'r, 'static> for TagList {
    fn respond_to(self, _req: &Request) -> response::Result<'static> {
        let json = serde_json::to_string(&self).unwrap();

        Response::build()
            .header(ContentType::JSON)
            .sized_body(None, Cursor::new(json))
            .ok()
    }
}
