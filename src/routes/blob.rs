use std::pin::Pin;
use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, Query, State};
use axum::routing::get;
use tokio::io::AsyncRead;

use super::macros::endpoint_fn_7_levels;
use crate::TrowServerState;
use crate::routes::extracts::ImageNamespace;
use crate::routes::macros::route_7_levels;
use crate::routes::response::errors::Error;
use crate::routes::response::trow_token::TrowToken;
use crate::services::blob_service::BlobReader;
use crate::utils::digest::Digest;

/*
---
Pulling a Layer
GET /v2/<name>/blobs/<digest>
name - name of the repository
digest - unique identifier for the blob to be downloaded
# Responses
200 - blob is downloaded
307 - redirect to another service for downloading (docker API, not OCI)
 */
async fn get_blob(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((repo, digest)): Path<(String, Digest)>,
    Query(query): Query<ImageNamespace>,
) -> Result<BlobReader<Pin<Box<dyn AsyncRead + Send>>>, Error> {
    Ok(state
        .services
        .blob
        .get_blob(repo, digest, query.ns.as_deref())
        .await?)
}

endpoint_fn_7_levels!(
    get_blob(
        auth_user: TrowToken,
        state: State<Arc<TrowServerState>>;
        path: [image_name, digest: Digest],
        query: Query<ImageNamespace>
    ) -> Result<BlobReader<Pin<Box<dyn AsyncRead + Send>>>, Error>
);

pub fn route(mut app: Router<Arc<TrowServerState>>) -> Router<Arc<TrowServerState>> {
    #[rustfmt::skip]
    route_7_levels!(
        app,
        "/v2" "/blobs/{digest}",
        get(get_blob, get_blob_2level, get_blob_3level, get_blob_4level, get_blob_5level, get_blob_6level, get_blob_7level)
    );
    app
}
