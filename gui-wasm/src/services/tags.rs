use serde::{Deserialize, Serialize};
use yew::{services::fetch::FetchTask, Callback};

use crate::error::ApiError;
use crate::services::api::Api;

pub struct TagsSvc {
    svc: Api,
}

impl TagsSvc {
    pub fn new() -> Self {
        Self { svc: Api::new() }
    }

    pub fn fetch(
        &mut self,
        repository: String,
        callback: Callback<Result<TagsResponse, ApiError>>,
    ) -> FetchTask {
        self.svc.get::<TagsResponse>(
            format!("{}/v2/{}/tags/list", self.svc.base_url, repository),
            callback,
        )
    }

    // pub fn fetch_by_limit(
    //     &mut self,
    //     limit: u32,
    //     repository: String,
    //     callback: Callback<Result<TagsResponse, ApiError>>,
    // ) -> FetchTask {
    //     self.svc
    //         .get::<TagsResponse>(format!("/v2/{}/tags/list?{}", repository, limit), callback)
    // }

    // pub fn fetch_by_limit_and_last_tag(
    //     &mut self,
    //     limit: u32,
    //     repository: String,
    //     last_tag: String,
    //     callback: Callback<Result<TagsResponse, ApiError>>,
    // ) -> FetchTask {
    //     self.svc.get::<TagsResponse>(
    //         format!("/v2/{}/tags/list?{}&{}", repository, limit, last_tag),
    //         callback,
    //     )
    // }
}

#[derive(Serialize, Default, Deserialize, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct TagsResponse {
    pub name: String,
    pub tags: Vec<String>,
}
