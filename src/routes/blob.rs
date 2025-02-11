use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::get;
use axum::Router;

use super::macros::endpoint_fn_7_levels;
use crate::registry::manifest::ManifestReference;
use crate::registry::server::PROXY_DIR;
use crate::registry::{BlobReader, Digest};
use crate::routes::macros::route_7_levels;
use crate::routes::response::errors::Error;
use crate::routes::response::trow_token::TrowToken;
use crate::TrowServerState;

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
                )))
            }
        };
        repo = format!("f/{}/{}", proxy_cfg.alias, image.get_repo())
    }
    sqlx::query_scalar!(
        r#"
        SELECT digest FROM blob
        JOIN repo_blob_association ON blob.digest = repo_blob_association.blob_digest
        WHERE digest = $1 AND repo_name = $2
        "#,
        digest_str,
        repo
    )
    .fetch_one(&state.db_ro)
    .await?;

    let stream = match state.registry.storage.get_blob_stream(&repo, &digest).await {
        Ok(stream) => stream,
        Err(_) => return Err(Error::InternalError),
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
