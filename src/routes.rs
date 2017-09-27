use std::io::Read;
use std::fs::File;
use std::string::ToString;
use rocket;
use uuid::Uuid;
use ring::digest;

use errors;
use response::MaybeResponse;
use response::empty::Empty;
use response::uuid::UuidResponse;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_v2root,
        get_manifest,
        check_image_manifest,
        get_blob,
        post_blob_uuid,
        check_existing_layer,
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
    ]
}

pub fn errors() -> Vec<rocket::Catcher> {
    errors![
        err_400,
        err_404,
        ]
        
}

#[error(400)]
fn err_400() -> MaybeResponse<Empty> {
    MaybeResponse::err(Empty)
}

#[error(404)]
fn err_404() -> MaybeResponse<Empty> {
    MaybeResponse::err(Empty)
}

/**
Routes of a 2.0 Registry

Version Check of the registry
GET /v2/

# Responses
200 - We Exist (and you are authenticated)
401 - Please Authorize (WWW-Authenticate header with instuctions).

# Headers
Docker-Distribution-API-Version: registry/2.0
*/

/// Some docs for this function
#[get("/v2")]
fn get_v2root() -> MaybeResponse<Empty> {
    MaybeResponse::ok(Empty)
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
#[get("/v2/<_name>/<_repo>/manifests/<reference>")]
fn get_manifest(
    _name: String,
    _repo: String,
    reference: String,
) -> MaybeResponse<Empty> {
    info!("Getting Manifest");
    match reference.as_str() {
        "good" => MaybeResponse::ok(Empty),
        _ => MaybeResponse::err(Empty),
    }
}
/*

---
Check for existance
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
 */
#[head("/v2/<_name>/<_repo>/manifests/<_reference>")]
fn check_image_manifest(_name: String, _repo: String, _reference: String) ->
    MaybeResponse<Empty> {
        MaybeResponse::err(Empty)
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
#[get("/v2/<_name>/<_repo>/blobs/<digest>")]
fn get_blob(_name: String, _repo: String, digest: String) -> MaybeResponse<Empty> {
    info!("Getting Blob");
    match digest.as_str() {
        "good" => MaybeResponse::ok(Empty),
        _ => MaybeResponse::err(Empty),
    }
}

/**

---
Pushing a Layer
POST /v2/<name>/blobs/uploads/
name - name of repository

# Headers
Location: /v2/<name>/blobs/uploads/<uuid>
Range: bytes=0-<offset>
Content-Length: 0
Docker-Upload-UUID: <uuid>

# Returns
202 - accepted
*/
#[post("/v2/<_name>/<_repo>/blobs/uploads/<_uuid>")]
fn post_blob_uuid(_name: String, _repo: String, _uuid: String) ->
    MaybeResponse<Empty> {
        MaybeResponse::err(Empty)
}

/*
---
Check for existing layer
HEAD /v2/<name>/blobs/<digest>
name - name of repository
digest - digest of blob to be checked

# Headers
Content-Length: <length of blob>
Docker-Content-Digest: <digest>

# Returns
200 - exists
404 - does not exist
 */
#[head("/v2/<name>/<repo>/blobs/<_digest>")]
fn check_existing_layer(name: String, repo: String, _digest: String) ->
    MaybeResponse<Empty> {
        debug!("Checking if {}/{} exists...", name, repo);
        MaybeResponse::err(Empty)
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
fn get_upload_progress(_name: String, _repo: String, _uuid: String) ->
    MaybeResponse<Empty> {
        MaybeResponse::err(Empty)
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

// TODO move this somewhere else
#[derive_FromForm]
#[derive(Debug)]
struct DigestStruct {
    query: bool,
    digest: String,
}

// TODO change this to return a type-safe thing rather than just 'String'
fn scratch_path(uuid: &String) -> String {
    format!("data/scratch/{}", uuid)
}

// TODO change this to return a type-safe thing rather than just 'String'
// TODO move this somewhere not in the routes directory...
fn hash_file(absolute_directory: String) -> Result<String, String> {
    debug!("Hashing file: {}", absolute_directory);
    match File::open(&absolute_directory) {
        Ok(mut file) => {
            let mut vec_file = &mut Vec::new();
            let _ = file.read_to_end(&mut vec_file);
            let sha = digest::digest(&digest::SHA256, &vec_file);

            // HACK: needs a fix of some description
            Ok(format!("{:?}", sha).to_lowercase())
        }
        Err(_) => Err(format!("could not open file: {}", absolute_directory))
    }
}

#[put("/v2/<_name>/<_repo>/blobs/uploads/<uuid>?<digest>")] // capture digest query string
fn put_blob(_name: String, _repo: String, uuid: String, digest: DigestStruct) ->
    MaybeResponse<Empty> {
        debug!("Completing layer upload with digest: {}", digest.digest);
        let hash = match hash_file(scratch_path(&uuid)) {
            Ok(v) => v,
            Err(_) => "".to_string(),
        };
        debug!("File Hash: {}", hash);

        match assert_eq!(hash, digest.digest) {
            () => MaybeResponse::err(Empty)
        };


        // hash uuid from scratch, if success, copy over to layers
        // UuidAccept
        match digest.digest.eq(&hash) {
            true => MaybeResponse::err(Empty),
            false  => MaybeResponse::err(Empty),
        }
}

#[patch("/v2/<name>/<repo>/blobs/uploads/<uuid>", data="<chunk>")]
fn patch_blob(name: String, repo: String, uuid: String, chunk: rocket::data::Data) ->
    MaybeResponse<UuidResponse> {
        let absolute_file = scratch_path(&uuid);
        debug!("Streaming out to {}", absolute_file);
        let file = chunk.stream_to_file(absolute_file);

        match file {
            Ok(_) => {
                let right = match file.map(|x| x.to_string()) {
                    Ok(x) => x.parse::<u32>().unwrap(),
                    Err(_) => 0,
                };
                MaybeResponse::ok(UuidResponse::Uuid {uuid, name, repo, left: 0, right})
            },
            Err(_) => MaybeResponse::err(UuidResponse::Empty)
        }
}

/*


---
Cancelling an upload
DELETE /v2/<name>/blobs/uploads/<uuid>

 */

#[delete("/v2/<_name>/<_repo>/blobs/uploads/<_uuid>")]
fn delete_upload(_name: String, _repo: String, _uuid: String) ->
    MaybeResponse<Empty> {
        MaybeResponse::err(Empty)
}
/*
---
Cross repo blob mounting (validate how regularly this is used)
POST /v2/<name>/blobs/uploads/?mount=<digest>&from=<repository name>

 */

#[post("/v2/<name>/<repo>/blobs/uploads")]
fn post_blob_upload(name: String, repo: String) ->
    MaybeResponse<UuidResponse> {
        let uuid = Uuid::new_v4();
        info!("Using Uuid: {:?}", uuid);
        MaybeResponse::ok(UuidResponse::Uuid {
            uuid: uuid.to_string(),
            name,
            repo,
            left: 0,
            right: 0,
        })
}
/*

---
Delete a layer
DELETE /v2/<name>/blobs/<digest>

*/
#[delete("/v2/<_name>/<_repo>/blobs/<_digest>")]
fn delete_blob(_name: String, _repo: String, _digest: String) ->
    MaybeResponse<Empty> {
        MaybeResponse::err(Empty)
}
/*

---
Pushing an image manifest
PUT /v2/<name>/manifests/<reference>
Content-Type: <manifest media type>

*/
#[put("/v2/<_name>/<_repo>/manifests/<_reference>")]
fn put_image_manifest(_name: String, _repo: String, _reference: String) ->
    MaybeResponse<Empty> {
        MaybeResponse::err(Empty)
}
/*
---
Listing Repositories
GET /v2/_catalog

*/
#[get("/v2/_catalog")]
fn get_catalog() ->
    MaybeResponse<Empty> {
        MaybeResponse::err(Empty)
}
/*
---
Listing Image Tags
GET /v2/<name>/tags/list

*/
#[delete("/v2/<_name>/<_repo>/tags/list")]
fn get_image_tags(_name: String, _repo: String) ->
    MaybeResponse<Empty> {
        MaybeResponse::err(Empty)
}
/*
---
Deleting an Image
DELETE /v2/<name>/manifests/<reference>

*/
#[delete("/v2/<_name>/<_repo>/manifests/<_reference>")]
fn delete_image_manifest(_name: String, _repo: String, _reference: String) ->
    MaybeResponse<Empty> {
        let _errors = errors::generate_errors(&[errors::ErrorType::UNSUPPORTED]);
        MaybeResponse::err(Empty)
}

/*
---
[1]: Could possibly be used to redirect a client to a local cache
*/
