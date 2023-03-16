use rocket::data::ToByteUnit;
use rocket::{delete, get, put};

use crate::client_interface::ClientInterface;
use crate::registry_interface::{digest, ManifestReader, ManifestStorage, StorageDriverError};
use crate::response::errors::Error;
use crate::response::trow_token::TrowToken;
use crate::types::{create_verified_manifest, ManifestDeleted, RepoName, VerifiedManifest};
use crate::TrowConfig;

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
#[get("/v2/<onename>/manifests/<reference>")]
pub async fn get_manifest(
    _auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    onename: String,
    reference: String,
) -> Result<ManifestReader, Error> {
    ci.get_manifest(&onename, &reference)
        .await
        .map_err(|_| Error::ManifestUnknown(reference))
}

#[get("/v2/<user>/<repo>/manifests/<reference>")]
pub async fn get_manifest_2level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    user: String,
    repo: String,
    reference: String,
) -> Result<ManifestReader, Error> {
    get_manifest(auth_user, ci, format!("{}/{}", user, repo), reference).await
}

/*
 * Process 3 level manifest path
 */
#[get("/v2/<org>/<user>/<repo>/manifests/<reference>")]
pub async fn get_manifest_3level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    org: String,
    user: String,
    repo: String,
    reference: String,
) -> Result<ManifestReader, Error> {
    get_manifest(
        auth_user,
        ci,
        format!("{}/{}/{}", org, user, repo),
        reference,
    )
    .await
}

/*
 * Process 4 level manifest path
 */
#[get("/v2/<fourth>/<org>/<user>/<repo>/manifests/<reference>")]
pub async fn get_manifest_4level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    fourth: String,
    org: String,
    user: String,
    repo: String,
    reference: String,
) -> Result<ManifestReader, Error> {
    get_manifest(
        auth_user,
        ci,
        format!("{}/{}/{}/{}", fourth, org, user, repo),
        reference,
    )
    .await
}

/*
 * Process 5 level manifest path
 */
#[get("/v2/<fifth>/<fourth>/<org>/<user>/<repo>/manifests/<reference>")]
pub async fn get_manifest_5level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    fifth: String,
    fourth: String,
    org: String,
    user: String,
    repo: String,
    reference: String,
) -> Result<ManifestReader, Error> {
    get_manifest(
        auth_user,
        ci,
        format!("{}/{}/{}/{}/{}", fifth, fourth, org, user, repo),
        reference,
    )
    .await
}

/*

---
Pushing an image manifest
PUT /v2/<name>/manifests/<reference>
Content-Type: <manifest media type>

 */
#[put("/v2/<repo_name>/manifests/<reference>", data = "<chunk>")]
pub async fn put_image_manifest(
    _auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    tc: &rocket::State<TrowConfig>,
    repo_name: String,
    reference: String,
    chunk: rocket::data::Data<'_>,
) -> Result<VerifiedManifest, Error> {
    let data = chunk.open(tc.max_manifest_size.mebibytes());

    match ci.store_manifest(&repo_name, &reference, data).await {
        Ok(digest) => Ok(create_verified_manifest(
            RepoName(repo_name),
            digest,
            reference,
        )),
        Err(StorageDriverError::InvalidName(name)) => Err(Error::NameInvalid(name)),
        Err(StorageDriverError::InvalidManifest) => Err(Error::ManifestInvalid("".to_string())),
        Err(StorageDriverError::InvalidContentRange) => Err(Error::ManifestInvalid(format!(
            "Content over data limit {} mebibytes",
            tc.max_blob_size
        ))),
        Err(_) => Err(Error::InternalError),
    }
}

/*
 * Parse 2 level <user>/<repo> style path and pass it to put_image_manifest
 */
#[put("/v2/<user>/<repo>/manifests/<reference>", data = "<chunk>")]
pub async fn put_image_manifest_2level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    tc: &rocket::State<TrowConfig>,
    user: String,
    repo: String,
    reference: String,
    chunk: rocket::data::Data<'_>,
) -> Result<VerifiedManifest, Error> {
    put_image_manifest(
        auth_user,
        ci,
        tc,
        format!("{}/{}", user, repo),
        reference,
        chunk,
    )
    .await
}

/*
 * Parse 3 level <org>/<user>/<repo> style path and pass it to put_image_manifest
 */
#[put("/v2/<org>/<user>/<repo>/manifests/<reference>", data = "<chunk>")]
pub async fn put_image_manifest_3level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    tc: &rocket::State<TrowConfig>,
    org: String,
    user: String,
    repo: String,
    reference: String,
    chunk: rocket::data::Data<'_>,
) -> Result<VerifiedManifest, Error> {
    put_image_manifest(
        auth_user,
        ci,
        tc,
        format!("{}/{}/{}", org, user, repo),
        reference,
        chunk,
    )
    .await
}

/*
 * Parse 4 level <fourth>/<org>/<user>/<repo> style path and pass it to put_image_manifest
 */
#[put(
    "/v2/<fourth>/<org>/<user>/<repo>/manifests/<reference>",
    data = "<chunk>"
)]
pub async fn put_image_manifest_4level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    tc: &rocket::State<TrowConfig>,
    fourth: String,
    org: String,
    user: String,
    repo: String,
    reference: String,
    chunk: rocket::data::Data<'_>,
) -> Result<VerifiedManifest, Error> {
    put_image_manifest(
        auth_user,
        ci,
        tc,
        format!("{}/{}/{}/{}", fourth, org, user, repo),
        reference,
        chunk,
    )
    .await
}

/*
 * Parse 5 level <fifth>/<fourth>/<org>/<user>/<repo> style path and pass it to put_image_manifest
 */
#[put(
    "/v2/<fifth>/<fourth>/<org>/<user>/<repo>/manifests/<reference>",
    data = "<chunk>"
)]
pub async fn put_image_manifest_5level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    tc: &rocket::State<TrowConfig>,
    fifth: String,
    fourth: String,
    org: String,
    user: String,
    repo: String,
    reference: String,
    chunk: rocket::data::Data<'_>,
) -> Result<VerifiedManifest, Error> {
    put_image_manifest(
        auth_user,
        ci,
        tc,
        format!("{}/{}/{}/{}/{}", fifth, fourth, org, user, repo),
        reference,
        chunk,
    )
    .await
}

/*
---
Deleting an Image
DELETE /v2/<name>/manifests/<reference>
*/

#[delete("/v2/<repo>/manifests/<digest>")]
pub async fn delete_image_manifest(
    _auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    repo: String,
    digest: String,
) -> Result<ManifestDeleted, Error> {
    let digest = digest::parse(&digest).map_err(|_| Error::Unsupported)?;
    match ci.delete_manifest(&repo, &digest).await {
        Ok(_) => Ok(ManifestDeleted {}),
        Err(StorageDriverError::Unsupported) => Err(Error::Unsupported),
        Err(StorageDriverError::InvalidManifest) => Err(Error::ManifestUnknown(repo)),
        Err(_) => Err(Error::InternalError),
    }
}

#[delete("/v2/<user>/<repo>/manifests/<digest>")]
pub async fn delete_image_manifest_2level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    user: String,
    repo: String,
    digest: String,
) -> Result<ManifestDeleted, Error> {
    delete_image_manifest(auth_user, ci, format!("{}/{}", user, repo), digest).await
}

#[delete("/v2/<org>/<user>/<repo>/manifests/<digest>")]
pub async fn delete_image_manifest_3level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    org: String,
    user: String,
    repo: String,
    digest: String,
) -> Result<ManifestDeleted, Error> {
    delete_image_manifest(auth_user, ci, format!("{}/{}/{}", org, user, repo), digest).await
}

#[delete("/v2/<fourth>/<org>/<user>/<repo>/manifests/<digest>")]
pub async fn delete_image_manifest_4level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    fourth: String,
    org: String,
    user: String,
    repo: String,
    digest: String,
) -> Result<ManifestDeleted, Error> {
    delete_image_manifest(
        auth_user,
        ci,
        format!("{}/{}/{}/{}", fourth, org, user, repo),
        digest,
    )
    .await
}

#[delete("/v2/<fifth>/<fourth>/<org>/<user>/<repo>/manifests/<digest>")]
pub async fn delete_image_manifest_5level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    fifth: String,
    fourth: String,
    org: String,
    user: String,
    repo: String,
    digest: String,
) -> Result<ManifestDeleted, Error> {
    delete_image_manifest(
        auth_user,
        ci,
        format!("{}/{}/{}/{}/{}", fifth, fourth, org, user, repo),
        digest,
    )
    .await
}
