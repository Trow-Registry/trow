use crate::manifest::{FromJson, Manifest};
use failure::{self, Error};
use std::collections::HashSet;
use std::fmt;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crypto::digest::Digest;
use crypto::sha2::Sha256;

pub mod trow_server {
    include!("../../protobuf/out/trow.rs");
}

use self::trow_server::{
    registry_server::Registry, BlobReadLocation, BlobRef, CatalogEntry, CatalogRequest,
    CompleteRequest, CompletedUpload, DownloadRef, ManifestReadLocation, ManifestRef,
    ManifestWriteDetails, Tag, UploadDetails, UploadRequest, VerifiedManifest,
    VerifyManifestRequest, WriteLocation,
};

static SUPPORTED_DIGESTS: [&'static str; 1] = ["sha256"];
static MANIFESTS_DIR: &'static str = "manifests";
static BLOBS_DIR: &'static str = "blobs";
static UPLOADS_DIR: &'static str = "scratch";

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
    blobs_path: PathBuf,
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

fn create_path(data_path: &str, dir: &str) -> Result<PathBuf, std::io::Error> {
    let data_path = Path::new(data_path);
    let dir_path = data_path.join(dir);
    if !dir_path.exists() {
        return match fs::create_dir_all(&dir_path) {
            Ok(_) => Ok(dir_path),
            Err(e) => {
                error!(
                    r#"
                Failed to create directory required by trow {:?}
                Please check the parent directory is writable by the trow user.
                {:?}"#,
                    dir_path, e
                );
                Err(e)
            }
        };
    };
    Ok(dir_path)
}

fn gen_digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.input(bytes);
    format!("sha256:{}", hasher.result_str())
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

/**
 * Checks a file matches the given digest.
 *
 * TODO: should be able to use range of hashes.
 * TODO: check if using a static for the hasher speeds things up.
 */
fn validate_digest(file: &PathBuf, digest: &str) -> Result<(), Error> {
    let f = File::open(file)?;
    let mut reader = BufReader::new(f);
    let mut hasher = Sha256::new();
    let mut buf = [0; 256]; // TODO: figure out best number here
    let mut bytes_read = reader.read(&mut buf[..])?;
    while bytes_read != 0 {
        hasher.input(&buf[..bytes_read]);
        bytes_read = reader.read(&mut buf[..])?;
    }

    let true_digest = format!("sha256:{}", hasher.result_str());
    if true_digest != digest {
        error!(
            "Upload did not match given digest. Was given {} but got {}",
            digest, true_digest
        );
        return Err(failure::err_msg(format!(
            "Upload did not match given digest. Was given {} but got {}",
            digest, true_digest
        )));
    }

    Ok(())
}

fn is_digest(maybe_digest: &str) -> bool {
    for alg in &SUPPORTED_DIGESTS {
        if maybe_digest.starts_with(&format!("{}:", alg)) {
            return true;
        }
    }

    false
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
        let scratch_path = create_path(data_path, UPLOADS_DIR)?;
        let blobs_path = create_path(data_path, BLOBS_DIR)?;
        let svc = TrowServer {
            active_uploads: Arc::new(RwLock::new(HashSet::new())),
            manifests_path,
            blobs_path,
            scratch_path,
            allow_prefixes,
            allow_images,
            deny_local_prefixes,
            deny_local_images,
        };
        Ok(svc)
    }

    fn get_upload_path_for_blob(&self, uuid: &str) -> PathBuf {
        self.scratch_path.join(uuid)
    }

    fn get_catalog_path_for_blob(&self, digest: &str) -> Result<PathBuf, Error> {
        let mut iter = digest.split(':');
        let alg = iter.next().ok_or(format_err!(
            "Digest {} did not contain alg component",
            digest
        ))?;
        if !SUPPORTED_DIGESTS.contains(&alg) {
            return Err(format_err!("Hash algorithm {} not supported", alg));
        }
        let val = iter.next().ok_or(format_err!(
            "Digest {} did not contain value component",
            digest
        ))?;
        assert_eq!(None, iter.next());
        Ok(self.blobs_path.join(alg).join(val))
    }

    fn get_path_for_manifest(&self, repo_name: &str, reference: &str) -> Result<PathBuf, Error> {
        if is_digest(reference) {
            return self.get_catalog_path_for_blob(reference);
        }
        Ok(self.manifests_path.join(repo_name).join(reference))
    }

    fn create_verified_manifest(
        &self,
        manifest_path: &PathBuf,
        verify_assets_exist: bool,
    ) -> Result<VerifiedManifest, Error> {
        let manifest_bytes = std::fs::read(&manifest_path)?;
        let manifest_json: serde_json::Value = serde_json::from_slice(&manifest_bytes)?;
        let manifest = Manifest::from_json(&manifest_json)?;

        if verify_assets_exist {
            //TODO: Need to make sure we find things indexed by digest or tag
            for digest in manifest.get_asset_digests() {
                let path = self.get_catalog_path_for_blob(digest)?;

                if !path.exists() {
                    return Err(format_err!(
                        "Failed to find artifact with digest {}",
                        digest
                    ));
                }
            }

            // TODO: check signature and names are correct on v1 manifests
            // AM: Actually can we just nuke v1 support?
        }

        //For performance, could generate only if verification is on, otherwise copy from somewhere
        Ok(VerifiedManifest {
            digest: gen_digest(&manifest_bytes),
            content_type: manifest.get_media_type().to_string(),
        })
    }

    fn create_manifest_read_location(
        &self,
        repo_name: String,
        reference: String,
        do_verification: bool,
    ) -> Result<ManifestReadLocation, Error> {
        //TODO: This isn't optimal
        let path = self.get_path_for_manifest(&repo_name, &reference)?;
        let vm = self.create_verified_manifest(&path, do_verification)?;
        Ok(ManifestReadLocation {
            content_type: vm.content_type.to_owned(),
            digest: vm.digest.to_owned(),
            path: path.to_string_lossy().to_string(),
        })
    }

    fn save_blob(&self, scratch_path: &PathBuf, digest: &str) -> Result<(), Error> {
        let digest_path = self.get_catalog_path_for_blob(digest)?;
        let repo_path = digest_path
            .parent()
            .ok_or_else(|| failure::err_msg("Error finding repository path"))?;

        if !repo_path.exists() {
            fs::create_dir_all(repo_path)?;
        }

        fs::copy(&scratch_path, &digest_path)?;
        Ok(())
    }

    fn validate_and_save_blob(&self, user_digest: &str, uuid: &str) -> Result<(), Error> {
        debug!("Saving blob {}", user_digest);

        let scratch_path = self.get_upload_path_for_blob(uuid);
        let res = match validate_digest(&scratch_path, user_digest) {
            Ok(_) => self.save_blob(&scratch_path, user_digest),
            Err(e) => Err(e),
        };

        //Not an error, even if it's not great
        fs::remove_file(&scratch_path).unwrap_or_else(|e| {
            error!(
                "Error deleting file {} {:?}",
                &scratch_path.to_string_lossy(),
                e
            )
        });

        res?;
        Ok(())
    }

    //Support functions for validate, would like to move these
    pub fn image_exists(&self, image: &Image) -> bool {
        match self.get_path_for_manifest(&image.repo, &image.tag) {
            Ok(f) => f.exists(),
            Err(_) => false,
        }
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

#[tonic::async_trait]
impl Registry for TrowServer {
    async fn request_upload(
        &self,
        request: Request<UploadRequest>,
    ) -> Result<Response<UploadDetails>, Status> {
        let uuid = Uuid::new_v4().to_string();
        let reply = UploadDetails { uuid: uuid.clone() };
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
            let path = self.get_upload_path_for_blob(&br.uuid);
            Ok(Response::new(WriteLocation {
                path: path.to_string_lossy().to_string(),
            }))
        } else {
            Err(Status::failed_precondition(format!(
                "No current upload matching {:?}",
                br
            )))
        }
    }

    async fn get_read_location_for_blob(
        &self,
        req: Request<DownloadRef>,
    ) -> Result<Response<BlobReadLocation>, Status> {
        let dr = req.into_inner();
        let path = self
            .get_catalog_path_for_blob(&dr.digest)
            .map_err(|e| Status::failed_precondition(format!("Error parsing digest {:?}", e)))?;

        if !path.exists() {
            warn!("Request for unknown blob: {:?}", path);
            Err(Status::failed_precondition(format!(
                "No blob found matching {:?}",
                dr
            )))
        } else {
            Ok(Response::new(BlobReadLocation {
                path: path.to_string_lossy().to_string(),
            }))
        }
    }

    async fn get_write_details_for_manifest(
        &self,
        _req: Request<ManifestRef>, // Expect to be used later in checks e.g. immutable tags
    ) -> Result<Response<ManifestWriteDetails>, Status> {
 
        //Give the manifest a UUID and save it to the uploads dir
        let uuid = Uuid::new_v4().to_string();

        let manifest_path = self.get_upload_path_for_blob(&uuid);
        Ok(Response::new(ManifestWriteDetails {
            path: manifest_path.to_string_lossy().to_string(),
            uuid,
        }))
    }

    async fn get_read_location_for_manifest(
        &self,
        req: Request<ManifestRef>,
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
        req: Request<CompleteRequest>,
    ) -> Result<Response<CompletedUpload>, Status> {
        let cr = req.into_inner();
        let ret = match self.validate_and_save_blob(&cr.user_digest, &cr.uuid) {
            Ok(_) => Ok(Response::new(CompletedUpload {
                digest: cr.user_digest.clone(),
            })),
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

    /**
     * Take uploaded manifest (which should be uuid in uploads), check it, put in catalog and
     * by blob digest
     */
    async fn verify_manifest(
        &self,
        req: Request<VerifyManifestRequest>,
    ) -> Result<Response<VerifiedManifest>, Status> {
        let req = req.into_inner();
        let mr = req.manifest.unwrap(); // Pissed off that the manifest is optional!
        let uploaded_manifest = self.get_upload_path_for_blob(&req.uuid);

        match self.create_verified_manifest(&uploaded_manifest, true) {
            Ok(vm) => {
                //move file to digest location and repo/tag

                let digest = vm.digest.clone();
                let mut ret = Ok(Response::new(vm));

                // TODO: can we simplify this with and_then?
                match self.save_blob(&uploaded_manifest, &digest) {
                    Ok(_) => {
                        let repo_dir = self.manifests_path.join(mr.repo_name);
                        let repo_path = repo_dir.join(mr.reference);
                        match fs::create_dir_all(&repo_dir).and_then(|_| fs::copy(&uploaded_manifest, &repo_path)) {
                            Ok(_) => (),
                            Err(e) => {
                                error!("Failure copying manifest from {:?} to {:?} {:?}", 
                                    &uploaded_manifest, &repo_path, e);
                                ret = Err(Status::internal("Internal error copying manifest"));
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failure saving blob {:?}", e);
                        ret = Err(Status::internal("Internal error copying manifest"));
                    }
                }
                fs::remove_file(&uploaded_manifest)
                    .unwrap_or_else(|e| error!("Failure deleting uploaded manifest {:?}", e));
                ret
            }
            Err(e) => {
                error!("Error verifying manifest {:?}", e);
                Err(Status::internal("Internal error verifying manifest"))
            }
        }
    }

    type GetCatalogStream = mpsc::Receiver<Result<CatalogEntry, Status>>;

    async fn get_catalog(
        &self,
        _request: Request<CatalogRequest>,
    ) -> Result<Response<Self::GetCatalogStream>, Status> {
        let (mut tx, rx) = mpsc::channel(4);
        let mut repos = HashSet::new();
        match visit_dirs(&self.manifests_path, &self.manifests_path, &mut repos) {
            Ok(_) => {
                tokio::spawn(async move {
                    for r in repos.iter() {
                        let ce = CatalogEntry {
                            repo_name: r.to_string(),
                        };
                        tx.send(Ok(ce)).await.expect("Error streaming catalog");
                    }
                });
                Ok(Response::new(rx))
            }
            Err(e) => {
                warn!("Error retreiving repository catalog {:?}", e);
                Err(Status::internal("Internal error streaming catalog"))
            }
        }
    }

    type ListTagsStream = mpsc::Receiver<Result<Tag, Status>>;

    async fn list_tags(
        &self,
        request: Request<CatalogEntry>,
    ) -> Result<Response<Self::ListTagsStream>, Status> {
        let (mut tx, rx) = mpsc::channel(4);
        let mut path = PathBuf::from(&self.manifests_path);
        let ce = request.into_inner();
        path.push(ce.repo_name);

        if let Ok(files) = fs::read_dir(path) {
            tokio::spawn(async move {
                for entry in files {
                    if let Ok(en) = entry {
                        let en_path = en.path();
                        if en_path.is_file() {
                            if let Some(tag_str) = en_path.file_name() {
                                let tag = Tag {
                                    tag: tag_str.to_string_lossy().to_string(),
                                };
                                tx.send(Ok(tag)).await.expect("Error streaming tags");
                            }
                        }
                    }
                }
            });
        }
        Ok(Response::new(rx))
    }
}
