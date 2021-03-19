use yew::services::fetch::Request;

use serde::Deserialize;
use yew::services::{storage::{Area, StorageService}, fetch::{FetchService, FetchTask, Response}};
use yew::{
    format::{Nothing, Text, Json},
    Callback,
};

use crate::error::ApiError;

const REGISTRY_KEY: &str = "trow.gui.registry_url";
const DEFAULT_REGISTRY_URL: &str = "https://0.0.0.0:8443";

#[derive(Default)]
pub struct Api {
    pub base_url: String
}

// refs
// https://doc.rust-lang.org/beta/nomicon/hrtb.html
// https://serde.rs/lifetimes.html

impl Api {
    pub fn new() -> Self {
        let storage = StorageService::new(Area::Local).expect("storage was disabled by the user"); 
        let registry_url = if let Json(Ok(registry_url_value)) = storage.restore(REGISTRY_KEY)  {
            registry_url_value
        } else {
            String::from(DEFAULT_REGISTRY_URL)
        };
        
        Self { base_url: registry_url}
    }

    pub fn builder<B, T>(
        &mut self,
        method: &str,
        url: String,
        body: B,
        callback: Callback<Result<T, ApiError>>,
    ) -> FetchTask
    where
        for<'de> T: Deserialize<'de> + 'static + std::fmt::Debug,
        B: Into<Text> + std::fmt::Debug,
    {
        let mut req_builder = Request::builder()
            .method(method)
            .uri(url.as_str())
            .header("Content-Type", "application/json");

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

    pub fn get<T>(&mut self, url: String, callback: Callback<Result<T, ApiError>>) -> FetchTask
    where
        for<'de> T: Deserialize<'de> + 'static + std::fmt::Debug,
    {
        self.builder("GET", url, Nothing, callback)
    }
}
