use serde::Deserialize;
use yew::services::{
    fetch::{FetchService, FetchTask, Request, Response},
    storage::{Area, StorageService},
};
use yew::{
    format::{Json, Nothing, Text},
    Callback,
};

use crate::error::ApiError;

#[derive(Default)]
pub struct Api {
    pub base_url: String,
}

// refs
// https://doc.rust-lang.org/beta/nomicon/hrtb.html
// https://serde.rs/lifetimes.html

impl Api {
    pub fn new() -> Self {
        let storage = StorageService::new(Area::Local).expect("storage was disabled by the user");

        let registry_url =
            if let Json(Ok(registry_url_value)) = storage.restore(crate::REGISTRY_KEY) {
                registry_url_value
            } else {
                String::from(crate::DEFAULT_REGISTRY_URL)
            };

        Self {
            base_url: registry_url,
        }
    }

    pub fn builder<B, T>(
        &mut self,
        method: &str,
        url: String,
        body: B,
        auth: Option<String>,
        callback: Callback<Result<T, ApiError>>,
    ) -> FetchTask
    where
        for<'de> T: Deserialize<'de> + 'static + std::fmt::Debug,
        B: Into<Text> + std::fmt::Debug,
    {
        let session_storage =
            StorageService::new(Area::Session).expect("storage was disabled by the user");

        let mut req_builder = Request::builder()
            .method(method)
            .uri(format!("{}{}", self.base_url, url))
            .header("Content-Type", "application/json");

        if let Some(b64_auth_string) = auth {
            req_builder = req_builder.header("Authorization", format!("Basic {}", b64_auth_string));
        } else {
            let auth_token = session_storage.restore(crate::AUTH_TOKEN_KEY);

            let token_auth_value = if let Json(Ok(auth_token_value)) = auth_token {
                auth_token_value
            };
            req_builder =
                req_builder.header("Authorization", format!("Bearer {:?}", token_auth_value))
        }

        let handler = move |response: Response<Text>| {
            if let (meta, Ok(data)) = response.into_parts() {
                log::debug!("Meta: {:?}", meta);
                log::debug!("Response: {:?}", data);
                let data: Result<T, _> = serde_json::from_str(&data);
                if let Ok(data) = data {
                    callback.emit(Ok(data))
                } else {
                    callback.emit(Err(ApiError::DeserializationFailed))
                }
            } else {
                callback.emit(Err(ApiError::ResponseParsingFailed))
            }
        };

        let request = req_builder.body(body).unwrap();
        log::debug!("Request: {:?}", request);

        FetchService::fetch(request, handler.into()).unwrap()
    }

    pub fn get<T>(
        &mut self,
        url: String,
        auth: Option<String>,
        callback: Callback<Result<T, ApiError>>,
    ) -> FetchTask
    where
        for<'de> T: Deserialize<'de> + 'static + std::fmt::Debug,
    {
        self.builder("GET", url, Nothing, auth, callback)
    }
}
