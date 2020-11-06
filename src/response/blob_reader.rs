use crate::types::BlobReader;
use rocket::http::Header;
use rocket::request::Request;
use rocket::response::{self, Responder, Stream};

impl<'r> Responder<'r> for BlobReader {
    fn respond_to(self, req: &Request) -> response::Result<'r> {
        let ct = Header::new("Content-Type", "application/octet-stream");
        let digest = Header::new("Docker-Content-Digest", self.digest().0.clone());

        let mut resp = Stream::from(self.get_reader()).respond_to(req)?;
        resp.set_header(ct);
        resp.set_header(digest);

        Ok(resp)
    }
}
