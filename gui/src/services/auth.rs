use serde::{Deserialize, Serialize};
use yew::{services::fetch::FetchTask, Callback};

use crate::error::ApiError;
use crate::services::api::Api;

#[derive(Serialize, Default, Deserialize, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct AuthResponse {
    pub token: String,
}

pub struct AuthSvc {
    svc: Api,
}

impl AuthSvc {
    pub fn new() -> Self {
        Self { svc: Api::new() }
    }

    pub fn login(
        &mut self,
        user: String,
        password: String,
        callback: Callback<Result<AuthResponse, ApiError>>,
    ) -> FetchTask {
        let encoded_auth = base64::encode(format!("{}:{}", user, password));
        self.svc
            .get::<AuthResponse>("/login".to_string(), Some(encoded_auth), callback)
    }
}
