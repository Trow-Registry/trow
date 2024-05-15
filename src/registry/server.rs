use std::path::PathBuf;
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

use super::image::RemoteImage;
use super::manifest::{manifest_media_type, Manifest, OCIManifest};
use super::proxy_auth::{ProxyClient, SingleRegistryProxyConfig};
use super::storage::{StorageBackendError, TrowStorageBackend};
use super::{ImageValidationConfig, RegistryProxiesConfig};
use crate::registry::api_types::Status;
use crate::registry::blob_storage::Stored;
use crate::registry::catalog_operations::HistoryEntry;
use crate::registry::digest::Digest;
use crate::registry::storage::WriteBlobRangeError;
use crate::registry::{BlobReader, ContentInfo, ManifestReader, StorageDriverError};
use crate::types::*;

pub static SUPPORTED_DIGESTS: [&str; 1] = ["sha256"];

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
    pub proxy_registry_config: Option<RegistryProxiesConfig>,
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

pub fn is_digest(maybe_digest: &str) -> bool {
    for alg in &SUPPORTED_DIGESTS {
        if maybe_digest.starts_with(&format!("{}:", alg)) {
            return true;
        }
    }

    false
}

impl TrowServer {
    pub async fn get_manifest(&self, name: &str, reference: &str) -> Result<ManifestReader> {
        let get_manifest = if name.starts_with("f/") {
            let (image, cfg) = match self.get_remote_image_and_cfg(name, reference) {
                Some(image) => image,
                None => {
                    return Err(anyhow!("No proxy config found for {name}:{reference}"));
                }
            };
            let digest = match self.download_remote_image(&image, &cfg).await {
                Ok(d) => d,
                Err(e) => {
                    event!(Level::ERROR, "Could not download remote image: {e:?}");
                    return Err(anyhow!("Could not download remote image: {e:?}"));
                }
            };
            self.storage
                .get_manifest("(fixme: none)", &digest.to_string())
                .await
        } else {
            self.storage.get_manifest(name, reference).await
        };

        let man = get_manifest.map_err(|e| {
            event!(Level::WARN, "Error getting manifest: {}", e);
            StorageDriverError::Internal
        })?;
        Ok(ManifestReader::new(
            man.get_media_type(),
            man.digest().clone(),
            man.raw().clone(),
        )
        .await)
    }

    pub async fn store_manifest<'a>(
        &self,
        repo: &str,
        reference: &str,
        data: Body,
    ) -> Result<Digest, StorageDriverError> {
        if repo.starts_with("f/") {
            return Err(StorageDriverError::InvalidName(format!(
                "Cannot upload manifest for proxied repo {repo}"
            )));
        }

        let man_bytes = axum::body::to_bytes(
            data,
            1024 * 1024 * 2, // 2MiB
        )
        .await
        .map_err(|_| StorageDriverError::InvalidManifest)?;

        self.storage
            .write_image_manifest(man_bytes.clone(), repo, reference, true)
            .await
            .map_err(|e| {
                event!(Level::ERROR, "Could not write manifest: {e}");
                StorageDriverError::Internal
            })?;

        Ok(Digest::digest_sha256(man_bytes.reader()).unwrap())
    }

    pub async fn delete_manifest(
        &self,
        repo_name: &str,
        digest: &Digest,
    ) -> Result<(), StorageDriverError> {
        event!(Level::WARN, "Manifest deletion is not correctly handled !");

        self.storage
            .delete_manifest(repo_name, digest)
            .await
            .map_err(|e| {
                event!(Level::ERROR, "Failed to delete manifest: {e}");
                StorageDriverError::Internal
            })
    }

    pub async fn get_blob(
        &self,
        repo_name: &str,
        digest: &Digest,
    ) -> Result<BlobReader<impl AsyncRead>, StorageDriverError> {
        event!(
            Level::DEBUG,
            "Getting read location for blob {} in {}",
            digest,
            repo_name
        );
        let stream = match self.storage.get_blob_stream(repo_name, digest).await {
            Ok(stream) => stream,
            Err(StorageBackendError::BlobNotFound(_)) => return Err(StorageDriverError::NotFound),
            Err(_) => return Err(StorageDriverError::Internal),
        };
        Ok(BlobReader::new(digest.clone(), stream).await)
    }

    pub async fn store_blob_chunk<'a>(
        &self,
        name: &str,
        upload_uuid: &str,
        content_info: Option<ContentInfo>,
        data: Body,
    ) -> Result<Stored, StorageDriverError> {
        // TODO: check that content length matches the body
        self.storage
            .write_blob_part_stream(
                upload_uuid,
                data.into_data_stream(),
                content_info.map(|d| d.range.0..d.range.1),
            )
            .await
            .map_err(|e| match e {
                WriteBlobRangeError::NotFound => {
                    StorageDriverError::InvalidName(format!("{} {}", name, upload_uuid))
                }
                WriteBlobRangeError::InvalidContentRange => StorageDriverError::InvalidContentRange,
                _ => StorageDriverError::Internal,
            })
    }

    pub async fn complete_and_verify_blob_upload(
        &self,
        _repo_name: &str,
        session_id: &str,
        digest: &Digest,
    ) -> Result<(), StorageDriverError> {
        self.storage
            .complete_blob_write(session_id, digest)
            .await
            .map_err(|e| match e {
                StorageBackendError::InvalidDigest => StorageDriverError::InvalidDigest,
                e => {
                    event!(Level::ERROR, "Could not complete upload: {}", e);
                    StorageDriverError::Internal
                }
            })?;
        Ok(())
    }

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
}

impl TrowServer {
    pub fn new(
        data_path: PathBuf,
        proxy_registry_config: Option<RegistryProxiesConfig>,
        image_validation_config: Option<ImageValidationConfig>,
    ) -> Result<Self> {
        let svc = Self {
            proxy_registry_config,
            image_validation_config,
            storage: TrowStorageBackend::new(data_path)?,
        };
        Ok(svc)
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
        if self
            .storage
            .get_blob_stream(remote_image.get_repo(), digest)
            .await
            .is_ok()
        {
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
        remote_image: &RemoteImage,
        proxy_cfg: &SingleRegistryProxyConfig,
    ) -> Result<Digest> {
        // Replace eg f/docker/alpine by f/docker/library/alpine
        let repo_name = format!("f/{}/{}", proxy_cfg.alias, remote_image.get_repo());

        let try_cl = match ProxyClient::try_new(proxy_cfg.clone(), remote_image).await {
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
            // The ref is a digest, no need to map tg to digest
            Ok(digest) => (Some(digest), None),
            // The ref is a tag, let's search for the digest
            Err(_) => {
                let local_digest = self
                    .storage
                    .get_manifest_digest(&repo_name, &remote_image.reference)
                    .await
                    .ok();
                let latest_digest = match &try_cl {
                    Some(cl) => self.get_digest_from_header(cl, remote_image).await,
                    _ => None,
                };
                if latest_digest == local_digest && local_digest.is_none() {
                    anyhow::bail!(
                        "Could not fetch digest for {}:{}",
                        repo_name,
                        remote_image.reference
                    );
                }
                (local_digest, latest_digest)
            }
        };

        let manifest_digests = [latest_digest, local_digest].into_iter().flatten();
        for mani_digest in manifest_digests {
            let have_manifest = self
                .storage
                .get_blob_stream("(fixme: none)", &mani_digest)
                .await
                .is_ok();
            if have_manifest {
                return Ok(mani_digest);
            }
            if try_cl.is_none() {
                event!(
                    Level::WARN,
                    "Missing manifest for proxied image, proxy client not available"
                );
            }
            if let Some(cl) = &try_cl {
                let img_ref_as_digest = Digest::try_from_raw(&remote_image.reference);
                let manifest_download = self
                    .download_manifest_and_layers(cl, remote_image, &repo_name)
                    .await;
                match (manifest_download, img_ref_as_digest) {
                    (Err(e), _) => {
                        event!(Level::WARN, "Failed to download proxied image: {}", e)
                    }
                    (Ok(()), Err(_)) => {
                        let write_tag = self
                            .storage
                            .write_tag(&repo_name, &remote_image.reference, &mani_digest)
                            .await;
                        match write_tag {
                            Ok(_) => return Ok(mani_digest),
                            Err(e) => event!(
                                Level::DEBUG,
                                "Internal error updating tag for proxied image ({})",
                                e
                            ),
                        }
                    }
                    (Ok(()), Ok(_)) => return Ok(mani_digest),
                }
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
            }
            Err(e) => Err(Status::Internal(format!("{e:?}"))),
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
        let manifests = match start_value {
            Some(repo) if !repo.is_empty() => manifests
                .into_iter()
                .skip_while(|m| *m != repo)
                .skip(1)
                .take(limit)
                .collect(),
            _ => {
                manifests.truncate(limit);
                manifests
            }
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
        match self.storage.is_ready().await {
            Ok(()) => true,
            Err(e) => {
                event!(Level::ERROR, "Storage backend not ready: {e}");
                false
            }
        }
    }

    pub async fn is_healthy(&self) -> bool {
        true
    }
}
