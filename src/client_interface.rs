use failure::Error;
use std::fs::OpenOptions;
use trow_protobuf::server::{UploadRequest, BlobRef};
use trow_protobuf::server_grpc::BackendClient;
use types::{create_upload_info, UploadInfo};
use std::io::prelude::*;

pub struct ClientInterface {
    backend: BackendClient,
}

impl ClientInterface {
    pub fn new(backend: BackendClient) -> Self {
        ClientInterface { backend }
    }

    /**
     * Ok so these functions are largely boilerplate to call the GRPC backend.
     * But doing it here lets us change things behind the scenes much cleaner.
     *
     * Frontend code becomes smaller and doesn't need to know about GRPC types.
     * In fact you could pull it out for a different implementation now by
     * just changing this file...
     **/

    pub fn request_upload(&self, repo_name: &str) -> Result<UploadInfo, Error> {
        let mut req = UploadRequest::new();
        req.set_repo_name(repo_name.to_owned());

        let response = self.backend.request_upload(&req)?;

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

        let mut br = BlobRef::new();
        br.set_uuid(uuid.to_owned());
        br.set_repo_name(repo_name.to_owned());

        let resp = self.backend.get_write_location_for_blob(&br)?;
        
        //For the moment we know it's a file location
        let file = OpenOptions::new().create(true).append(true).open(resp.path)?;
        Ok(file)
    }
    
}
