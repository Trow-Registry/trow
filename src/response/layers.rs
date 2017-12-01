use failure::Error;

use rocket::State;
use rocket::http::{Header, Status};
use rocket::response::{Responder, Response};
use rocket::request::Request;

use config;
use errors;
use util;

#[derive(Debug)]
pub enum LayerExists {
    True { digest: String, length: u64 },
    False,
}

impl LayerExists {
    pub fn handle(
        config: State<config::Config>,
        name: String,
        repo: String,
        digest: String,
    ) -> Result<LayerExists, Error> {
        use std;
        Err(Error::from(std::fmt::Error))
    }
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
            }
            LayerExists::False => Response::build().status(Status::NotFound).ok(),
        }
    }
}
