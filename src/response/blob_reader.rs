use crate::types::BlobReader;
use rocket::http::Header;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};

impl<'r> Responder<'r> for BlobReader {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        let ct = Header::new("Content-Type", "application/octet-stream");
        let digest = Header::new("Docker-Content-Digest", self.digest().0.clone());

        // Important to used sized_body in order to have content length set correctly
        let mut resp = Response::build().sized_body(self.get_reader()).ok()?;
        resp.set_header(ct);
        resp.set_header(digest);

        Ok(resp)
    }
}
