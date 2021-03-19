use crate::client_interface::{ClientInterface, RegistryError};
use crate::response::authenticate::Authenticate;
use crate::response::errors::Error;
use crate::response::html::HTML;
use crate::response::trow_token::ValidBasicToken;
use crate::response::trow_token::{self, TrowToken};
use crate::response::upload_info::UploadInfo;
use crate::types::*;
use crate::TrowConfig;
use rocket::http::uri::{Origin, Uri};
use rocket::request::Request;
use rocket::State;
use rocket_contrib::json::{Json, JsonValue};
use std::io::Read;
use std::str;
use tonic::Code;

mod health;
mod metrics;
mod readiness;
mod catalog;
mod tags;
mod manifest;
mod blob;
mod login;

//ENORMOUS TODO: at the moment we spawn a whole runtime for each request,
//which is hugely inefficient. Need to figure out how to use thread-local
//for each runtime or move to Warp and share the runtime.
use tokio::runtime::Runtime;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_v2root,
        get_homepage,
        health::healthz,
        readiness::readiness,
        metrics::metrics,
        login,
        login::options_login,
        get_manifest,
        get_manifest_2level,
        get_manifest_3level,
        get_manifest_4level,
        manifest::options_manifest,
        manifest::options_manifest_2level,
        manifest::options_manifest_3level,
        manifest::options_manifest_4level,
        put_image_manifest,
        put_image_manifest_2level,
        put_image_manifest_3level,
        put_image_manifest_4level,
        get_blob,
        get_blob_2level,
        get_blob_3level,
        get_blob_4level,
        blob::options_blob,
        blob::options_blob_2level,
        blob::options_blob_3level,
        blob::options_blob_4level,
        put_blob,
        put_blob_2level,
        put_blob_3level,
        put_blob_4level,
        patch_blob,
        patch_blob_2level,
        patch_blob_3level,
        patch_blob_4level,
        post_blob_upload,
        post_blob_upload_2level,
        post_blob_upload_3level,
        post_blob_upload_4level,
        post_blob_upload_5level,
        list_tags,
        list_tags_2level,
        list_tags_3level,
        list_tags_4level,
        tags::options_tags,
        tags::options_tags_2level,
        tags::options_tags_3level,
        tags::options_tags_4level,
        get_catalog,
        catalog::options_catalog,
        validate_image,
        delete_blob,
        delete_blob_2level,
        delete_blob_3level,
        delete_blob_4level,
        delete_image_manifest,
        delete_image_manifest_2level,
        delete_image_manifest_3level,
        delete_image_manifest_4level,
        get_manifest_history,
        get_manifest_history_2level,
        get_manifest_history_3level,
        get_manifest_history_4level,
    ]
    /* The following routes used to have stub methods, but I removed them as they were cluttering the code
          post_blob_uuid,
          get_upload_progress,
          delete_upload,
          admin routes,
          admin_get_uuids

    To find the stubs, go to https://github.com/ContainerSolutions/trow/tree/4b007088bb0657a98238870d9aaca638e01f6487
    Please add tests for any routes that you recover.
    */
}

pub fn catchers() -> Vec<rocket::Catcher> {
    catchers![not_found, no_auth]
}

/*
 * v2 - throw Empty
 */
#[get("/v2")]
fn get_v2root(_auth_user: TrowToken) -> Json<JsonValue> {
    Json(json!({}))
}
/*
 * Welcome message
 */
#[get("/")]
fn get_homepage<'a>() -> HTML<'a> {
    const ROOT_RESPONSE: &str = "<!DOCTYPE html><html><body>
<h1>Welcome to Trow, the cluster registry</h1>
</body></html>";

    HTML(ROOT_RESPONSE)
}

// Want non HTML return for 404 for docker client
#[catch(404)]
fn not_found(_: &Request) -> Json<String> {
    Json("404 page not found".to_string())
}

#[catch(401)]
fn no_auth(_req: &Request) -> Authenticate {
    Authenticate {}
}

/* login should it be /v2/login?
 * this is where client will attempt to login
 *
 * If login is called with a valid bearer token, return session token
 */
#[get("/login")]
fn login(auth_user: ValidBasicToken, tc: State<TrowConfig>) -> Result<TrowToken, Error> {
    trow_token::new(auth_user, tc).map_err(|_| Error::InternalError)
}

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
fn get_manifest(
    _auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    onename: String,
    reference: String,
) -> Result<ManifestReader, Error> {
    let rn = RepoName(onename);
    let f = ci.get_reader_for_manifest(&rn, &reference);
    let mut rt = Runtime::new().unwrap();
    rt.block_on(f)
        .map_err(|_| Error::ManifestUnknown(reference))
}

#[get("/v2/<user>/<repo>/manifests/<reference>")]
fn get_manifest_2level(
    _auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    user: String,
    repo: String,
    reference: String,
) -> Option<ManifestReader> {
    let rn = RepoName(format!("{}/{}", user, repo));
    let r = ci.get_reader_for_manifest(&rn, &reference);
    let mut rt = Runtime::new().unwrap();
    rt.block_on(r).ok()
}

/*
 * Process 3 level manifest path
 */
#[get("/v2/<org>/<user>/<repo>/manifests/<reference>")]
fn get_manifest_3level(
    _auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    org: String,
    user: String,
    repo: String,
    reference: String,
) -> Option<ManifestReader> {
    let rn = RepoName(format!("{}/{}/{}", org, user, repo));
    let r = ci.get_reader_for_manifest(&rn, &reference);
    Runtime::new().unwrap().block_on(r).ok()
}

/*
 * Process 4 level manifest path
 */
#[get("/v2/<fourth>/<org>/<user>/<repo>/manifests/<reference>")]
fn get_manifest_4level(
    _auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    fourth: String,
    org: String,
    user: String,
    repo: String,
    reference: String,
) -> Option<ManifestReader> {
    let rn = RepoName(format!("{}/{}/{}/{}", fourth, org, user, repo));
    let r = ci.get_reader_for_manifest(&rn, &reference);
    Runtime::new().unwrap().block_on(r).ok()
}

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
fn get_blob(
    _auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    name_repo: String,
    digest: String,
) -> Option<BlobReader> {
    let rn = RepoName(name_repo);
    let d = Digest(digest);
    let r = ci.get_reader_for_blob(&rn, &d);
    Runtime::new().unwrap().block_on(r).ok()
}

/*
 * Parse 2 level <repo>/<name> style path and pass it to get_blob
 */

#[get("/v2/<name>/<repo>/blobs/<digest>")]
fn get_blob_2level(
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
fn get_blob_3level(
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
fn get_blob_4level(
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
 *
 * TODO: allow uploading of final data
 * TODO: add other failure states
 */
#[put("/v2/<repo_name>/blobs/uploads/<uuid>?<digest>", data = "<chunk>")]
fn put_blob(
    _auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    repo_name: String,
    uuid: String,
    digest: String,
    chunk: rocket::data::Data,
) -> Result<AcceptedUpload, Error> {
    let repo = RepoName(repo_name.clone());
    let uuid = Uuid(uuid);

    let mut rt = Runtime::new().unwrap();
    let mut data: Box<dyn Read> = Box::new(chunk.open());

    match rt.block_on(ci.upload_blob(&repo, &uuid, &digest, &mut data)) {
        Ok(au) => Ok(au),
        Err(RegistryError::InvalidNameOrUUID) => Err(Error::NameInvalid(repo_name)),
        Err(_) => Err(Error::InternalError),
    }
}

/*
 * Parse 2 level <repo>/<name> style path and pass it to put_blob
 */
#[put("/v2/<repo>/<name>/blobs/uploads/<uuid>?<digest>", data = "<chunk>")]
fn put_blob_2level(
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
fn put_blob_3level(
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
fn put_blob_4level(
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
fn patch_blob(
    _auth_user: TrowToken,
    info: Option<ContentInfo>,
    ci: rocket::State<ClientInterface>,
    repo_name: String,
    uuid: String,
    chunk: rocket::data::Data,
) -> Result<UploadInfo, Error> {
    let repo = RepoName(repo_name.clone());
    let uuid = Uuid(uuid);

    let mut rt = Runtime::new().unwrap();
    let mut data: Box<dyn Read> = Box::new(chunk.open());

    match rt.block_on(ci.upload_blob_chunk(&repo, &uuid, info, &mut data)) {
        Ok(au) => Ok(au),
        Err(RegistryError::InvalidNameOrUUID) => Err(Error::NameInvalid(repo_name)),
        Err(RegistryError::InvalidContentRange) => Err(Error::BlobUploadInvalid),
        Err(_) => Err(Error::InternalError),
    }
}

/*
 * Parse 2 level <repo>/<name> style path and pass it to patch_blob
 */
#[patch("/v2/<repo>/<name>/blobs/uploads/<uuid>", data = "<chunk>")]
fn patch_blob_2level(
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
fn patch_blob_3level(
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
fn patch_blob_4level(
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
fn post_blob_upload(
    uri: &Origin, // This is a mess, but needed to check for ?digest
    _auth_user: TrowToken,
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

    let rn = repo_name.clone();
    let repo_name = RepoName(repo_name);
    let mut rt = Runtime::new().unwrap();

    let req = ci.request_upload(&repo_name);

    let up_info = rt.block_on(req).map_err(|e| {
        warn!("Error getting ref from backend: {}", e);
        let e = e.downcast::<tonic::Status>();
        if let Ok(ts) = e {
            match ts.code() {
                Code::InvalidArgument => Error::NameInvalid(rn),
                _ => Error::InternalError,
            }
        } else {
            Error::InternalError
        }
    })?;

    if let Some(digest) = uri.query() {
        if digest.starts_with("digest=") {
            //Have a monolithic upload

            let mut data: Box<dyn Read> = Box::new(data.open());
            let digest = &Uri::percent_decode_lossy(&digest["digest=".len()..].as_bytes());

            return match rt.block_on(ci.upload_blob(
                &repo_name,
                &up_info.uuid(),
                &digest,
                &mut data,
            )) {
                Ok(au) => Ok(Upload::Accepted(au)),
                Err(RegistryError::InvalidNameOrUUID) => Err(Error::BlobUnknown),
                Err(_) => Err(Error::InternalError),
            };
        }
    }
    Ok(Upload::Info(up_info))
}

/*
 * Parse 2 level <repo>/<name> style path and pass it to put_blob_upload_onename
 */
#[post("/v2/<repo>/<name>/blobs/uploads", data = "<data>")]
fn post_blob_upload_2level(
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
fn post_blob_upload_3level(
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
fn post_blob_upload_4level(
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
fn post_blob_upload_5level(
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

/*

---
Pushing an image manifest
PUT /v2/<name>/manifests/<reference>
Content-Type: <manifest media type>

 */
#[put("/v2/<repo_name>/manifests/<reference>", data = "<chunk>")]
fn put_image_manifest(
    _auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    repo_name: String,
    reference: String,
    chunk: rocket::data::Data,
) -> Result<VerifiedManifest, Error> {
    let rn = repo_name.clone();
    let repo = RepoName(repo_name);

    let mut rt = Runtime::new().unwrap();
    let mut data: Box<dyn Read> = Box::new(chunk.open());
    match rt.block_on(ci.upload_manifest(&repo, &reference, &mut data)) {
        Ok(vm) => Ok(vm),
        Err(RegistryError::InvalidName) => Err(Error::NameInvalid(rn)),
        Err(RegistryError::InvalidManifest) => Err(Error::ManifestInvalid),
        Err(_) => Err(Error::InternalError),
    }
}

/*
 * Parse 2 level <user>/<repo> style path and pass it to put_image_manifest
 */
#[put("/v2/<user>/<repo>/manifests/<reference>", data = "<chunk>")]
fn put_image_manifest_2level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
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
fn put_image_manifest_3level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
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
fn put_image_manifest_4level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
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
fn delete_image_manifest(
    _auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    repo: String,
    digest: String,
) -> Result<ManifestDeleted, Error> {
    let repo_str = repo.clone();
    let repo = RepoName(repo);
    let digest = Digest(digest);
    let r = ci.delete_manifest(&repo, &digest);
    Runtime::new().unwrap().block_on(r).map_err(|e| {
        let e = e.downcast::<tonic::Status>();
        if let Ok(ts) = e {
            match ts.code() {
                Code::InvalidArgument => Error::Unsupported,
                Code::NotFound => Error::ManifestUnknown(repo_str),
                _ => Error::InternalError,
            }
        } else {
            Error::InternalError
        }
    })
}

#[delete("/v2/<user>/<repo>/manifests/<digest>")]
fn delete_image_manifest_2level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    user: String,
    repo: String,
    digest: String,
) -> Result<ManifestDeleted, Error> {
    delete_image_manifest(auth_user, ci, format!("{}/{}", user, repo), digest)
}

#[delete("/v2/<org>/<user>/<repo>/manifests/<digest>")]
fn delete_image_manifest_3level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    org: String,
    user: String,
    repo: String,
    digest: String,
) -> Result<ManifestDeleted, Error> {
    delete_image_manifest(auth_user, ci, format!("{}/{}/{}", org, user, repo), digest)
}

#[delete("/v2/<fourth>/<org>/<user>/<repo>/manifests/<digest>")]
fn delete_image_manifest_4level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
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

/**
 * Deletes the given blob.
 *
 * Really unsure about this method - why should the user delete a blob?
 * TODO: This should probably be denied if the blob is referenced by any manifests
 * (manifest should be deleted first)
 */
#[delete("/v2/<repo>/blobs/<digest>")]
fn delete_blob(
    _auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    repo: String,
    digest: String,
) -> Result<BlobDeleted, Error> {
    let repo = RepoName(repo);
    let digest = Digest(digest);
    let r = ci.delete_blob(&repo, &digest);
    Runtime::new()
        .unwrap()
        .block_on(r)
        .map_err(|_| Error::BlobUnknown)
}

#[delete("/v2/<user>/<repo>/blobs/<digest>")]
fn delete_blob_2level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    user: String,
    repo: String,
    digest: String,
) -> Result<BlobDeleted, Error> {
    delete_blob(auth_user, ci, format!("{}/{}", user, repo), digest)
}

#[delete("/v2/<org>/<user>/<repo>/blobs/<digest>")]
fn delete_blob_3level(
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
fn delete_blob_4level(
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

#[get("/v2/_catalog?<n>&<last>")]
fn get_catalog(
    _auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    n: Option<u32>,
    last: Option<String>,
) -> Result<RepoCatalog, Error> {
    let limit = n.unwrap_or(std::u32::MAX);
    let last_repo = last.unwrap_or_default();
    let cat = ci.get_catalog(limit, &last_repo);
    match Runtime::new().unwrap().block_on(cat) {
        Ok(c) => Ok(c),
        Err(_) => Err(Error::InternalError),
    }
}

#[get("/v2/<repo_name>/tags/list?<last>&<n>")]
fn list_tags(
    _auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    repo_name: String,
    last: Option<String>,
    n: Option<u32>,
) -> Result<TagList, Error> {
    let rn = RepoName(repo_name);
    let limit = n.unwrap_or(std::u32::MAX);
    let last_tag = last.unwrap_or_default();

    let tags = ci.list_tags(&rn, limit, &last_tag);
    match Runtime::new().unwrap().block_on(tags) {
        Ok(c) => Ok(c),
        Err(_) => Err(Error::InternalError),
    }
}

#[get("/v2/<user>/<repo>/tags/list?<last>&<n>")]
fn list_tags_2level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    user: String,
    repo: String,
    last: Option<String>,
    n: Option<u32>,
) -> Result<TagList, Error> {
    list_tags(auth_user, ci, format!("{}/{}", user, repo), last, n)
}

#[get("/v2/<org>/<user>/<repo>/tags/list?<last>&<n>")]
fn list_tags_3level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    org: String,
    user: String,
    repo: String,
    last: Option<String>,
    n: Option<u32>,
) -> Result<TagList, Error> {
    list_tags(auth_user, ci, format!("{}/{}/{}", org, user, repo), last, n)
}

#[get("/v2/<fourth>/<org>/<user>/<repo>/tags/list?<last>&<n>")]
fn list_tags_4level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    fourth: String,
    org: String,
    user: String,
    repo: String,
    last: Option<String>,
    n: Option<u32>,
) -> Result<TagList, Error> {
    list_tags(
        auth_user,
        ci,
        format!("{}/{}/{}/{}", fourth, org, user, repo),
        last,
        n,
    )
}

// TODO add support for pagination
#[get("/<onename>/manifest_history/<reference>?<last>&<n>")]
fn get_manifest_history(
    _auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    onename: String,
    reference: String,
    last: Option<String>,
    n: Option<u32>,
) -> Result<ManifestHistory, Error> {
    let limit = n.unwrap_or(std::u32::MAX);
    let last_digest = last.unwrap_or_default();

    let rn = RepoName(onename);
    let f = ci.get_manifest_history(&rn, &reference, limit, &last_digest);
    let mut rt = Runtime::new().unwrap();
    rt.block_on(f)
        .map_err(|_| Error::ManifestUnknown(reference))
}

#[get("/<user>/<repo>/manifest_history/<reference>?<last>&<n>")]
fn get_manifest_history_2level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    user: String,
    repo: String,
    reference: String,
    last: Option<String>,
    n: Option<u32>,
) -> Result<ManifestHistory, Error> {
    get_manifest_history(
        auth_user,
        ci,
        format!("{}/{}", user, repo),
        reference,
        last,
        n,
    )
}

#[get("/<org>/<user>/<repo>/manifest_history/<reference>?<last>&<n>")]
fn get_manifest_history_3level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    org: String,
    user: String,
    repo: String,
    reference: String,
    last: Option<String>,
    n: Option<u32>,
) -> Result<ManifestHistory, Error> {
    get_manifest_history(
        auth_user,
        ci,
        format!("{}/{}/{}", org, user, repo),
        reference,
        last,
        n,
    )
}

#[get("/<fourth>/<org>/<user>/<repo>/manifest_history/<reference>?<last>&<n>")]
fn get_manifest_history_4level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    fourth: String,
    org: String,
    user: String,
    repo: String,
    reference: String,
    last: Option<String>,
    n: Option<u32>,
) -> Result<ManifestHistory, Error> {
    get_manifest_history(
        auth_user,
        ci,
        format!("{}/{}/{}/{}", fourth, org, user, repo),
        reference,
        last,
        n,
    )
}

//Might want to move this stuff somewhere else
//Kubernetes webhooks for admitting images
//Update to use rocket_contrib::Json
//Just using String for debugging
#[post("/validate-image", data = "<image_data>")]
fn validate_image(
    ci: rocket::State<ClientInterface>,
    tc: rocket::State<TrowConfig>,
    image_data: Json<AdmissionReview>,
) -> Json<AdmissionReview> {
    /*
     * The return type is a little complicated. Always return a 200 including for disallowed images. The JSON is an
     * AdmissionReview object with an AdmissionResponse entry. The object sent to this endpoint can be reused, or
     * a new created with the same UID.
     *
     * The docs on this stuff is a bit lacking, it's easiest to refer to the Go code in kubernetes/api.
     */
    let mut resp_data = image_data.clone();
    match image_data.0.request {
        Some(req) => {
            let r = ci.validate_admission(&req, &tc.host_names);
            match Runtime::new().unwrap().block_on(r) {
                Ok(res) => {
                    resp_data.response = Some(res);
                    Json(resp_data)
                }
                Err(e) => {
                    resp_data.response = Some(AdmissionResponse {
                        uid: req.uid.clone(),
                        allowed: false,
                        status: Some(Status {
                            status: "Failure".to_owned(),
                            message: Some(format!("Internal Error {:?}", e)),
                            code: None,
                        }),
                    });
                    Json(resp_data)
                }
            }
        }

        None => {
            resp_data.response = Some(AdmissionResponse {
                uid: "UNKNOWN".to_string(),
                allowed: false,
                status: Some(Status {
                    status: "Failure".to_owned(),
                    message: Some("No request found in review object".to_owned()),
                    code: None,
                }),
            });

            Json(resp_data)
        }
    }
}
