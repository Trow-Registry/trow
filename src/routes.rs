use std::fs;
use std::io::Write;
use std::path::Path;
use std::str;

use failure;
use types::create_upload_info;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use grpc;
use manifest::{self, FromJson, Manifest};
use response::accepted_upload::AcceptedUpload;
use response::empty::Empty;
use response::errors::Error;
use response::html::HTML;
use response::manifest_upload::ManifestUpload;
use response::upload_info::UploadInfo;
use rocket::request::{self, FromRequest, Request};
use rocket::response::NamedFile;
use rocket::{self, Outcome};
use serde_json;
use state;
use types::Layer;
use client_interface::ClientInterface;

static DATA_DIR: &'static str = "data";
static MANIFESTS_DIR: &'static str = "manifests";
static LAYERS_DIR: &'static str = "layers";

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_v2root,
        get_homepage,
        get_manifest,
        get_manifest_2level,
        get_manifest_3level,
        get_blob,
        get_blob_qualified,
        put_blob_qualified_3level,
        get_blob_qualified_3level,
        patch_blob_qualified_3level,
        put_blob,
        put_blob_qualified,
        patch_blob,
        patch_blob_qualified,
        post_blob_upload,
        post_blob_upload_3level,
        post_blob_upload_onename,
        put_image_manifest,
        put_image_manifest_qualified,
        put_image_manifest_qualified_3level,
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
fn get_manifest(onename: String, reference: String) -> Option<Manifest> {
    let path = format!("{}/{}/{}/{}", DATA_DIR, MANIFESTS_DIR, onename, reference);
    info!("Path: {}", path);
    let path = Path::new(&path);

    //Parse the manifest to get the response type
    //We could do this faster by storing in appropriate folder and streaming file
    //directly
    if path.exists() {
        return match fs::File::open(path) {
            Ok(f) => serde_json::from_reader(f).ok(),
            Err(_) => None,
        };
    }

    None
}

#[get("/v2/<user>/<repo>/manifests/<reference>")]
fn get_manifest_2level(user: String, repo: String, reference: String) -> Option<Manifest> {
    let path = format!(
        "{}/{}/{}/{}/{}",
        DATA_DIR, MANIFESTS_DIR, user, repo, reference
    );
    info!("Path: {}", path);
    let path = Path::new(&path);

    //Parse the manifest to get the response type
    //We could do this faster by storing in appropriate folder and streaming file
    //directly
    if path.exists() {
        return match fs::File::open(path) {
            Ok(f) => serde_json::from_reader(f).ok(),
            Err(_) => None,
        };
    }

    None
}

/*
 * Process 3 level manifest path - not sure this one is needed
 */
#[get("/v2/<org>/<user>/<repo>/manifests/<reference>")]
fn get_manifest_3level(
    org: String,
    user: String,
    repo: String,
    reference: String,
) -> Option<Manifest> {
    let path = format!(
        "{}/{}/{}/{}/{}/{}",
        DATA_DIR, MANIFESTS_DIR, org, user, repo, reference
    );
    info!("Path: {}", path);
    let path = Path::new(&path);

    //Parse the manifest to get the response type
    //We could do this faster by storing in appropriate folder and streaming file
    //directly
    if path.exists() {
        return match fs::File::open(path) {
            Ok(f) => serde_json::from_reader(f).ok(),
            Err(_) => None,
        };
    }

    None
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
fn get_blob(name_repo: String, digest: String, _auth_user: AuthorisedUser) -> Option<NamedFile> {
    let path = format!("{}/{}/{}/{}", DATA_DIR, LAYERS_DIR, name_repo, digest);
    info!("Path: {}", path);
    let path = Path::new(&path);

    if path.exists() {
        NamedFile::open(path).ok()
    } else {
        None
    }
}
/*
 * Parse 2 level <repo>/<name> style path and pass it to get_blob
 */

#[get("/v2/<name>/<repo>/blobs/<digest>")]
fn get_blob_qualified(
    name: String,
    repo: String,
    digest: String,
    auth_user: AuthorisedUser,
) -> Option<NamedFile> {
    get_blob(format!("{}/{}", name, repo), digest, auth_user)
}

/*
 * Parse 3 level <org>/<repo>/<name> style path and pass it to get_blob
 */
#[get("/v2/<org>/<name>/<repo>/blobs/<digest>")]
fn get_blob_qualified_3level(
    org: String,
    name: String,
    repo: String,
    digest: String,
    auth_user: AuthorisedUser,
) -> Option<NamedFile> {
    get_blob(format!("{}/{}/{}", org, name, repo), digest, auth_user)
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

#[derive_FromForm]
struct UploadQuery {
    _query: bool,
    digest: String,
}

#[put("/v2/<repo_name>/blobs/uploads/<uuid>?<query>")]
fn put_blob(
    config: rocket::State<ClientInterface>,
    repo_name: String,
    uuid: String,
    query: UploadQuery,
) -> Result<AcceptedUpload, Error> {
    match AcceptedUpload::handle(config, repo_name, uuid, query.digest) {
        Ok(x) => Ok(x),
        Err(_) => Err(Error::InternalError),
    }
}

/*
 * Parse 2 level <repo>/<name> style path and pass it to put_blob
 */
#[put("/v2/<repo>/<name>/blobs/uploads/<uuid>?<query>")]
fn put_blob_qualified(
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
fn put_blob_qualified_3level(
    config: rocket::State<ClientInterface>,
    org: String,
    repo: String,
    name: String,
    uuid: String,
    query: UploadQuery,
) -> Result<AcceptedUpload, Error> {
    put_blob(config, format!("{}/{}/{}", org, repo, name), uuid, query)
}

//TODO: Move this shite.
fn uuid_exists(handler: rocket::State<ClientInterface>, layer: &Layer) -> Result<bool, failure::Error> {
    let backend = handler.backend();
    let mut req = grpc::backend::Layer::new();
    req.set_repo_name(layer.repo_name.to_owned());
    req.set_digest(layer.digest.to_owned());

    let response = backend.uuid_exists(&req)?;
    debug!("UuidExists: {:?}", response.get_success());
    if response.get_success() {
        Ok(true)
    } else {
        Err(Error::DigestInvalid.into())
    }
}

/*

Uploads a blob or chunk of a blog.

Checks UUID. Returns UploadInfo with range set to correct position.

*/
#[patch("/v2/<repo_name>/blobs/uploads/<uuid>", data = "<chunk>")]
fn patch_blob(
    handler: rocket::State<ClientInterface>,
    repo_name: String,
    uuid: String,
    chunk: rocket::data::Data,
) -> Result<UploadInfo, Error> {
    //This needs to change to be a blob, no digest, or just go away
    //There is no digest at the minute; that comes at put stage
    let layer = Layer {
        repo_name: repo_name.clone(),
        digest: uuid.clone(),
    };
    //TODO change to is_valid_uuid()
    //Should return path to write to or URL, client should *not*
    //be in charge of this
    if uuid_exists(handler, &layer).is_ok() {
        let absolute_file = state::uuid::scratch_path(&uuid);
        debug!("Streaming out to {}", absolute_file);
        let len = chunk.stream_to_file(absolute_file);

        match len {
            Ok(len) => Ok(create_upload_info(
                uuid,
                repo_name,
                (0, len as u32),
            )),
            Err(_) => Err(Error::InternalError),
        }
    } else {
        // TODO: pipe breaks if we don't accept the whole file
        // Currently makes us prone to DOS attack
        warn!("Uuid {} does not exist, piping to /dev/null", uuid);
        let _ = chunk.stream_to_file("/dev/null");
        Err(Error::BlobUnknown)
    }
}

/*
 * Parse 2 level <repo>/<name> style path and pass it to patch_blob
 */
#[patch("/v2/<repo>/<name>/blobs/uploads/<uuid>", data = "<chunk>")]
fn patch_blob_qualified(
    handler: rocket::State<ClientInterface>,
    repo: String,
    name: String,
    uuid: String,
    chunk: rocket::data::Data,
) -> Result<UploadInfo, Error> {
    patch_blob(handler, format!("{}/{}", repo, name), uuid, chunk)
}

/*
 * Parse 3 level <org>/<repo>/<name> style path and pass it to patch_blob
 */
#[patch(
    "/v2/<org>/<repo>/<name>/blobs/uploads/<uuid>",
    data = "<chunk>"
)]
fn patch_blob_qualified_3level(
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
fn post_blob_upload_onename(
    handler: rocket::State<ClientInterface>,
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
    let backend = handler.backend();
    let mut req = grpc::backend::CreateUuidRequest::new();
    req.set_repo_name(repo_name.clone());

    let response = backend.create_uuid(&req).map_err(|e| {
        //TODO should be stronger than a warn
        warn!("Error getting ref from backend: {}", e);
        Error::InternalError
    })?;
    debug!("Client received: {:?}", response);

    Ok(create_upload_info(
        response.get_uuid().to_owned(),
        repo_name,
        (0, 0),
    ))
}

/*
 * Parse 2 level <repo>/<name> style path and pass it to put_blob_upload_onename
 */
#[post("/v2/<repo>/<name>/blobs/uploads")]
fn post_blob_upload(
    handler: rocket::State<ClientInterface>,
    repo: String,
    name: String,
) -> Result<UploadInfo, Error> {
    info!("upload {}/{}", repo, name);
    post_blob_upload_onename(handler, format!("{}/{}", repo, name))
}
/*
 * Parse 3 level <org>/<repo>/<name> style path and pass it to put_blob_upload_onename
 */
#[post("/v2/<org>/<repo>/<name>/blobs/uploads")]
fn post_blob_upload_3level(
    handler: rocket::State<ClientInterface>,
    org: String,
    repo: String,
    name: String,
) -> Result<UploadInfo, Error> {
    info!("upload 3 way {}/{}/{}", org, repo, name);
    post_blob_upload_onename(handler, format!("{}/{}/{}", org, repo, name))
}

/*

---
Pushing an image manifest
PUT /v2/<name>/manifests/<reference>
Content-Type: <manifest media type>

 */
#[put("/v2/<repo_name>/manifests/<reference>", data = "<chunk>")]
fn put_image_manifest(
    repo_name: String,
    reference: String,
    chunk: rocket::data::Data,
) -> Result<ManifestUpload, Error> {
    let mut manifest_bytes = Vec::new();
    //TODO From this point on, should stream to backend
    //Note that back end will need to have manifest, user, repo, ref
    //and possibly some sort of auth token
    //Needs to return digest & location or error
    //Just do this synchronous, let grpc deal with timeouts
    chunk.stream_to(&mut manifest_bytes).unwrap();
    // TODO: wouldn't shadowing be better here?
    let raw_manifest = str::from_utf8(&manifest_bytes).unwrap();
    let manifest_json: serde_json::Value = serde_json::from_str(raw_manifest).unwrap();
    let manifest = match manifest::Manifest::from_json(&manifest_json) {
        Ok(x) => x,
        Err(_) => return Err(Error::ManifestInvalid),
    };

    for digest in manifest.get_asset_digests() {
        let path = format!("{}/{}/{}/{}", DATA_DIR, LAYERS_DIR, repo_name, digest);
        info!("Path: {}", path);
        let path = Path::new(&path);

        if !path.exists() {
            warn!("Layer does not exist in repo");
            return Err(Error::ManifestInvalid);
        }
    }

    // TODO: check signature and names are correct on v1 manifests

    // save manifest file

    let manifest_directory = format!("{}/{}/{}/", DATA_DIR, MANIFESTS_DIR, repo_name);
    let manifest_path = format!("{}/{}", manifest_directory, reference);
    fs::create_dir_all(manifest_directory).unwrap();
    let mut file = fs::File::create(manifest_path).unwrap();
    file.write_all(raw_manifest.as_bytes()).unwrap();

    let digest = gen_digest(raw_manifest.as_bytes());
    let location = format!(
        "http://localhost:5000/v2/{}/manifests/{}",
        repo_name, digest
    );

    Ok(ManifestUpload { digest, location })
}

/*
 * Parse 2 level <user>/<repo> style path and pass it to put_image_manifest
 */
#[put("/v2/<user>/<repo>/manifests/<reference>", data = "<chunk>")]
fn put_image_manifest_qualified(
    user: String,
    repo: String,
    reference: String,
    chunk: rocket::data::Data,
) -> Result<ManifestUpload, Error> {
    put_image_manifest(format!("{}/{}", user, repo), reference, chunk)
}

/*
 * Parse 3 level <org>/<user>/<repo> style path and pass it to put_image_manifest
 */
#[put(
    "/v2/<org>/<user>/<repo>/manifests/<reference>",
    data = "<chunk>"
)]
fn put_image_manifest_qualified_3level(
    org: String,
    user: String,
    repo: String,
    reference: String,
    chunk: rocket::data::Data,
) -> Result<ManifestUpload, Error> {
    put_image_manifest(format!("{}/{}/{}", org, user, repo), reference, chunk)
}
fn gen_digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.input(bytes);
    format!("sha256:{}", hasher.result_str())
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
