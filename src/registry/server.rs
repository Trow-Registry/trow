use std::path::PathBuf;
use std::str;

use anyhow::anyhow;
use anyhow::Result;
use axum::body::Body;
use bytes::Buf;
use bytes::Bytes;
use futures::AsyncRead;
use lazy_static::lazy_static;
use regex::Regex;
use sea_orm::ActiveModelTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityOrSelect;
use sea_orm::ModelTrait;
use sea_orm::QueryOrder;
use sea_orm::QuerySelect;
use thiserror::Error;
use tracing::{event, Level};
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::entity::{Set, NotSet};
use sea_orm::ColumnTrait;

use super::manifest::ManifestReference;
// use super::manifest::Manifest;
use super::proxy::RegistryProxiesConfig;
use super::storage::{StorageBackendError, TrowStorageBackend};
use super::ImageValidationConfig;
use crate::registry::api_types::Status;
use crate::registry::blob_storage::Stored;
use crate::registry::catalog_operations::HistoryEntry;
use crate::registry::digest::Digest;
use crate::registry::storage::WriteBlobRangeError;
use crate::registry::{BlobReader, ContentInfo, ManifestReader, RegistryError};
use crate::types::*;
use crate::entity;

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
    db: DatabaseConnection,
}

impl TrowServer {
    pub fn new(
        db: DatabaseConnection,
        data_path: PathBuf,
        proxy_registry_config: Option<RegistryProxiesConfig>,
        image_validation_config: Option<ImageValidationConfig>,
    ) -> Result<Self> {
        let proxy_registry_config = proxy_registry_config.unwrap_or_default();

        let svc = Self {
            db,
            proxy_registry_config,
            image_validation_config,
            storage: TrowStorageBackend::new(data_path)?,
        };
        Ok(svc)
    }

    pub async fn get_manifest(&self, name: &str, reference: &str) -> Result<ManifestReader> {
        let mut reference = ManifestReference::try_from_str(reference)?;

        if reference.digest().is_none() && name.starts_with(PROXY_DIR) {
            if let Some((proxy_cfg, img)) = self
                .proxy_registry_config
                .get_proxy_config(name, reference)
                .await
            {
                reference = ManifestReference::Digest(proxy_cfg.download_remote_image(&img, self).await?);
            }
        }
        if let Some(ManifestReference::Tag(tag)) = reference {
            let manifest = entity::Manifest::find()
                .inner_join(entity::Tag::Entity)
                .inner_join(entity::repo::Entity)
                .filter(entity::repo::Column::Name.eq(name))
                .filter(entity::tag::Column::Tag.eq(tag))
                .one(&self.db).await?;

            let tag = entity::Tag::find()
                .left_join(entity::repo::Entity)
                .filter(entity::repo::Column::Name.eq(name))
                .filter(entity::tag::Column::Tag.eq(tag))
                .one(&self.db).await?;
            if let Some(tag) = tag {
                reference = ManifestReference::Digest(Digest::try_from_raw(&tag.manifest_digest).unwrap());
            }
        }
        let digest = match digest {
            Some(d) => d,
            None => return Err(anyhow!("Could not find {name}:{reference}"))
        };
        let man = self.storage.get_manifest(name, &digest).await.map_err(|e| {
            event!(Level::WARN, "Error getting manifest: {}", e);
            RegistryError::Internal
        })?;
        Ok(
            ManifestReader::new(
            man.get_media_type(),
            man.digest().clone(),
            man.raw().clone(),
        )
        .await
        )
    }

    pub async fn store_manifest<'a>(
        &self,
        repo: &str,
        reference: &str,
        raw_manifest: Bytes,
    ) -> Result<Digest, RegistryError> {
        if repo.starts_with(PROXY_DIR) {
            return Err(RegistryError::InvalidName(format!(
                "Cannot upload manifest for proxied repo {repo}"
            )));
        }
        let digest = Digest::digest(&mut raw_manifest.reader()).unwrap();
        let manifest = Manifest::from_bytes(raw_manifest).map_err(|_| {
            RegistryError::InvalidManifest
        })?;
        entity::manifest::ActiveModel {
            digest: digest.to_string(),
            repo: repo,
            size: ""

        }

        // entity::Manifest::insert()



        self.storage
            .write_image_manifest(raw_manifest.clone(), repo, reference, true)
            .await
            .map_err(|e| {
                event!(Level::ERROR, "Could not write manifest: {e}");
                RegistryError::Internal
            })?;

        Ok(Digest::digest_sha256(man_bytes.reader()).unwrap())
    }

    pub async fn delete_manifest(
        &self,
        repo_name: &str,
        digest: &Digest,
    ) -> Result<(), RegistryError> {
        event!(Level::WARN, "Manifest deletion is not correctly handled !");

        self.storage
            .delete_manifest(repo_name, digest)
            .await
            .map_err(|e| {
                event!(Level::ERROR, "Failed to delete manifest: {e}");
                RegistryError::Internal
            })
    }

    pub async fn get_blob(
        &self,
        repo_name: &str,
        digest: &Digest,
    ) -> Result<BlobReader<impl AsyncRead>, RegistryError> {
        event!(
            Level::DEBUG,
            "Getting read location for blob {} in {}",
            digest,
            repo_name
        );
        let stream = match self.storage.get_blob_stream(repo_name, digest).await {
            Ok(stream) => stream,
            Err(StorageBackendError::BlobNotFound(_)) => return Err(RegistryError::NotFound),
            Err(_) => return Err(RegistryError::Internal),
        };
        Ok(BlobReader::new(digest.clone(), stream).await)
    }

    pub async fn store_blob_chunk<'a>(
        &self,
        name: &str,
        upload_uuid: &str,
        content_info: Option<ContentInfo>,
        data: Body,
    ) -> Result<Stored, RegistryError> {
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
                    RegistryError::InvalidName(format!("{} {}", name, upload_uuid))
                }
                WriteBlobRangeError::InvalidContentRange => RegistryError::InvalidContentRange,
                _ => RegistryError::Internal,
            })
    }

    pub async fn complete_and_verify_blob_upload(
        &self,
        _repo_name: &str,
        session_id: &str,
        digest: &Digest,
    ) -> Result<(), RegistryError> {
        let upload = entity::Upload::find_by_id(session_id)
            .one(&self.db)
            .await?.ok_or(RegistryError::NotFound)?;

        self.storage
            .complete_blob_write(session_id, digest)
            .await?;
        let blob_size = upload.offset;
        upload.delete(&self.db).await?;
        entity::blob::ActiveModel {
            digest: Set(digest.to_string()),
            size: Set(blob_size),
            last_accessed: NotSet,
            ..Default::default()
        }.insert(&self.db).await?;

        Ok(())
    }

    pub async fn get_tags(
        &self,
        repo: &str,
        start_value: Option<&str>,
        num_results: Option<u32>,
    ) -> Result<Vec<String>, RegistryError> {
        let num_results = num_results.unwrap_or(u32::MAX);
        let start_id = if let Some(val) = start_value {
            entity::Tag::find()
            .filter(entity::tag::Column::Tag.eq(val))
            .column(entity::tag::Column::Id)
            .one(&self.db).await?.ok_or(RegistryError::InvalidName(val.to_string()))?.id
        } else {
            i32::MAX
        };
        entity::Tag::find().order_by("id", "dsc").filter(filter) .limit()

        self.list_tags(repo, num_results, start_value)
            .await
            .map_err(|_| RegistryError::Internal)
    }

    pub async fn get_history(
        &self,
        repo: &str,
        name: &str,
        start_value: Option<&str>,
        num_results: Option<u32>,
    ) -> Result<Vec<HistoryEntry>, RegistryError> {
        let num_results = num_results.unwrap_or(u32::MAX);
        let start_value = start_value.unwrap_or_default();

        self.get_manifest_history(repo, name, num_results, start_value)
            .await
            .map_err(|_| RegistryError::Internal)
    }

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
        let is_digest = Digest::try_from_raw(reference).is_ok();
        if is_digest {
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
