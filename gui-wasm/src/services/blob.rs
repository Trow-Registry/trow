use yew::{services::fetch::FetchTask, Callback};

use crate::services::api::Api;

use crate::error::ApiError;
use crate::types::blob::Blob;

pub struct BlobSvc {
    svc: Api,
}

impl BlobSvc {
    pub fn new() -> Self {
        Self { svc: Api::new() }
    }

    pub fn fetch(
        &mut self,
        repository: String,
        digest: String,
        callback: Callback<Result<Blob, ApiError>>,
    ) -> FetchTask {
        self.svc
            .get::<Blob>(format!("/v2/{}/blobs/{}", repository, digest), callback)
    }
}
