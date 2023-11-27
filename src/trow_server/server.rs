use std::collections::HashSet;
use std::fs::{self, DirEntry, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::{io, str};

use anyhow::{anyhow, Context, Result};
use async_recursion::async_recursion;
use chrono::prelude::*;
use futures::TryFutureExt;
use futures::future::try_join_all;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{self, Method};
use crate::registry_interface::Metrics;

use super::errors::{DigestValidationError, ProxyError};
use tokio::time::Duration;
use tracing::{event, Level};
use uuid::Uuid;
use object_store::ObjectStore;
use tokio::io::AsyncWriteExt;
use futures::stream::StreamExt;

use super::api_types::*;
use super::digest::sha256_tag_digest;
use super::image::RemoteImage;
use super::manifest::{manifest_media_type, FromJson, Manifest};
use super::proxy_auth::{ProxyClient, SingleRegistryProxyConfig};
use super::temporary_file::TemporaryFile;
use super::{metrics, ImageValidationConfig, RegistryProxiesConfig};
use super::storage::TrowStorageBackend;

pub static SUPPORTED_DIGESTS: [&str; 1] = ["sha256"];
static MANIFESTS_DIR: &str = "manifests";
static BLOBS_DIR: &str = "blobs";
static UPLOADS_DIR: &str = "scratch";

static PROXY_DIR: &str = "f/"; //Repositories starting with this are considered proxies
static DIGEST_HEADER: &str = "Docker-Content-Digest";


/// Struct implementing callbacks for the Frontend
#[derive(Clone, Debug)]
pub struct TrowServer {
    storage: TrowStorageBackend,
    /// active_uploads: a HashSet of all uuids that are currently being tracked
    active_uploads: Arc<RwLock<HashSet<Upload>>>,
    pub proxy_registry_config: Option<RegistryProxiesConfig>,
    pub image_validation_config: Option<ImageValidationConfig>,
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


pub fn is_digest(maybe_digest: &str) -> bool {
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
        proxy_registry_config: Option<RegistryProxiesConfig>,
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
        Ok(PathBuf::from("blobs").join(alg).join(val))
    }

    fn get_digest_from_manifest(&self, repo_name: &str, reference: &str) -> Result<String> {
        get_digest_from_manifest_path(self.manifests_path.join(repo_name).join(reference))
    }

    async fn save_tag(&self, digest: &str, repo_name: &str, tag: &str) -> Result<()> {
        // Tag files should contain list of digests with timestamp
        // Last line should always be the current digest

        let repo_dir = self.manifests_path.join(repo_name);
        // fs::create_dir_all(&repo_dir)?;

        let ts = Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true);
        let contents = format!("{} {}\n", digest, ts).into_bytes();

        self.file_locks
            .write()
            .unwrap()
            .insert(repo_name.to_string());

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
        manifest_uuid: &str,
        verify_assets_exist: bool,
    ) -> Result<VerifiedManifest> {
        let manifest_bytes = std::fs::read(manifest_path)?;
        let manifest_json: serde_json::Value =
            serde_json::from_slice(&manifest_bytes).context("not valid json")?;
        let manifest = Manifest::from_json(&manifest_json).context("not a valid manifest")?;

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
    ) -> Option<(RemoteImage, SingleRegistryProxyConfig)> {
        // All proxies are under "f_"
        if repo_name.starts_with(PROXY_DIR) && self.proxy_registry_config.is_some() {
            let proxy_config = self.proxy_registry_config.as_ref().unwrap();

            let segments = repo_name.splitn(3, '/').collect::<Vec<_>>();
            debug_assert_eq!(segments[0], "f");
            let proxy_alias = segments[1].to_string();
            let repo = segments[2].to_string();

            for proxy in proxy_config.registries.iter() {
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
            event!(Level::DEBUG, "Already have blob {}", digest);
            return Ok(());
        }
        let addr = format!("{}/blobs/{}", remote_image.get_base_uri(), digest);
        event!(Level::INFO, "Downloading blob {}", addr);
        let resp = cl.authenticated_request(Method::GET, &addr).send().await.context("GET blob failed")?;
        self.storage.write_blob_stream(digest, resp.bytes_stream()).await.context("Failed to write blob")?;

        Ok(())
    }

    #[async_recursion]
    async fn download_manifest_and_layers(
        &self,
        cl: &ProxyClient,
        remote_image: &RemoteImage,
        local_repo_name: &str,
    ) -> Result<()> {
        event!(
            Level::DEBUG,
            "Downloading manifest + layers for {}",
            remote_image
        );
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
        let bytes = resp.bytes().await?;

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

        self.storage.write_image_manifest(bytes, local_repo_name, &remote_image.reference).await?;

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
                event!(
                    Level::ERROR,
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
        proxy_cfg: SingleRegistryProxyConfig,
    ) -> Result<String> {
        // Replace eg f/docker/alpine by f/docker/library/alpine
        let repo_name = format!("f/{}/{}", proxy_cfg.alias, remote_image.get_repo());

        let try_cl = match ProxyClient::try_new(proxy_cfg.clone(), &remote_image).await {
            Ok(cl) => Some(cl),
            Err(e) => {
                event!(
                    Level::ERROR,
                    "Could not create client for proxied registry {}: {:?}",
                    proxy_cfg.host,
                    e
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
                                event!(
                                    Level::DEBUG,
                                    "Internal error updating tag for proxied image ({})",
                                    e
                                )
                            }
                        },
                        Ok(_) => return Ok(digest),
                        Err(e) => event!(Level::WARN, "Failed to download proxied image: {}", e),
                    };
                }
                false => event!(
                    Level::WARN,
                    "Missing manifest for proxied image, proxy client not available"
                ),
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
            event!(
                Level::INFO,
                "Request for proxied repo {}:{} maps to {}",
                repo_name,
                reference,
                remote_image
            );
            // These are not up to date and should not be used !
            drop(repo_name);
            drop(reference);
            if self.proxy_registry_config.as_ref().unwrap().offline {
                let repo_name = format!("f/{}/{}", proxy_cfg.alias, remote_image.get_repo());
                self.get_path_for_manifest(&repo_name, &remote_image.reference)?
            } else {
                let digest = self.download_remote_image(remote_image, proxy_cfg).await?;
                self.get_catalog_path_for_blob(&digest)?
            }
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
        event!(Level::DEBUG, "Saving blob {}", user_digest);

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

// Registry
impl TrowServer {
    pub async fn request_upload(&self, ur: UploadRequest) -> Result<UploadDetails, Status> {
        let repo_name = ur.repo_name;
        if self.is_writable_repo(&repo_name) {
            let uuid = Uuid::new_v4().to_string();
            let reply = UploadDetails { uuid: uuid.clone() };
            let upload = Upload { repo_name, uuid };
            {
                self.active_uploads.write().unwrap().insert(upload);
                event!(Level::DEBUG, "Upload Table: {:?}", self.active_uploads);
            }
            Ok(reply)
        } else {
            Err(Status::InvalidArgument(format!(
                "Repository {} is not writable",
                repo_name
            )))
        }
    }

    pub async fn get_write_sync_for_blob(
        &self,
        br: UploadRef,
    ) {
        let upload = Upload {
            repo_name: br.repo_name.clone(),
            uuid: br.uuid.clone(),
        };

        // "We unwrap() the return value to assert that we are not expecting
        // threads to ever fail while holding the lock."
        let locked_set = self.active_uploads.read().unwrap();

        let a = if locked_set.contains(&upload) {
            drop(locked_set);
            let path = self.get_upload_path_for_blob(&br.uuid);
            Ok(WriteLocation {
                path: path.to_string_lossy().to_string(),
            })
        } else {
            Err(Status::FailedPrecondition(format!(
                "No current upload matching {:?}",
                br
            )))
        }

        Ok(())
    }

    pub async fn get_write_location_for_blob(
        &self,
        br: UploadRef,
    ) -> Result<WriteLocation, Status> {
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
            Ok(WriteLocation {
                path: path.to_string_lossy().to_string(),
            })
        } else {
            Err(Status::FailedPrecondition(format!(
                "No current upload matching {:?}",
                br
            )))
        }
    }

    pub async fn get_read_location_for_blob(
        &self,
        br: BlobRef,
    ) -> Result<BlobReadLocation, Status> {
        metrics::TOTAL_BLOB_REQUESTS.inc();
        let path = self
            .get_catalog_path_for_blob(&br.digest)
            .map_err(|e| Status::InvalidArgument(format!("Error parsing digest {:?}", e)))?;

        if !path.exists() {
            event!(Level::WARN, "Request for unknown blob: {:?}", path);
            Err(Status::NotFound(format!("No blob found matching {:?}", br)))
        } else {
            Ok(BlobReadLocation {
                path: path.to_string_lossy().to_string(),
            })
        }
    }

    /**
     * TODO: check if blob referenced by manifests. If so, refuse to delete.
     */
    pub async fn delete_blob(&self, br: BlobRef) -> Result<BlobDeleted, Status> {
        let path = self
            .get_catalog_path_for_blob(&br.digest)
            .map_err(|e| Status::InvalidArgument(format!("Error parsing digest {:?}", e)))?;
        if !path.exists() {
            event!(Level::WARN, "Request for unknown blob: {:?}", path);
            Err(Status::NotFound(format!("No blob found matching {:?}", br)))
        } else {
            fs::remove_file(&path)
                .map_err(|e| {
                    event!(Level::ERROR, "Failed to delete blob {:?} {:?}", br, e);
                    Status::Internal("Internal error deleting blob".to_owned())
                })
                .and(Ok(BlobDeleted {}))
        }
    }

    pub async fn delete_manifest(&self, mr: ManifestRef) -> Result<ManifestDeleted, Status> {
        if !is_digest(&mr.reference) {
            return Err(Status::InvalidArgument(format!(
                "Manifests can only be deleted by digest. Got {}",
                mr.reference
            )));
        }
        let digest = mr.reference;
        //For the repo, go through all tags and see if they reference the digest. Delete them.
        //Can only delete manifest if no other tags in any repo reference it

        let ri = RepoIterator::new(&self.manifests_path.join(&mr.repo_name)).map_err(|e| {
            event!(Level::ERROR, "Problem reading manifest catalog {:?}", e);
            Status::FailedPrecondition("Repository not found".to_owned())
        })?;

        //TODO: error if no manifest matches?
        ri.filter(|de| does_manifest_match_digest(de, &digest))
            .for_each(|man| match fs::remove_file(man.path()) {
                Ok(_) => (),
                Err(e) => event!(Level::DEBUG, "Failed to delete manifest {:?} {:?}", &man, e),
            });

        Ok(ManifestDeleted {})
    }

    pub async fn get_write_details_for_manifest(
        &self,
        mr: ManifestRef,
    ) -> Result<ManifestWriteDetails, Status> {
        let repo_name = mr.repo_name;
        if self.is_writable_repo(&repo_name) {
            //Give the manifest a UUID and save it to the uploads dir
            let uuid = Uuid::new_v4().to_string();

            let manifest_path = self.get_upload_path_for_blob(&uuid);
            Ok(ManifestWriteDetails {
                path: manifest_path.to_string_lossy().to_string(),
                uuid,
            })
        } else {
            Err(Status::InvalidArgument(format!(
                "Repository {} is not writable",
                repo_name
            )))
        }
    }

    pub async fn get_read_location_for_manifest(
        &self,
        mr: ManifestRef,
    ) -> Result<ManifestReadLocation, Status> {
        //Don't actually need to verify here; could set to false

        metrics::TOTAL_MANIFEST_REQUESTS.inc();
        match self
            .create_manifest_read_location(mr.repo_name, mr.reference, true)
            .await
        {
            Ok(vm) => Ok(vm),
            Err(e) => {
                event!(Level::WARN, "Internal error with manifest: {:?}", e);
                Err(Status::Internal(
                    "Internal error finding manifest".to_owned(),
                ))
            }
        }
    }


    async fn get_blob(&self, uuid: String) {
        self.data_store.

    }

    /**
     * Take uploaded manifest (which should be uuid in uploads), check it, put in catalog and
     * by blob digest
     */
    pub async fn verify_manifest(
        &self,
        req: VerifyManifestRequest,
    ) -> Result<VerifiedManifest, Status> {
        let mr = req.manifest.unwrap(); // Pissed off that the manifest is optional!
        let uploaded_manifest = self.get_upload_path_for_blob(&req.uuid);

        match self.create_verified_manifest(&uploaded_manifest, true) {
            Ok(vm) => {
                // copy manifest to blobs and add tag
                let digest = vm.digest.clone();
                self.save_blob(&uploaded_manifest, &digest)
                    .and(self.save_tag(&digest, &mr.repo_name, &mr.reference).await)
                    .map(|_| vm)
                    .map_err(|e| {
                        event!(
                            Level::ERROR,
                            "Failure cataloguing manifest {}/{} {:?}",
                            &mr.repo_name,
                            &mr.reference,
                            e
                        );
                        Status::Internal("Internal error copying manifest".to_owned())
                    })
            }
            Err(e) => {
                event!(Level::ERROR, "Error verifying manifest: {:?}", e);
                Err(Status::InvalidArgument(
                    "Failed to verify manifest".to_owned(),
                ))
            }
        }
    }

    pub async fn complete_upload(&self, cr: CompleteRequest) -> Result<CompletedUpload, Status> {
        let ret = match self.validate_and_save_blob(&cr.user_digest, &cr.uuid) {
            Ok(_) => Ok(CompletedUpload {
                digest: cr.user_digest.clone(),
            }),
            Err(e) => match e.downcast::<DigestValidationError>() {
                Ok(v_e) => Err(Status::InvalidArgument(v_e.to_string())),
                Err(e) => {
                    event!(Level::WARN, "Failure when saving layer: {:?}", e);
                    Err(Status::Internal("Internal error saving layer".to_owned()))
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
            event!(Level::WARN, "Upload {:?} not found when deleting", upload);
        }
        ret
    }

    pub async fn get_catalog(&self, cr: CatalogRequest) -> Result<Vec<CatalogEntry>, Status> {
        let mut manifests = self.storage.list_repos().await.map_err(|e| {
            Status::Internal(format!("Internal error streaming catalog: {e}"))
        })?;
        let limit = cr.limit as usize;
        let manifests = if !cr.last_repo.is_empty() {
            manifests.truncate(limit);
            manifests
        } else {
            manifests.into_iter().skip_while(|m| *m != cr.last_repo).skip(1).take(limit).collect()
        };

        Ok(manifests
            .into_iter()
            .map(|repo_name| CatalogEntry { repo_name })
            .collect())
    }

    pub async fn list_tags(&self, ltr: ListTagsRequest) -> Result<Vec<Tag>, Status> {
        let limit = ltr.limit as usize;

        let mut tags = self.storage.list_repo_tags(&ltr.repo_name).await.map_err(|e| {
            event!(Level::ERROR, "Error listing catalog repo tags {:?}", e);
            Status::Internal("Internal error streaming catalog".to_owned())
        })?;
        tags.sort();

        let partial_catalog: Vec<String> = if ltr.last_tag.is_empty() {
            tags.truncate(limit);
            tags
        } else {
            tags
                .into_iter()
                .skip_while(|t| t != &ltr.last_tag)
                .skip(1)
                .take(limit)
                .collect()
        };

        Ok(partial_catalog
            .into_iter()
            .map(|tag| Tag {
                tag,
            })
            .collect())
    }

    pub async fn get_manifest_history(
        &self,
        mr: ManifestHistoryRequest,
    ) -> Result<Vec<ManifestHistoryEntry>, Status> {
        if is_digest(&mr.tag) {
            return Err(Status::InvalidArgument(
                "Require valid tag (not digest) to search for history".to_owned(),
            ));
        }
        let mut manifest_history = self.storage.get_manifest_history(&mr.repo_name, &mr.tag).await.map_err(|e| {
            event!(Level::ERROR, "Error listing manifest history: {e}");
            Status::Internal("Could not list manifest history".to_owned())
        })?;

        let limit = mr.limit as usize;

        let entries = if !mr.last_digest.is_empty() {
            manifest_history.truncate(limit);
            manifest_history
        } else {
            manifest_history.into_iter().skip_while(|entry| entry.digest != mr.last_digest).skip(1).take(limit).collect()
        };

        Ok(entries)
    }

    // Readiness check
    pub async fn is_ready(&self) -> ReadyStatus {
        ReadyStatus {
            is_ready: true,
            message: String::from("Ready"),
        }
    }

    pub async fn is_healthy(&self) -> HealthStatus {
        HealthStatus {
            is_healthy: true,
            message: String::from("Healthy"),
        }
    }

    pub async fn get_metrics(&self, _request: MetricsRequest) -> Result<MetricsResponse, Status> {
        Ok(MetricsResponse { metrics: String::new()})
    }
}
