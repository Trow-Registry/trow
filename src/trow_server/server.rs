use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::str;

use anyhow::{anyhow, Context, Result};
use async_recursion::async_recursion;
use axum::body::Body;
use bytes::Buf;
use futures::future::try_join_all;
use futures::AsyncRead;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{self, Method};
use thiserror::Error;
use tracing::{event, Level};
use trow_server::api_types::{BlobRef, MetricsResponse};

use super::image::RemoteImage;
use super::manifest::{manifest_media_type, Manifest, OCIManifest};
use super::proxy_auth::{ProxyClient, SingleRegistryProxyConfig};
use super::storage::{is_path_writable, StorageBackendError, TrowStorageBackend};
use super::{metrics, ImageValidationConfig, RegistryProxiesConfig};
use crate::registry_interface::blob_storage::Stored;
use crate::registry_interface::catalog_operations::HistoryEntry;
use crate::registry_interface::digest::{Digest, DigestAlgorithm};
use crate::registry_interface::{
    BlobReader, ContentInfo, ManifestReader, StorageDriverError,
};
use crate::trow_server;
use crate::trow_server::api_types::Status;
use crate::trow_server::storage::WriteBlobRangeError;
use crate::types::{self, *};

pub static SUPPORTED_DIGESTS: [&str; 1] = ["sha256"];
static MANIFESTS_DIR: &str = "manifests";
static BLOBS_DIR: &str = "blobs";
static UPLOADS_DIR: &str = "scratch";

pub static PROXY_DIR: &str = "f/"; //Repositories starting with this are considered proxies
static DIGEST_HEADER: &str = "Docker-Content-Digest";

/* Struct implementing callbacks for the Frontend
 *
 * _manifests_path_: path to where the manifests are
 * _layers_path_: path to where blobs are stored
 * _scratch_path_: path to temporary storage for uploads
 *
 * Each "route" gets a clone of this struct.
 * The Arc makes sure they all point to the same data.
 */
#[derive(Clone, Debug)]
pub struct TrowServer {
    pub storage: TrowStorageBackend,
    // deprecated
    manifests_path: PathBuf,
    blobs_path: PathBuf,
    scratch_path: PathBuf,

    pub proxy_registry_config: Option<RegistryProxiesConfig>,
    pub image_validation_config: Option<ImageValidationConfig>,
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
struct Upload {
    repo_name: String,
    uuid: String,
}

// TODO: Each function should have it's own enum of the errors it can return
// There must be a standard pattern for this somewhere...
#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("Invalid repository or tag")]
    InvalidName,
    #[error("Invalid manifest")]
    InvalidManifest,
    #[error("Internal registry error")]
    Internal,
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

fn create_path(data_path: &Path, dir: &str) -> Result<PathBuf, std::io::Error> {
    let dir_path = data_path.join(dir);
    if !dir_path.exists() {
        return match fs::create_dir_all(&dir_path) {
            Ok(_) => Ok(dir_path),
            Err(e) => {
                event!(
                    Level::ERROR,
                    r#"
                Failed to create directory required by trow {:?}
                Please check the parent directory is writable by the trow user.
                {:?}"#,
                    dir_path,
                    e
                );
                Err(e)
            }
        };
    };
    Ok(dir_path)
}

/**
 * Checks a file matches the given digest.
 *
 * TODO: should be able to use range of hashes.
 * TODO: check if using a static for the hasher speeds things up.
 */
fn validate_digest(file: &PathBuf, digest: &Digest) -> Result<()> {
    let f = File::open(file)?;
    let reader = BufReader::new(f);

    let calculated_digest = Digest::try_sha256(reader)?;

    if calculated_digest != *digest {
        event!(
            Level::ERROR,
            "Upload did not match given digest. Was given {} but got {}",
            digest,
            calculated_digest
        );
        return Err(anyhow!(DigestValidationError {
            user_digest: digest.to_string(),
            actual_digest: calculated_digest.to_string(),
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

impl TrowServer {
    pub async fn get_manifest(
        &self,
        name: &str,
        tag: &str,
    ) -> Result<ManifestReader, StorageDriverError> {
        let _rn = name.to_string();
// heeeeeeeeeeeeeeeeeeeeeeeeeeeere
        if name.starts_with("f/") {
            let (image, cfg) = match self.get_remote_image_and_cfg(name, tag) {
                Some(image) => image,
                None =>  {
                    return Err(StorageDriverError::InvalidName(format!("No proxy config found for {name}:{tag}")));
                }
            };
            match self.download_remote_image(image, cfg).await {
                Ok(_digest) => (),
                Err(e) => {
                    event!(Level::ERROR, "Could not download remote image: {e:?}");
                    return Err(StorageDriverError::Internal);
                }
            };
        }

        let man = self.storage.get_manifest(name, tag).await.map_err(|e| {
            event!(Level::WARN, "Error getting manifest: {}", e);
            StorageDriverError::Internal
        })?;
        ManifestReader::new(
            man.get_media_type(),
            man.digest().clone(),
            man.raw().clone(),
        )
        .await
    }

    pub async fn store_manifest<'a>(
        &self,
        name: &str,
        tag: &str,
        data: Body,
    ) -> Result<Digest, StorageDriverError> {
        let repo = name.to_string();

        match self.upload_manifest(&repo, tag, data).await {
            Ok(vm) => Ok(vm.digest().clone()),
            Err(RegistryError::InvalidName) => {
                Err(StorageDriverError::InvalidName(format!("{}:{}", name, tag)))
            }
            Err(RegistryError::InvalidManifest) => Err(StorageDriverError::InvalidManifest),
            Err(_) => Err(StorageDriverError::Internal),
        }
    }

    pub async fn delete_manifest(
        &self,
        _name: &str,
        _digest: &Digest,
    ) -> Result<(), StorageDriverError> {
        unimplemented!("Manifest deletion not yet implemented");
    }

    async fn has_manifest(&self, _name: &str, _algo: &DigestAlgorithm, _reference: &str) -> bool {
        todo!()
    }

    pub async fn get_blob(
        &self,
        name: &str,
        digest: &Digest,
    ) -> Result<BlobReader<impl AsyncRead>, StorageDriverError> {
        let rn = name.to_string();
        let br = self.get_reader_for_blob(&rn, digest).await.map_err(|e| {
            event!(Level::WARN, "Error getting blob: {}", e);
            StorageDriverError::Internal
        })?;

        Ok(br)
    }

    pub async fn store_blob_chunk<'a>(
        &self,
        name: &str,
        upload_uuid: &str,
        data_info: Option<ContentInfo>,
        data: Body,
    ) -> Result<Stored, StorageDriverError> {
        // TODO: check that content length matches the body
        self.storage
            .write_blob_part_stream(
                upload_uuid,
                data.into_data_stream(),
                data_info.map(|d| d.range.0..d.range.1),
            )
            .await
            .map_err(|e| match e {
                WriteBlobRangeError::NotFound => {
                    StorageDriverError::InvalidName(format!("{} {}", name, upload_uuid))
                }
                WriteBlobRangeError::InvalidContentRange => StorageDriverError::InvalidContentRange,
                _ => StorageDriverError::Internal,
            })?;

        Ok(Stored {
            total_stored: 0,
            chunk: 0,
        })
    }

    pub async fn complete_and_verify_blob_upload(
        &self,
        _repo_name: &str,
        session_id: &str,
        digest: &Digest,
    ) -> Result<(), StorageDriverError> {
        self.storage.complete_blob_write(session_id, digest).await
            .map_err(|e| match e {
                StorageBackendError::InvalidDigest => StorageDriverError::InvalidDigest,
                e => {
                    event!(Level::ERROR, "Could not complete upload: {}", e);
                    StorageDriverError::Internal
                }
            })?;
        Ok(())
    }

    // pub async fn delete_blob(&self, name: &str, digest: &Digest) -> Result<(), StorageDriverError> {
    //     event!(
    //         Level::INFO,
    //         "Attempting to delete blob {} in {}",
    //         digest,
    //         name
    //     );
    //     let rn = (name.to_string());

    //     self.delete_blob_local(&rn, digest)
    //         .await
    //         .map_err(|_| StorageDriverError::InvalidDigest)?;
    //     Ok(())
    // }


    // async fn get_catalog(
    //     &self,
    //     start_value: Option<&str>,
    //     num_results: Option<u32>,
    // ) -> Result<Vec<String>, StorageDriverError> {
    //     let num_results = num_results.unwrap_or(u32::MAX);
    //     let start_value = start_value.unwrap_or_default();

    //     self.get_catalog_part(num_results, start_value)
    //         .await
    //         .map_err(|_| StorageDriverError::Internal)
    //         .map(|rc| rc.raw())
    // }

    pub async fn get_tags(
        &self,
        repo: &str,
        start_value: Option<&str>,
        num_results: Option<u32>,
    ) -> Result<Vec<String>, StorageDriverError> {
        let num_results = num_results.unwrap_or(u32::MAX);
        let start_value = start_value.unwrap_or_default();

        self.list_tags(repo, num_results, start_value)
            .await
            .map_err(|_| StorageDriverError::Internal)
    }

    pub async fn get_history(
        &self,
        repo: &str,
        name: &str,
        start_value: Option<&str>,
        num_results: Option<u32>,
    ) -> Result<Vec<HistoryEntry>, StorageDriverError> {
        let num_results = num_results.unwrap_or(u32::MAX);
        let start_value = start_value.unwrap_or_default();

        self.get_manifest_history(repo, name, num_results, start_value)
            .await
            .map_err(|_| StorageDriverError::Internal)
    }

    // async fn validate_admission(
    //     &self,
    //     admission_req: &AdmissionRequest<Pod>,
    //     host_name: &str,
    // ) -> AdmissionResponse {
    //     self.validate_admission_internal(admission_req, host_name)
    //         .await
    //         .unwrap_or_else(|e| {
    //             AdmissionResponse::from(admission_req).deny(format!("Internal error: {}", e))
    //         })
    // }

    // async fn mutate_admission(
    //     &self,
    //     admission_req: &AdmissionRequest<Pod>,
    //     host_name: &str,
    // ) -> AdmissionResponse {
    //     self.mutate_admission_internal(admission_req, host_name)
    //         .await
    //         .unwrap_or_else(|e| {
    //             AdmissionResponse::from(admission_req).deny(format!("Internal error: {}", e))
    //         })
    // }

    // async fn is_healthy(&self) -> bool {
    //     self.is_healthy().await.is_healthy
    // }

    // async fn is_ready(&self) -> bool {
    //     self.is_ready().await.is_ready
    // }

    // async fn get_metrics(
    //     &self,
    // ) -> Result<MetricsResponse, crate::registry_interface::MetricsError> {
    //     self.get_metrics().await.map_err(|_| MetricsError::Internal)
    // }

    // async fn complete_upload(
    //     &self,
    //     repo_name: &str,
    //     uuid: &str,
    //     digest: &Digest,
    // ) -> Result<(), Status> {
    //     event!(
    //         Level::INFO,
    //         "Complete Upload called for repository {} with upload id {} digest {}",
    //         repo_name,
    //         uuid,
    //         digest
    //     );

    //     let req = CompleteRequest {
    //         repo_name: repo_name.to_string(),
    //         uuid: uuid.to_string(),
    //         user_digest: digest.to_string(),
    //     };

    //     self.trow_server.complete_upload(req).await?;

    //     Ok(())
    // }

    async fn upload_manifest(
        &self,
        repo_name: &String,
        reference: &str,
        manifest: Body,
    ) -> Result<types::VerifiedManifest, RegistryError> {
        let man_bytes = axum::body::to_bytes(
            manifest,
            1024 * 1024 * 2, // 2MiB
        )
        .await
        .map_err(|_| RegistryError::Internal)?;

        self.storage
            .write_image_manifest(man_bytes.clone(), repo_name, reference, true)
            .await
            .map_err(|_| RegistryError::Internal)?;

        Ok(VerifiedManifest::new(
            None,
            repo_name.clone(),
            Digest::try_sha256(man_bytes.reader()).unwrap(),
            reference.to_string(),
        ))
    }

    async fn get_reader_for_manifest(
        &self,
        repo_name: &String,
        reference: &str,
    ) -> Result<ManifestReader> {
        event!(
            Level::DEBUG,
            "Getting read location for {} with ref {}",
            repo_name,
            reference
        );

        let man = self
            .storage
            .get_manifest(repo_name, reference)
            .await
            .map_err(|e| {
                event!(Level::WARN, "Error getting manifest: {}", e);
                StorageDriverError::Internal
            })?;

        Ok(ManifestReader::new(
            man.get_media_type(),
            man.digest().clone(),
            man.raw().clone(),
        )
        .await?)
    }

    // async fn get_manifest_history(
    //     &self,
    //     repo_name: &str,
    //     reference: &str,
    //     limit: u32,
    //     last_digest: &str,
    // ) -> Result<ManifestHistory> {
    //     event!(
    //         Level::INFO,
    //         "Getting manifest history for repo {} ref {} limit {} last_digest {}",
    //         repo_name,
    //         reference,
    //         limit,
    //         last_digest
    //     );
    //     let mr = ManifestHistoryRequest {
    //         tag: reference.to_owned(),
    //         repo_name: repo_name.to_string(),
    //         limit,
    //         last_digest: last_digest.to_owned(),
    //     };
    //     let stream = self.trow_server.get_manifest_history(mr).await?;
    //     let mut history = ManifestHistory::new(format!("{}:{}", repo_name, reference));

    //     for entry in stream {
    //         history.insert(entry.digest, entry.date);
    //     }

    //     Ok(history)
    // }

    async fn get_reader_for_blob(
        &self,
        repo_name: &String,
        digest: &Digest,
    ) -> Result<BlobReader<impl AsyncRead>> {
        event!(
            Level::DEBUG,
            "Getting read location for blob {} in {}",
            digest,
            repo_name
        );
        let _br = BlobRef {
            digest: digest.to_string(),
            repo_name: repo_name.clone(),
        };
        let stream = self.storage.get_blob_stream(repo_name, digest).await?;

        let reader = BlobReader::new(digest.clone(), stream).await;
        Ok(reader)
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

impl TrowServer {
    pub fn new(
        data_path: PathBuf,
        proxy_registry_config: Option<RegistryProxiesConfig>,
        image_validation_config: Option<ImageValidationConfig>,
    ) -> Result<Self> {
        let manifests_path = create_path(&data_path, MANIFESTS_DIR)?;
        let scratch_path = create_path(&data_path, UPLOADS_DIR)?;
        let blobs_path = create_path(&data_path, BLOBS_DIR)?;

        let svc = TrowServer {
            manifests_path,
            blobs_path,
            scratch_path,
            proxy_registry_config,
            image_validation_config,
            storage: TrowStorageBackend::new(data_path.into())?,
        };
        Ok(svc)
    }

    fn get_upload_path_for_blob(&self, uuid: &str) -> PathBuf {
        self.scratch_path.join(uuid)
    }

    fn get_catalog_path_for_blob(&self, digest: &Digest) -> Result<PathBuf> {
        Ok(self.blobs_path.join(digest.algo_str()).join(&digest.hash))
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
        digest: &Digest,
    ) -> Result<()> {
        if self.get_catalog_path_for_blob(digest)?.exists() {
            event!(Level::DEBUG, "Already have blob {}", digest);
            return Ok(());
        }
        let addr = format!("{}/blobs/{}", remote_image.get_base_uri(), digest);
        event!(Level::INFO, "Downloading blob {}", addr);
        let resp = cl
            .authenticated_request(Method::GET, &addr)
            .send()
            .await
            .context("GET blob failed")?;
        self.storage
            .write_blob_stream(digest, resp.bytes_stream(), true)
            .await
            .context("Failed to write blob")?;
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
        let mani = Manifest::from_bytes(bytes)?;
        match mani.parsed() {
            OCIManifest::List(_) => {
                let images_to_dl = mani
                    .get_local_asset_digests()?
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
            OCIManifest::V2(_) => {
                let digests: Vec<_> = mani.get_local_asset_digests()?;

                let futures = digests
                    .iter()
                    .map(|digest| self.download_blob(cl, remote_image, digest));
                try_join_all(futures).await?;
            }
        }
        self.storage
            .write_image_manifest(mani.raw(), local_repo_name, &remote_image.reference, false)
            .await?;

        Ok(())
    }

    async fn get_digest_from_header(
        &self,
        cl: &ProxyClient,
        image: &RemoteImage,
    ) -> Option<Digest> {
        let resp = cl
            .authenticated_request(Method::HEAD, &image.get_manifest_url())
            .headers(create_accept_header())
            .send()
            .await;
        match resp.as_ref().map(|r| r.headers().get(DIGEST_HEADER)) {
            Err(e) => {
                event!(
                    Level::ERROR,
                    "Remote registry didn't respond correctly to HEAD request {}",
                    e
                );
                None
            }
            Ok(None) => {
                event!(
                    Level::ERROR,
                    "Remote registry didn't send header {DIGEST_HEADER}",
                );
                None
            }
            Ok(Some(header)) => {
                let digest_str = header.to_str().unwrap();
                let digest = Digest::try_from_raw(digest_str).unwrap();
                Some(digest)
            }
        }
    }

    /// returns the downloaded digest
    async fn download_remote_image(
        &self,
        remote_image: RemoteImage,
        proxy_cfg: SingleRegistryProxyConfig,
    ) -> Result<Digest> {
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
        let remote_img_ref_digest = Digest::try_from_raw(&remote_image.reference);
        let (local_digest, latest_digest) = match remote_img_ref_digest {
            Ok(digest) => (Some(digest), None),
            Err(_) => {
                let local_digest = self
                    .storage
                    .get_manifest_digest(&repo_name, &remote_image.reference)
                    .await
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
            }
        };

        // let (local_digest, latest_digest) = if ref_is_digest {
        //     (Some(remote_image.reference.clone()), None)
        // } else {
        //     let local_digest = self
        //         .storage
        //         .get_manifest_digest(&repo_name, &remote_image.reference)
        //         .await
        //         .ok();
        //     let mut latest_digest = match &try_cl {
        //         Some(cl) => self.get_digest_from_header(cl, &remote_image).await,
        //         _ => None,
        //     };
        //     if latest_digest == local_digest {
        //         if local_digest.is_none() {
        //             anyhow::bail!(
        //                 "Could not fetch digest for {}:{}",
        //                 repo_name,
        //                 remote_image.reference
        //             );
        //         }
        //         // if both are the same, no need to try to pull
        //         latest_digest = None;
        //     }
        //     (local_digest, latest_digest)
        // };

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
                        Ok(_) if !is_digest(&remote_image.reference) => match self
                            .storage
                            .write_tag(&repo_name, &remote_image.reference, &digest)
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

    // async fn create_manifest_read_location(
    //     &self,
    //     repo_name: String,
    //     reference: String,
    //     do_verification: bool,
    // ) -> Result<ManifestReadLocation> {
    //     let path = if let Some((remote_image, proxy_cfg)) =
    //         self.get_remote_image_and_cfg(&repo_name, &reference)
    //     {
    //         event!(
    //             Level::INFO,
    //             "Request for proxied repo {}:{} maps to {}",
    //             repo_name,
    //             reference,
    //             remote_image
    //         );
    //         // These are not up to date and should not be used !
    //         drop(repo_name);
    //         drop(reference);
    //         if self.proxy_registry_config.as_ref().unwrap().offline {
    //             let repo_name = format!("f/{}/{}", proxy_cfg.alias, remote_image.get_repo());
    //             self.get_path_for_manifest(&repo_name, &remote_image.reference)
    //                 .await?
    //         } else {
    //             let digest = self.download_remote_image(remote_image, proxy_cfg).await?;
    //             self.get_catalog_path_for_blob(&digest)?
    //         }
    //     } else {
    //         self.get_path_for_manifest(&repo_name, &reference).await?
    //     };

    //     let vm = self.create_verified_manifest(&path, do_verification)?;
    //     Ok(ManifestReadLocation {
    //         content_type: vm.content_type.to_owned(),
    //         digest: vm.digest,
    //         path: path.to_string_lossy().to_string(),
    //     })
    // }
}

// Registry
impl TrowServer {
    /**
     * TODO: check if blob referenced by manifests. If so, refuse to delete.
     */
    pub async fn delete_blob(&self, name: &str, digest: &Digest) -> Result<BlobDeleted, Status> {
        // if !is_digest(digest) {
        //     return Err(Status::InvalidArgument(format!(
        //         "Invalid digest: {}",
        //         digest
        //     )));
        // }
        match self.storage.delete_blob(digest).await {
            Ok(_) => Ok(BlobDeleted {}),
            Err(e) => {
                event!(
                    Level::ERROR,
                    "Failed to delete blob {} {:?} {:?}",
                    name,
                    digest,
                    e
                );
                Err(Status::Internal("Internal error deleting blob".to_owned()))
            }
        }
    }

    // pub async fn delete_manifest(&self, _mr: ManifestRef) -> Result<ManifestDeleted, Status> {
    //     // event!(Level::ERROR, "Manifest deletion requested but not handled !");
    //     unimplemented!("Manifest deletion not yet implemented")
    //     // Ok(ManifestDeleted {})
    // }

    pub async fn complete_upload(
        &self,
        repo_name: &str,
        uuid: &str,
        digest: &Digest,
    ) -> Result<(), Status> {
        let ret = match self.storage.complete_blob_write(uuid, digest).await {
            Ok(_) => Ok(()),
            Err(StorageBackendError::InvalidDigest) => {
                Err(Status::InvalidArgument("Digest does not match".to_owned()))
            },
            Err(e) => Err(Status::Internal(format!("{e:?}")))
        };

        //delete uuid from uploads tracking
        let _upload = Upload {
            repo_name: repo_name.to_string(),
            uuid: uuid.to_string(),
        };
        ret
    }

    pub async fn get_catalog(
        &self,
        start_value: Option<&str>,
        num_results: Option<u32>,
    ) -> Result<Vec<String>, Status> {
        let mut manifests = self
            .storage
            .list_repos()
            .await
            .map_err(|e| Status::Internal(format!("Internal error streaming catalog: {e}")))?;
        let limit = num_results.unwrap_or(u32::MAX) as usize;
        let manifests = if let Some(repo) = start_value {
            manifests
                .into_iter()
                .skip_while(|m| *m != repo)
                .skip(1)
                .take(limit)
                .collect()
        } else {
            manifests.truncate(limit);
            manifests
        };

        Ok(manifests)
    }

    pub async fn list_tags(
        &self,
        repo_name: &str,
        limit: u32,
        last_tag: &str,
    ) -> Result<Vec<String>, Status> {
        let mut tags = self.storage.list_repo_tags(repo_name).await.map_err(|e| {
            event!(Level::ERROR, "Error listing catalog repo tags {:?}", e);
            Status::Internal("Internal error streaming catalog".to_owned())
        })?;
        tags.sort();
        let limit = limit as usize;

        let partial_catalog: Vec<String> = if last_tag.is_empty() {
            tags.truncate(limit);
            tags
        } else {
            tags.into_iter()
                .skip_while(|t| t != last_tag)
                .skip(1)
                .take(limit)
                .collect()
        };

        Ok(partial_catalog)
    }

    pub async fn get_manifest_history(
        &self,
        repo_name: &str,
        reference: &str,
        limit: u32,
        last_digest: &str,
    ) -> Result<Vec<HistoryEntry>, Status> {
        if is_digest(reference) {
            return Err(Status::InvalidArgument(
                "Require valid tag (not digest) to search for history".to_owned(),
            ));
        }

        let mut manifest_history = self
            .storage
            .get_manifest_history(repo_name, reference)
            .await
            .map_err(|e| {
                event!(Level::ERROR, "Error listing manifest history: {e}");
                Status::Internal("Could not list manifest history".to_owned())
            })?;

        let limit = limit as usize;
        let entries = if last_digest.is_empty() {
            manifest_history.truncate(limit);
            manifest_history
        } else {
            manifest_history
                .into_iter()
                .skip_while(|entry| entry.digest != last_digest)
                .skip(1)
                .take(limit)
                .collect()
        };

        Ok(entries)
    }

    // Readiness check
    pub async fn is_ready(&self) -> bool {
        for path in &[&self.scratch_path, &self.manifests_path, &self.blobs_path] {
            match is_path_writable(path) {
                Ok(true) => {}
                Ok(false) => {
                    event!(Level::WARN, "{} is not writable", path.to_string_lossy());
                    return false;
                }
                Err(error) => {
                    event!(
                        Level::WARN,
                        "Error checking path {}: {}",
                        path.to_string_lossy(),
                        error
                    );
                    return false;
                }
            }
        }

        // All paths writable
        true
    }

    pub async fn is_healthy(&self) -> bool {
        true
    }

    pub async fn get_metrics(&self) -> Result<MetricsResponse, Status> {
        match metrics::gather_metrics(&self.blobs_path) {
            Ok(metrics) => {
                let reply = MetricsResponse { metrics };
                Ok(reply)
            }

            Err(error) => Err(Status::Unavailable(error.to_string())),
        }
    }
}
