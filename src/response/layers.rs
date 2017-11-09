use rocket::http::{Header, Status};
use rocket::response::{Responder, Response};
use rocket::request::Request;

#[derive(Debug)]
pub enum LayerExists {
    True {
        digest: String,
        length: u64,
    },
    False,
}
    

impl<'r> Responder<'r> for LayerExists {
    fn respond_to(self, _req: &Request) -> Result<Response<'r>, Status> {
        match self {
            LayerExists::True { digest, length } => {
                let digest_header = Header::new("Docker-Content-Digest", digest);
                // TODO: figure out what is wrong here.
                let content_length = Header::new("X-Content-Length", format!("{}", length));
                Response::build()
                    .header(digest_header)
                    .header(content_length)
                    .ok()
            },
            LayerExists::False => {
                Response::build()
                    .status(Status::NotFound)
                    .ok()
            },
        }
    }
}
