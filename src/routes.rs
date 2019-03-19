use std::str;

use client_interface::ClientInterface;
use response::authenticate::Authenticate;
use response::empty::Empty;
use response::errors::Error;
use response::html::HTML;
use response::trow_token::ValidBasicToken;
use response::trow_token::{self, TrowToken};
use response::upload_info::UploadInfo;
use rocket;
use rocket::request::Request;
use rocket::State;
use rocket_contrib::json::{Json,JsonValue};
use types::*;
use TrowConfig;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_v2root,
        get_homepage,
        login,
        get_manifest,
        get_manifest_2level,
        get_manifest_3level,
        put_image_manifest,
        put_image_manifest_2level,
        put_image_manifest_3level,
        delete_image_manifest,
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
        list_tags,
        list_tags_2level,
        list_tags_3level,
        get_catalog,
        validate_image
    ]
    /* The following routes used to have stub methods, but I removed them as they were cluttering the code
          post_blob_uuid,
          get_upload_progress,
          delete_upload,
          delete_blob,
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
    warn!("matched");
    ci.get_reader_for_manifest(&RepoName(onename), &reference)
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
    ci.get_reader_for_manifest(&RepoName(format!("{}/{}", user, repo)), &reference)
        .ok()
}

/*
 * Process 3 level manifest path - not sure this one is needed
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
    _auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    name_repo: String,
    digest: String,
) -> Option<BlobReader> {
    ci.get_reader_for_blob(&RepoName(name_repo), &Digest(digest))
        .ok()
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

/**
 * Completes the upload.
 *
 * TODO: allow uploading of final data
 * TODO: add other failure states
 */
#[put("/v2/<repo_name>/blobs/uploads/<uuid>?<digest>")]
fn put_blob(
    _auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    repo_name: String,
    uuid: String,
    digest: String,
) -> Result<AcceptedUpload, Error> {
    ci.complete_upload(&RepoName(repo_name), &Uuid(uuid), &Digest(digest))
        .map_err(|_| Error::InternalError)
}

/*
 * Parse 2 level <repo>/<name> style path and pass it to put_blob
 */
#[put("/v2/<repo>/<name>/blobs/uploads/<uuid>?<digest>")]
fn put_blob_2level(
    auth_user: TrowToken,
    config: rocket::State<ClientInterface>,
    repo: String,
    name: String,
    uuid: String,
    digest: String,
) -> Result<AcceptedUpload, Error> {
    put_blob(auth_user, config, format!("{}/{}", repo, name), uuid, digest)
}

/*
 * Parse 3 level <org>/<repo>/<name> style path and pass it to put_blob
 */
#[put("/v2/<org>/<repo>/<name>/blobs/uploads/<uuid>?<digest>")]
fn put_blob_3level(
    auth_user: TrowToken,
    config: rocket::State<ClientInterface>,
    org: String,
    repo: String,
    name: String,
    uuid: String,
    digest: String,
) -> Result<AcceptedUpload, Error> {
    put_blob(auth_user, config, format!("{}/{}/{}", org, repo, name), uuid, digest)
}

/*

Uploads a blob or chunk of a blog.

Checks UUID. Returns UploadInfo with range set to correct position.

*/
#[patch("/v2/<repo_name>/blobs/uploads/<uuid>", data = "<chunk>")]
fn patch_blob(
    _auth_user: TrowToken,
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
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    repo: String,
    name: String,
    uuid: String,
    chunk: rocket::data::Data,
) -> Result<UploadInfo, Error> {
    patch_blob(auth_user, ci, format!("{}/{}", repo, name), uuid, chunk)
}

/*
 * Parse 3 level <org>/<repo>/<name> style path and pass it to patch_blob
 */
#[patch("/v2/<org>/<repo>/<name>/blobs/uploads/<uuid>", data = "<chunk>")]
fn patch_blob_3level(
    auth_user: TrowToken,
    handler: rocket::State<ClientInterface>,
    org: String,
    repo: String,
    name: String,
    uuid: String,
    chunk: rocket::data::Data,
) -> Result<UploadInfo, Error> {
    patch_blob(auth_user, handler, format!("{}/{}/{}", org, repo, name), uuid, chunk)
}

/*
 Starting point for an uploading a new image or new version of an image.

 We respond with details of location and UUID to upload to with patch/put.

 No data is being transferred yet.
*/
#[post("/v2/<repo_name>/blobs/uploads")]
fn post_blob_upload(
    _auth_user: TrowToken,
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
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    repo: String,
    name: String,
) -> Result<UploadInfo, Error> {
    info!("upload {}/{}", repo, name);
    post_blob_upload(auth_user, ci, format!("{}/{}", repo, name))
}

/*
 * Parse 3 level <org>/<repo>/<name> style path and pass it to put_blob_upload_onename
 */
#[post("/v2/<org>/<repo>/<name>/blobs/uploads")]
fn post_blob_upload_3level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    org: String,
    repo: String,
    name: String,
) -> Result<UploadInfo, Error> {
    info!("upload 3 way {}/{}/{}", org, repo, name);
    post_blob_upload(auth_user, ci, format!("{}/{}/{}", org, repo, name))
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
        }
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
    put_image_manifest(auth_user, ci, format!("{}/{}", user, repo), reference, chunk)
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
    put_image_manifest(auth_user, ci, format!("{}/{}/{}", org, user, repo), reference, chunk)
}

/*
---
Deleting an Image
DELETE /v2/<name>/manifests/<reference>
*/

#[delete("/v2/<_name>/<_repo>/manifests/<_reference>")]
fn delete_image_manifest(_auth_user: TrowToken, _name: String, _repo: String, _reference: String) -> Result<Empty, Error> {
    Err(Error::Unsupported)
}

#[get("/v2/_catalog")]
fn get_catalog(_auth_user: TrowToken, ci: rocket::State<ClientInterface>) -> Result<RepoCatalog, Error> {
    match ci.get_catalog() {
        Ok(c) => Ok(c),
        Err(_) => Err(Error::InternalError),
    }
}

#[get("/v2/<repo_name>/tags/list")]
fn list_tags(_auth_user: TrowToken, ci: rocket::State<ClientInterface>, repo_name: String) -> Result<TagList, Error> {
    match ci.list_tags(&RepoName(repo_name)) {
        Ok(c) => Ok(c),
        Err(_) => Err(Error::InternalError),
    }
}

#[get("/v2/<user>/<repo>/tags/list")]
fn list_tags_2level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    user: String,
    repo: String,
) -> Result<TagList, Error> {
    list_tags(auth_user, ci, format!("{}/{}", user, repo))
}

#[get("/v2/<org>/<user>/<repo>/tags/list")]
fn list_tags_3level(
    auth_user: TrowToken,
    ci: rocket::State<ClientInterface>,
    org: String,
    user: String,
    repo: String,
) -> Result<TagList, Error> {
    list_tags(auth_user, ci, format!("{}/{}/{}", org, user, repo))
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
        Some(req) => match ci.validate_admission(&req, &tc.host_names) {
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
                        message: Some(format!("Internal Error {:?}", e).to_owned()),
                        code: None,
                    }),
                });
                Json(resp_data)
            }
        },

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
