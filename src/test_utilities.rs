#![cfg(test)]

use std::sync::Arc;

use axum::body::Body;
use axum::Router;
use http_body_util::BodyExt;
use hyper::body::Buf;
use hyper::Response;
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use serde::de::DeserializeOwned;
use test_temp_dir::TestTempDir;

use crate::migrations::Migrator;
use crate::{routes, TrowConfig, TrowServerState};

pub const DIST_API_HEADER: &str = "Docker-Distribution-API-Version";
pub const UPLOAD_HEADER: &str = "Docker-Upload-Uuid";
pub const LOCATION_HEADER: &str = "Location";
pub const RANGE_HEADER: &str = "Range";

pub async fn trow_router<F: FnOnce(&mut TrowConfig)>(
    custom_cfg: F,
    temp_dir: Option<&TestTempDir>,
) -> (DatabaseConnection, Arc<TrowServerState>, Router) {
    let mut trow_builder = crate::TrowConfig::new();
    let db = Database::connect("sqlite::memory:").await.unwrap();
    Migrator::refresh(&db).await.unwrap();
    trow_builder.db_connection = Some(db.clone());
    if let Some(dir) = temp_dir {
        trow_builder.data_dir = dir.as_path_untracked().to_owned();
    }

    custom_cfg(&mut trow_builder);
    let state = trow_builder.build_server_state().await.unwrap();
    let router = routes::create_app(state.clone());
    (db, state, router)
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
