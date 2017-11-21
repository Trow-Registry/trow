use failure::Error;
use rocket::State;
use rocket::http::{Header, Status};
use rocket::response::{Responder, Response};
use rocket::request::Request;


use config;
use controller::uuid as cuuid;
use errors;
use state;
use util;
use http_capnp::lycaon;

const BASE_URL: &str = "http://localhost:8000";

#[derive(Debug, Serialize)]
pub enum UuidAcceptResponse {
    DigestMismatch,
    UuidAccept {
        uuid: String,
        digest: String,
        name: String,
        repo: String,
    },
    UuidDelete,
    UnknownError,
}

impl UuidAcceptResponse {
    pub fn handle(
        config: State<config::Config>,
        name: String,
        repo: String,
        uuid: String,
        digest: cuuid::DigestStruct,
    ) -> Result<UuidAcceptResponse, Error> {
        let mut handler = util::CapnpInterface::uuid_interface(&config)?;
        let mut msg = handler
            .builder
            .init_root::<lycaon::uuid_interface::uuid::Builder>();
        let proxy = handler.proxy.and_then(|proxy| {
            // TODO: this is a current hack to get around dynamic dispatch issues
            // with the proxy handler. This is _super_ fragile!
            if let util::CapnpInterface::Uuid(client) = proxy {
                Ok(client)
            } else {
                Err(errors::Server::CapnpInterfaceError("Uuid").into())
            }
        })?;

        let mut req = proxy.save_layer_request();
        msg.set_uuid(&uuid);
        let _response = req.get().set_uuid(msg.as_reader())
            .map_err(|e| Error::from(e))
            .and(handler.core.and_then(|mut core| {
                core.run(req.send().promise).map_err(|e| Error::from(e))
            }))?;
        let _hash = state::uuid::hash_file(state::uuid::scratch_path(&uuid))
            .and_then(|hash| {
                if hash != digest.digest {
                    return Err(errors::Client::DIGEST_INVALID.into());
                }
                Ok(hash)
            })?;
        state::uuid::save_layer(&uuid, &digest.digest)?;
        state::uuid::mark_delete(&uuid)?;

        Ok(UuidAcceptResponse::UuidAccept {
            uuid,
            digest: digest.digest,
            name,
            repo,
        })
    }
}

impl<'r> Responder<'r> for UuidAcceptResponse {
    fn respond_to(self, _req: &Request) -> Result<Response<'r>, Status> {
        use self::UuidAcceptResponse::*;

        match self {
            UuidAccept {
                name,
                repo,
                digest,
                uuid: _,
            } => {
                let location = format!("{}/v2/{}/{}/blobs/{}", BASE_URL, name, repo, digest);
                let location = Header::new("Location", location);
                let digest = Header::new("Docker-Content-Digest", digest);
                Response::build()
                    .status(Status::Created)
                    .header(location)
                    .header(digest)
                    .ok()
            }
            DigestMismatch => {
                debug!("Digest mismatched");
                Response::build().status(Status::NotFound).ok()
            }
            UuidDelete => Response::build().status(Status::NoContent).ok(),
            UnknownError => Response::build().status(Status::NotFound).ok(),
        }
    }
}

#[cfg(test)]
mod test {
    use rocket::http::Status;
    use response::uuid::UuidResponse;

    use test::test_helpers::test_route;
    fn build_response() -> UuidResponse {
        UuidResponse::Uuid {
            // TODO: keep this as a real Uuid!
            uuid: String::from("whatever"),
            name: String::from("moredhel"),
            repo: String::from("test"),
            left: 0,
            right: 0,
        }
    }

    #[test]
    fn uuid_uuid() {
        let response = test_route(build_response());
        let headers = response.headers();
        assert_eq!(response.status(), Status::Accepted);
        assert!(headers.contains("Docker-Upload-UUID"));
        assert!(headers.contains("Location"));
        assert!(headers.contains("Range"));
    }

    #[test]
    fn uuid_empty() {
        let response = test_route(UuidResponse::Empty);
        assert_eq!(response.status(), Status::NotFound);
    }
}
