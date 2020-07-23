use crate::manifest::{FromJson, Manifest};
use chrono::prelude::*;
use failure::{self, Error};
use std::collections::HashSet;
use std::fmt;
use std::fs::{self, DirEntry, File};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};
use uuid::Uuid;
use prost_types::Timestamp;

use std::io;
use fs3; 

use crypto::digest::Digest;
use crypto::sha2::Sha256;

pub mod trow_server {
    include!("../../protobuf/out/trow.rs");
}

use self::trow_server::*;
use crate::server::trow_server::registry_server::Registry;


use prometheus::{TextEncoder, Encoder};

use crate::statics::{FREE_SPACE, AVAILABLE_SPACE, TOTAL_SPACE, TOTAL_MANIFEST_REQUESTS, TOTAL_BLOB_REQUESTS};

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

fn does_manifest_match_digest(manifest: &DirEntry, digest: &str) -> bool {
    digest
        == match get_digest_from_manifest_path(manifest.path()) {
            Ok(test_digest) => test_digest,
            Err(e) => {
                warn!("Failure reading repo {:?}", e);
                "NO_MATCH".to_string()
            }
        }
}

struct RepoIterator {
    paths: Vec<Result<DirEntry, std::io::Error>>,
}

impl RepoIterator {
    fn new(base_dir: &Path) -> Result<RepoIterator, Error> {
        let paths = fs::read_dir(base_dir)?.collect();
        Ok(RepoIterator { paths })
    }
}

impl Iterator for RepoIterator {
    type Item = DirEntry;
    fn next(&mut self) -> Option<Self::Item> {
        match self.paths.pop() {
            None => None,
            Some(res_path) => match res_path {
                Err(e) => {
                    warn!("Error iterating over repos {:?}", e);
                    self.next()
                }
                Ok(path) => {
                    if path.file_type().unwrap().is_dir() {
                        let new_paths = fs::read_dir(path.path()).unwrap();
                        self.paths.extend(new_paths);
                        self.next()
                    } else {
                        Some(path)
                    }
                }
            },
        }
    }
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

fn is_path_writable(path: &PathBuf) -> io::Result<bool> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    let permissions = metadata.permissions();
    Ok(!permissions.readonly())
}

fn get_digest_from_manifest_path<P: AsRef<Path>>(path: P) -> Result<String, Error> {
    let digest_date = fs::read_to_string(path)?;
    //Should be digest followed by date, but allow for digest only
    Ok(digest_date
        .split(' ')
        .next()
        .unwrap_or(&digest_date)
        .to_string())
}

// Query disk metrics
fn query_disk_metrics(path: &PathBuf) {
    let data_path = path.parent().unwrap();
    let available_space =  fs3::available_space(data_path).unwrap_or(0);
    AVAILABLE_SPACE.set(available_space as i64);
    let free_space  =  fs3::free_space(data_path).unwrap_or(0);
    FREE_SPACE.set(free_space as i64);
    let total_space =  fs3::total_space(data_path).unwrap_or(0);
    TOTAL_SPACE.set(total_space as i64);
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

    // Given a manifest digest, check if it is referenced by any tag in the repo
    fn verify_manifest_digest_in_repo(&self, repo_name: &str, digest: &str) -> Result<bool, Error> {
        let mut ri = RepoIterator::new(&self.manifests_path.join(repo_name))?;
        let res = ri.find(|de| does_manifest_match_digest(de, &digest));
        Ok(res.is_some())
    }

    fn get_digest_from_manifest(&self, repo_name: &str, tag: &str) -> Result<String, Error> {
        get_digest_from_manifest_path(self.manifests_path.join(repo_name).join(tag))
    }

    fn save_tag(&self, digest: &str, repo_name: &str, tag: &str) -> Result<(), Error> {
        // Tag files should contain list of digests with timestamp
        // First line should always be the current digest

        let repo_dir = self.manifests_path.join(repo_name);
        let repo_path = repo_dir.join(tag);
        fs::create_dir_all(&repo_dir)?;

        let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true);
        let contents = format!("{} {}\n", digest, ts);

        if let Ok(mut f) = fs::File::open(&repo_path) {
            let mut buf = Vec::new();
            buf.extend(contents.as_bytes().iter());
            f.read_to_end(&mut buf)?;
            // TODO: Probably best to write to temporary file and then copy over.
            // Will be closer to atomic
            fs::write(&repo_path, buf)?;
        } else {
            fs::File::create(&repo_path).and_then(|mut f| f.write_all(contents.as_bytes()))?;
        }
        Ok(())
    }

    fn get_path_for_manifest(&self, repo_name: &str, reference: &str) -> Result<PathBuf, Error> {
        let digest = if is_digest(reference) {
            if !self.verify_manifest_digest_in_repo(repo_name, reference)? {
                error!("Digest {} not in repository {}", reference, repo_name);
                return Err(failure::err_msg(format!(
                    "Digest {} not in repository {}",
                    reference, repo_name
                )));
            }
            reference.to_string()
        } else {
            //Content of tag is the digest
            self.get_digest_from_manifest(repo_name, reference)?
        };

        return self.get_catalog_path_for_blob(&digest);
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
            for digest in manifest.get_local_asset_digests() {
                let path = self.get_catalog_path_for_blob(digest)?;

                if !path.exists() {
                    return Err(format_err!(
                        "Failed to find artifact with digest {}",
                        digest
                    ));
                }
            }
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
        req: Request<UploadRef>,
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
        req: Request<BlobRef>,
    ) -> Result<Response<BlobReadLocation>, Status> {
        TOTAL_BLOB_REQUESTS.inc();
        let br = req.into_inner();
        let path = self
            .get_catalog_path_for_blob(&br.digest)
            .map_err(|e| Status::invalid_argument(format!("Error parsing digest {:?}", e)))?;

        if !path.exists() {
            warn!("Request for unknown blob: {:?}", path);
            Err(Status::not_found(format!(
                "No blob found matching {:?}",
                br
            )))
        } else {
            Ok(Response::new(BlobReadLocation {
                path: path.to_string_lossy().to_string(),
            }))
        }
    }

    /**
     * TODO: check if blob referenced by manifests. If so, refuse to delete.
     */
    async fn delete_blob(&self, req: Request<BlobRef>) -> Result<Response<BlobDeleted>, Status> {
        let br = req.into_inner();
        let path = self
            .get_catalog_path_for_blob(&br.digest)
            .map_err(|e| Status::invalid_argument(format!("Error parsing digest {:?}", e)))?;
        if !path.exists() {
            warn!("Request for unknown blob: {:?}", path);
            Err(Status::not_found(format!(
                "No blob found matching {:?}",
                br
            )))
        } else {
            fs::remove_file(&path)
                .map_err(|e| {
                    error!("Failed to delete blob {:?} {:?}", br, e);
                    Status::internal("Internal error deleting blob")
                })
                .and(Ok(Response::new(BlobDeleted {})))
        }
    }

    async fn delete_manifest(
        &self,
        req: Request<ManifestRef>,
    ) -> Result<Response<ManifestDeleted>, Status> {
        let mr = req.into_inner();
        if !is_digest(&mr.reference) {
            return Err(Status::invalid_argument(format!(
                "Manifests can only be deleted by digest. Got {}",
                mr.reference
            )));
        }
        let digest = mr.reference;
        //For the repo, go through all tags and see if they reference the digest. Delete them.
        //Can only delete manifest if no other tags in any repo reference it

        let ri = RepoIterator::new(&self.manifests_path.join(&mr.repo_name)).map_err(|e| {
            error!("Problem reading manifest catalog {:?}", e);
            Status::failed_precondition("Repository not found")
        })?;

        //TODO: error if no manifest matches?
        ri.filter(|de| does_manifest_match_digest(de, &digest))
            .for_each(|man| match fs::remove_file(man.path()) {
                Ok(_) => (),
                Err(e) => error!("Failed to delete manifest {:?} {:?}", &man, e),
            });

        Ok(Response::new(ManifestDeleted {}))
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
        // Increase manifest request metrics counter
        TOTAL_MANIFEST_REQUESTS.inc();
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
                // copy manifest to blobs and add tag

                let digest = vm.digest.clone();

                let ret = self
                    .save_blob(&uploaded_manifest, &digest)
                    .and(self.save_tag(&digest, &mr.repo_name, &mr.reference))
                    .map(|_| Response::new(vm))
                    .map_err(|e| {
                        error!(
                            "Failure cataloguing manifest {}/{} {:?}",
                            &mr.repo_name, &mr.reference, e
                        );
                        Status::internal("Internal error copying manifest")
                    });

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
        request: Request<CatalogRequest>,
    ) -> Result<Response<Self::GetCatalogStream>, Status> {
        let cr = request.into_inner();
        let limit = cr.limit as usize;

        let (mut tx, rx) = mpsc::channel(4);
        let catalog: HashSet<String> = RepoIterator::new(&self.manifests_path)
            .map_err(|e| {
                error!("Error accessing catalog {:?}", e);
                Status::internal("Internal error streaming catalog")
            })?
            .map(|de| de.path())
            .filter_map(|p| p.parent().map(|p| p.to_path_buf()))
            .filter_map(|r| {
                r.strip_prefix(&self.manifests_path)
                    .ok()
                    .map(|p| p.to_path_buf())
            })
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        let partial_catalog: Vec<String> = if cr.last_repo.is_empty() {
            catalog.into_iter().take(limit).collect()
        } else {
            catalog
                .into_iter()
                .skip_while(|t| t != &cr.last_repo)
                .skip(1)
                .take(limit)
                .collect()
        };

        tokio::spawn(async move {
            for repo_name in partial_catalog {
                let ce = CatalogEntry { repo_name };

                tx.send(Ok(ce)).await.expect("Error streaming catalog");
            }
        });
        Ok(Response::new(rx))
    }

    type ListTagsStream = mpsc::Receiver<Result<Tag, Status>>;

    async fn list_tags(
        &self,
        request: Request<ListTagsRequest>,
    ) -> Result<Response<Self::ListTagsStream>, Status> {
        let (mut tx, rx) = mpsc::channel(4);
        let mut path = PathBuf::from(&self.manifests_path);

        let ltr = request.into_inner();

        let limit = ltr.limit as usize;
        path.push(&ltr.repo_name);

        let mut catalog: Vec<String> = RepoIterator::new(&path)
            .map_err(|e| {
                error!("Error accessing catalog {:?}", e);
                Status::internal("Internal error streaming catalog")
            })?
            .map(|de| de.path().file_name().unwrap().to_string_lossy().to_string())
            .collect();
        catalog.sort();
        let partial_catalog: Vec<String> = if ltr.last_tag.is_empty() {
            catalog.into_iter().take(limit).collect()
        } else {
            catalog
                .into_iter()
                .skip_while(|t| t != &ltr.last_tag)
                .skip(1)
                .take(limit)
                .collect()
        };

        tokio::spawn(async move {
            for tag in partial_catalog {
                tx.send(Ok(Tag {
                    tag: tag.to_string(),
                }))
                .await
                .expect("Error streaming tags");
            }
        });
        Ok(Response::new(rx))
    }

    type GetManifestHistoryStream = mpsc::Receiver<Result<ManifestHistoryEntry, Status>>;

    async fn get_manifest_history(
        &self,
        request: Request<ManifestHistoryRequest>,
    ) -> Result<Response<Self::GetManifestHistoryStream>, Status> {

        let mr = request.into_inner();
        if is_digest(&mr.tag) {
            return Err(Status::invalid_argument("Require valid tag (not digest) to search for history"));
        }

        let file = File::open(self.manifests_path.join(&mr.repo_name).join(&mr.tag))?;
        let reader = BufReader::new(file);

        let (mut tx, rx) = mpsc::channel(4);
        tokio::spawn(async move {

            let mut searching_for_digest = mr.last_digest != ""; //Looking for a digest iff it's not empty

            let mut sent = 0;
            for line in reader.lines() {
                if line.is_ok() {

                    let line = line.unwrap();
                    let (digest, date) = match line.find(' ') {

                        Some(ind) => {
                            let (digest_str, date_str) = line.split_at(ind);

                            if searching_for_digest {
                                if digest_str == mr.last_digest {
                                    searching_for_digest = false;
                                }
                                //Remember we want digest following matched digest
                                continue;
                            }

                            let dt_r = DateTime::parse_from_rfc3339(date_str.trim());

                            let ts = if let Ok(dt) = dt_r {
                                Some(Timestamp { seconds: dt.timestamp(), nanos: dt.timestamp_subsec_nanos() as i32})
                            } else {
                                warn!("Failed to parse timestamp {}", date_str);
                                None
                            };
                            (digest_str, ts)
                        },
                        None => {
                            warn!("No timestamp found in manifest");
                            (line.as_ref(), None)
                        }
                    };

                    let entry = ManifestHistoryEntry {
                        digest: digest.to_string(),
                        date
                    };
                    tx.send(Ok(entry))
                        .await
                       .expect("Error streaming manifest history");

                    sent = sent + 1;
                    if sent >= mr.limit {
                        break;
                    }
                }
            }
        });
        Ok(Response::new(rx))
    }

     // Readiness check
     async fn is_ready(
        &self,
        _request: Request<ReadinessRequest>,
    ) -> Result<Response<ReadyStatus>, Status> {

        for path in &[&self.scratch_path, &self.manifests_path, &self.blobs_path] {
            
            match is_path_writable(path) {
                Ok(true) => {},
                Ok(false) => {
                    return Err(Status::unavailable(format!("{} is not writable", path.to_string_lossy())));
                    },
                Err(error) => {
                    return Err(Status::unavailable(error.to_string()));
                }
            }
        }

        //All paths writable
        let reply = trow_server::ReadyStatus {
            message: String::from("Ready"),
        };

        return Ok(Response::new(reply));
            
    }

    async fn is_healthy(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<HealthStatus>, Status> {
        
        let reply = trow_server::HealthStatus {
            message: String::from("Healthy")
        };
        Ok(Response::new(reply))
    }

    async fn get_metrics(
        &self,
        _request: Request<MetricsRequest>,
    ) -> Result<Response<MetricsResponse>, Status> {
        query_disk_metrics(&mut self.blobs_path.clone());
        
        let encoder = TextEncoder::new();
        
        // Gather all prometheus metrics of DEFAULT_REGISTRY 
        // * disk
        // * total manifest requests
        // * total blob requests

        let metric_families = prometheus::gather();
        let mut buffer = vec![];
        
        match encoder.encode(&metric_families, &mut buffer) {
            Ok(_) => {   
                let metrics =  String::from_utf8(buffer).unwrap();

                let reply = trow_server::MetricsResponse {                             
                    metrics: metrics,
                    errored: false,
                    message: String::from("")
                };
                Ok(Response::new(reply))
            },

            Err(error) => {
                return Err(Status::unavailable(error.to_string()));
            }
        } 
    }
}