use failure::Error;
use trow_protobuf::backend::{CreateUuidRequest, Layer};
use trow_protobuf::backend_grpc::BackendClient;
use types::{self, create_upload_info, UploadInfo};

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

    //TODO: Change to get path for uuid
    //TODO: layer type change is shite
    pub fn uuid_exists(&self, layer: &types::Layer) -> Result<bool, Error> {
        let mut req = Layer::new();
        req.set_repo_name(layer.repo_name.to_owned());
        req.set_digest(layer.digest.to_owned());

        let response = self.backend.uuid_exists(&req)?;
        //TODO: get_success is probably a really bad overloading
        //would be better to use an option or result somehow
        //Returning Ok(false) seems pure shite
        debug!("UuidExists: {:?}", response.get_success());
        Ok(response.get_success())
    }
}
