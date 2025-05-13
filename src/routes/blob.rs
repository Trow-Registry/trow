use std::sync::Arc;

use axum::Router;
use axum::extract::{Path, State};
use axum::routing::get;

use super::macros::endpoint_fn_7_levels;
use crate::TrowServerState;
use crate::registry::manifest::ManifestReference;
use crate::registry::server::PROXY_DIR;
use crate::registry::{BlobReader, Digest};
use crate::routes::macros::route_7_levels;
use crate::routes::response::errors::Error;
use crate::routes::response::trow_token::TrowToken;

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
    Path((mut repo, digest)): Path<(String, Digest)>,
) -> Result<BlobReader<impl tokio::io::AsyncRead>, Error> {
    let digest_str = digest.as_str();
    if repo.starts_with(PROXY_DIR) {
        let (proxy_cfg, image) = match state
            .registry
            .config
            .registry_proxies
            .get_proxy_config(&repo, &ManifestReference::Digest(digest.clone()))
            .await
        {
            Some(cfg) => cfg,
            None => {
                return Err(Error::NameInvalid(format!(
                    "No registered proxy matches {repo}"
                )));
            }
        };
        // This is important to transform f/docker/ubuntu into f/docker/_library/ubuntu
        repo = format!("f/{}/{}", proxy_cfg.alias, image.get_repo())
    }
    let rowid = sqlx::query_scalar!(
        r#"
        SELECT b.rowid as "rowid!" FROM blob b
        JOIN repo_blob_association rba ON b.digest = rba.blob_digest
        WHERE b.digest = $1 AND rba.repo_name = $2
        "#,
        digest_str,
        repo
    )
    .fetch_one(&state.db_ro)
    .await?;
    sqlx::query!(
        "UPDATE blob SET last_accessed=unixepoch() WHERE rowid=$1",
        rowid
    )
    .execute(&state.db_rw)
    .await?;

    let stream = match state
        .registry
        .storage
        .get_blob_stream(&repo, digest.as_str())
        .await
    {
        Ok(stream) => stream,
        Err(_) => return Err(Error::Internal),
    };
    Ok(BlobReader::new(digest, stream).await)
}

endpoint_fn_7_levels!(
    get_blob(
        auth_user: TrowToken,
        state: State<Arc<TrowServerState>>;
        path: [image_name, digest: Digest]
    ) -> Result<BlobReader<impl tokio::io::AsyncRead>, Error>
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
