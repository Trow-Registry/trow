use failure::Error;

use rocket::State;
use rocket::http::{Header, Status};
use rocket::response::{Responder, Response};
use rocket::request::Request;

use config;
use types::Layer;

use grpc::backend;
use util;

#[derive(Debug, Clone)]
pub enum LayerExists {
    True { digest: String, length: u64 },
    False,
}

impl LayerExists {
    pub fn handle(
        handler: State<config::BackendHandler>,
        layer: Layer,
    ) -> Result<LayerExists, Error> {
        let backend = handler.backend();

        let mut proto_layer = backend::Layer::new();
        proto_layer.set_name(layer.name);
        proto_layer.set_repo(layer.repo);
        proto_layer.set_digest(layer.digest.clone());

        let reply = backend
            .layer_exists(proto_layer)
            .expect("layerexists RPC failed");
        debug!("Client received: {:?}", reply);

        match reply.get_success() {
            true => Ok(LayerExists::True {
                digest: layer.digest,
                length: reply.get_length(),
            }),
            false => Err(util::std_err("blob doesn't exist")),
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

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen, QuickCheck, TestResult};

    impl Arbitrary for LayerExists {
        fn arbitrary<G>(g: &mut G) -> Self
        where
            G: Gen,
        {
            let digest_len = g.gen_range(1, 256);
            let length = g.gen_range(1, 256000000);

            let digest: String = g.gen_ascii_chars().take(digest_len).collect();
            let digest: String = format!("sha256:{}", digest);

            LayerExists::True { digest, length }
        }
    }

    #[test]
    #[ignore]
    fn test_process_layer() {
        fn inner(_layer: LayerExists) -> TestResult {
            TestResult::failed()
        }
        QuickCheck::new()
            .tests(100)
            .max_tests(1000)
            .quickcheck(inner as fn(LayerExists) -> TestResult);
    }
}
