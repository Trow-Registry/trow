use failure::{self, Error};
use std::fs::OpenOptions;
use trow_protobuf::backend::{CreateUuidRequest, Layer};
use trow_protobuf::backend_grpc::BackendClient;
use types::{create_upload_info, UploadInfo};
use state;
use std::io::prelude::*;

pub struct ClientInterface {
    backend: BackendClient,
}

impl ClientInterface {
    pub fn new(backend: BackendClient) -> Self {
        ClientInterface { backend }
    }

    //TODO: delete
    pub fn backend(&self) -> &BackendClient {
        &self.backend
    }

    /**
     * Ok so these functions are largely boilerplate to call the GRPC backend.
     * But doing it here lets us change things behind the scenes much cleaner.
     *
     * Frontend code becomes smaller and doesn't need to know about GRPC types.
     * In fact you could pull it out for a different implementation now by
     * just changing this file...
     **/

    /*
     * change this sodding name.
     */
    pub fn request_upload(&self, repo_name: &str) -> Result<UploadInfo, Error> {
        let mut req = CreateUuidRequest::new();
        req.set_repo_name(repo_name.to_owned());

        let response = self.backend.create_uuid(&req)?;
        debug!("Client received: {:?}", response);

        Ok(create_upload_info(
            response.get_uuid().to_owned(),
            repo_name.to_string(),
            (0, 0),
        ))
    }

    pub fn get_write_sink_for_upload (
        &self,
        repo_name: &str,
        uuid: &str,
    ) -> Result<impl Write, Error> {

        //TODO move path gen to backend and rearchitect.
        let mut req = Layer::new();
        req.set_repo_name(repo_name.to_owned());
        req.set_digest(uuid.to_owned());

        let response = self.backend.uuid_exists(&req)?;

        match response.get_success() {
            true => {
                let path = state::uuid::scratch_path(&uuid);
                let file = OpenOptions::new().create(true).append(true).open(path)?;
                Ok(file)
            }
            //TODO: return proper error
            false => Err(failure::err_msg("UUID unknown"))
        }
    }
}
