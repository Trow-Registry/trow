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
        let mut handler = util::CapnpInterface::layer_interface(&config)?;
        let mut msg = handler.builder.init_root::<lycaon::layer::Builder>();
        let proxy = handler.proxy.and_then(|proxy| {
            // TODO: this is a current hack to get around dynamic dispatch issues
            // with the proxy handler
            if let util::CapnpInterface::Layer(client) = proxy {
                Ok(client)
            } else {
                Err(errors::Server::CapnpInterfaceError("Layer").into())
            }
        })?;
        let mut req = proxy.layer_exists_request();
        msg.set_digest(&digest);
        msg.set_name(&name);
        msg.set_repo(&repo);
        let response = req.get()
            .set_layer(msg.as_reader())
            .map_err(|e| Error::from(e))
            .and(handler.core.and_then(|mut core| {
                core.run(req.send().promise).map_err(|e| Error::from(e))
            }))?;
        let response = response.get()?;
        let response = response.get_result()?;
        let exists = response.get_exists();
        let length = response.get_length();
        match exists {
            true => Ok(LayerExists::True { digest, length }),
            false => Err(
                errors::Server::FileNotFound(format!("{}", digest.clone())).into(),
            ),
        }
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
