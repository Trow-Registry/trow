use std::sync::Arc;

use axum::extract::{BodyStream, Path, State};
use axum::headers::HeaderMap;

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
pub async fn get_manifest_2level(
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    Path((one, two, reference)): Path<(String, String, String)>,
) -> Result<ManifestReader, Error> {
    get_manifest(auth_user, state, Path((format!("{one}/{two}"), reference))).await
}
pub async fn get_manifest_3level(
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    Path((one, two, three, reference)): Path<(String, String, String, String)>,
) -> Result<ManifestReader, Error> {
    get_manifest(
        auth_user,
        state,
        Path((format!("{one}/{two}/{three}"), reference)),
    )
    .await
}
pub async fn get_manifest_4level(
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    Path((one, two, three, four, reference)): Path<(String, String, String, String, String)>,
) -> Result<ManifestReader, Error> {
    get_manifest(
        auth_user,
        state,
        Path((format!("{one}/{two}/{three}/{four}"), reference)),
    )
    .await
}
pub async fn get_manifest_5level(
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    Path((one, two, three, four, five, reference)): Path<(
        String,
        String,
        String,
        String,
        String,
        String,
    )>,
) -> Result<ManifestReader, Error> {
    get_manifest(
        auth_user,
        state,
        Path((format!("{one}/{two}/{three}/{four}/{five}"), reference)),
    )
    .await
}

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
pub async fn put_image_manifest_2level(
    headers: HeaderMap,
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    Path((one, two, reference)): Path<(String, String, String)>,
    chunk: BodyStream,
) -> Result<VerifiedManifest, Error> {
    put_image_manifest(
        headers,
        auth_user,
        state,
        Path((format!("{one}/{two}"), reference)),
        chunk,
    )
    .await
}
pub async fn put_image_manifest_3level(
    headers: HeaderMap,
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    Path((one, two, three, reference)): Path<(String, String, String, String)>,
    chunk: BodyStream,
) -> Result<VerifiedManifest, Error> {
    put_image_manifest(
        headers,
        auth_user,
        state,
        Path((format!("{one}/{two}/{three}"), reference)),
        chunk,
    )
    .await
}
pub async fn put_image_manifest_4level(
    headers: HeaderMap,
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    Path((one, two, three, four, reference)): Path<(String, String, String, String, String)>,
    chunk: BodyStream,
) -> Result<VerifiedManifest, Error> {
    put_image_manifest(
        headers,
        auth_user,
        state,
        Path((format!("{one}/{two}/{three}/{four}"), reference)),
        chunk,
    )
    .await
}
pub async fn put_image_manifest_5level(
    headers: HeaderMap,
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    Path((one, two, three, four, five, reference)): Path<(
        String,
        String,
        String,
        String,
        String,
        String,
    )>,
    chunk: BodyStream,
) -> Result<VerifiedManifest, Error> {
    put_image_manifest(
        headers,
        auth_user,
        state,
        Path((format!("{one}/{two}/{three}/{four}/{five}"), reference)),
        chunk,
    )
    .await
}

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
pub async fn delete_image_manifest_2level(
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    Path((one, two, digest)): Path<(String, String, String)>,
) -> Result<ManifestDeleted, Error> {
    delete_image_manifest(auth_user, state, Path((format!("{one}/{two}"), digest))).await
}
pub async fn delete_image_manifest_3level(
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    Path((one, two, three, digest)): Path<(String, String, String, String)>,
) -> Result<ManifestDeleted, Error> {
    delete_image_manifest(
        auth_user,
        state,
        Path((format!("{one}/{two}/{three}"), digest)),
    )
    .await
}
pub async fn delete_image_manifest_4level(
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    Path((one, two, three, four, digest)): Path<(String, String, String, String, String)>,
) -> Result<ManifestDeleted, Error> {
    delete_image_manifest(
        auth_user,
        state,
        Path((format!("{one}/{two}/{three}/{four}"), digest)),
    )
    .await
}
pub async fn delete_image_manifest_5level(
    auth_user: TrowToken,
    state: State<Arc<TrowServerState>>,
    Path((one, two, three, four, five, digest)): Path<(
        String,
        String,
        String,
        String,
        String,
        String,
    )>,
) -> Result<ManifestDeleted, Error> {
    delete_image_manifest(
        auth_user,
        state,
        Path((format!("{one}/{two}/{three}/{four}/{five}"), digest)),
    )
    .await
}
