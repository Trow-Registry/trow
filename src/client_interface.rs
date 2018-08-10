use failure::Error;
use std::fs::OpenOptions;
use trow_protobuf::server::{UploadRequest, BlobRef};
use trow_protobuf::server_grpc::BackendClient;
use types::{create_upload_info, UploadInfo, AcceptedUpload, Layer, create_accepted_upload};
use std::io::prelude::*;
use std::path::Path;
use std::fs;

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

    pub fn complete_upload(&self, repo_name: &str, uuid: &str, digest: &str) -> Result<AcceptedUpload, Error> {

         // 1. copy file to new location
        //let backend = handler.backend();
        let layer = Layer {
            repo_name: repo_name.to_owned(),
            digest: digest.to_owned(),
        };

        let digest_path = format!("data/layers/{}/{}", layer.repo_name, layer.digest);
        let path = format!("data/layers/{}", layer.repo_name);
        let scratch_path = format!("data/scratch/{}", uuid);
        debug!("Saving file");
        // 1.1 check direcory exists
        if !Path::new(&path).exists() {
            fs::create_dir_all(path)?;
        }
        fs::copy(&scratch_path, digest_path)?;
        // 2. delete uploaded temporary file
        debug!("Deleting file: {}", uuid);
        fs::remove_file(scratch_path)?;
        Ok(create_accepted_upload(uuid.to_owned(), digest.to_owned(), repo_name.to_owned()))
        // 3. delete uuid from the backend
        // TODO is this process right? Should the backend be doing this?!
        /*
        let mut layer = server::Layer::new();
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
        */
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
