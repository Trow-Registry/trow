use std;
use std::sync::{Arc, Mutex};

use failure::Error;
use futures::Future;
use grpcio::{self, RpcStatus, RpcStatusCode};
use manifest::{FromJson, Manifest};
use serde_json;
use std::fs;
use std::path::Path;
use trow_protobuf;
use trow_protobuf::server::*;
use uuid::Uuid;

use crypto::digest::Digest;
use crypto::sha2::Sha256;

static DATA_DIR: &'static str = "data";
static MANIFESTS_DIR: &'static str = "manifests";
static LAYERS_DIR: &'static str = "layers";

/*
 * TODO: figure out what needs to be stored in the backend
 * and what it's keyed on
 * probably need a path
 *
 * remember will probably want to split out metadata for search
 *
 * Accepted Upload is borked atm
 */
/// Struct implementing callbacks for the Frontend
///
/// _uploads_: a HashSet of all uuids that are currently being tracked
#[derive(Clone)]
pub struct BackendService {
    uploads: Arc<Mutex<std::collections::HashSet<Layer>>>,
}

impl BackendService {
    pub fn new() -> Self {
        BackendService {
            uploads: Arc::new(Mutex::new(std::collections::HashSet::new())),
        }
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
struct Layer {
    repo_name: String,
    digest: String,
}

//TODO: fix
fn get_path_for_uuid(uuid: &str) -> String {
    format!("data/scratch/{}", uuid)
}

fn gen_digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.input(bytes);
    format!("sha256:{}", hasher.result_str())
}

fn save_layer(repo_name: &str, user_digest: &str, uuid: &str) -> Result<(), Error> {
    debug!("Saving layer {}", user_digest);

    //TODO: This is wrong; user digest needs to be verified and potentially changed to our own digest
    //if we want to use consistent compression alg
    let digest_path = format!("data/layers/{}/{}", repo_name, user_digest);
    let path = format!("data/layers/{}", repo_name);
    let scratch_path = format!("data/scratch/{}", uuid);

    if !Path::new(&path).exists() {
        fs::create_dir_all(path)?;
    }

    fs::copy(&scratch_path, digest_path)?;

    //Not an error, even if it's not great
    fs::remove_file(&scratch_path)
        .unwrap_or_else(|e| warn!("Error deleting file {} {:?}", &scratch_path, e));

    Ok(())
}

impl trow_protobuf::server_grpc::Backend for BackendService {
    fn get_write_location_for_blob(
        &self,
        ctx: grpcio::RpcContext,
        blob_ref: BlobRef,
        sink: grpcio::UnarySink<WriteLocation>,
    ) {
        let set = self.uploads.lock().unwrap();
        //LAYER MUST DIE!
        let layer = Layer {
            repo_name: blob_ref.get_repo_name().to_owned(),
            digest: blob_ref.get_uuid().to_owned(),
        };

        if set.contains(&layer) {
            let path = get_path_for_uuid(blob_ref.get_uuid());
            let mut w = WriteLocation::new();
            w.set_path(path);
            let f = sink
                .success(w)
                .map_err(move |e| warn!("Failed sending to client {:?}", e));
            ctx.spawn(f);
        } else {
            let f = sink
                .fail(RpcStatus::new(
                    RpcStatusCode::Unknown,
                    Some("UUID Not Known".to_string()),
                )).map_err(|e| warn!("Received request for unknown UUID {:?}", e));
            ctx.spawn(f);
        }
    }

    fn get_read_location_for_blob(
        &self,
        ctx: grpcio::RpcContext,
        dr: DownloadRef,
        sink: grpcio::UnarySink<ReadLocation>,
    ) {
        //TODO: test that it exists

        let path = format!(
            "{}/{}/{}/{}",
            DATA_DIR,
            LAYERS_DIR,
            dr.get_repo_name(),
            dr.get_digest()
        );
        if !Path::new(&path).exists() {
            let f = sink
                .fail(RpcStatus::new(
                    RpcStatusCode::Unknown,
                    Some("Blob Not Known".to_string()),
                )).map_err(|e| warn!("Received request for unknown blob {:?}", e));
            ctx.spawn(f);
        } else {
            let mut r = ReadLocation::new();
            r.set_path(path);
            let f = sink
                .success(r)
                .map_err(move |e| warn!("Failed sending to client {:?}", e));
            ctx.spawn(f);
        }
    }

    fn get_write_location_for_manifest(
        &self,
        ctx: grpcio::RpcContext,
        mr: ManifestRef,
        sink: grpcio::UnarySink<super::server::WriteLocation>,
    ) {
        //TODO: First save to temporary file and copy over after verify
        let manifest_directory = format!("{}/{}/{}/", DATA_DIR, MANIFESTS_DIR, mr.get_repo_name());
        let manifest_path = format!("{}/{}", manifest_directory, mr.get_reference());

        match fs::create_dir_all(manifest_directory) {
            Ok(_) => {
                let mut w = WriteLocation::new();
                w.set_path(manifest_path);
                let f = sink
                    .success(w)
                    .map_err(move |e| warn!("Failed sending to client {:?}", e));
                ctx.spawn(f);
            }
            Err(e) => {
                warn!("Internal error creating directory {:?}", e);
                let f = sink
                    .fail(RpcStatus::new(
                        RpcStatusCode::Internal,
                        Some("Internal error creating directory".to_string()),
                    )).map_err(|e| warn!("Failed to send error message to client {:?}", e));
                ctx.spawn(f);
            }
        }
    }

    fn get_read_location_for_manifest(
        &self,
        ctx: grpcio::RpcContext,
        mr: ManifestRef,
        sink: grpcio::UnarySink<VerifiedManifest>,
    ) {
        let manifest_directory = format!("{}/{}/{}/", DATA_DIR, MANIFESTS_DIR, mr.get_repo_name());
        let manifest_path = format!("{}/{}", manifest_directory, mr.get_reference());

        let manifest_bytes = std::fs::read(&manifest_path).unwrap();
        let manifest_json: serde_json::Value = serde_json::from_slice(&manifest_bytes).unwrap();
        let manifest = Manifest::from_json(&manifest_json).unwrap();

        let mut vm = VerifiedManifest::new();
        vm.set_location(manifest_path);
        vm.set_digest(gen_digest(&manifest_bytes));
        vm.set_content_type(manifest.get_media_type().to_string());
        let f = sink
            .success(vm)
            .map_err(move |e| warn!("Failed sending to client {:?}", e));
        ctx.spawn(f);
    }

    fn request_upload(
        &self,
        ctx: grpcio::RpcContext,
        req: UploadRequest,
        sink: grpcio::UnarySink<UploadDetails>,
    ) {
        let mut resp = UploadDetails::new();
        let layer = Layer {
            repo_name: req.get_repo_name().to_owned(),
            //WTF?!
            digest: Uuid::new_v4().to_string(),
        };
        {
            self.uploads.lock().unwrap().insert(layer.clone());
            debug!("Hash Table: {:?}", self.uploads);
        }
        resp.set_uuid(layer.digest.to_owned());
        let f = sink
            .success(resp)
            .map_err(|e| warn!("failed to reply! {:?}", e));
        ctx.spawn(f);
    }

    fn complete_upload(
        &self,
        ctx: grpcio::RpcContext,
        cr: CompleteRequest,
        sink: grpcio::UnarySink<CompletedUpload>,
    ) {
        match save_layer(cr.get_repo_name(), cr.get_user_digest(), cr.get_uuid()) {
            Ok(_) => {
                let mut cu = CompletedUpload::new();
                cu.set_digest(cr.get_user_digest().to_string());
                let f = sink
                    .success(cu)
                    .map_err(move |e| warn!("failed to reply! {:?}", e));
                ctx.spawn(f);
            }
            Err(_) => {
                let f = sink
                    .fail(RpcStatus::new(
                        RpcStatusCode::Internal,
                        Some("Internal error saving file".to_string()),
                    )).map_err(|e| warn!("Internal error saving file {:?}", e));
                ctx.spawn(f);
            }
        }

        //delete uuid from uploads tracking
        let layer = Layer {
            repo_name: cr.get_repo_name().to_string(),
            digest: cr.get_user_digest().to_string(),
        };

        let mut set = self.uploads.lock().unwrap();
        set.remove(&layer);
    }

    fn verify_manifest(
        &self,
        ctx: grpcio::RpcContext,
        mr: ManifestRef,
        sink: grpcio::UnarySink<VerifiedManifest>,
    ) {
        // TODO: wouldn't shadowing be better here?
        let manifest_directory = format!("{}/{}/{}/", DATA_DIR, MANIFESTS_DIR, mr.get_repo_name());
        let manifest_path = format!("{}/{}", manifest_directory, mr.get_reference());
        //let mut file = File::open(manifest_path).unwrap();
        let manifest_bytes = std::fs::read(manifest_path).unwrap();
        let manifest_json: serde_json::Value = serde_json::from_slice(&manifest_bytes).unwrap();
        let manifest = Manifest::from_json(&manifest_json).unwrap();

        //TODO: Need to make sure we find things indexed by digest or tag
        for digest in manifest.get_asset_digests() {
            let path = format!(
                "{}/{}/{}/{}",
                DATA_DIR,
                LAYERS_DIR,
                mr.get_repo_name(),
                digest
            );
            info!("Path: {}", path);
            let path = Path::new(&path);

            if !path.exists() {
                warn!("Layer does not exist in repo");
                //TODO: Error out here
            }
        }

        // TODO: check signature and names are correct on v1 manifests

        // save manifest file

        let digest = gen_digest(&manifest_bytes);
        let location = format!(
            "http://localhost:5000/v2/{}/manifests/{}",
            mr.get_repo_name(),
            digest
        );

        let mut mv = VerifiedManifest::new();
        mv.set_location(location);
        mv.set_digest(digest);
        mv.set_content_type(manifest.get_media_type().to_string());
        let f = sink
            .success(mv)
            .map_err(move |e| warn!("failed to reply! {:?}", e));
        ctx.spawn(f);
    }
}
