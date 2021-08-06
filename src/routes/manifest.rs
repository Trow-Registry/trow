use crate::client_interface::ClientInterface;
use crate::registry_interface::{digest, ManifestReader, ManifestStorage, StorageDriverError};
use crate::response::errors::Error;
use crate::response::trow_token::TrowToken;
use crate::types::{create_verified_manifest, ManifestDeleted, RepoName, VerifiedManifest};

use std::io::Read;

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
pub fn get_manifest(
    _auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    onename: String,
    reference: String,
) -> Result<ManifestReader, Error> {
    ci.get_manifest(&onename, &reference)
        .map_err(|_| Error::ManifestUnknown(reference))
}

#[get("/v2/<user>/<repo>/manifests/<reference>")]
pub fn get_manifest_2level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    user: String,
    repo: String,
    reference: String,
) -> Result<ManifestReader, Error> {
    get_manifest(auth_user, ci, format!("{}/{}", user, repo), reference)
}

/*
 * Process 3 level manifest path
 */
#[get("/v2/<org>/<user>/<repo>/manifests/<reference>")]
pub fn get_manifest_3level(
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
}

/*
 * Process 4 level manifest path
 */
#[get("/v2/<fourth>/<org>/<user>/<repo>/manifests/<reference>")]
pub fn get_manifest_4level(
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
}

/*

---
Pushing an image manifest
PUT /v2/<name>/manifests/<reference>
Content-Type: <manifest media type>

 */
#[put("/v2/<repo_name>/manifests/<reference>", data = "<chunk>")]
pub fn put_image_manifest(
    _auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    repo_name: String,
    reference: String,
    chunk: rocket::data::Data,
) -> Result<VerifiedManifest, Error> {
    let mut data: Box<dyn Read> = Box::new(chunk.open());

    match ci.store_manifest(&repo_name, &reference, &mut data) {
        Ok(digest) => Ok(create_verified_manifest(
            RepoName(repo_name),
            digest,
            reference,
        )),
        Err(StorageDriverError::InvalidName(name)) => Err(Error::NameInvalid(name)),
        Err(StorageDriverError::InvalidManifest) => Err(Error::ManifestInvalid),
        Err(_) => Err(Error::InternalError),
    }
}

/*
 * Parse 2 level <user>/<repo> style path and pass it to put_image_manifest
 */
#[put("/v2/<user>/<repo>/manifests/<reference>", data = "<chunk>")]
pub fn put_image_manifest_2level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    user: String,
    repo: String,
    reference: String,
    chunk: rocket::data::Data,
) -> Result<VerifiedManifest, Error> {
    put_image_manifest(
        auth_user,
        ci,
        format!("{}/{}", user, repo),
        reference,
        chunk,
    )
}

/*
 * Parse 3 level <org>/<user>/<repo> style path and pass it to put_image_manifest
 */
#[put("/v2/<org>/<user>/<repo>/manifests/<reference>", data = "<chunk>")]
pub fn put_image_manifest_3level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    org: String,
    user: String,
    repo: String,
    reference: String,
    chunk: rocket::data::Data,
) -> Result<VerifiedManifest, Error> {
    put_image_manifest(
        auth_user,
        ci,
        format!("{}/{}/{}", org, user, repo),
        reference,
        chunk,
    )
}

/*
 * Parse 4 level <fourth>/<org>/<user>/<repo> style path and pass it to put_image_manifest
 */
#[put(
    "/v2/<fourth>/<org>/<user>/<repo>/manifests/<reference>",
    data = "<chunk>"
)]
pub fn put_image_manifest_4level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    fourth: String,
    org: String,
    user: String,
    repo: String,
    reference: String,
    chunk: rocket::data::Data,
) -> Result<VerifiedManifest, Error> {
    put_image_manifest(
        auth_user,
        ci,
        format!("{}/{}/{}/{}", fourth, org, user, repo),
        reference,
        chunk,
    )
}

/*
---
Deleting an Image
DELETE /v2/<name>/manifests/<reference>
*/

#[delete("/v2/<repo>/manifests/<digest>")]
pub fn delete_image_manifest(
    _auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    repo: String,
    digest: String,
) -> Result<ManifestDeleted, Error> {
    let digest = digest::parse(&digest).map_err(|_| Error::Unsupported)?;
    match ci.delete_manifest(&repo, &digest) {
        Ok(_) => Ok(ManifestDeleted {}),
        Err(StorageDriverError::Unsupported) => Err(Error::Unsupported),
        Err(StorageDriverError::InvalidManifest) => Err(Error::ManifestUnknown(repo)),
        Err(_) => Err(Error::InternalError),
    }
}

#[delete("/v2/<user>/<repo>/manifests/<digest>")]
pub fn delete_image_manifest_2level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    user: String,
    repo: String,
    digest: String,
) -> Result<ManifestDeleted, Error> {
    delete_image_manifest(auth_user, ci, format!("{}/{}", user, repo), digest)
}

#[delete("/v2/<org>/<user>/<repo>/manifests/<digest>")]
pub fn delete_image_manifest_3level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    org: String,
    user: String,
    repo: String,
    digest: String,
) -> Result<ManifestDeleted, Error> {
    delete_image_manifest(auth_user, ci, format!("{}/{}/{}", org, user, repo), digest)
}

#[delete("/v2/<fourth>/<org>/<user>/<repo>/manifests/<digest>")]
pub fn delete_image_manifest_4level(
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
}
