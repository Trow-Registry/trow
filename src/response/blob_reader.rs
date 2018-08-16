use rocket::http::Header;
use rocket::request::Request;
use rocket::response::{self, Responder, Stream};
use types::BlobReader;

impl<'r> Responder<'r> for BlobReader {
    fn respond_to(self, req: &Request) -> response::Result<'r> {

        let mut resp = Stream::from(self.get_reader()).respond_to(req)?;
        //Not sure this is the right content type
        let ct = Header::new(
                    "Content-Type",
                    "application/octet-stream",
                );
        resp.set_header(ct);

        Ok(resp)
    }
}