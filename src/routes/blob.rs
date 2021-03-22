use crate::client_interface::ClientInterface;
use crate::registry_interface::{digest, BlobReader, BlobStorage, ContentInfo, StorageDriverError};
use crate::response::errors::Error;
use crate::response::trow_token::TrowToken;
use crate::response::upload_info::UploadInfo;
use crate::types::{
    create_accepted_upload, create_upload_info, AcceptedUpload, BlobDeleted, RepoName, Upload, Uuid,
};

use rocket::http::uri::{Origin, Uri};
use rocket_contrib::json::{Json, JsonValue};

use std::io::Read;

/*
---
Pulling a Layer
GET /v2/<name>/blobs/<digest>
name - name of the repository
digest - unique identifier for the blob to be downoaded

# Responses
200 - blob is downloaded
307 - redirect to another service for downloading[1]
 */

#[get("/v2/<name_repo>/blobs/<digest>")]
pub fn get_blob(
    _auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    name_repo: String,
    digest: String,
) -> Option<BlobReader> {
    let digest = digest::parse(&digest);
    match digest {
        Ok(d) => ci.get_blob(&name_repo, &d).ok(),
        Err(_) => None,
    }
}

/*
 * Parse 2 level <repo>/<name> style path and pass it to get_blob
 */

#[get("/v2/<name>/<repo>/blobs/<digest>")]
pub fn get_blob_2level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    name: String,
    repo: String,
    digest: String,
) -> Option<BlobReader> {
    get_blob(auth_user, ci, format!("{}/{}", name, repo), digest)
}

/*
 * Parse 3 level <org>/<repo>/<name> style path and pass it to get_blob
 */
#[get("/v2/<org>/<name>/<repo>/blobs/<digest>")]
pub fn get_blob_3level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    org: String,
    name: String,
    repo: String,
    digest: String,
) -> Option<BlobReader> {
    get_blob(auth_user, ci, format!("{}/{}/{}", org, name, repo), digest)
}

/*
 * Parse 4 level <org>/<repo>/<name> style path and pass it to get_blob
 */
#[get("/v2/<fourth>/<org>/<name>/<repo>/blobs/<digest>")]
pub fn get_blob_4level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    fourth: String,
    org: String,
    name: String,
    repo: String,
    digest: String,
) -> Option<BlobReader> {
    get_blob(
        auth_user,
        ci,
        format!("{}/{}/{}/{}", fourth, org, name, repo),
        digest,
    )
}

/*
---
Monolithic Upload
PUT /v2/<name>/blobs/uploads/<uuid>?digest=<digest>
Content-Length: <size of layer>
Content-Type: application/octet-stream

<Layer Binary Data>
 */

/**
 * Completes the upload.
 */
#[put("/v2/<repo_name>/blobs/uploads/<uuid>?<digest>", data = "<chunk>")]
pub fn put_blob(
    _auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    repo_name: String,
    uuid: String,
    digest: String,
    chunk: rocket::data::Data,
) -> Result<AcceptedUpload, Error> {
    let mut data: Box<dyn Read> = Box::new(chunk.open());

    let size = match ci.store_blob_chunk(&repo_name, &uuid, None, &mut data) {
        Ok(size) => size,
        Err(StorageDriverError::InvalidName(name)) => return Err(Error::NameInvalid(name)),
        Err(StorageDriverError::InvalidContentRange) => return Err(Error::BlobUploadInvalid),
        Err(_) => return Err(Error::InternalError),
    };

    let digest_obj = digest::parse(&digest).map_err(|_| Error::DigestInvalid)?;
    ci.complete_and_verify_blob_upload(&repo_name, &uuid, &digest_obj)
        .map_err(|e| match e {
            StorageDriverError::InvalidDigest => Error::DigestInvalid,
            _ => Error::InternalError,
        })?;

    Ok(create_accepted_upload(
        digest_obj,
        RepoName(repo_name),
        Uuid(uuid),
        (0, (size as u32)),
    ))
}

/*
 * Parse 2 level <repo>/<name> style path and pass it to put_blob
 */
#[put("/v2/<repo>/<name>/blobs/uploads/<uuid>?<digest>", data = "<chunk>")]
pub fn put_blob_2level(
    auth_user: TrowToken,
    config: rocket::State<ClientInterface>,
    repo: String,
    name: String,
    uuid: String,
    digest: String,
    chunk: rocket::data::Data,
) -> Result<AcceptedUpload, Error> {
    put_blob(
        auth_user,
        config,
        format!("{}/{}", repo, name),
        uuid,
        digest,
        chunk,
    )
}

/*
 * Parse 3 level <org>/<repo>/<name> style path and pass it to put_blob
 */
#[put(
    "/v2/<org>/<repo>/<name>/blobs/uploads/<uuid>?<digest>",
    data = "<chunk>"
)]
pub fn put_blob_3level(
    auth_user: TrowToken,
    config: rocket::State<ClientInterface>,
    org: String,
    repo: String,
    name: String,
    uuid: String,
    digest: String,
    chunk: rocket::data::Data,
) -> Result<AcceptedUpload, Error> {
    put_blob(
        auth_user,
        config,
        format!("{}/{}/{}", org, repo, name),
        uuid,
        digest,
        chunk,
    )
}

/*
 * Parse 4 level <org>/<repo>/<name> style path and pass it to put_blob
 */
#[put(
    "/v2/<fourth>/<org>/<repo>/<name>/blobs/uploads/<uuid>?<digest>",
    data = "<chunk>"
)]
pub fn put_blob_4level(
    auth_user: TrowToken,
    config: rocket::State<ClientInterface>,

    fourth: String,
    org: String,
    repo: String,
    name: String,
    uuid: String,
    digest: String,
    chunk: rocket::data::Data,
) -> Result<AcceptedUpload, Error> {
    put_blob(
        auth_user,
        config,
        format!("{}/{}/{}/{}", fourth, org, repo, name),
        uuid,
        digest,
        chunk,
    )
}

/*

---
Chunked Upload

PATCH /v2/<name>/blobs/uploads/<uuid>
Content-Length: <size of chunk>
Content-Range: <start of range>-<end of range>
Content-Type: application/octet-stream

<Layer Chunk Binary Data>
---

Uploads a blob or chunk of a blog.

Checks UUID. Returns UploadInfo with range set to correct position.

*/
#[patch("/v2/<repo_name>/blobs/uploads/<uuid>", data = "<chunk>")]
pub fn patch_blob(
    _auth_user: TrowToken,
    info: Option<ContentInfo>,
    ci: rocket::State<ClientInterface>,
    repo_name: String,
    uuid: String,
    chunk: rocket::data::Data,
) -> Result<UploadInfo, Error> {
    let mut data: Box<dyn Read> = Box::new(chunk.open());

    match ci.store_blob_chunk(&repo_name, &uuid, info, &mut data) {
        Ok(size) => {
            let repo_name = RepoName(repo_name);
            let uuid = Uuid(uuid);
            Ok(create_upload_info(uuid, repo_name, (0, size as u32)))
        }
        Err(StorageDriverError::InvalidName(name)) => Err(Error::NameInvalid(name)),
        Err(StorageDriverError::InvalidContentRange) => Err(Error::BlobUploadInvalid),
        Err(_) => Err(Error::InternalError),
    }
}

/*
 * Parse 2 level <repo>/<name> style path and pass it to patch_blob
 */
#[patch("/v2/<repo>/<name>/blobs/uploads/<uuid>", data = "<chunk>")]
pub fn patch_blob_2level(
    auth_user: TrowToken,
    info: Option<ContentInfo>,
    ci: rocket::State<ClientInterface>,
    repo: String,
    name: String,
    uuid: String,
    chunk: rocket::data::Data,
) -> Result<UploadInfo, Error> {
    patch_blob(
        auth_user,
        info,
        ci,
        format!("{}/{}", repo, name),
        uuid,
        chunk,
    )
}

/*
 * Parse 3 level <org>/<repo>/<name> style path and pass it to patch_blob
 */
#[patch("/v2/<org>/<repo>/<name>/blobs/uploads/<uuid>", data = "<chunk>")]
pub fn patch_blob_3level(
    auth_user: TrowToken,
    info: Option<ContentInfo>,
    handler: rocket::State<ClientInterface>,
    org: String,
    repo: String,
    name: String,
    uuid: String,
    chunk: rocket::data::Data,
) -> Result<UploadInfo, Error> {
    patch_blob(
        auth_user,
        info,
        handler,
        format!("{}/{}/{}", org, repo, name),
        uuid,
        chunk,
    )
}

/*
 * Parse 4 level <org>/<repo>/<name> style path and pass it to patch_blob
 */
#[patch(
    "/v2/<fourth>/<org>/<repo>/<name>/blobs/uploads/<uuid>",
    data = "<chunk>"
)]
pub fn patch_blob_4level(
    auth_user: TrowToken,
    info: Option<ContentInfo>,
    handler: rocket::State<ClientInterface>,
    fourth: String,
    org: String,
    repo: String,
    name: String,
    uuid: String,
    chunk: rocket::data::Data,
) -> Result<UploadInfo, Error> {
    patch_blob(
        auth_user,
        info,
        handler,
        format!("{}/{}/{}/{}", fourth, org, repo, name),
        uuid,
        chunk,
    )
}

/*
 Starting point for an uploading a new image or new version of an image.

 We respond with details of location and UUID to upload to with patch/put.

 No data is being transferred _unless_ the request ends with "?digest".
 In this case the whole blob is attached.
*/
#[post("/v2/<repo_name>/blobs/uploads", data = "<data>")]
pub fn post_blob_upload(
    uri: &Origin, // This is a mess, but needed to check for ?digest
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    repo_name: String,
    data: rocket::data::Data,
) -> Result<Upload, Error> {
    /*
    Ask the backend for a UUID.

    We should also need to do some checking that the user is allowed
    to upload first.

    If using a true UUID it is possible for the frontend to generate
    and tell the backend what the UUID is. This is a potential
    optimisation, but is arguably less flexible.
    */

    let uuid = ci.start_blob_upload(&repo_name).map_err(|e| match e {
        StorageDriverError::InvalidName(n) => Error::NameInvalid(n),
        _ => Error::InternalError,
    })?;

    if let Some(digest) = uri.query() {
        if digest.starts_with("digest=") {
            //Have a monolithic upload with data

            let digest = &Uri::percent_decode_lossy(&digest["digest=".len()..].as_bytes());
            return put_blob(
                auth_user,
                ci,
                repo_name.to_string(),
                uuid,
                digest.to_string(),
                data,
            )
            .map(|r| Upload::Accepted(r));
        }
    }

    Ok(Upload::Info(create_upload_info(
        Uuid(uuid),
        RepoName(repo_name.clone()),
        (0, 0),
    )))
}

/*
 * Parse 2 level <repo>/<name> style path and pass it to put_blob_upload_onename
 */
#[post("/v2/<repo>/<name>/blobs/uploads", data = "<data>")]
pub fn post_blob_upload_2level(
    //digest: PossibleDigest, //create requestguard to handle /?digest
    uri: &Origin,
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    repo: String,
    name: String,
    data: rocket::data::Data,
) -> Result<Upload, Error> {
    post_blob_upload(uri, auth_user, ci, format!("{}/{}", repo, name), data)
}

/*
 * Parse 3 level <org>/<repo>/<name> style path and pass it to put_blob_upload_onename
 */
#[post("/v2/<org>/<repo>/<name>/blobs/uploads", data = "<data>")]
pub fn post_blob_upload_3level(
    //digest: PossibleDigest, //create requestguard to handle /?digest
    uri: &Origin,
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    org: String,
    repo: String,
    name: String,
    data: rocket::data::Data,
) -> Result<Upload, Error> {
    post_blob_upload(
        uri,
        auth_user,
        ci,
        format!("{}/{}/{}", org, repo, name),
        data,
    )
}

/*
 * Parse 4 level <fourth>/<org>/<repo>/<name> style path
 */
#[post("/v2/<fourth>/<org>/<repo>/<name>/blobs/uploads", data = "<data>")]
pub fn post_blob_upload_4level(
    //digest: PossibleDigest, //create requestguard to handle /?digest
    uri: &Origin,
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    fourth: String,
    org: String,
    repo: String,
    name: String,
    data: rocket::data::Data,
) -> Result<Upload, Error> {
    post_blob_upload(
        uri,
        auth_user,
        ci,
        format!("{}/{}/{}/{}", fourth, org, repo, name),
        data,
    )
}

/*
 * Parse 5 level path and error.
 *
 * We really shouldn't error any number of paths, but it doesn't seem easy with Rocket.
 *
 * This should return a proper JSON error such as NAME_INVALID, but that causes the Docker
 * client to retry. Passing non-json causes an error and a reasonable message to the user.
 */
#[post(
    "/v2/<fifth>/<fourth>/<org>/<repo>/<name>/blobs/uploads",
    data = "<_data>"
)]
pub fn post_blob_upload_5level(
    _auth_user: TrowToken,
    fifth: String,
    fourth: String,
    org: String,
    repo: String,
    name: String,
    _data: rocket::data::Data,
) -> rocket::response::status::BadRequest<String> {
    rocket::response::status::BadRequest(Some(format!(
        "Repository names are limited to 4 levels: {}/{}/{}/{}/{} is not allowed",
        fifth, fourth, org, repo, name
    )))
}

/**
 * Deletes the given blob.
 *
 * Really unsure about this method - why should the user delete a blob?
 * TODO: This should probably be denied if the blob is referenced by any manifests
 * (manifest should be deleted first)
 */
#[delete("/v2/<repo>/blobs/<digest>")]
pub fn delete_blob(
    _auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    repo: String,
    digest: String,
) -> Result<BlobDeleted, Error> {
    let digest = digest::parse(&digest).map_err(|_| Error::DigestInvalid)?;
    ci.delete_blob(&repo, &digest)
        .map_err(|_| Error::BlobUnknown)?;
    Ok(BlobDeleted {})
}

#[delete("/v2/<user>/<repo>/blobs/<digest>")]
pub fn delete_blob_2level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    user: String,
    repo: String,
    digest: String,
) -> Result<BlobDeleted, Error> {
    delete_blob(auth_user, ci, format!("{}/{}", user, repo), digest)
}

#[delete("/v2/<org>/<user>/<repo>/blobs/<digest>")]
pub fn delete_blob_3level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    org: String,
    user: String,
    repo: String,
    digest: String,
) -> Result<BlobDeleted, Error> {
    delete_blob(auth_user, ci, format!("{}/{}/{}", org, user, repo), digest)
}

#[delete("/v2/<fourth>/<org>/<user>/<repo>/blobs/<digest>")]
pub fn delete_blob_4level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    fourth: String,
    org: String,
    user: String,
    repo: String,
    digest: String,
) -> Result<BlobDeleted, Error> {
    delete_blob(
        auth_user,
        ci,
        format!("{}/{}/{}/{}", fourth, org, user, repo),
        digest,
    )
}

#[options("/v2/<name_repo>/blobs/<digest>")]
pub fn options_blob(name_repo: String, digest: String) -> Json<JsonValue> {
    let _ = name_repo;
    let _ = digest;
    Json(json!({}))
}

#[options("/v2/<name>/<repo>/blobs/<digest>")]
pub fn options_blob_2level(name: String, repo: String, digest: String) -> Json<JsonValue> {
    options_blob(format!("{}/{}", name, repo), digest)
}

#[options("/v2/<org>/<name>/<repo>/blobs/<digest>")]
pub fn options_blob_3level(
    org: String,
    name: String,
    repo: String,
    digest: String,
) -> Json<JsonValue> {
    options_blob(format!("{}/{}/{}", org, name, repo), digest)
}

#[options("/v2/<fourth>/<org>/<name>/<repo>/blobs/<digest>")]
pub fn options_blob_4level(
    fourth: String,
    org: String,
    name: String,
    repo: String,
    digest: String,
) -> Json<JsonValue> {
    options_blob(format!("{}/{}/{}/{}", fourth, org, name, repo), digest)
}
