use std::string::ToString;
use rocket;

use config;
use errors;
use response::{MaybeResponse, RegistryResponse};
use response::empty::Empty;
use response::layers::LayerExists;
use response::uuid::UuidResponse;
use response::uuidaccept::UuidAcceptResponse;
use response::catalog::Catalog;
use response::html::HTML;

use controller::uuid as cuuid;

pub fn routes() -> Vec<rocket::Route> {
    routes![
        get_test_route,

        get_v2root,
        get_homepage,
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
    errors![err_400, err_404,]
}

#[error(400)]
fn err_400() -> MaybeResponse<Empty> {
    MaybeResponse::err(Empty)
}

#[error(404)]
fn err_404() -> MaybeResponse<Empty> {
    MaybeResponse::err(Empty)
}


#[get("/testing")]
fn get_test_route(config: rocket::State<config::Config>) -> MaybeResponse<Empty> {
    use capnp_rpc::{RpcSystem, twoparty, rpc_twoparty_capnp};
    use http_capnp::lycaon;

    use tokio_core::reactor;
    use tokio_io::AsyncRead;
    use futures::Future;

    use std::net::ToSocketAddrs;

    let address = format!("localhost:{}", config.console_port);
    let mut core = reactor::Core::new().unwrap();
    let handle = core.handle();

    let addr = address.to_socket_addrs().unwrap().next().expect(
        "could not parse address",
    );
    info!("Connecting to address: {}", address);
    if let Ok(stream) = core.run(::tokio_core::net::TcpStream::connect(&addr, &handle)) {
        stream.set_nodelay(true).unwrap();
        let (reader, writer) = stream.split();

        let rpc_network = Box::new(twoparty::VatNetwork::new(
            reader,
            writer,
            rpc_twoparty_capnp::Side::Client,
            Default::default(),
        ));

        let mut rpc_system = RpcSystem::new(rpc_network, None);
        let lycaon_proxy: lycaon::Client = rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);
        let interface = lycaon_proxy.get_message_interface_request().send();
        let proxy = interface.pipeline.get_if();


        handle.spawn(rpc_system.map_err(|_e| ()));

        let mut req = proxy.get_request();
        req.get().set_num(12);
        let session = req.send();
        let response = core.run(session.promise).unwrap();

        let response = response.get().unwrap();
        let msg = response.get_msg().unwrap();
        info!("Success!!");
        info!(
            "Response: (text = {:?}, number = {:?})",
            msg.get_text(),
            msg.get_number()
        );
    } else {
        warn!("Issue connecting to Console, please try again later");
    }
    MaybeResponse::err(Empty)
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
fn get_v2root() -> MaybeResponse<Empty> {
    MaybeResponse::ok(Empty)
}

#[get("/")]
fn get_homepage<'a>() -> RegistryResponse<HTML<'a>> {
    RegistryResponse(HTML(
        "<!DOCTYPE html><html><body>
    <h1>Welcome to Lycaon, the King of Registries</h1>
    </body></html>",
    ))
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
fn get_manifest(_name: String, _repo: String, reference: String) -> MaybeResponse<Empty> {
    info!("Getting Manifest");
    match reference.as_str() {
        "good" => MaybeResponse::ok(Empty),
        _ => MaybeResponse::err(Empty),
    }
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
 */
#[head("/v2/<_name>/<_repo>/manifests/<_reference>")]
fn check_image_manifest(_name: String, _repo: String, _reference: String) -> MaybeResponse<Empty> {
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
fn post_blob_uuid(_name: String, _repo: String, _uuid: String) -> MaybeResponse<Empty> {
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
#[head("/v2/<name>/<repo>/blobs/<digest>")]
fn check_existing_layer(
    config: rocket::State<config::Config>,
    name: String,
    repo: String,
    digest: String,
) -> MaybeResponse<LayerExists> {
    debug!("Checking if {}/{} exists...", name, repo);

    use capnp_rpc::{RpcSystem, twoparty, rpc_twoparty_capnp};
    use http_capnp::lycaon;

    use tokio_core::reactor;
    use tokio_io::AsyncRead;
    use futures::Future;

    use std::net::ToSocketAddrs;

    // TODO: this can /all/ be cleaned up considerably...
    let address = format!("localhost:{}", config.console_port);
    let mut core = reactor::Core::new().unwrap();
    let handle = core.handle();

    let addr = address.to_socket_addrs().and_then(|mut addr| {
        let err = Err("could not parse address".to_string());
        // The below piece of code is actually handled by using
        // `.or_ok()`, but it is not a solution until I can find a
        // proper error handler.
        match addr.next() {
            Some(x) => Ok(x),
            // TODO: This is a hack and will actually cause the code to panic when trying to unwrap.
            // A proper fix needs to be done for this, but it does make the type-checker happy...
            // This is a duplicate of some code in the state/mod.rs file.
            None => Err(err.unwrap()),
        }
    });

    info!("Connecting to address: {}", address);
    let stream = addr.and_then(|addr| {
        core.run(::tokio_core::net::TcpStream::connect(&addr, &handle))
    });

    if let Ok(stream) = stream {
        stream.set_nodelay(true).expect("could not set nodelay");
        let (reader, writer) = stream.split();

        let rpc_network = Box::new(twoparty::VatNetwork::new(
            reader,
            writer,
            rpc_twoparty_capnp::Side::Client,
            Default::default(),
        ));

        let mut rpc_system = RpcSystem::new(rpc_network, None);
        let lycaon_proxy: lycaon::Client = rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);
        let interface = lycaon_proxy.get_layer_interface_request().send();
        let proxy = interface.pipeline.get_if();


        handle.spawn(rpc_system.map_err(|_e| ()));

        let mut req = proxy.layer_exists_request();
        let mut message2 = ::capnp::message::Builder::new(::capnp::message::HeapAllocator::new());
        let mut msg = message2.init_root::<lycaon::layer::Builder>();
        msg.set_digest(&digest);
        msg.set_name(&name);
        msg.set_repo(&repo);
        req.get().set_layer(msg.as_reader()).expect(
            "could not set layer",
        );
        let session = req.send();
        match core.run(session.promise) {
            Ok(response) => {
                let response = response.get();
                let msg = response
                    .and_then(|response| {
                        response.get_result().and_then(|response| {
                            let exists = response.get_exists();
                            let length = response.get_length();
                            match exists {
                                true => Ok(Ok(LayerExists::True { digest, length })),
                                false => Ok(Err(LayerExists::False)),
                            }
                        })
                    })
                    .unwrap();

                match msg {
                    Ok(v) => {
                        info!("Path found!");
                        MaybeResponse::ok(v)
                    }
                    Err(e) => MaybeResponse::err(e),
                }
            }
            Err(_) => MaybeResponse::err(LayerExists::False),
        }
    } else {
        warn!("Issue connecting to Console, please try again later");
        MaybeResponse::err(LayerExists::False)
    }
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
fn get_upload_progress(_name: String, _repo: String, _uuid: String) -> MaybeResponse<Empty> {
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

#[put("/v2/<name>/<repo>/blobs/uploads/<uuid>?<digest>")] // capture digest query string
fn put_blob(
    name: String,
    repo: String,
    uuid: String,
    digest: cuuid::DigestStruct,
) -> MaybeResponse<UuidAcceptResponse> {
    debug!("Completing layer upload with digest: {}", digest.digest);
    let hash = match cuuid::hash_file(cuuid::scratch_path(&uuid)) {
        Ok(v) => v,
        Err(_) => "".to_string(),
    };
    debug!("File Hash: {}", hash);

    match assert_eq!(hash, digest.digest) {
        () => MaybeResponse::err(UuidAcceptResponse::DigestMismatch),
    };


    // hash uuid from scratch, if success, copy over to layers
    // UuidAccept
    match digest.digest.eq(&hash) {
        true => {
            let digest = digest.digest;
            // 1. copy file to layers (with new name)
            if let Ok(_) = cuuid::save_layer(&uuid, &digest) {
                // 2. delete old layer
                warn!("Deleting scratch file: {}", &uuid);
                if let Ok(_) = cuuid::mark_delete(&uuid) {
                    // 3. return success
                    MaybeResponse::err(UuidAcceptResponse::UuidAccept {
                        uuid,
                        digest,
                        name,
                        repo,
                    })
                } else {
                    panic!("file could not be deleted");
                }
            } else {
                panic!("file could not be copied");
            }
        }
        false => {
            warn!("expected {}, got {}", digest.digest, hash);
            MaybeResponse::err(UuidAcceptResponse::DigestMismatch)
        }
    }
}

#[patch("/v2/<name>/<repo>/blobs/uploads/<uuid>", data = "<chunk>")]
fn patch_blob(
    name: String,
    repo: String,
    uuid: String,
    chunk: rocket::data::Data,
) -> MaybeResponse<UuidResponse> {
    let absolute_file = cuuid::scratch_path(&uuid);
    debug!("Streaming out to {}", absolute_file);
    let file = chunk.stream_to_file(absolute_file);

    match file {
        Ok(_) => {
            let right = match file.map(|x| x.to_string()) {
                Ok(x) => x.parse::<u32>().unwrap(),
                Err(_) => 0,
            };
            MaybeResponse::ok(UuidResponse::Uuid {
                uuid,
                name,
                repo,
                left: 0,
                right,
            })
        }
        Err(_) => MaybeResponse::err(UuidResponse::Empty),
    }
}

/*


---
Cancelling an upload
DELETE /v2/<name>/blobs/uploads/<uuid>

 */

/// This route assumes that no more data will be uploaded to the specified uuid.
#[delete("/v2/<_name>/<_repo>/blobs/uploads/<uuid>")]
fn delete_upload(_name: String, _repo: String, uuid: String) -> MaybeResponse<UuidAcceptResponse> {
    match cuuid::mark_delete(&uuid) {
        Ok(_) => RegistryResponse(UuidAcceptResponse::UuidDelete),
        Err(_) => panic!("Figure out what to put here too..."),
    }
}
/*
---
Cross repo blob mounting (validate how regularly this is used)
POST /v2/<name>/blobs/uploads/?mount=<digest>&from=<repository name>

 */

#[post("/v2/<name>/<repo>/blobs/uploads")]
fn post_blob_upload(name: String, repo: String) -> MaybeResponse<UuidResponse> {
    let uuid = cuuid::gen_uuid();
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
fn delete_blob(_name: String, _repo: String, _digest: String) -> MaybeResponse<Empty> {
    MaybeResponse::err(Empty)
}
/*

---
Pushing an image manifest
PUT /v2/<name>/manifests/<reference>
Content-Type: <manifest media type>

*/
#[put("/v2/<_name>/<_repo>/manifests/<_reference>")]
fn put_image_manifest(_name: String, _repo: String, _reference: String) -> MaybeResponse<Empty> {
    MaybeResponse::err(Empty)
}
/*
---
Listing Repositories
GET /v2/_catalog

*/
#[get("/v2/_catalog")]
fn get_catalog() -> MaybeResponse<Catalog> {
    MaybeResponse::err(Catalog)
}
/*
---
Listing Image Tags
GET /v2/<name>/tags/list

*/
#[delete("/v2/<_name>/<_repo>/tags/list")]
fn get_image_tags(_name: String, _repo: String) -> MaybeResponse<Empty> {
    MaybeResponse::err(Empty)
}
/*
---
Deleting an Image
DELETE /v2/<name>/manifests/<reference>

*/
#[delete("/v2/<_name>/<_repo>/manifests/<_reference>")]
fn delete_image_manifest(_name: String, _repo: String, _reference: String) -> MaybeResponse<Empty> {
    let _errors = errors::generate_errors(&[errors::ErrorType::UNSUPPORTED]);
    MaybeResponse::err(Empty)
}

/*
---
[1]: Could possibly be used to redirect a client to a local cache
*/
