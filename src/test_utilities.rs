use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use http_body_util::BodyExt;
use hyper::Response;
use serde::de::DeserializeOwned;
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions, SqliteSynchronous,
};
use test_temp_dir::TestTempDir;

use crate::repositories::Repositories;
use crate::{TrowConfig, TrowServerState, routes};

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

/// Create an `Arc<Repositories>` backed by an in-memory SQLite database.
/// Runs all migrations. Use for unit tests that need database access without
/// spinning up a full Trow instance.
pub async fn repos_in_memory() -> Arc<Repositories> {
    let options = SqliteConnectOptions::new()
        .filename(":memory:")
        .synchronous(SqliteSynchronous::Normal)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .unwrap();

    sqlx::migrate!().run(&pool).await.unwrap();
    Arc::new(Repositories::from_pools(pool.clone(), pool))
}

/// Create an `Arc<SqlitePool>` backed by an in-memory SQLite database
/// with migrations applied. Useful when tests need direct pool access.
pub async fn pool_in_memory() -> SqlitePool {
    let options = SqliteConnectOptions::new()
        .filename(":memory:")
        .synchronous(SqliteSynchronous::Normal)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .unwrap();

    sqlx::migrate!().run(&pool).await.unwrap();
    pool
}

// Persistence fixtures.
//
// Route and integration tests must not reach for repositories directly; these
// helpers are the single place that seeds the database and the storage backend
// on their behalf.

/// Insert a manifest row plus its repository association, as the registry
/// would after a successful push.
pub async fn seed_manifest(state: &TrowServerState, repo_name: &str, digest: &str, bytes: &[u8]) {
    seed_manifest_blob(state, digest, bytes).await;
    seed_manifest_assoc(state, repo_name, digest).await;
}

/// Insert only the manifest row, for manifests shared by several repositories.
pub async fn seed_manifest_blob(state: &TrowServerState, digest: &str, bytes: &[u8]) {
    sqlx::query!(
        "INSERT INTO manifest (digest, json, blob) VALUES ($1, jsonb($2), $2)",
        digest,
        bytes,
    )
    .execute(state.services.repos().db_rw())
    .await
    .unwrap();
}

/// Associate an existing manifest with a repository.
pub async fn seed_manifest_assoc(state: &TrowServerState, repo_name: &str, digest: &str) {
    sqlx::query!(
        "INSERT INTO repo_blob_assoc (repo_name, manifest_digest) VALUES ($1, $2)",
        repo_name,
        digest
    )
    .execute(state.services.repos().db_rw())
    .await
    .unwrap();
}

/// Number of repositories a manifest is still referenced by.
pub async fn manifest_repo_count(state: &TrowServerState, digest: &str) -> i64 {
    sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM manifest WHERE digest = $1"#,
        digest
    )
    .fetch_one(state.services.repos().db_ro())
    .await
    .unwrap()
}

/// Insert an in-progress blob upload row with a pre-filled offset.
pub async fn seed_blob_upload(state: &TrowServerState, uuid: &str, repo: &str, offset: i64) {
    sqlx::query!(
        "INSERT INTO blob_upload (uuid, offset, repo) VALUES ($1, $2, $3)",
        uuid,
        offset,
        repo
    )
    .execute(state.services.repos().db_rw())
    .await
    .unwrap();
}

/// Whether a blob upload row with this UUID exists.
pub async fn blob_upload_exists(state: &TrowServerState, uuid: &str) -> bool {
    let count: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) as "count!" FROM blob_upload WHERE uuid = $1"#,
        uuid
    )
    .fetch_one(state.services.repos().db_ro())
    .await
    .unwrap();
    count > 0
}

/// Insert a tag pointing at an existing manifest.
pub async fn seed_tag(state: &TrowServerState, repo: &str, tag: &str, manifest_digest: &str) {
    sqlx::query!(
        "INSERT INTO tag (tag, repo, manifest_digest) VALUES ($1, $2, $3)",
        tag,
        repo,
        manifest_digest
    )
    .execute(state.services.repos().db_rw())
    .await
    .unwrap();
}

/// Every tag in the database, as `(repo, tag, manifest digest)`.
pub async fn all_tags(state: &TrowServerState) -> Vec<(String, String, String)> {
    sqlx::query!("SELECT repo, tag, manifest_digest FROM tag")
        .fetch_all(state.services.repos().db_ro())
        .await
        .unwrap()
        .into_iter()
        .map(|t| (t.repo, t.tag, t.manifest_digest))
        .collect()
}

/// Write bytes into an in-progress upload, as a previous PATCH would.
pub async fn seed_upload_content(
    state: &TrowServerState,
    uuid: &uuid::Uuid,
    content: &'static [u8],
) -> u64 {
    let stream = futures::stream::iter(vec![Ok::<_, std::io::Error>(bytes::Bytes::from_static(
        content,
    ))]);
    state
        .services
        .storage()
        .write_blob_part_stream(uuid, stream, None)
        .await
        .unwrap()
        .total_stored as u64
}

pub async fn response_body_json<T: DeserializeOwned>(resp: Response<Body>) -> T {
    let buf = resp
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    match serde_json::from_slice(&buf) {
        Ok(val) => val,
        Err(err) => {
            let text = String::from_utf8_lossy(&buf);
            panic!("unable to deserialize response body: {err:?} {text}");
        }
    }
}

/// test_temp_dir if thread name != module path, which is the case in parametrized tests
pub fn test_temp_dir_from_thread_name(mod_path: &str) -> TestTempDir {
    let path = {
        let (crate_, _) = mod_path.split_once("::").unwrap();
        let thread = std::thread::current();
        let thread = thread.name().unwrap();
        let (t_mod, fn_) = thread.rsplit_once("::").unwrap();
        format!("{crate_}::{t_mod}::{fn_}")
    };

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

pub(crate) use resp_header;
pub(crate) use test_temp_dir;
