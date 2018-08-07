use grpc::backend_grpc::BackendClient;
use grpc::backend::CreateUuidRequest;
use failure::Error;
use types::{UploadInfo, create_upload_info};


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
    pub fn create_uuid(&self, repo_name: &str) -> Result<UploadInfo, Error> {
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
}
