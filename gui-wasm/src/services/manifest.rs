use yew::{services::fetch::FetchTask, Callback};

use crate::error::ApiError;
use crate::services::api::Api;
use crate::types::manifest::Manifest;

pub struct ManifestSvc {
    svc: Api,
}

impl ManifestSvc {
    pub fn new() -> Self {
        Self { svc: Api::new() }
    }

    pub fn fetch(
        &mut self,
        repository: String,
        reference: String,
        callback: Callback<Result<Manifest, ApiError>>,
    ) -> FetchTask {
        self.svc.get::<Manifest>(
            format!("/v2/{}/manifests/{}", repository, reference),
            callback,
        )
    }
}
