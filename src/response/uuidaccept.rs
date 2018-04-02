use failure;
use rocket::State;
use rocket::http::{Header, Status};
use rocket::response::{Responder, Response};
use rocket::request::Request;

use config;
use grpc::backend;
use types;
use response::errors;

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
}

fn construct_digest_path(layer: &types::Layer) -> String {
    format!("data/layers/{}/{}/{}", layer.name, layer.repo, layer.digest)
}

impl UuidAcceptResponse {
    pub fn handle(
        handler: State<config::BackendHandler>,
        name: String,
        repo: String,
        uuid: String,
        digest: String,
    ) -> Result<UuidAcceptResponse, failure::Error> {
        use std::fs;
        use std::path;
        // 1. copy file to new location
        let backend = handler.backend();
        let layer = types::Layer {
            name: name.clone(),
            repo: repo.clone(),
            digest: digest.clone(),
        };
        let digest_path = construct_digest_path(&layer);
        let path = format!("data/layers/{}/{}", layer.name, layer.repo);
        let scratch_path = format!("data/scratch/{}", uuid);
        debug!("Saving file");
        // 1.1 check direcory exists
        if !path::Path::new(&path).exists() {
            fs::create_dir_all(path)?;
        }
        fs::copy(&scratch_path, digest_path)?;
        // 2. delete uploaded temporary file
        debug!("Deleting file: {}", uuid);
        fs::remove_file(scratch_path)?;
        // 3. delete uuid from the backend
        let mut layer = backend::Layer::new();
        layer.set_name(name.clone());
        layer.set_repo(repo.clone());
        layer.set_digest(uuid.clone());
        let resp = backend.delete_uuid(&layer)?;
        // 4. Construct response
        if resp.get_success() {
            Ok(UuidAcceptResponse::UuidAccept{uuid, digest, name, repo})
        } else {
            warn!("Function is not implemented");
            Err(failure::err_msg("Not implemented"))
        }
    }

    pub fn delete_upload(
        handler: State<config::BackendHandler>,
        layer: &types::Layer,
    ) -> Result<UuidAcceptResponse, errors::Error> {
        let backend = handler.backend();
        let mut req = backend::Layer::new();
        req.set_name(layer.name.to_owned());
        req.set_repo(layer.repo.to_owned());
        req.set_digest(layer.digest.to_owned());

        //Log errors, don't send details to client
        let response = match backend.cancel_upload(&req) {
            Ok(r) => r,
            Err(e) => {
                //why can't I call error!?
                debug!("Error calling backend {:?}", e);
                return Err(errors::Error::InternalError);
            }
        };

        debug!("Return: {:?}", response);
        if response.get_success() {
            Ok(UuidAcceptResponse::UuidDelete)
        } else {
            Err(errors::Error::BlobUploadUnknown)
        }
    }
}

impl<'r> Responder<'r> for UuidAcceptResponse {
    fn respond_to(self, _req: &Request) -> Result<Response<'r>, Status> {
        use self::UuidAcceptResponse::*;

        match self {
            UuidAccept {
                name, repo, digest, ..
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
        }
    }
}

#[cfg(test)]
mod test {
    use rocket::http::Status;
    use response::uuid::UuidResponse;

    use response::test_helper::test_route;
    fn build_response() -> UuidResponse {
        UuidResponse::Uuid {
            // TODO: keep this as a real Uuid!
            uuid: String::from("whatever"),
            name: String::from("moredhel"),
            repo: String::from("test"),
            range: (0, 0),
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
