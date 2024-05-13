use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path, State};
use digest::Digest;

use super::extracts::AlwaysHost;
use super::macros::endpoint_fn_7_levels;
use crate::registry::{digest, ManifestReader, StorageDriverError};
use crate::response::errors::Error;
use crate::response::trow_token::TrowToken;
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
pub async fn get_manifest(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((name, reference)): Path<(String, String)>,
) -> Result<ManifestReader, Error> {
    state
        .client
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
pub async fn put_image_manifest(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    AlwaysHost(host): AlwaysHost,
    Path((repo_name, reference)): Path<(String, String)>,
    chunk: Body,
) -> Result<VerifiedManifest, Error> {
    match state
        .client
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
pub async fn delete_image_manifest(
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((repo, digest)): Path<(String, String)>,
) -> Result<ManifestDeleted, Error> {
    let digest = Digest::try_from_raw(&digest).map_err(|_| Error::Unsupported)?;
    match state.client.delete_manifest(&repo, &digest).await {
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
