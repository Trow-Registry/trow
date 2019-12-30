use tonic::{Request, Response, Status};
use tokio::sync::mpsc;
use uuid::Uuid;
use failure::{self, Error};
use std::sync::{Arc, RwLock};
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use crate::manifest::{FromJson, Manifest};
use std::fs;

use crypto::digest::Digest;
use crypto::sha2::Sha256;

pub mod trow_server {
    include!("../../protobuf/out/trow.rs");
}

use trow_server::{
    registry_server::Registry,
    UploadRequest, UploadDetails, CatalogEntry, CatalogRequest, Tag, BlobRef, 
    WriteLocation, DownloadRef, BlobReadLocation, ManifestRef, ManifestReadLocation,
    VerifiedManifest, CompleteRequest, CompletedUpload
};


static MANIFESTS_DIR: &'static str = "manifests";
static LAYERS_DIR: &'static str = "layers";
static SCRATCH_DIR: &'static str = "scratch";

/* Struct implementing callbacks for the Frontend
 *
 * _active_uploads_: a HashSet of all uuids that are currently being tracked
 * _manifests_path_: path to where the manifests are
 * _layers_path_: path to where blobs are stored
 * _scratch_path_: path to temporary storage for uploads
 *
 * Each "route" gets a clone of this struct.
 * The Arc makes sure they all point to the same data.
 */
#[derive(Clone)]
pub struct TrowServer {
    active_uploads: Arc<RwLock<HashSet<Upload>>>,
    manifests_path: PathBuf,
    layers_path: PathBuf,
    scratch_path: PathBuf,
    allow_prefixes: Vec<String>,
    allow_images: Vec<String>,
    deny_local_prefixes: Vec<String>,
    deny_local_images: Vec<String>,
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
struct Upload {
    repo_name: String,
    uuid: String,
}

fn create_path(data_path: &str, dir: &str) -> Result<PathBuf, Error> {
    let data_path = Path::new(data_path);
    let dir_path = data_path.join(dir);
    if !dir_path.exists() {
        fs::create_dir_all(&dir_path)?;
    }
    Ok(dir_path)
}


fn gen_digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.input(bytes);
    format!("sha256:{}", hasher.result_str())
}

impl TrowServer {
        pub fn new(
            data_path: &str,
            allow_prefixes: Vec<String>,
            allow_images: Vec<String>,
            deny_local_prefixes: Vec<String>,
            deny_local_images: Vec<String>,
        ) -> Result<Self, Error> {
            let manifests_path = create_path(data_path, MANIFESTS_DIR)?;
            let scratch_path = create_path(data_path, SCRATCH_DIR)?;
            let layers_path = create_path(data_path, LAYERS_DIR)?;
            let svc = TrowServer {
                active_uploads: Arc::new(RwLock::new(HashSet::new())),
                manifests_path,
                layers_path,
                scratch_path,
                allow_prefixes,
                allow_images,
                deny_local_prefixes,
                deny_local_images,
            };
            Ok(svc)
        }


    fn get_path_for_blob_upload(&self, uuid: &str) -> PathBuf {
        self.scratch_path.join(uuid)
    }

    fn get_path_for_layer(&self, repo_name: &str, digest: &str) -> PathBuf {
        self.layers_path.join(repo_name).join(digest)
    }

    fn get_path_for_manifest(&self, repo_name: &str, reference: &str) -> PathBuf {
        self.manifests_path.join(repo_name).join(reference)
    }

    fn get_scratch_path_for_uuid(&self, uuid: &str) -> PathBuf {
        self.scratch_path.join(uuid)
    }

    fn create_verified_manifest(
        &self,
        repo_name: String,
        reference: String,
        do_verification: bool,
    ) -> Result<VerifiedManifest, Error> {
        let manifest_path = self.get_path_for_manifest(&repo_name, &reference);

        let manifest_bytes = std::fs::read(&manifest_path)?;
        let manifest_json: serde_json::Value = serde_json::from_slice(&manifest_bytes)?;
        let manifest = Manifest::from_json(&manifest_json)?;

        if do_verification {
            //TODO: Need to make sure we find things indexed by digest or tag
            for digest in manifest.get_asset_digests() {
                let path = self.get_path_for_layer(&repo_name, &digest);
                info!("Path: {:?}", path);

                if !path.exists() {
                    return Err(format_err!("Failed to find {} in {}", digest, repo_name));
                }
            }

            // TODO: check signature and names are correct on v1 manifests
        }

        //For performance, could generate only if verification is on, otherwise copy from somewhere
        Ok(VerifiedManifest {
            digest: gen_digest(&manifest_bytes),
            content_type: manifest.get_media_type().to_string()
        })
    }

    fn create_manifest_read_location(
        &self,
        repo_name: String,
        reference: String,
        do_verification: bool,
    ) -> Result<ManifestReadLocation, Error> {
        //TODO: This isn't optimal
        let path = self.get_path_for_manifest(&repo_name, &reference);
        let vm = self.create_verified_manifest(repo_name, reference, do_verification)?;
        Ok( ManifestReadLocation {
            content_type: vm.content_type.to_owned(),
            digest: vm.digest.to_owned(),
            path: path.to_string_lossy().to_string()
        })
    }

    fn save_layer(&self, repo_name: &str, user_digest: &str, uuid: &str) -> Result<(), Error> {
        debug!("Saving layer {}", user_digest);

        //TODO: This is wrong; user digest needs to be verified and potentially changed to our own digest
        //if we want to use consistent compression alg

        let digest_path = self.get_path_for_layer(repo_name, user_digest);
        let repo_path = digest_path
            .parent()
            .ok_or_else(|| failure::err_msg("Error finding repository path"))?;

        if !repo_path.exists() {
            fs::create_dir_all(repo_path)?;
        }

        let scratch_path = self.get_scratch_path_for_uuid(uuid);
        fs::copy(&scratch_path, &digest_path)?;

        //Not an error, even if it's not great
        fs::remove_file(&scratch_path).unwrap_or_else(|e| {
            warn!(
                "Error deleting file {} {:?}",
                &scratch_path.to_string_lossy(),
                e
            )
        });

        Ok(())
    }
}

#[tonic::async_trait]
impl Registry for TrowServer {

    async fn request_upload(
        &self,
        request: Request<UploadRequest>,
    ) -> Result<Response<UploadDetails>, Status> {
        
        let uuid = Uuid::new_v4().to_string();
        let reply = UploadDetails{uuid: uuid.clone()};
        let upload = Upload {
            repo_name: request.into_inner().repo_name.to_owned(),
            uuid,
        };
        {
            self.active_uploads.write().unwrap().insert(upload);
            debug!("Hash Table: {:?}", self.active_uploads);
        }

        Ok(Response::new(reply))
    }

    async fn get_write_location_for_blob(
        &self,
        req: Request<BlobRef>,
    ) -> Result<Response<WriteLocation>, Status> {

        let br = req.into_inner();
        let upload = Upload {
            repo_name: br.repo_name.clone(),
            uuid: br.uuid.clone(),
        };

        // Apparently unwrap() is correct here. From the docs:
        // "We unwrap() the return value to assert that we are not expecting
        // threads to ever fail while holding the lock."

        let set = self.active_uploads.read().unwrap();
        if set.contains(&upload) {
            let path = self.get_path_for_blob_upload(&br.uuid);
            Ok(Response::new(
                WriteLocation {path: path.to_string_lossy().to_string()}))

        } else {
            Err(Status::failed_precondition(format!("No current upload matching {:?}", br)))
        }
    }

    async fn get_read_location_for_blob(
        &self,
        req: Request<DownloadRef>
    ) -> Result<Response<BlobReadLocation>, Status> {
        //TODO: test that it exists

        let dr = req.into_inner();
        let path = self.get_path_for_layer(&dr.repo_name, &dr.digest);

        if !path.exists() {
            warn!("Request for unknown blob: {:?}", path);
            Err(Status::failed_precondition(format!("No blob found matching {:?}", dr)))

        } else {
            Ok(Response::new( BlobReadLocation {path: path.to_string_lossy().to_string() }))
        }
    }

    async fn get_write_location_for_manifest(
        &self,
        req: Request<ManifestRef>
    ) -> Result<Response<WriteLocation>, Status> {
        //TODO: First save to temporary file and copy over after verify

        let mr = req.into_inner();
        let manifest_path = self.get_path_for_manifest(&mr.repo_name, &mr.reference);
        let manifest_dir = manifest_path.parent().unwrap();

        match fs::create_dir_all(manifest_dir) {
            Ok(_) => {
                Ok(Response::new(
                    WriteLocation {path: manifest_path.to_string_lossy().to_string()}))
            }
            Err(e) => {
                warn!("Internal error creating directory {:?}", e);
                Err(Status::internal("Failed to create directory for manifest"))
            }
        }
    }

    async fn get_read_location_for_manifest(
        &self,
        req: Request<ManifestRef>
    ) -> Result<Response<ManifestReadLocation>, Status> {
        //Don't actually need to verify here; could set to false

        let mr = req.into_inner();
        // TODO refactor to return directly
        match self.create_manifest_read_location(mr.repo_name, mr.reference, true) {
            Ok(vm) => Ok(Response::new(vm)),
            Err(e) => {
                warn!("Internal error with manifest {:?}", e);
                Err(Status::internal("Internal error finding manifest"))
            }
        }
    }

    async fn complete_upload(
        &self,
        req: Request<CompleteRequest>
    ) -> Result<Response<CompletedUpload>, Status> {

        let cr = req.into_inner();
        let ret = match self.save_layer(&cr.repo_name, &cr.user_digest, &cr.uuid) {
            Ok(_) => {
                Ok(Response::new(CompletedUpload { digest: cr.user_digest.clone() }))
            }
            Err(e) => {
                warn!("Failure when saving layer: {:?}", e);
                Err(Status::internal("Internal error saving layer"))
            }
        };

        //delete uuid from uploads tracking
        let upload = Upload {
            repo_name: cr.repo_name.clone(),
            uuid: cr.uuid.clone(),
        };

        let mut set = self.active_uploads.write().unwrap();
        if !set.remove(&upload) {
            warn!("Upload {:?} not found when deleting", upload);
        }
        ret
    }

    async fn verify_manifest(
        &self,
        req: Request<ManifestRef>
    ) -> Result<Response<VerifiedManifest>, Status> {

        let mr = req.into_inner();
        match self.create_verified_manifest(
            mr.repo_name.clone(),
            mr.reference.clone(),
            true,
        ) {
            Ok(vm) => Ok(Response::new(vm)),
            Err(e) => {
                warn!("Error verifying manifest {:?}", e);
                Err(Status::internal("Internal error verifying manifest"))
            }
        }
    }

    type GetCatalogStream = mpsc::Receiver<Result<CatalogEntry, Status>>;

    async fn get_catalog(
        &self,
        _request: Request<CatalogRequest>,
    ) -> Result<Response<Self::GetCatalogStream>, Status> {
        warn!("get catalog unimplemented");
        unimplemented!()
    }

    type ListTagsStream = mpsc::Receiver<Result<Tag, Status>>;


    async fn list_tags(
        &self,
        _request: Request<CatalogEntry>,
    ) -> Result<Response<Self::ListTagsStream>, Status> {
        warn!("list tags unimplemented");
        unimplemented!()
    }
}
    