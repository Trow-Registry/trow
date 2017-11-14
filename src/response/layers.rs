// use std::io::{Error, ErrorKind};
use failure::Error;

use rocket::State;
use rocket::http::{Header, Status};
use rocket::response::{Responder, Response};
use rocket::request::Request;

use config;
use errors;
use util;
use http_capnp::lycaon;

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
        util::connect_backend(&config)
            .and_then(|mut handler: util::CapnpConnection| {
                let mut msg = handler.builder.init_root::<lycaon::layer::Builder>();
                let mut req = handler.proxy.layer_exists_request();
                msg.set_digest(&digest);
                msg.set_name(&name);
                msg.set_repo(&repo);
                req.get()
                    .set_layer(msg.as_reader())
                    .map_err(|e| Error::from(e))
                    .and(handler.core.and_then(|mut core| {
                        core.run(req.send().promise).map_err(|e| Error::from(e))
                    }))
                    .and_then(|response| {
                        response.get().map_err(|e| Error::from(e)).and_then(
                            |response| {
                                response.get_result().map_err(|e| Error::from(e)).and_then(
                                    |response| {
                                        let exists = response.get_exists();
                                        let length = response.get_length();
                                        match exists {
                                            true => Ok(LayerExists::True { digest, length }),
                                            false => Err(
                                                errors::Server::FileNotFound(
                                                    format!("{}", digest.clone()),
                                                ).into(),
                                            ),
                                        }
                                    },
                                )
                            },
                        )
                    })
                    .map_err(|e| e.into())
            })
            .map_err(|e| {
                warn!("{}", e.cause());
                errors::Client::BLOB_UNKNOWN.into()
            })

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
