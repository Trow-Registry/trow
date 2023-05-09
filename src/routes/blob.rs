use crate::client_interface::ClientInterface;
use crate::registry_interface::{digest, BlobReader, BlobStorage, ContentInfo, StorageDriverError};
use crate::response::errors::Error;
use crate::response::trow_token::TrowToken;
use crate::response::upload_info::UploadInfo;
use crate::types::{
    create_accepted_upload, create_upload_info, AcceptedUpload, BlobDeleted, RepoName, Upload, Uuid,
};
use crate::TrowConfig;
use anyhow::Result;
use rocket::data::ToByteUnit;
use rocket::http::uri::Origin;
use rocket::{delete, get, patch, post, put};

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
pub async fn get_blob(
    _auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    name_repo: String,
    digest: String,
) -> Option<BlobReader> {
    let digest = digest::parse(&digest);
    match digest {
        Ok(d) => ci.get_blob(&name_repo, &d).await.ok(),
        Err(_) => None,
    }
}

/*
 * Parse 2 level <repo>/<name> style path and pass it to get_blob
 */

#[get("/v2/<name>/<repo>/blobs/<digest>")]
pub async fn get_blob_2level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    name: String,
    repo: String,
    digest: String,
) -> Option<BlobReader> {
    get_blob(auth_user, ci, format!("{}/{}", name, repo), digest).await
}

/*
 * Parse 3 level <org>/<repo>/<name> style path and pass it to get_blob
 */
#[get("/v2/<org>/<name>/<repo>/blobs/<digest>")]
pub async fn get_blob_3level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    org: String,
    name: String,
    repo: String,
    digest: String,
) -> Option<BlobReader> {
    get_blob(auth_user, ci, format!("{}/{}/{}", org, name, repo), digest).await
}

/*
 * Parse 4 level <org>/<repo>/<name> style path and pass it to get_blob
 */
#[get("/v2/<fourth>/<org>/<name>/<repo>/blobs/<digest>")]
pub async fn get_blob_4level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
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
    .await
}

/*
 * Parse 5 level <org>/<repo>/<name> style path and pass it to get_blob
 */
#[get("/v2/<fifth>/<fourth>/<org>/<name>/<repo>/blobs/<digest>")]
pub async fn get_blob_5level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    fifth: String,
    fourth: String,
    org: String,
    name: String,
    repo: String,
    digest: String,
) -> Option<BlobReader> {
    get_blob(
        auth_user,
        ci,
        format!("{}/{}/{}/{}/{}", fifth, fourth, org, name, repo),
        digest,
    )
    .await
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
pub async fn put_blob(
    _auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    tc: &rocket::State<TrowConfig>,
    repo_name: String,
    uuid: String,
    digest: String,
    chunk: rocket::data::Data<'_>,
) -> Result<AcceptedUpload, Error> {
    let ds = chunk.open(tc.max_blob_size.mebibytes());

    let size = match ci.store_blob_chunk(&repo_name, &uuid, None, ds).await {
        Ok(stored) => {
            if !stored.complete {
                return Err(Error::BlobUploadInvalid(format!(
                    "Content over data limit {} mebibytes",
                    tc.max_blob_size
                )));
            } else {
                stored.total_stored
            }
        }
        Err(StorageDriverError::InvalidName(name)) => return Err(Error::NameInvalid(name)),
        Err(StorageDriverError::InvalidContentRange) => {
            return Err(Error::BlobUploadInvalid(
                "Invalid Content Range".to_string(),
            ))
        }
        Err(_) => return Err(Error::InternalError),
    };

    let digest_obj = digest::parse(&digest).map_err(|_| Error::DigestInvalid)?;
    ci.complete_and_verify_blob_upload(&repo_name, &uuid, &digest_obj)
        .await
        .map_err(|e| match e {
            StorageDriverError::InvalidDigest => Error::DigestInvalid,
            _ => Error::InternalError,
        })?;

    Ok(create_accepted_upload(
        digest_obj,
        RepoName(repo_name),
        Uuid(uuid),
        (0, (size as u32).saturating_sub(1)), // Note first byte is 0
    ))
}

/*
 * Parse 2 level <repo>/<name> style path and pass it to put_blob
 */
#[put("/v2/<repo>/<name>/blobs/uploads/<uuid>?<digest>", data = "<chunk>")]
pub async fn put_blob_2level(
    auth_user: TrowToken,
    config: &rocket::State<ClientInterface>,
    tc: &rocket::State<TrowConfig>,
    repo: String,
    name: String,
    uuid: String,
    digest: String,
    chunk: rocket::data::Data<'_>,
) -> Result<AcceptedUpload, Error> {
    put_blob(
        auth_user,
        config,
        tc,
        format!("{}/{}", repo, name),
        uuid,
        digest,
        chunk,
    )
    .await
}

/*
 * Parse 3 level <org>/<repo>/<name> style path and pass it to put_blob
 */
#[put(
    "/v2/<org>/<repo>/<name>/blobs/uploads/<uuid>?<digest>",
    data = "<chunk>"
)]
pub async fn put_blob_3level(
    auth_user: TrowToken,
    config: &rocket::State<ClientInterface>,
    tc: &rocket::State<TrowConfig>,
    org: String,
    repo: String,
    name: String,
    uuid: String,
    digest: String,
    chunk: rocket::data::Data<'_>,
) -> Result<AcceptedUpload, Error> {
    put_blob(
        auth_user,
        config,
        tc,
        format!("{}/{}/{}", org, repo, name),
        uuid,
        digest,
        chunk,
    )
    .await
}

/*
 * Parse 4 level <org>/<repo>/<name> style path and pass it to put_blob
 */
#[put(
    "/v2/<fourth>/<org>/<repo>/<name>/blobs/uploads/<uuid>?<digest>",
    data = "<chunk>"
)]
pub async fn put_blob_4level(
    auth_user: TrowToken,
    config: &rocket::State<ClientInterface>,
    tc: &rocket::State<TrowConfig>,
    fourth: String,
    org: String,
    repo: String,
    name: String,
    uuid: String,
    digest: String,
    chunk: rocket::data::Data<'_>,
) -> Result<AcceptedUpload, Error> {
    put_blob(
        auth_user,
        config,
        tc,
        format!("{}/{}/{}/{}", fourth, org, repo, name),
        uuid,
        digest,
        chunk,
    )
    .await
}

/*
 * Parse 4 level <org>/<repo>/<name> style path and pass it to put_blob
 */
#[put(
    "/v2/<fifth>/<fourth>/<org>/<repo>/<name>/blobs/uploads/<uuid>?<digest>",
    data = "<chunk>"
)]
pub async fn put_blob_5level(
    auth_user: TrowToken,
    config: &rocket::State<ClientInterface>,
    tc: &rocket::State<TrowConfig>,
    fifth: String,
    fourth: String,
    org: String,
    repo: String,
    name: String,
    uuid: String,
    digest: String,
    chunk: rocket::data::Data<'_>,
) -> Result<AcceptedUpload, Error> {
    put_blob(
        auth_user,
        config,
        tc,
        format!("{}/{}/{}/{}/{}", fifth, fourth, org, repo, name),
        uuid,
        digest,
        chunk,
    )
    .await
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

Uploads a blob or chunk of a blob.

Checks UUID. Returns UploadInfo with range set to correct position.

*/
#[patch("/v2/<repo_name>/blobs/uploads/<uuid>", data = "<chunk>")]
pub async fn patch_blob(
    _auth_user: TrowToken,
    info: Option<ContentInfo>,
    ci: &rocket::State<ClientInterface>,
    tc: &rocket::State<TrowConfig>,
    repo_name: String,
    uuid: String,
    chunk: rocket::data::Data<'_>,
) -> Result<UploadInfo, Error> {
    let data = chunk.open(tc.max_blob_size.mebibytes());

    match ci.store_blob_chunk(&repo_name, &uuid, info, data).await {
        Ok(stored) => {
            let repo_name = RepoName(repo_name);
            let uuid = Uuid(uuid);
            if !stored.complete {
                Err(Error::BlobUploadInvalid(format!(
                    "Content over data limit {} mebibytes",
                    tc.max_blob_size
                )))
            } else {
                Ok(create_upload_info(
                    uuid,
                    repo_name,
                    (0, (stored.total_stored as u32).saturating_sub(1)), // First byte is 0
                ))
            }
        }
        Err(StorageDriverError::InvalidName(name)) => Err(Error::NameInvalid(name)),
        Err(StorageDriverError::InvalidContentRange) => Err(Error::BlobUploadInvalid(
            "Invalid Content Range".to_string(),
        )),
        Err(_) => Err(Error::InternalError),
    }
}

/*
 * Parse 2 level <repo>/<name> style path and pass it to patch_blob
 */
#[patch("/v2/<repo>/<name>/blobs/uploads/<uuid>", data = "<chunk>")]
pub async fn patch_blob_2level(
    auth_user: TrowToken,
    info: Option<ContentInfo>,
    ci: &rocket::State<ClientInterface>,
    tc: &rocket::State<TrowConfig>,
    repo: String,
    name: String,
    uuid: String,
    chunk: rocket::data::Data<'_>,
) -> Result<UploadInfo, Error> {
    patch_blob(
        auth_user,
        info,
        ci,
        tc,
        format!("{}/{}", repo, name),
        uuid,
        chunk,
    )
    .await
}

/*
 * Parse 3 level <org>/<repo>/<name> style path and pass it to patch_blob
 */
#[patch("/v2/<org>/<repo>/<name>/blobs/uploads/<uuid>", data = "<chunk>")]
pub async fn patch_blob_3level(
    auth_user: TrowToken,
    info: Option<ContentInfo>,
    handler: &rocket::State<ClientInterface>,
    tc: &rocket::State<TrowConfig>,
    org: String,
    repo: String,
    name: String,
    uuid: String,
    chunk: rocket::data::Data<'_>,
) -> Result<UploadInfo, Error> {
    patch_blob(
        auth_user,
        info,
        handler,
        tc,
        format!("{}/{}/{}", org, repo, name),
        uuid,
        chunk,
    )
    .await
}

/*
 * Parse 4 level <org>/<repo>/<name> style path and pass it to patch_blob
 */
#[patch(
    "/v2/<fourth>/<org>/<repo>/<name>/blobs/uploads/<uuid>",
    data = "<chunk>"
)]
pub async fn patch_blob_4level(
    auth_user: TrowToken,
    info: Option<ContentInfo>,
    handler: &rocket::State<ClientInterface>,
    tc: &rocket::State<TrowConfig>,
    fourth: String,
    org: String,
    repo: String,
    name: String,
    uuid: String,
    chunk: rocket::data::Data<'_>,
) -> Result<UploadInfo, Error> {
    patch_blob(
        auth_user,
        info,
        handler,
        tc,
        format!("{}/{}/{}/{}", fourth, org, repo, name),
        uuid,
        chunk,
    )
    .await
}

/*
 * Parse 5 level <org>/<repo>/<name> style path and pass it to patch_blob
 */
#[patch(
    "/v2/<fifth>/<fourth>/<org>/<repo>/<name>/blobs/uploads/<uuid>",
    data = "<chunk>"
)]
pub async fn patch_blob_5level(
    auth_user: TrowToken,
    info: Option<ContentInfo>,
    handler: &rocket::State<ClientInterface>,
    tc: &rocket::State<TrowConfig>,
    fifth: String,
    fourth: String,
    org: String,
    repo: String,
    name: String,
    uuid: String,
    chunk: rocket::data::Data<'_>,
) -> Result<UploadInfo, Error> {
    patch_blob(
        auth_user,
        info,
        handler,
        tc,
        format!("{}/{}/{}/{}/{}", fifth, fourth, org, repo, name),
        uuid,
        chunk,
    )
    .await
}

/*
 Starting point for an uploading a new image or new version of an image.

 We respond with details of location and UUID to upload to with patch/put.

 No data is being transferred _unless_ the request ends with "?digest".
 In this case the whole blob is attached.
*/
#[post("/v2/<repo_name>/blobs/uploads", data = "<data>")]
pub async fn post_blob_upload(
    uri: &Origin<'_>, // This is a mess, but needed to check for ?digest
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    tc: &rocket::State<TrowConfig>,
    repo_name: String,
    data: rocket::data::Data<'_>,
) -> Result<Upload, Error> {
    /*
    Ask the backend for a UUID.

    We should also need to do some checking that the user is allowed
    to upload first.

    If using a true UUID it is possible for the frontend to generate
    and tell the backend what the UUID is. This is a potential
    optimisation, but is arguably less flexible.
    */

    let uuid = ci
        .start_blob_upload(&repo_name)
        .await
        .map_err(|e| match e {
            StorageDriverError::InvalidName(n) => Error::NameInvalid(n),
            _ => Error::InternalError,
        })?;

    if let Some(digest) = uri.query() {
        if digest.starts_with("digest=") {
            //Have a monolithic upload with data

            //Unwrap must be safe given above statement
            let digest = digest
                .strip_prefix("digest=")
                .unwrap()
                .percent_decode_lossy();
            return put_blob(
                auth_user,
                ci,
                tc,
                repo_name.to_string(),
                uuid,
                digest.to_string(),
                data,
            )
            .await
            .map(Upload::Accepted);
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
pub async fn post_blob_upload_2level(
    //digest: PossibleDigest, //create requestguard to handle /?digest
    uri: &Origin<'_>,
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    tc: &rocket::State<TrowConfig>,
    repo: String,
    name: String,
    data: rocket::data::Data<'_>,
) -> Result<Upload, Error> {
    post_blob_upload(uri, auth_user, ci, tc, format!("{}/{}", repo, name), data).await
}

/*
 * Parse 3 level <org>/<repo>/<name> style path and pass it to put_blob_upload_onename
 */
#[post("/v2/<org>/<repo>/<name>/blobs/uploads", data = "<data>")]
pub async fn post_blob_upload_3level(
    //digest: PossibleDigest, //create requestguard to handle /?digest
    uri: &Origin<'_>,
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    tc: &rocket::State<TrowConfig>,
    org: String,
    repo: String,
    name: String,
    data: rocket::data::Data<'_>,
) -> Result<Upload, Error> {
    post_blob_upload(
        uri,
        auth_user,
        ci,
        tc,
        format!("{}/{}/{}", org, repo, name),
        data,
    )
    .await
}

/*
 * Parse 4 level <fourth>/<org>/<repo>/<name> style path
 */
#[post("/v2/<fourth>/<org>/<repo>/<name>/blobs/uploads", data = "<data>")]
pub async fn post_blob_upload_4level(
    //digest: PossibleDigest, //create requestguard to handle /?digest
    uri: &Origin<'_>,
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    tc: &rocket::State<TrowConfig>,
    fourth: String,
    org: String,
    repo: String,
    name: String,
    data: rocket::data::Data<'_>,
) -> Result<Upload, Error> {
    post_blob_upload(
        uri,
        auth_user,
        ci,
        tc,
        format!("{}/{}/{}/{}", fourth, org, repo, name),
        data,
    )
    .await
}

/*
 * Parse 5 level <fith>/<fourth>/<org>/<repo>/<name> style path
 */
#[post(
    "/v2/<fifth>/<fourth>/<org>/<repo>/<name>/blobs/uploads",
    data = "<data>"
)]
pub async fn post_blob_upload_5level(
    //digest: PossibleDigest, //create requestguard to handle /?digest
    uri: &Origin<'_>,
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    tc: &rocket::State<TrowConfig>,
    fifth: String,
    fourth: String,
    org: String,
    repo: String,
    name: String,
    data: rocket::data::Data<'_>,
) -> Result<Upload, Error> {
    post_blob_upload(
        uri,
        auth_user,
        ci,
        tc,
        format!("{}/{}/{}/{}/{}", fifth, fourth, org, repo, name),
        data,
    )
    .await
}

/*
 * Parse 6 level path and error.
 *
 * We really shouldn't error any number of paths, but it doesn't seem easy with Rocket.
 *
 * This should return a proper JSON error such as NAME_INVALID, but that causes the Docker
 * client to retry. Passing non-json causes an error and a reasonable message to the user.
 */
#[post(
    "/v2/<sixth>/<fifth>/<fourth>/<org>/<repo>/<name>/blobs/uploads",
    data = "<_data>"
)]
pub fn post_blob_upload_6level(
    _auth_user: TrowToken,
    sixth: String,
    fifth: String,
    fourth: String,
    org: String,
    repo: String,
    name: String,
    _data: rocket::data::Data,
) -> rocket::response::status::BadRequest<String> {
    rocket::response::status::BadRequest(Some(format!(
        "Repository names are limited to 5 levels: {}/{}/{}/{}/{}/{} is not allowed",
        sixth, fifth, fourth, org, repo, name
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
pub async fn delete_blob(
    _auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    repo: String,
    digest: String,
) -> Result<BlobDeleted, Error> {
    let digest = digest::parse(&digest).map_err(|_| Error::DigestInvalid)?;
    ci.delete_blob(&repo, &digest)
        .await
        .map_err(|_| Error::BlobUnknown)?;
    Ok(BlobDeleted {})
}

#[delete("/v2/<user>/<repo>/blobs/<digest>")]
pub async fn delete_blob_2level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    user: String,
    repo: String,
    digest: String,
) -> Result<BlobDeleted, Error> {
    delete_blob(auth_user, ci, format!("{}/{}", user, repo), digest).await
}

#[delete("/v2/<org>/<user>/<repo>/blobs/<digest>")]
pub async fn delete_blob_3level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    org: String,
    user: String,
    repo: String,
    digest: String,
) -> Result<BlobDeleted, Error> {
    delete_blob(auth_user, ci, format!("{}/{}/{}", org, user, repo), digest).await
}

#[delete("/v2/<fourth>/<org>/<user>/<repo>/blobs/<digest>")]
pub async fn delete_blob_4level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
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
    .await
}

#[delete("/v2/<fifth>/<fourth>/<org>/<user>/<repo>/blobs/<digest>")]
pub async fn delete_blob_5level(
    auth_user: TrowToken,
    ci: &rocket::State<ClientInterface>,
    fifth: String,
    fourth: String,
    org: String,
    user: String,
    repo: String,
    digest: String,
) -> Result<BlobDeleted, Error> {
    delete_blob(
        auth_user,
        ci,
        format!("{}/{}/{}/{}/{}", fifth, fourth, org, user, repo),
        digest,
    )
    .await
}
