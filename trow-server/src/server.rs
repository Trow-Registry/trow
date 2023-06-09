use std::collections::HashSet;
use std::fs::{self, DirEntry, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::{io, str};

use anyhow::{anyhow, Result};
use async_recursion::async_recursion;
use chrono::prelude::*;
use futures::future::try_join_all;
use log::{debug, error, info, warn};
use prost_types::Timestamp;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{self, Method};
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tokio::time::Duration;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use self::trow_server::*;
use crate::digest::sha256_tag_digest;
use crate::image::RemoteImage;
use crate::manifest::{manifest_media_type, FromJson, Manifest};
use crate::proxy_auth::ProxyClient;
use crate::server::trow_server::registry_server::Registry;
use crate::temporary_file::TemporaryFile;
use crate::{metrics, ImageValidationConfig, RegistryProxyConfig};

pub mod trow_server {
    include!("../../trow-protobuf/out/trow.rs");
}

static SUPPORTED_DIGESTS: [&str; 1] = ["sha256"];
static MANIFESTS_DIR: &str = "manifests";
static BLOBS_DIR: &str = "blobs";
static UPLOADS_DIR: &str = "scratch";

static PROXY_DIR: &str = "f/"; //Repositories starting with this are considered proxies
static DIGEST_HEADER: &str = "Docker-Content-Digest";

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
    pub proxy_registry_config: Vec<RegistryProxyConfig>,
    pub image_validation_config: Option<ImageValidationConfig>,
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
struct Upload {
    repo_name: String,
    uuid: String,
}

#[derive(Error, Debug)]
#[error("Error getting proxied repo {msg:?}")]
pub struct ProxyError {
    msg: String,
}

#[derive(Error, Debug)]
#[error("Expected digest {user_digest:?} but got {actual_digest:?}")]
pub struct DigestValidationError {
    user_digest: String,
    actual_digest: String,
}

pub fn create_accept_header() -> HeaderMap {
    const ACCEPT: [&str; 4] = [
        manifest_media_type::OCI_V1,
        manifest_media_type::DOCKER_V2,
        manifest_media_type::DOCKER_LIST,
        manifest_media_type::OCI_INDEX,
    ];

    let mut headers = HeaderMap::new();
    headers.insert(
        reqwest::header::ACCEPT,
        HeaderValue::from_str(&ACCEPT.join(", ")).unwrap(),
    );
    headers
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
    fn new(base_dir: &Path) -> Result<RepoIterator> {
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
fn validate_digest(file: &PathBuf, digest: &str) -> Result<()> {
    let f = File::open(file)?;
    let reader = BufReader::new(f);

    let calculated_digest = sha256_tag_digest(reader)?;

    if calculated_digest != digest {
        error!(
            "Upload did not match given digest. Was given {} but got {}",
            digest, calculated_digest
        );
        return Err(anyhow!(DigestValidationError {
            user_digest: digest.to_string(),
            actual_digest: calculated_digest,
        }));
    }

    Ok(())
}

pub fn is_digest(maybe_digest: &str) -> bool {
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

fn get_digest_from_manifest_path<P: AsRef<Path>>(path: P) -> Result<String> {
    let manifest = fs::read_to_string(path)?;
    let latest_digest_line = manifest
        .lines()
        .last()
        .ok_or_else(|| anyhow!("Empty manifest"))?;
    // Each line is `{digest} {date}`
    let latest_digest = latest_digest_line
        .split(' ')
        .next()
        .ok_or_else(|| anyhow!("Invalid manifest line: `{}`", latest_digest_line))?;

    Ok(latest_digest.to_string())
}

impl TrowServer {
    pub fn new(
        data_path: &str,
        proxy_registry_config: Vec<RegistryProxyConfig>,
        image_validation_config: Option<ImageValidationConfig>,
    ) -> Result<Self> {
        let manifests_path = create_path(data_path, MANIFESTS_DIR)?;
        let scratch_path = create_path(data_path, UPLOADS_DIR)?;
        let blobs_path = create_path(data_path, BLOBS_DIR)?;

        let svc = TrowServer {
            active_uploads: Arc::new(RwLock::new(HashSet::new())),
            manifests_path,
            blobs_path,
            scratch_path,
            proxy_registry_config,
            image_validation_config,
        };
        Ok(svc)
    }

    fn get_upload_path_for_blob(&self, uuid: &str) -> PathBuf {
        self.scratch_path.join(uuid)
    }

    fn get_catalog_path_for_blob(&self, digest: &str) -> Result<PathBuf> {
        let mut iter = digest.split(':');
        let alg = iter
            .next()
            .ok_or_else(|| anyhow!("Digest {} did not contain alg component", digest))?;
        if !SUPPORTED_DIGESTS.contains(&alg) {
            return Err(anyhow!("Hash algorithm {} not supported", alg));
        }
        let val = iter
            .next()
            .ok_or_else(|| anyhow!("Digest {} did not contain value component", digest))?;
        assert_eq!(None, iter.next());
        Ok(self.blobs_path.join(alg).join(val))
    }

    fn get_digest_from_manifest(&self, repo_name: &str, reference: &str) -> Result<String> {
        get_digest_from_manifest_path(self.manifests_path.join(repo_name).join(reference))
    }

    async fn save_tag(&self, digest: &str, repo_name: &str, tag: &str) -> Result<()> {
        // Tag files should contain list of digests with timestamp
        // Last line should always be the current digest

        let repo_dir = self.manifests_path.join(repo_name);
        fs::create_dir_all(&repo_dir)?;

        let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true);
        let contents = format!("{} {}\n", digest, ts).into_bytes();

        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&repo_dir.join(tag))
            .await?;
        file.write_all(&contents).await?;

        Ok(())
    }

    fn get_path_for_manifest(&self, repo_name: &str, reference: &str) -> Result<PathBuf> {
        let digest = if is_digest(reference) {
            reference.to_string()
        } else {
            self.get_digest_from_manifest(repo_name, reference)?
        };

        self.get_catalog_path_for_blob(&digest)
    }

    fn create_verified_manifest(
        &self,
        manifest_path: &PathBuf,
        verify_assets_exist: bool,
    ) -> Result<VerifiedManifest> {
        let manifest_bytes = std::fs::read(manifest_path)?;
        let manifest_json: serde_json::Value = serde_json::from_slice(&manifest_bytes)?;
        let manifest = Manifest::from_json(&manifest_json)?;

        if verify_assets_exist {
            for digest in manifest.get_local_asset_digests() {
                let path = self.get_catalog_path_for_blob(digest)?;

                if !path.exists() {
                    return Err(anyhow!("Failed to find artifact with digest {}", digest));
                }
            }
        }

        // Calculate the digest: sha256:...
        let reader = BufReader::new(manifest_bytes.as_slice());
        let digest = sha256_tag_digest(reader)?;

        // For performance, could generate only if verification is on, otherwise copy from somewhere
        Ok(VerifiedManifest {
            digest,
            content_type: manifest.get_media_type(),
        })
    }

    /**
    If repo is proxied to another registry, this will return the details of the remote image.
    If the repo isn't proxied None is returned
    **/
    fn get_remote_image_and_cfg(
        &self,
        repo_name: &str,
        reference: &str,
    ) -> Option<(RemoteImage, RegistryProxyConfig)> {
        //All proxies are under "f_"
        if repo_name.starts_with(PROXY_DIR) {
            let segments = repo_name.splitn(3, '/').collect::<Vec<_>>();
            debug_assert_eq!(segments[0], "f");
            let proxy_alias = segments[1].to_string();
            let repo = segments[2].to_string();

            for proxy in self.proxy_registry_config.iter() {
                if proxy.alias == proxy_alias {
                    let image = RemoteImage::new(&proxy.host, repo, reference.into());
                    return Some((image, proxy.clone()));
                }
            }
        }
        None
    }

    /// Download a blob that is part of `remote_image`.
    async fn download_blob(
        &self,
        cl: &ProxyClient,
        remote_image: &RemoteImage,
        digest: &str,
    ) -> Result<()> {
        if self.get_catalog_path_for_blob(digest)?.exists() {
            info!("Already have blob {}", digest);
            return Ok(());
        }
        let path = self.scratch_path.join(digest);
        let mut file = match TemporaryFile::open_for_writing(path.clone()).await? {
            Some(f) => f,
            None => {
                info!("Waiting for concurrently fetched blob {}", digest);
                while path.exists() {
                    // wait for download to be done (temp file to be moved)
                    // TODO: use notify crate instead
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
                return Ok(());
            }
        };

        let addr = format!("{}/blobs/{}", remote_image.get_base_uri(), digest);
        info!("Downloading blob {}", addr);
        let resp = cl.authenticated_request(Method::GET, &addr).send().await?;

        file.write_stream(resp.bytes_stream()).await?;
        self.save_blob(file.path(), digest)?;
        Ok(())
    }

    #[async_recursion]
    async fn download_manifest_and_layers(
        &self,
        cl: &ProxyClient,
        remote_image: &RemoteImage,
        local_repo_name: &str,
    ) -> Result<()> {
        debug!("Downloading manifest + layers for {}", remote_image);
        let resp = cl
            .authenticated_request(Method::GET, &remote_image.get_manifest_url())
            .headers(create_accept_header())
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow!(
                "GET {} returned unexpected {}",
                &remote_image.get_manifest_url(),
                resp.status()
            ));
        }

        let mut buf =
            TemporaryFile::open_for_writing(self.scratch_path.join(Uuid::new_v4().to_string()))
                .await?
                .unwrap();
        let bytes = resp.bytes().await?;
        buf.write_all(&bytes).await?;

        let mani: Manifest = serde_json::from_slice(&bytes)?;
        match mani {
            Manifest::List(_) => {
                let images_to_dl = mani
                    .get_local_asset_digests()
                    .into_iter()
                    .map(|digest| {
                        let mut image = remote_image.clone();
                        image.reference = digest.to_string();
                        image
                    })
                    .collect::<Vec<_>>();
                let futures = images_to_dl
                    .iter()
                    .map(|img| self.download_manifest_and_layers(cl, img, local_repo_name));
                try_join_all(futures).await?;
            }
            Manifest::V2(_) => {
                let futures = mani
                    .get_local_asset_digests()
                    .into_iter()
                    .map(|digest| self.download_blob(cl, remote_image, digest));
                try_join_all(futures).await?;
            }
        }

        //Save out manifest
        let f = File::open(buf.path())?;
        let reader = BufReader::new(f);
        let calculated_digest = sha256_tag_digest(reader)?;

        self.save_blob(buf.path(), &calculated_digest)?;
        self.save_tag(&calculated_digest, local_repo_name, &remote_image.reference)
            .await?;

        Ok(())
    }

    async fn get_digest_from_header(
        &self,
        cl: &ProxyClient,
        image: &RemoteImage,
    ) -> Option<String> {
        let resp = cl
            .authenticated_request(Method::HEAD, &image.get_manifest_url())
            .headers(create_accept_header())
            .send()
            .await;

        match resp {
            Err(e) => {
                error!(
                    "Remote registry didn't respond correctly to HEAD request {}",
                    e
                );
                None
            }
            Ok(resp) => resp.headers().get(DIGEST_HEADER).map(|digest| {
                let digest = format!("{:?}", digest);
                digest.trim_matches('"').to_string()
            }),
        }
    }

    /// returns the downloaded digest
    async fn download_remote_image(
        &self,
        remote_image: RemoteImage,
        proxy_cfg: RegistryProxyConfig,
    ) -> Result<String> {
        // Replace eg f/docker/alpine by f/docker/library/alpine
        let repo_name = format!("f/{}/{}", proxy_cfg.alias, remote_image.get_repo());

        let try_cl = match ProxyClient::try_new(proxy_cfg.clone(), &remote_image).await {
            Ok(cl) => Some(cl),
            Err(e) => {
                error!(
                    "Could not create client for proxied registry {}: {}",
                    proxy_cfg.host, e
                );
                None
            }
        };
        let ref_is_digest = is_digest(&remote_image.reference);

        let (local_digest, latest_digest) = if ref_is_digest {
            (Some(remote_image.reference.clone()), None)
        } else {
            let local_digest = self
                .get_digest_from_manifest(&repo_name, &remote_image.reference)
                .ok();
            let mut latest_digest = match &try_cl {
                Some(cl) => self.get_digest_from_header(cl, &remote_image).await,
                _ => None,
            };
            if latest_digest == local_digest {
                if local_digest.is_none() {
                    anyhow::bail!(
                        "Could not fetch digest for {}:{}",
                        repo_name,
                        remote_image.reference
                    );
                }
                // if both are the same, no need to try to pull
                latest_digest = None;
            }
            (local_digest, latest_digest)
        };

        let digests = [latest_digest, local_digest].into_iter().flatten();

        for digest in digests {
            // if let Some(latest_digest) = latest_digest {
            let have_manifest = self.get_catalog_path_for_blob(&digest)?.exists();
            match have_manifest {
                true => return Ok(digest),
                false if try_cl.is_some() => {
                    match self
                        .download_manifest_and_layers(
                            try_cl.as_ref().unwrap(),
                            &remote_image,
                            &repo_name,
                        )
                        .await
                    {
                        Ok(_) if !ref_is_digest => match self
                            .save_tag(&digest, &repo_name, &remote_image.reference)
                            .await
                        {
                            Ok(_) => return Ok(digest),
                            Err(e) => {
                                error!("Internal error updating tag for proxied image ({})", e)
                            }
                        },
                        Ok(_) => return Ok(digest),
                        Err(e) => warn!("Failed to download proxied image: {}", e),
                    };
                }
                false => warn!("Missing manifest for proxied image, proxy client not available"),
            }
        }

        Err(anyhow!(
            "Could not fetch manifest for proxied image {}:{}",
            repo_name,
            remote_image.reference
        ))
    }

    async fn create_manifest_read_location(
        &self,
        repo_name: String,
        reference: String,
        do_verification: bool,
    ) -> Result<ManifestReadLocation> {
        let path = if let Some((remote_image, proxy_cfg)) =
            self.get_remote_image_and_cfg(&repo_name, &reference)
        {
            info!(
                "Request for proxied repo {}:{} maps to {}",
                repo_name, reference, remote_image
            );
            let digest = self.download_remote_image(remote_image, proxy_cfg).await?;
            // These are not up to date and should not be used !
            drop(repo_name);
            drop(reference);

            self.get_catalog_path_for_blob(&digest)?
        } else {
            self.get_path_for_manifest(&repo_name, &reference)?
        };

        let vm = self.create_verified_manifest(&path, do_verification)?;
        Ok(ManifestReadLocation {
            content_type: vm.content_type.to_owned(),
            digest: vm.digest,
            path: path.to_string_lossy().to_string(),
        })
    }

    /// Moves blob from scratch to blob catalog
    fn save_blob(&self, scratch_path: &Path, digest: &str) -> Result<()> {
        let digest_path = self.get_catalog_path_for_blob(digest)?;
        let repo_path = digest_path
            .parent()
            .ok_or_else(|| anyhow!("Error finding repository path"))?;

        if !repo_path.exists() {
            fs::create_dir_all(repo_path)?;
        }
        fs::rename(scratch_path, &digest_path)?;
        Ok(())
    }

    fn validate_and_save_blob(&self, user_digest: &str, uuid: &str) -> Result<()> {
        debug!("Saving blob {}", user_digest);

        let scratch_path = self.get_upload_path_for_blob(uuid);
        let res = match validate_digest(&scratch_path, user_digest) {
            Ok(_) => self.save_blob(&scratch_path, user_digest),
            Err(e) => Err(e),
        };

        res?;
        Ok(())
    }

    fn is_writable_repo(&self, repo_name: &str) -> bool {
        if repo_name.starts_with(PROXY_DIR) {
            return false;
        }

        true
    }
}

#[tonic::async_trait]
impl Registry for TrowServer {
    async fn request_upload(
        &self,
        request: Request<UploadRequest>,
    ) -> Result<Response<UploadDetails>, Status> {
        let repo_name = request.into_inner().repo_name;
        if self.is_writable_repo(&repo_name) {
            let uuid = Uuid::new_v4().to_string();
            let reply = UploadDetails { uuid: uuid.clone() };
            let upload = Upload { repo_name, uuid };
            {
                self.active_uploads.write().unwrap().insert(upload);
                debug!("Upload Table: {:?}", self.active_uploads);
            }
            Ok(Response::new(reply))
        } else {
            Err(Status::invalid_argument(format!(
                "Repository {} is not writable",
                repo_name
            )))
        }
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
        metrics::TOTAL_BLOB_REQUESTS.inc();
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
        req: Request<ManifestRef>,
    ) -> Result<Response<ManifestWriteDetails>, Status> {
        let repo_name = req.into_inner().repo_name;
        if self.is_writable_repo(&repo_name) {
            //Give the manifest a UUID and save it to the uploads dir
            let uuid = Uuid::new_v4().to_string();

            let manifest_path = self.get_upload_path_for_blob(&uuid);
            Ok(Response::new(ManifestWriteDetails {
                path: manifest_path.to_string_lossy().to_string(),
                uuid,
            }))
        } else {
            Err(Status::invalid_argument(format!(
                "Repository {} is not writable",
                repo_name
            )))
        }
    }

    async fn get_read_location_for_manifest(
        &self,
        req: Request<ManifestRef>,
    ) -> Result<Response<ManifestReadLocation>, Status> {
        //Don't actually need to verify here; could set to false

        let mr = req.into_inner();
        metrics::TOTAL_MANIFEST_REQUESTS.inc();
        match self
            .create_manifest_read_location(mr.repo_name, mr.reference, true)
            .await
        {
            Ok(vm) => Ok(Response::new(vm)),
            Err(e) => {
                warn!("Internal error with manifest: {:?}", e);
                Err(Status::internal("Internal error finding manifest"))
            }
        }
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
                self.save_blob(&uploaded_manifest, &digest)
                    .and(self.save_tag(&digest, &mr.repo_name, &mr.reference).await)
                    .map(|_| Response::new(vm))
                    .map_err(|e| {
                        error!(
                            "Failure cataloguing manifest {}/{} {:?}",
                            &mr.repo_name, &mr.reference, e
                        );
                        Status::internal("Internal error copying manifest")
                    })
            }
            Err(e) => {
                error!("Error verifying manifest {:?}", e);
                Err(Status::invalid_argument("Failed to verify manifest"))
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
            Err(e) => match e.downcast::<DigestValidationError>() {
                Ok(v_e) => Err(Status::invalid_argument(v_e.to_string())),
                Err(e) => {
                    warn!("Failure when saving layer: {:?}", e);
                    Err(Status::internal("Internal error saving layer"))
                }
            },
        };

        //delete uuid from uploads tracking
        let upload = Upload {
            repo_name: cr.repo_name.clone(),
            uuid: cr.uuid,
        };

        let mut set = self.active_uploads.write().unwrap();
        if !set.remove(&upload) {
            warn!("Upload {:?} not found when deleting", upload);
        }
        ret
    }

    type GetCatalogStream = ReceiverStream<Result<CatalogEntry, Status>>;

    async fn get_catalog(
        &self,
        request: Request<CatalogRequest>,
    ) -> Result<Response<Self::GetCatalogStream>, Status> {
        let cr = request.into_inner();
        let limit = cr.limit as usize;

        let (tx, rx) = mpsc::channel(4);
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
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type ListTagsStream = ReceiverStream<Result<Tag, Status>>;

    async fn list_tags(
        &self,
        request: Request<ListTagsRequest>,
    ) -> Result<Response<Self::ListTagsStream>, Status> {
        let (tx, rx) = mpsc::channel(4);
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
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    type GetManifestHistoryStream = ReceiverStream<Result<ManifestHistoryEntry, Status>>;

    async fn get_manifest_history(
        &self,
        request: Request<ManifestHistoryRequest>,
    ) -> Result<Response<Self::GetManifestHistoryStream>, Status> {
        let mr = request.into_inner();
        if is_digest(&mr.tag) {
            return Err(Status::invalid_argument(
                "Require valid tag (not digest) to search for history",
            ));
        }

        let manifest_path = self.manifests_path.join(&mr.repo_name).join(&mr.tag);

        let file = File::open(&manifest_path);

        if file.is_err() {
            return Err(Status::not_found(format!(
                "Could not find the requested manifest at: {}",
                &manifest_path.to_str().unwrap()
            )));
        }

        // It's safe to unwrap here
        let reader = BufReader::new(file.unwrap());

        let (tx, rx) = mpsc::channel(4);
        tokio::spawn(async move {
            let mut searching_for_digest = !mr.last_digest.is_empty(); //Looking for a digest iff it's not empty

            let mut sent = 0;
            for line in reader.lines().flatten() {
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
                            Some(Timestamp {
                                seconds: dt.timestamp(),
                                nanos: dt.timestamp_subsec_nanos() as i32,
                            })
                        } else {
                            warn!("Failed to parse timestamp {}", date_str);
                            None
                        };
                        (digest_str, ts)
                    }
                    None => {
                        warn!("No timestamp found in manifest");
                        (line.as_ref(), None)
                    }
                };

                let entry = ManifestHistoryEntry {
                    digest: digest.to_string(),
                    date,
                };
                tx.send(Ok(entry))
                    .await
                    .expect("Error streaming manifest history");

                sent += 1;
                if sent >= mr.limit {
                    break;
                }
            }
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    // Readiness check
    async fn is_ready(
        &self,
        _request: Request<ReadinessRequest>,
    ) -> Result<Response<ReadyStatus>, Status> {
        for path in &[&self.scratch_path, &self.manifests_path, &self.blobs_path] {
            match is_path_writable(path) {
                Ok(true) => {}
                Ok(false) => {
                    return Err(Status::unavailable(format!(
                        "{} is not writable",
                        path.to_string_lossy()
                    )));
                }
                Err(error) => {
                    return Err(Status::unavailable(error.to_string()));
                }
            }
        }

        //All paths writable
        let reply = trow_server::ReadyStatus {
            message: String::from("Ready"),
        };

        Ok(Response::new(reply))
    }

    async fn is_healthy(
        &self,
        _request: Request<HealthRequest>,
    ) -> Result<Response<HealthStatus>, Status> {
        let reply = trow_server::HealthStatus {
            message: String::from("Healthy"),
        };
        Ok(Response::new(reply))
    }

    async fn get_metrics(
        &self,
        _request: Request<MetricsRequest>,
    ) -> Result<Response<MetricsResponse>, Status> {
        match metrics::gather_metrics(&self.blobs_path) {
            Ok(metrics) => {
                let reply = trow_server::MetricsResponse { metrics };
                Ok(Response::new(reply))
            }

            Err(error) => Err(Status::unavailable(error.to_string())),
        }
    }
}
