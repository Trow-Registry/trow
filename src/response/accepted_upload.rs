use failure;
use rocket::http::{Header, Status};
use rocket::request::Request;
use rocket::response::{Responder, Response};
use rocket::State;
use response::get_base_url;
use client_interface::ClientInterface;

use trow_protobuf::backend;
use types;

#[derive(Debug, Serialize)]
pub struct AcceptedUpload {
    uuid: String,
    digest: String,
    repo_name: String,
}

fn create_accepted_upload(uuid: String, digest: String, repo_name: String) -> AcceptedUpload {
    AcceptedUpload {
        uuid,
        digest,
        repo_name,
    }
}

fn construct_digest_path(layer: &types::Layer) -> String {
    format!("data/layers/{}/{}", layer.repo_name, layer.digest)
}

impl AcceptedUpload {
    pub fn handle(
        handler: State<ClientInterface>,
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
            Ok(create_accepted_upload(uuid, digest, repo_name))
        } else {
            warn!("Failed to remove UUID");
            Err(failure::err_msg("Not implemented"))
        }
    }
}

impl<'r> Responder<'r> for AcceptedUpload {
    fn respond_to(self, req: &Request) -> Result<Response<'r>, Status> {

        let location = format!("{}/v2/{}/blobs/{}", get_base_url(req), self.repo_name, self.digest);
        let location_header = Header::new("Location", location);
        let digest_header = Header::new("Docker-Content-Digest", self.digest);
        Response::build()
            .status(Status::Created)
            .header(location_header)
            .header(digest_header)
            .ok()
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
            (0, 0),
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
