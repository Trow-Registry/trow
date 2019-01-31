use std;
use std::sync::{Arc, RwLock};

use failure::{self, Error};
use futures::{stream, Future, Sink};
use grpcio::{self, RpcStatus, RpcStatusCode, WriteFlags};
use manifest::{FromJson, Manifest};
use serde_json;
use std::collections::HashSet;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use trow_protobuf;
use trow_protobuf::server::*;
use uuid::Uuid;

use crypto::digest::Digest;
use crypto::sha2::Sha256;

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
pub struct TrowService {
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

#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    pub host: String, //Including port, docker.io by default
    pub repo: String, //Between host and : including any /s
    pub tag: String,  //Bit after the :, latest by default
}

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}:{}", self.host, self.repo, self.tag)
    }
}

impl TrowService {
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
        let svc = TrowService {
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

    /**
     * TODO: needs to handle either tag or digest as reference.
     */
    fn get_path_for_manifest(&self, repo_name: &str, reference: &str) -> PathBuf {
        self.manifests_path.join(repo_name).join(reference)
    }

    fn get_path_for_layer(&self, repo_name: &str, digest: &str) -> PathBuf {
        self.layers_path.join(repo_name).join(digest)
    }

    fn get_scratch_path_for_uuid(&self, uuid: &str) -> PathBuf {
        self.scratch_path.join(uuid)
    }

    pub fn image_exists(&self, image: &Image) -> bool {
        self.get_path_for_manifest(&image.repo, &image.tag).exists()
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

        let mut vm = VerifiedManifest::new();

        //For performance, could generate only if verification is on, otherwise copy from somewhere
        vm.set_digest(gen_digest(&manifest_bytes));
        vm.set_content_type(manifest.get_media_type().to_string());
        Ok(vm)
    }

    fn create_manifest_read_location(
        &self,
        repo_name: String,
        reference: String,
        do_verification: bool,
    ) -> Result<ManifestReadLocation, Error> {
        //This isn't optimal
        let path = self.get_path_for_manifest(&repo_name, &reference);
        let vm = self.create_verified_manifest(repo_name, reference, do_verification)?;
        let mut mrl = ManifestReadLocation::new();
        mrl.set_content_type(vm.get_content_type().to_string());
        mrl.set_digest(vm.get_digest().to_string());
        mrl.set_path(path.to_string_lossy().to_string());
        Ok(mrl)
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

    pub fn is_local_denied(&self, image: &Image) -> bool {
        //Try matching both with and without host name
        //Deny images are expected without host as always local
        let full_name = format!("{}", image);
        let name_without_host = format!("{}:{}", image.repo, image.tag);

        for prefix in &self.deny_local_prefixes {
            if full_name.starts_with(prefix) || name_without_host.starts_with(prefix) {
                info!("Image {} matches prefix {} on deny list", image, prefix);
                return true;
            }
        }

        for name in &self.deny_local_images {
            if &full_name == name || &name_without_host == name {
                info!("Image {} matches image {} on deny list", image, name);
                return true;
            }
        }

        false
    }

    pub fn is_allowed(&self, image: &Image) -> bool {
        //Have full names with host here
        let name = format!("{}", image);

        for prefix in &self.allow_prefixes {
            if name.starts_with(prefix) {
                info!("Image {} matches prefix {} on allow list", name, prefix);
                return true;
            }
        }

        for a_name in &self.allow_images {
            if &name == a_name {
                info!("Image {} matches image {} on allow list", name, a_name);
                return true;
            }
        }

        false
    }
}

/**
 * Visits each subdir and adds path to set if there are files in the directory.
 *
 * Could be made more generic by taking a function argument.
 */
fn visit_dirs(dir: &Path, base: &Path, repos: &mut HashSet<String>) -> Result<(), Error> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, base, repos)?;
            } else if let Some(d) = path.parent() {
                    let repo = d.strip_prefix(base)?;
                    repos.insert(repo.to_string_lossy().to_string());
            }
        }
    }
    Ok(())
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

impl trow_protobuf::server_grpc::Registry for TrowService {
    fn get_write_location_for_blob(
        &self,
        ctx: grpcio::RpcContext,
        blob_ref: BlobRef,
        sink: grpcio::UnarySink<WriteLocation>,
    ) {
        let upload = Upload {
            repo_name: blob_ref.get_repo_name().to_owned(),
            uuid: blob_ref.get_uuid().to_owned(),
        };

        // Apparently unwrap() is correct here. From the docs:
        // "We unwrap() the return value to assert that we are not expecting
        // threads to ever fail while holding the lock."

        let set = self.active_uploads.read().unwrap();
        if set.contains(&upload) {
            let path = self.get_path_for_blob_upload(blob_ref.get_uuid());
            let mut w = WriteLocation::new();
            w.set_path(path.to_string_lossy().to_string());
            let f = sink
                .success(w)
                .map_err(move |e| warn!("Failed sending to client {:?}", e));
            ctx.spawn(f);
        } else {
            warn!("Request for write location for unknown upload");
            let f = sink
                .fail(RpcStatus::new(
                    RpcStatusCode::Unknown,
                    Some("UUID Not Known".to_string()),
                ))
                .map_err(|e| warn!("Failure sending error to client {:?}", e));
            ctx.spawn(f);
        }
    }

    fn get_read_location_for_blob(
        &self,
        ctx: grpcio::RpcContext,
        dr: DownloadRef,
        sink: grpcio::UnarySink<BlobReadLocation>,
    ) {
        //TODO: test that it exists

        let path = self.get_path_for_layer(dr.get_repo_name(), dr.get_digest());

        if !path.exists() {
            warn!("Request for unknown blob");
            let f = sink
                .fail(RpcStatus::new(
                    RpcStatusCode::Unknown,
                    Some("Blob Not Known".to_string()),
                ))
                .map_err(|e| warn!("Failure sending error to client {:?}", e));
            ctx.spawn(f);
        } else {
            let mut r = BlobReadLocation::new();
            r.set_path(path.to_string_lossy().to_string());
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

        let manifest_path = self.get_path_for_manifest(mr.get_repo_name(), mr.get_reference());
        let manifest_dir = manifest_path.parent().unwrap();

        match fs::create_dir_all(manifest_dir) {
            Ok(_) => {
                let mut w = WriteLocation::new();
                w.set_path(manifest_path.to_string_lossy().to_string());
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
                    ))
                    .map_err(|e| warn!("Failed to send error message to client {:?}", e));
                ctx.spawn(f);
            }
        }
    }

    fn get_read_location_for_manifest(
        &self,
        ctx: grpcio::RpcContext,
        mr: ManifestRef,
        sink: grpcio::UnarySink<ManifestReadLocation>,
    ) {
        //Don't actually need to verify here; could set to false

        match self.create_manifest_read_location(mr.repo_name, mr.reference, true) {
            Ok(vm) => {
                let f = sink
                    .success(vm)
                    .map_err(move |e| warn!("Failed sending to client {:?}", e));
                ctx.spawn(f);
            }
            Err(e) => {
                warn!("Internal error with manifest {:?}", e);
                let f = sink
                    .fail(RpcStatus::new(
                        RpcStatusCode::Internal,
                        Some("Internal error finding manifest".to_string()),
                    ))
                    .map_err(|e| warn!("Failed to send error message to client {:?}", e));
                ctx.spawn(f);
            }
        }
    }

    fn request_upload(
        &self,
        ctx: grpcio::RpcContext,
        req: UploadRequest,
        sink: grpcio::UnarySink<UploadDetails>,
    ) {
        let mut resp = UploadDetails::new();

        let uuid = Uuid::new_v4().to_string();
        resp.set_uuid(uuid.clone());

        let upload = Upload {
            repo_name: req.get_repo_name().to_owned(),
            uuid,
        };
        {
            self.active_uploads.write().unwrap().insert(upload);
            debug!("Hash Table: {:?}", self.active_uploads);
        }

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
        match self.save_layer(cr.get_repo_name(), cr.get_user_digest(), cr.get_uuid()) {
            Ok(_) => {
                let mut cu = CompletedUpload::new();
                cu.set_digest(cr.get_user_digest().to_string());
                let f = sink
                    .success(cu)
                    .map_err(move |e| warn!("failed to reply! {:?}", e));
                ctx.spawn(f);
            }
            Err(e) => {
                warn!("Failure when saving layer: {:?}", e);
                let f = sink
                    .fail(RpcStatus::new(
                        RpcStatusCode::Internal,
                        Some("Internal error saving file".to_string()),
                    ))
                    .map_err(|e| warn!("Internal error saving file {:?}", e));
                ctx.spawn(f);
            }
        }

        //delete uuid from uploads tracking
        let upload = Upload {
            repo_name: cr.get_repo_name().to_string(),
            uuid: cr.get_uuid().to_string(),
        };

        let mut set = self.active_uploads.write().unwrap();
        if !set.remove(&upload) {
            warn!("Upload {:?} not found when deleting", upload);
        }
    }

    fn verify_manifest(
        &self,
        ctx: grpcio::RpcContext,
        mr: ManifestRef,
        sink: grpcio::UnarySink<VerifiedManifest>,
    ) {
        match self.create_verified_manifest(
            mr.get_repo_name().to_string(),
            mr.get_reference().to_string(),
            true,
        ) {
            Ok(vm) => {
                let f = sink
                    .success(vm)
                    .map_err(move |e| warn!("failed to reply! {:?}", e));
                ctx.spawn(f);
            }
            Err(e) => {
                warn!("Error verifying manifest {:?}", e);
                let f = sink
                    .fail(RpcStatus::new(
                        RpcStatusCode::Internal,
                        Some("Problem verifying manifest".to_string()),
                    ))
                    .map_err(|e| warn!("Internal error saving file {:?}", e));
                ctx.spawn(f);
            }
        }
    }

    fn get_catalog(
        &self,
        ctx: grpcio::RpcContext,
        _: CatalogRequest,
        sink: grpcio::ServerStreamingSink<CatalogEntry>,
    ) {
        let mut repos = HashSet::new();
        match visit_dirs(&self.manifests_path, &self.manifests_path, &mut repos) {
            Ok(_) => {
                let repo_list: Vec<_> = repos
                    .iter()
                    .map(|r| {
                        let mut ce = CatalogEntry::new();
                        ce.set_repo_name(r.to_string());
                        (ce, WriteFlags::default())
                    })
                    .collect();
                let f = sink
                    .send_all(stream::iter_ok::<_, grpcio::Error>(repo_list))
                    .map(|_| {})
                    .map_err(|e| warn!("Failed to handle catalog request: {:?}", e));
                ctx.spawn(f)
            }
            Err(e) => {
                warn!("Error retreiving repository catalog {:?}", e);
                let f = sink
                    .fail(RpcStatus::new(
                        RpcStatusCode::Internal,
                        Some("Problem retrieving catalog".to_string()),
                    ))
                    .map_err(|e| warn!("Internal error sending response {:?}", e));
                ctx.spawn(f);
            }
        }
    }

    fn list_tags(
        &self,
        ctx: grpcio::RpcContext,
        ce: CatalogEntry,
        sink: grpcio::ServerStreamingSink<Tag>,
    ) {
        let mut tags = Vec::new();
        let mut path = PathBuf::from(&self.manifests_path);
        path.push(ce.get_repo_name());

        if let Ok(files) = fs::read_dir(path) {
            for entry in files {
                if let Ok(en) = entry {
                    let en_path = en.path();
                    if en_path.is_file() {
                        if let Some(tag_str) = en_path.file_name() {
                            let mut tag = Tag::new();
                            tag.set_tag(tag_str.to_string_lossy().to_string());
                            tags.push((tag, WriteFlags::default()));
                        }
                    }
                }
            }
        }

        let f = sink
            .send_all(stream::iter_ok::<_, grpcio::Error>(tags))
            .map(|_| {})
            .map_err(|e| warn!("Failed to respond to ListTags request: {:?}", e));
        ctx.spawn(f);
    }
}
