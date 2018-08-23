use std::str;

use client_interface::ClientInterface;
use response::empty::Empty;
use response::errors::Error;
use response::html::HTML;
use response::upload_info::UploadInfo;
use rocket::request::{self, FromRequest, Request};
use rocket::{self, Outcome};
use types::{
    create_upload_info, AcceptedUpload, BlobReader, Digest,
    ManifestReader, RepoName, Uuid, VerifiedManifest,
};

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_v2root,
        get_homepage,
        get_manifest,
        get_manifest_2level,
        get_manifest_3level,
        get_blob,
        get_blob_2level,
        get_blob_3level,
        put_blob,
        put_blob_2level,
        put_blob_3level,
        patch_blob,
        patch_blob_2level,
        patch_blob_3level,
        post_blob_upload,
        post_blob_upload_2level,
        post_blob_upload_3level,
        put_image_manifest,
        put_image_manifest_2level,
        put_image_manifest_3level,
        delete_image_manifest,
    ]
    /* The following routes used to have stub methods, but I removed them as they were cluttering the code
          post_blob_uuid,
          get_upload_progress,
          delete_upload,
          delete_blob,
          get_catalog,
          get_image_tags,
          admin routes,
          admin_get_uuids

    To find the stubs, go to https://github.com/ContainerSolutions/trow/tree/4b007088bb0657a98238870d9aaca638e01f6487
    Please add tests for any routes that you recover.
    */
}

struct AuthorisedUser(String);
impl<'a, 'r> FromRequest<'a, 'r> for AuthorisedUser {
    type Error = ();
    fn from_request(_req: &'a Request<'r>) -> request::Outcome<AuthorisedUser, ()> {
        Outcome::Success(AuthorisedUser("test".to_owned()))
    }
}
/*
Registry root.

Returns 200.
*/
#[get("/v2")]
fn get_v2root() -> Empty {
    Empty
}

#[get("/")]
fn get_homepage<'a>() -> HTML<'a> {
    const ROOT_RESPONSE: &str = "<!DOCTYPE html><html><body>
<h1>Welcome to Trow, the cluster registry</h1>
</body></html>";

    HTML(ROOT_RESPONSE)
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
    ci: rocket::State<ClientInterface>,
    onename: String,
    reference: String,
) -> Option<ManifestReader> {
    //Need to handle error
    ci.get_reader_for_manifest(&RepoName(onename), &reference)
        .ok()
}

#[get("/v2/<user>/<repo>/manifests/<reference>")]
fn get_manifest_2level(
    ci: rocket::State<ClientInterface>,
    user: String,
    repo: String,
    reference: String,
) -> Option<ManifestReader> {
    ci.get_reader_for_manifest(&RepoName(format!("{}/{}", user, repo)), &reference)
        .ok()
}

/*
 * Process 3 level manifest path - not sure this one is needed
 */
#[get("/v2/<org>/<user>/<repo>/manifests/<reference>")]
fn get_manifest_3level(
    ci: rocket::State<ClientInterface>,
    org: String,
    user: String,
    repo: String,
    reference: String,
) -> Option<ManifestReader> {
    ci.get_reader_for_manifest(&RepoName(format!("{}/{}/{}", org, user, repo)), &reference)
        .ok()
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
    ci: rocket::State<ClientInterface>,
    name_repo: String,
    digest: String,
    _auth_user: AuthorisedUser,
) -> Option<BlobReader> {
    ci.get_reader_for_blob(&RepoName(name_repo), &Digest(digest))
        .ok()
}
/*
 * Parse 2 level <repo>/<name> style path and pass it to get_blob
 */

#[get("/v2/<name>/<repo>/blobs/<digest>")]
fn get_blob_2level(
    ci: rocket::State<ClientInterface>,
    name: String,
    repo: String,
    digest: String,
    auth_user: AuthorisedUser,
) -> Option<BlobReader> {
    get_blob(ci, format!("{}/{}", name, repo), digest, auth_user)
}

/*
 * Parse 3 level <org>/<repo>/<name> style path and pass it to get_blob
 */
#[get("/v2/<org>/<name>/<repo>/blobs/<digest>")]
fn get_blob_3level(
    ci: rocket::State<ClientInterface>,
    org: String,
    name: String,
    repo: String,
    digest: String,
    auth_user: AuthorisedUser,
) -> Option<BlobReader> {
    get_blob(ci, format!("{}/{}/{}", org, name, repo), digest, auth_user)
}
/*
---
Monolithic Upload
PUT /v2/<name>/blobs/uploads/<uuid>?digest=<digest>
Content-Length: <size of layer>
Content-Type: application/octet-stream

<Layer Binary Data>
---
Chunked Upload (Don't implement until Monolithic works)
Must be implemented as docker only supports this
PATCH /v2/<name>/blobs/uploads/<uuid>
Content-Length: <size of chunk>
Content-Range: <start of range>-<end of range>
Content-Type: application/octet-stream

<Layer Chunk Binary Data>
 */

#[derive(FromForm)]
struct UploadQuery {
    _query: bool,
    digest: String,
}

/**
 * Completes the upload.
 *
 * TODO: allow uploading of final data
 * TODO: add other failure states
 */
#[put("/v2/<repo_name>/blobs/uploads/<uuid>?<query>")]
fn put_blob(
    ci: rocket::State<ClientInterface>,
    repo_name: String,
    uuid: String,
    query: UploadQuery,
) -> Result<AcceptedUpload, Error> {
    ci.complete_upload(&RepoName(repo_name), &Uuid(uuid), &Digest(query.digest))
        .map_err(|_| Error::InternalError)
}

/*
 * Parse 2 level <repo>/<name> style path and pass it to put_blob
 */
#[put("/v2/<repo>/<name>/blobs/uploads/<uuid>?<query>")]
fn put_blob_2level(
    config: rocket::State<ClientInterface>,
    repo: String,
    name: String,
    uuid: String,
    query: UploadQuery,
) -> Result<AcceptedUpload, Error> {
    put_blob(config, format!("{}/{}", repo, name), uuid, query)
}

/*
 * Parse 3 level <org>/<repo>/<name> style path and pass it to put_blob
 */
#[put("/v2/<org>/<repo>/<name>/blobs/uploads/<uuid>?<query>")]
fn put_blob_3level(
    config: rocket::State<ClientInterface>,
    org: String,
    repo: String,
    name: String,
    uuid: String,
    query: UploadQuery,
) -> Result<AcceptedUpload, Error> {
    put_blob(config, format!("{}/{}/{}", org, repo, name), uuid, query)
}

/*

Uploads a blob or chunk of a blog.

Checks UUID. Returns UploadInfo with range set to correct position.

*/
#[patch("/v2/<repo_name>/blobs/uploads/<uuid>", data = "<chunk>")]
fn patch_blob(
    ci: rocket::State<ClientInterface>,
    repo_name: String,
    uuid: String,
    chunk: rocket::data::Data,
) -> Result<UploadInfo, Error> {
    let repo = RepoName(repo_name);
    let uuid = Uuid(uuid);
    let sink = ci.get_write_sink_for_upload(&repo, &uuid);

    match sink {
        Ok(mut sink) => {
            //TODO: for the moment we'll just append, but this should seek to correct position
            //according to spec shouldn't allow out-of-order uploads, so verify start address (from header)
            //is same as current address
            let len = chunk.stream_to(&mut sink);
            match len {
                //TODO: For chunked upload this should be start pos to end pos
                Ok(len) => Ok(create_upload_info(uuid, repo, (0, len as u32))),
                Err(_) => Err(Error::InternalError),
            }
        }
        Err(_) => {
            // TODO: this conflates rpc errors with uuid not existing
            // TODO: pipe breaks if we don't accept the whole file
            // Possibly makes us prone to DOS attack?
            warn!("Uuid {} does not exist, piping to /dev/null", uuid);
            let _ = chunk.stream_to_file("/dev/null");
            Err(Error::BlobUnknown)
        }
    }
}

/*
 * Parse 2 level <repo>/<name> style path and pass it to patch_blob
 */
#[patch("/v2/<repo>/<name>/blobs/uploads/<uuid>", data = "<chunk>")]
fn patch_blob_2level(
    ci: rocket::State<ClientInterface>,
    repo: String,
    name: String,
    uuid: String,
    chunk: rocket::data::Data,
) -> Result<UploadInfo, Error> {
    patch_blob(ci, format!("{}/{}", repo, name), uuid, chunk)
}

/*
 * Parse 3 level <org>/<repo>/<name> style path and pass it to patch_blob
 */
#[patch(
    "/v2/<org>/<repo>/<name>/blobs/uploads/<uuid>",
    data = "<chunk>"
)]
fn patch_blob_3level(
    handler: rocket::State<ClientInterface>,
    org: String,
    repo: String,
    name: String,
    uuid: String,
    chunk: rocket::data::Data,
) -> Result<UploadInfo, Error> {
    patch_blob(handler, format!("{}/{}/{}", org, repo, name), uuid, chunk)
}
/*
  Starting point for an uploading a new image or new version of an image.

  We respond with details of location and UUID to upload to with patch/put.

  No data is being transferred yet.
 */
#[post("/v2/<repo_name>/blobs/uploads")]
fn post_blob_upload(
    ci: rocket::State<ClientInterface>,
    repo_name: String,
) -> Result<UploadInfo, Error> {
    /*
    Ask the backend for a UUID.

    We should also need to do some checking that the user is allowed
    to upload first.

    If using a true UUID it is possible for the frontend to generate
    and tell the backend what the UUID is. This is a potential
    optimisation, but is arguably less flexible.
    */
    ci.request_upload(&RepoName(repo_name)).map_err(|e| {
        warn!("Error getting ref from backend: {}", e);
        Error::InternalError
    })
}

/*
 * Parse 2 level <repo>/<name> style path and pass it to put_blob_upload_onename
 */
#[post("/v2/<repo>/<name>/blobs/uploads")]
fn post_blob_upload_2level(
    ci: rocket::State<ClientInterface>,
    repo: String,
    name: String,
) -> Result<UploadInfo, Error> {
    info!("upload {}/{}", repo, name);
    post_blob_upload(ci, format!("{}/{}", repo, name))
}

/*
 * Parse 3 level <org>/<repo>/<name> style path and pass it to put_blob_upload_onename
 */
#[post("/v2/<org>/<repo>/<name>/blobs/uploads")]
fn post_blob_upload_3level(
    ci: rocket::State<ClientInterface>,
    org: String,
    repo: String,
    name: String,
) -> Result<UploadInfo, Error> {
    info!("upload 3 way {}/{}/{}", org, repo, name);
    post_blob_upload(ci, format!("{}/{}/{}", org, repo, name))
}

/*

---
Pushing an image manifest
PUT /v2/<name>/manifests/<reference>
Content-Type: <manifest media type>

 */
#[put("/v2/<repo_name>/manifests/<reference>", data = "<chunk>")]
fn put_image_manifest(
    ci: rocket::State<ClientInterface>,
    repo_name: String,
    reference: String,
    chunk: rocket::data::Data,
) -> Result<VerifiedManifest, Error> {
    let repo = RepoName(repo_name);
    match ci
        .get_write_sink_for_manifest(&repo, &reference)
        .map(|mut sink| chunk.stream_to(&mut sink))
    {
        Ok(_) => {
            //This can probably be moved to responder
            match ci.verify_manifest(&repo, &reference) {
                Ok(vm) => Ok(vm),
                Err(_) => Err(Error::ManifestInvalid),
            }
        },
        Err(_) => Err(Error::InternalError),
    }
}

/*
 * Parse 2 level <user>/<repo> style path and pass it to put_image_manifest
 */
#[put("/v2/<user>/<repo>/manifests/<reference>", data = "<chunk>")]
fn put_image_manifest_2level(
    ci: rocket::State<ClientInterface>,
    user: String,
    repo: String,
    reference: String,
    chunk: rocket::data::Data,
) -> Result<VerifiedManifest, Error> {
    put_image_manifest(ci, format!("{}/{}", user, repo), reference, chunk)
}

/*
 * Parse 3 level <org>/<user>/<repo> style path and pass it to put_image_manifest
 */
#[put(
    "/v2/<org>/<user>/<repo>/manifests/<reference>",
    data = "<chunk>"
)]
fn put_image_manifest_3level(
    ci: rocket::State<ClientInterface>,
    org: String,
    user: String,
    repo: String,
    reference: String,
    chunk: rocket::data::Data,
) -> Result<VerifiedManifest, Error> {
    put_image_manifest(ci, format!("{}/{}/{}", org, user, repo), reference, chunk)
}

/*
---
Deleting an Image
DELETE /v2/<name>/manifests/<reference>
*/

#[delete("/v2/<_name>/<_repo>/manifests/<_reference>")]
fn delete_image_manifest(_name: String, _repo: String, _reference: String) -> Result<Empty, Error> {
    Err(Error::Unsupported)
}
