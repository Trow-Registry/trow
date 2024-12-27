#![cfg(test)]

use std::sync::Arc;

use axum::body::Body;
use axum::Router;
use http_body_util::BodyExt;
use hyper::body::Buf;
use hyper::Response;
use serde::de::DeserializeOwned;
use test_temp_dir::TestTempDir;

use crate::{routes, TrowConfig, TrowServerState};

pub const DIST_API_HEADER: &str = "Docker-Distribution-API-Version";
pub const UPLOAD_HEADER: &str = "Docker-Upload-Uuid";
pub const LOCATION_HEADER: &str = "Location";
pub const RANGE_HEADER: &str = "Range";

pub async fn trow_router<F: FnOnce(&mut TrowConfig)>(
    custom_cfg: F,
    temp_dir: Option<&TestTempDir>,
) -> (Arc<TrowServerState>, Router) {
    let mut trow_builder = crate::TrowConfig::new();
    trow_builder.db_connection = Some("sqlite::memory:".to_string());
    if let Some(dir) = temp_dir {
        trow_builder.data_dir = dir.as_path_untracked().to_owned();
    }

    custom_cfg(&mut trow_builder);
    let state = trow_builder.build_server_state().await.unwrap();
    let router = routes::create_app(state.clone());
    (state, router)
}

pub async fn response_body_json<T: DeserializeOwned>(resp: Response<Body>) -> T {
    let reader = resp
        .into_body()
        .collect()
        .await
        .unwrap()
        .aggregate()
        .reader();
    serde_json::from_reader(reader).unwrap()
}
