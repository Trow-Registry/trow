use trow_protobuf::backend_grpc::BackendClient;

pub struct ClientInterface {
    backend: BackendClient,
}

impl ClientInterface {
    pub fn new(backend: BackendClient) -> Self {
        ClientInterface { backend }
    }

    pub fn backend(&self) -> &BackendClient {
        &self.backend
    }
}