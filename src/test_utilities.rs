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
    temp_dir: &TestTempDir,
) -> (Arc<TrowServerState>, Router) {
    let mut trow_builder = crate::TrowConfig::new();
    trow_builder.data_dir = temp_dir.as_path_untracked().to_owned();
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

/// test_temp_dir if thread name != module path, which is the case in parametrized tests
pub fn test_temp_dir_from_thread_name(mod_path: &str) -> TestTempDir {
    let path = {
        let (crate_, _) = mod_path.split_once("::").unwrap();
        let thread = std::thread::current();
        let thread = thread.name().unwrap();
        let (t_mod, fn_) = thread.rsplit_once("::").unwrap();
        Ok::<_, anyhow::Error>(format!("{crate_}::{t_mod}::{fn_}"))
    }
    .expect("unable to calculate complete test function path");

    test_temp_dir::TestTempDir::from_complete_item_path(&path)
}

macro_rules! resp_header {
    ($name:expr, $value:expr) => {
        $name.headers().get($value).unwrap().to_str().unwrap()
    };
}

macro_rules! test_temp_dir { {} => {
    $crate::test_utilities::test_temp_dir_from_thread_name(module_path!())
} }

pub(crate) use {resp_header, test_temp_dir};
