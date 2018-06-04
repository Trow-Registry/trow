use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str;

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use manifest::{self, FromJson, Manifest};
use response::empty::Empty;
use response::errors::Error;
use response::html::HTML;
use response::manifest_upload::ManifestUpload;
use response::upload_info::{self, UploadInfo};
use response::uuidaccept::UuidAcceptResponse;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::response::NamedFile;
use rocket::{self, Outcome};
use serde_json;
use state;
use types::Layer;
use backend;
use grpc;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_v2root,
        get_homepage,
        get_manifest,
        get_blob,
        put_blob,
        patch_blob,
        post_blob_upload,
        put_image_manifest,
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

struct Blob {
    file: PathBuf,
}
impl<'a, 'r> FromRequest<'a, 'r> for Blob {
    type Error = ();
    fn from_request(req: &'a Request<'r>) -> request::Outcome<Blob, ()> {
        //Look up catalogue to see if we have it
        let name = req.get_param::<String>(0).unwrap();
        let repo = req.get_param::<String>(1).unwrap();
        let digest = req.get_param::<String>(2).unwrap();
        let path = format!("data/layers/{}/{}/{}", name, repo, digest);
        info!("Path: {}", path);
        let path = Path::new(&path);

        if path.exists() {
            return Outcome::Success(Blob {
                file: path.to_path_buf(),
            });
        }
        Outcome::Failure((Status::NotFound, ()))
    }
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
#[get("/v2/<user>/<repo>/manifests/<reference>")]
fn get_manifest(user: String, repo: String, reference: String) -> Option<Manifest> {
    let path = format!("data/manifests/{}/{}/{}", user, repo, reference);
    info!("Path: {}", path);
    let path = Path::new(&path);

    //Parse the manifest to get the response type
    //We could do this faster by storing in appropriate folder and streaming file
    //directly
    if path.exists() {
        let file = fs::File::open(path).unwrap();
        let m: Manifest = serde_json::from_reader(file).unwrap();
        return Some(m);
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
#[get("/v2/<_name>/<_repo>/blobs/<_digest>")]
fn get_blob(
    _name: String,
    _repo: String,
    _digest: String,
    _auth_user: AuthorisedUser,
    blob: Blob,
) -> Option<NamedFile> {
    /*
     * I suspect calling the guards directly would be better.
     * We generally don't need to work through a call chain
     * (e.g. admin user -> authorised user -> anon user methods)
     * and can either error/run happy path.
     */
    info!("Getting Blob");
    NamedFile::open(blob.file).ok()
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

#[put("/v2/<name>/<repo>/blobs/uploads/<uuid>?<query>")] // capture digest query string
fn put_blob(
    config: rocket::State<backend::BackendHandler>,
    name: String,
    repo: String,
    uuid: String,
    query: UploadQuery,
) -> Result<UuidAcceptResponse, Error> {
    match UuidAcceptResponse::handle(config, name, repo, uuid, query.digest) {
        Ok(x) => Ok(x),
        Err(_) => Err(Error::InternalError),
    }
}

/*

Uploads a blob or chunk of a blog.

Checks UUID. Returns UploadInfo with range set to correct position.

*/
#[patch("/v2/<name>/<repo>/blobs/uploads/<uuid>", data = "<chunk>")]
fn patch_blob(
    handler: rocket::State<backend::BackendHandler>,
    name: String,
    repo: String,
    uuid: String,
    chunk: rocket::data::Data,
) -> Result<UploadInfo, Error> {

    //This needs to change to be a blob, no digest, or just go away
    //There is no digest at the minute; that comes at put stage
    let layer = Layer {
        name: name.clone(),
        repo: repo.clone(),
        digest: uuid.clone(),
    };
    //TODO change to is_valid_uuid()
    //Should return path to write to or URL, client should *not*
    //be in charge of this
    if UploadInfo::uuid_exists(handler, &layer).is_ok() {
        let absolute_file = state::uuid::scratch_path(&uuid);
        debug!("Streaming out to {}", absolute_file);
        let len = chunk.stream_to_file(absolute_file);

        match len {
            Ok(len) => Ok(upload_info::create_upload_info(
                uuid,
                name,
                repo,
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
  Starting point for an uploading a new image or new version of an image.

  We respond with details of location and UUID to upload to with patch/put.

  No data is being transferred yet.
 */
#[post("/v2/<name>/<repo>/blobs/uploads")]
fn post_blob_upload(
    handler: rocket::State<backend::BackendHandler>,
    name: String,
    repo: String,
) -> Result<UploadInfo, Error> {

    //Ask the backend for a UUID
    let backend = handler.backend();
    let mut req = grpc::backend::CreateUuidRequest::new();
    req.set_repo_name(format!("{}/{}", name, repo));

    let response = backend
        .create_uuid(&req)
        .map_err(|e| {
            //TODO should be stronger than a warn
            warn!("Error getting ref from backend: {}", e);
            Error::InternalError})?;
    debug!("Client received: {:?}", response);

    Ok(upload_info::create_upload_info(
        response.get_uuid().to_owned(),
        name,
        repo,
        (0, 0),
    ))
}

/*

---
Pushing an image manifest
PUT /v2/<name>/manifests/<reference>
Content-Type: <manifest media type>

 */
#[put("/v2/<user>/<repo>/manifests/<reference>", data = "<chunk>")]
fn put_image_manifest(
    user: String,
    repo: String,
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
        let path = format!("data/layers/{}/{}/{}", user, repo, digest);
        let path = Path::new(&path);

        if !path.exists() {
            warn!("Layer does not exist in repo");
            return Err(Error::ManifestInvalid);
        }
    }

    // TODO: check signature and names are correct on v1 manifests

    // save manifest file

    let manifest_directory = format!("./data/manifests/{}/{}", user, repo);
    let manifest_path = format!("{}/{}", manifest_directory, reference);
    fs::create_dir_all(manifest_directory).unwrap();
    let mut file = fs::File::create(manifest_path).unwrap();
    file.write_all(raw_manifest.as_bytes()).unwrap();

    let digest = gen_digest(raw_manifest.as_bytes());
    let location = format!(
        "http://localhost:5000/v2/{}/{}/manifests/{}",
        user, repo, digest
    );

    Ok(ManifestUpload { digest, location })
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
