use rocket::http::Header;
use rocket::request::Request;
use rocket::response::{self, Responder, Stream};
use types::ManifestReader;

impl<'r> Responder<'r> for ManifestReader {
    fn respond_to(self, req: &Request) -> response::Result<'r> {

        let ct = Header::new(
                    "Content-Type",
                    self.content_type().to_string(),
                );
        let mut resp = Stream::from(self.get_reader()).respond_to(req)?;
        resp.set_header(ct);

        Ok(resp)
    }
}