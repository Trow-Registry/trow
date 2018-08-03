use failure;
use rocket::State;
use rocket::http::{Header, Status};
use rocket::request::Request;
use rocket::response::{Responder, Response};

use grpc::backend;
use response::errors;
use types;
use backend as be;

//TODO: WTF?
const BASE_URL: &str = "http://localhost:8000";

//shouldn't be an enum
#[derive(Debug, Serialize)]
pub enum AcceptedUpload {
    DigestMismatch,
    UuidAccept {
        uuid: String,
        digest: String,
        repo_name: String,
    },
    UuidDelete,
}

fn construct_digest_path(layer: &types::Layer) -> String {
    format!("data/layers/{}/{}", layer.repo_name, layer.digest)
}

impl AcceptedUpload {
    pub fn handle(
        handler: State<be::BackendHandler>,
        repo_name: String,
        uuid: String,
        digest: String,
    ) -> Result<AcceptedUpload, failure::Error> {
        use std::fs;
        use std::path;
        // 1. copy file to new location
        let backend = handler.backend();
        let layer = types::Layer {
            repo_name: repo_name.clone(),
            digest: digest.clone(),
        };
        let digest_path = construct_digest_path(&layer);
        let path = format!("data/layers/{}", layer.repo_name);
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
        // TODO is this process right? Should the backend be doing this?!
        let mut layer = backend::Layer::new();
        layer.set_repo_name(repo_name.clone());
        layer.set_digest(uuid.clone());
        let resp = backend.delete_uuid(&layer)?;
        // 4. Construct response
        if resp.get_success() {
            Ok(AcceptedUpload::UuidAccept {
                uuid,
                digest,
                repo_name,
            })
        } else {
            warn!("Failed to remove UUID");
            Err(failure::err_msg("Not implemented"))
        }
    }

    pub fn delete_upload(
        handler: State<be::BackendHandler>,
        layer: &types::Layer,
    ) -> Result<AcceptedUpload, errors::Error> {
        let backend = handler.backend();
        let mut req = backend::Layer::new();
        req.set_repo_name(layer.repo_name.to_owned());
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
            Ok(AcceptedUpload::UuidDelete)
        } else {
            Err(errors::Error::BlobUploadUnknown)
        }
    }
}

impl<'r> Responder<'r> for AcceptedUpload {
    fn respond_to(self, _req: &Request) -> Result<Response<'r>, Status> {
        use self::AcceptedUpload::{DigestMismatch, UuidAccept, UuidDelete};

        match self {
            UuidAccept {
                repo_name, digest, ..
            } => {
                let location = format!("{}/v2/{}/blobs/{}", BASE_URL, repo_name, digest);
                let location = Header::new("Location", location);
                let digest = Header::new("Docker-Content-Digest", digest);
                Response::build()
                    .status(Status::Created)
                    .header(location)
                    .header(digest)
                    .ok()
            }
            DigestMismatch => {
                //TODO: Needs to be an error. Fix this FFS.
                warn!("Digest mismatched");
                Response::build().status(Status::NotFound).ok()
            }
            UuidDelete => Response::build().status(Status::NoContent).ok(),
        }
    }
}

#[cfg(test)]
mod test {
    use response::upload_info::{create_upload_info, UploadInfo};
    use rocket::http::Status;

    use response::test_helper::test_route;
    fn build_response() -> UploadInfo {
        create_upload_info(
            String::from("whatever"),
            String::from("moredhel/test"),
            (0, 0)
        )
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
}
