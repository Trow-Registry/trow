use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::Router;
use digest::Digest;

use super::extracts::AlwaysHost;
use super::macros::endpoint_fn_7_levels;
use crate::registry::{digest, ManifestReader, StorageDriverError};
use crate::routes::macros::route_7_levels;
use crate::routes::response::errors::Error;
use crate::routes::response::trow_token::TrowToken;
use crate::types::{ManifestDeleted, VerifiedManifest};
use crate::TrowServerState;

/*
---
Pulling an image
GET /v2/<name>/manifests/<reference>

# Parameters
name - The name of the image
reference - either a tag or a digest

# Client Headers
Accept: manifest-version

# Headers
Accept: manifest-version
?Docker-Content-Digest: digest of manifest file

# Returns
200 - return the manifest
404 - manifest not known to the registry
 */
async fn get_manifest(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((name, reference)): Path<(String, String)>,
) -> Result<ManifestReader, Error> {
    state
        .registry
        .get_manifest(&name, &reference)
        .await
        .map_err(|_| Error::ManifestUnknown(reference))
}

endpoint_fn_7_levels!(
    get_manifest(
        auth_user: TrowToken,
        state: State<Arc<TrowServerState>>;
        path: [image_name, reference]
    ) -> Result<ManifestReader, Error>
);

/*

---
Pushing an image manifest
PUT /v2/<name>/manifests/<reference>
Content-Type: <manifest media type>

 */
async fn put_image_manifest(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    AlwaysHost(host): AlwaysHost,
    Path((repo_name, reference)): Path<(String, String)>,
    chunk: Body,
) -> Result<VerifiedManifest, Error> {
    match state
        .registry
        .store_manifest(&repo_name, &reference, chunk)
        .await
    {
        Ok(digest) => Ok(VerifiedManifest::new(
            Some(host),
            repo_name,
            digest,
            reference,
        )),
        Err(StorageDriverError::InvalidName(name)) => Err(Error::NameInvalid(name)),
        Err(StorageDriverError::InvalidManifest) => Err(Error::ManifestInvalid("".to_string())),
        Err(_) => Err(Error::InternalError),
    }
}
endpoint_fn_7_levels!(
    put_image_manifest(
        auth_user: TrowToken,
        state: State<Arc<TrowServerState>>,
        host: AlwaysHost;
        path: [image_name, reference],
        chunk: Body
    ) -> Result<VerifiedManifest, Error>
);

/*
---
Deleting an Image
DELETE /v2/<name>/manifests/<reference>
*/
async fn delete_image_manifest(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((repo, digest)): Path<(String, String)>,
) -> Result<ManifestDeleted, Error> {
    let digest = Digest::try_from_raw(&digest).map_err(|_| Error::Unsupported)?;
    match state.registry.delete_manifest(&repo, &digest).await {
        Ok(_) => Ok(ManifestDeleted {}),
        Err(StorageDriverError::Unsupported) => Err(Error::Unsupported),
        Err(StorageDriverError::InvalidManifest) => Err(Error::ManifestUnknown(repo)),
        Err(_) => Err(Error::InternalError),
    }
}
endpoint_fn_7_levels!(
    delete_image_manifest(
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>;
    path: [image_name, digest]
    ) -> Result<ManifestDeleted, Error>
);

pub fn route(mut app: Router<Arc<TrowServerState>>) -> Router<Arc<TrowServerState>> {
    #[rustfmt::skip]
    route_7_levels!(
        app,
        "/v2" "/manifests/:reference",
        get(get_manifest, get_manifest_2level, get_manifest_3level, get_manifest_4level, get_manifest_5level, get_manifest_6level, get_manifest_7level),
        put(put_image_manifest, put_image_manifest_2level, put_image_manifest_3level, put_image_manifest_4level, put_image_manifest_5level, put_image_manifest_6level, put_image_manifest_7level),
        delete(delete_image_manifest, delete_image_manifest_2level, delete_image_manifest_3level, delete_image_manifest_4level, delete_image_manifest_5level, delete_image_manifest_6level, delete_image_manifest_7level)
    );
    app
}
