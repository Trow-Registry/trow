use std::sync::Arc;

use axum::extract::{BodyStream, Path, State};
use axum::headers::HeaderMap;

use super::macros::endpoint_fn_7_levels;
use crate::registry_interface::{digest, ManifestReader, ManifestStorage, StorageDriverError};
use crate::response::errors::Error;
use crate::response::get_base_url;
use crate::response::trow_token::TrowToken;
use crate::types::{ManifestDeleted, RepoName, VerifiedManifest};
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
    headers: HeaderMap,
    _auth_user: TrowToken,
    State(state): State<Arc<TrowServerState>>,
    Path((repo_name, reference)): Path<(String, String)>,
    chunk: BodyStream,
) -> Result<VerifiedManifest, Error> {
    let base_url = get_base_url(&headers, &state.config);

    match state
        .client
        .store_manifest(&repo_name, &reference, chunk)
        .await
    {
        Ok(digest) => Ok(VerifiedManifest::new(
            Some(base_url),
            RepoName(repo_name),
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
        headers: HeaderMap,
        auth_user: TrowToken,
        state: State<Arc<TrowServerState>>;
        path: [image_name, reference],
        chunk: BodyStream
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
    let digest = digest::parse(&digest).map_err(|_| Error::Unsupported)?;
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
