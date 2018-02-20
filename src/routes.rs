use std::path::{Path, PathBuf};
use std::str;

use response::errors::Error;
use config;
use response::admin::Admin;
use response::empty::Empty;
use response::uuid::UuidResponse;
use response::manifest_upload::ManifestUpload;
use response::uuidaccept::UuidAcceptResponse;
use response::catalog::Catalog;
use response::html::HTML;
use rocket::request::{self, FromRequest, Request};
use rocket::{self, Outcome};
use rocket::http::Status;
use rocket::response::NamedFile;
use serde_json;
use crypto::sha2::Sha256;
use crypto::digest::Digest;
use manifest::{self, FromJson};
use state;
use types::Layer;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_v2root,
        get_homepage,
        get_manifest,
        check_image_manifest,
        get_blob,
        post_blob_uuid,
        get_upload_progress,
        put_blob,
        patch_blob,
        delete_upload,
        post_blob_upload,
        delete_blob,
        put_image_manifest,
        get_catalog,
        get_image_tags,
        delete_image_manifest,
        // admin routes
        admin_get_uuids,
    ]
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
            return Outcome::Success(Blob { file: path.to_path_buf() });
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

struct Manifest {
    file: PathBuf,
}

impl<'a, 'r> FromRequest<'a, 'r> for Manifest {
    type Error = ();
    fn from_request(req: &'a Request<'r>) -> request::Outcome<Manifest, ()> {
        //Look up catalogue to see if we have it
        let reference = req.get_param::<String>(2).unwrap();

        if reference == "test_manifest" {
            return Outcome::Success(Manifest { file: PathBuf::from("./README.md") });
        }
        Outcome::Failure((Status::NotFound, ()))
    }
}

/// Routes of a 2.0 Registry
///
/// Version Check of the registry
/// GET /v2/
///
/// # Responses
/// 200 - We Exist (and you are authenticated)
/// 401 - Please Authorize (WWW-Authenticate header with instuctions).
///
/// # Headers
/// Docker-Distribution-API-Version: registry/2.0
#[get("/v2")]
fn get_v2root() -> Empty {
    Empty
}

#[get("/")]
fn get_homepage<'a>() -> HTML<'a> {
    const ROOT_RESPONSE: &'static str = "<!DOCTYPE html><html><body>
<h1>Welcome to Lycaon, the King of Registries</h1>
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
#[get("/v2/<_name>/<_repo>/manifests/<_reference>")]
fn get_manifest(
    _name: String,
    _repo: String,
    _reference: String,
    manifest: Manifest,
) -> Option<NamedFile> {
    info!("Getting Manifest");
    NamedFile::open(manifest.file).ok()
}
/*

---
Check for existence
HEAD /v2/<name>/manifests/<reference>

# Parameters
name - The name of the image
reference - either a tag or a digest

# Headers
Content-Length: size of manifest
?Docker-Content-Digest: digest of manifest file

# Returns
200 - manifest exists
404 - manifest does not exist

NOTE: this may not be needed due to autogen of HEAD methods.
 */
#[head("/v2/<_name>/<_repo>/manifests/<_reference>")]
fn check_image_manifest(_name: String, _repo: String, _reference: String) -> Empty {
    Empty
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

/// Pushing a Layer
/// POST /v2/<name>/blobs/uploads/
/// name - name of repository
///
/// # Headers
/// Location: /v2/<name>/blobs/uploads/<uuid>
/// Range: bytes=0-<offset>
/// Content-Length: 0
/// Docker-Upload-UUID: <uuid>
///
/// # Returns
/// 202 - accepted
#[post("/v2/<_name>/<_repo>/blobs/uploads/<_uuid>")]
fn post_blob_uuid(_name: String, _repo: String, _uuid: String) -> Empty {
    Empty
}

/*
---
Upload Progress
GET /v2/<name>/blobs/uploads/<uuid>
name - name of registry
uuid - unique id for the upload that is to be checked

# Client Headers
Host: <registry host>

# Headers
Location: /v2/<name>/blobs/uploads/<uuid>
Range: bytes=0-<offset>
Docker-Upload-UUID: <uuid>

# Returns
204
 */
#[get("/v2/<_name>/<_repo>/blobs/uploads/<_uuid>")]
fn get_upload_progress(_name: String, _repo: String, _uuid: String) -> Empty {
    Empty
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
    query: bool,
    digest: String
}

#[put("/v2/<name>/<repo>/blobs/uploads/<uuid>?<query>")] // capture digest query string
fn put_blob(
    config: rocket::State<config::BackendHandler>,
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

#[patch("/v2/<name>/<repo>/blobs/uploads/<uuid>", data = "<chunk>")]
fn patch_blob(
    handler: rocket::State<config::BackendHandler>,
    name: String,
    repo: String,
    uuid: String,
    chunk: rocket::data::Data,
) -> UuidResponse {
    let layer = Layer {
        name: name.clone(),
        repo: repo.clone(),
        digest: uuid.clone(),
    };
    if let Ok(_) = UuidResponse::uuid_exists(handler, &layer) {
        let absolute_file = state::uuid::scratch_path(&uuid);
        debug!("Streaming out to {}", absolute_file);
        let len = chunk.stream_to_file(absolute_file);

        match len {
            Ok(len) => {
                UuidResponse::Uuid {
                    uuid,
                    name,
                    repo,
                    range: (0, len as u32),
                }
            }
            Err(_) => UuidResponse::Empty,
        }

    } else {
        // TODO: pipe breaks if we don't accept the whole file...
        // AM - shouldn't this return a 4xx? IllegalArgument or something?
        warn!("Uuid {} does not exist, piping to /dev/null", uuid);
        let _ = chunk.stream_to_file("/dev/null");
        UuidResponse::Empty
    }
}

/*


---
Cancelling an upload
DELETE /v2/<name>/blobs/uploads/<uuid>

 */

/// This route assumes that no more data will be uploaded to the specified uuid.
#[delete("/v2/<name>/<repo>/blobs/uploads/<uuid>")]
fn delete_upload(
    handler: rocket::State<config::BackendHandler>,
    name: String,
    repo: String,
    uuid: String,
) -> Result<UuidAcceptResponse, Error> {
    UuidAcceptResponse::delete_upload(handler, &Layer::new(name, repo, uuid))
}

/*
---
Cross repo blob mounting (validate how regularly this is used)
POST /v2/<name>/blobs/uploads/?mount=<digest>&from=<repository name>

 */

#[post("/v2/<name>/<repo>/blobs/uploads")]
fn post_blob_upload(
    handler: rocket::State<config::BackendHandler>,
    name: String,
    repo: String,
) -> UuidResponse {
    UuidResponse::handle(handler, name, repo)
        .map_err(|e| {
            warn!("Uuid Generate: {}", e);
        })
        .unwrap_or(UuidResponse::Empty)
}

/*

---
Delete a layer
DELETE /v2/<name>/blobs/<digest>

 */
#[delete("/v2/<_name>/<_repo>/blobs/<_digest>")]
fn delete_blob(_name: String, _repo: String, _digest: String) -> Empty {
    Empty
}

/*

---
Pushing an image manifest
PUT /v2/<name>/manifests/<reference>
Content-Type: <manifest media type>

 */
#[put("/v2/<name>/<repo>/manifests/<reference>", data = "<chunk>")]
fn put_image_manifest(
    name: String,
    repo: String,
    reference: String,
    chunk: rocket::data::Data,
) -> Result<ManifestUpload, Error> {
    /*
     * 0. accept manifest file. (done)
     * 1. Check signature is valid (skip for now)
     * 2. Verify name/repo:tag match with manifest
     * 3. Verify all layers exist
     * 4. Save Manifest file
     *
     * - do size check before accepting file
     */

    let mut manifest_bytes = Vec::new();
    chunk.stream_to(&mut manifest_bytes).unwrap();
    let raw_manifest = str::from_utf8(&manifest_bytes).unwrap();
    let manifest_json: serde_json::Value = serde_json::from_str(&raw_manifest).unwrap();
    let manifest = match manifest::Manifest::from_json(&manifest_json) {
        Ok(x) => x,
        Err(_) => return Err(Error::ManifestInvalid)
    };

    /*
    if let manifest::Manifest::v2(_) = manifest {
        return Err(Error::Unsupported);
    }
    */

    let manifest_v1 = match manifest {
         manifest::Manifest::V2(_) => return Err(Error::Unsupported),
         manifest::Manifest::V1(m1) => m1 
    };

    // TODO: check signature is valid

    debug!("verifying name/repo");
    // Verify name/repo:tag match with manifest
    if format!("{}/{}", name, repo) != manifest_v1.name || manifest_v1.tag != reference {
        warn!("name and repo don't match!");
        return Err(Error::Unsupported)
    }

    debug!("verifying layers");
    // Verify all layers exist
    for layer in manifest_v1.fs_layers.into_iter() {
        // Same code as respond_to for Blob struct
        let path = format!("data/layers/{}/{}/{}", name, repo, layer.blob_sum);
        let path = Path::new(&path);

        if !path.exists() {
            warn!("Layer does not exist in repo");
            return Err(Error::Unsupported);
        }
    }

    let mut hasher = Sha256::new();
    hasher.input(&raw_manifest.as_bytes());
    let digest = format!("sha256:{}", hasher.result_str());
    let location = format!("http://localhost:5000/v2/alpine/manifests/{}", digest);

    // save manifest file
    use std::fs;
    use std::io::Write;
    let manifest_directory = format!("./data/manifests/{}/{}", name, repo);
    let manifest_path = format!("{}/{}", manifest_directory, reference);
    fs::create_dir_all(manifest_directory).unwrap();
    let mut file = fs::File::create(manifest_path).unwrap();
    file.write_all(&raw_manifest.as_bytes()).unwrap();

    Ok(ManifestUpload{digest, location})
}

/*
---
Listing Repositories
GET /v2/_catalog

 */
#[get("/v2/_catalog")]
fn get_catalog() -> Catalog {
    Catalog
}
/*
---
Listing Image Tags
GET /v2/<name>/tags/list

 */
#[get("/v2/<_name>/<_repo>/tags/list")]
fn get_image_tags(_name: String, _repo: String) -> Result<Empty, Error> {
    Err(Error::Unsupported)
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

#[get("/admin/uuids")]
fn admin_get_uuids(handler: rocket::State<config::BackendHandler>) -> Admin {
    Admin::get_uuids(handler).unwrap_or(Admin::Uuids(vec![]))
}

/*
---
[1]: Could possibly be used to redirect a client to a local cache
 */
