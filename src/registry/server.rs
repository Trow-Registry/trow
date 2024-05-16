use std::path::PathBuf;
use std::str;

use anyhow::Result;
use axum::body::Body;
use bytes::Buf;
use futures::AsyncRead;
use thiserror::Error;
use tracing::{event, Level};

use super::proxy::RegistryProxiesConfig;
use super::storage::{StorageBackendError, TrowStorageBackend};
use super::ImageValidationConfig;
use crate::registry::api_types::Status;
use crate::registry::blob_storage::Stored;
use crate::registry::catalog_operations::HistoryEntry;
use crate::registry::digest::Digest;
use crate::registry::storage::WriteBlobRangeError;
use crate::registry::{BlobReader, ContentInfo, ManifestReader, StorageDriverError};
use crate::types::*;

pub static SUPPORTED_DIGESTS: [&str; 1] = ["sha256"];

pub static PROXY_DIR: &str = "f/"; //Repositories starting with this are considered proxies

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
    pub proxy_registry_config: RegistryProxiesConfig,
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
        let get_manifest = if name.starts_with(PROXY_DIR) {
            if let Some((proxy_cfg, img)) = self
                .proxy_registry_config
                .get_proxy_config(name, reference)
                .await
            {
                let digest = proxy_cfg.download_remote_image(&img, self).await?;
                self.storage
                    .get_manifest("(fixme: none)", &digest.to_string())
                    .await
            } else {
                self.storage.get_manifest(name, reference).await
            }
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
                content_info.map(|d| d.range.0..=d.range.1),
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
        let proxy_registry_config = proxy_registry_config.unwrap_or_default();

        let svc = Self {
            proxy_registry_config,
            image_validation_config,
            storage: TrowStorageBackend::new(data_path)?,
        };
        Ok(svc)
    }
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
