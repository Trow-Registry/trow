use crate::registry_interface::ManifestReader;
use rocket::http::Header;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};

impl<'r> Responder<'r> for ManifestReader {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        let ct = Header::new("Content-Type", self.content_type().to_string());
        let digest = Header::new("Docker-Content-Digest", self.digest().to_string());

        // Important to used sized_body in order to have content length set correctly
        let mut resp = Response::build().sized_body(self.get_reader()).ok()?;
        resp.set_header(ct);
        resp.set_header(digest);

        Ok(resp)
    }
}
