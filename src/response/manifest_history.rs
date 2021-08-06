use std::io::Cursor;

use crate::registry_interface::ManifestHistory;
use rocket::http::ContentType;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};

impl<'r> Responder<'r, 'static> for ManifestHistory {
    fn respond_to(self, _req: &Request) -> response::Result<'static> {
        let json = serde_json::to_string(&self).unwrap();

        Response::build()
            .header(ContentType::JSON)
            .sized_body(None, Cursor::new(json))
            .ok()
    }
}
