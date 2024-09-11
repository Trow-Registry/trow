use std::path::PathBuf;
use std::str;

use anyhow::{anyhow, Result};
use axum::body::Body;
use futures::AsyncRead;
use sea_orm::entity::{NotSet, Set};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, ModelTrait, QueryFilter,
};
use tracing::{event, Level};

use super::manifest::ManifestReference;
// use super::manifest::Manifest;
use super::proxy::RegistryProxiesConfig;
use super::storage::{StorageBackendError, TrowStorageBackend};
use super::ImageValidationConfig;
use crate::entity;
use crate::registry::api_types::Status;
use crate::registry::blob_storage::Stored;
use crate::registry::digest::Digest;
use crate::registry::storage::WriteBlobRangeError;
use crate::registry::{BlobReader, ContentInfo, ManifestReader, RegistryError};
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

    pub async fn get_manifest(
        &self,
        db: &impl ConnectionTrait,
        name: &str,
        reference: &str,
    ) -> Result<ManifestReader> {
        let mut reference = ManifestReference::try_from_str(reference)?;

        if reference.digest().is_none() && name.starts_with(PROXY_DIR) {
            if let Some((proxy_cfg, img)) = self
                .proxy_registry_config
                .get_proxy_config(name, &reference)
                .await
            {
                reference =
                    ManifestReference::Digest(proxy_cfg.download_remote_image(&img, self).await?);
            }
        }
        if let ManifestReference::Tag(tag) = &reference {
            let tag = entity::tag::Entity::find()
                .filter(entity::tag::Column::Repo.eq(name))
                .filter(entity::tag::Column::Tag.eq(tag))
                .one(db)
                .await?;
            if let Some(tag) = tag {
                reference = ManifestReference::Digest(tag.manifest_digest.clone());
            }
        }
        let digest = match reference {
            ManifestReference::Digest(d) => d,
            ManifestReference::Tag(t) => return Err(anyhow!("Could not find {name}:{t}")),
        };
        let man = self
            .storage
            .get_manifest(name, &digest)
            .await
            .map_err(|e| {
                event!(Level::WARN, "Error getting manifest: {}", e);
                RegistryError::Internal
            })?;
        Ok(ManifestReader::new(
            "zozio".to_string(), // man.media_type().as_ref().unwrap().to_string(),
            digest,
            man,
        )
        .await)
    }

    // pub async fn store_manifest<'a>(
    //     &self,
    //     repo: &str,
    //     reference: &str,
    //     raw_manifest: Bytes,
    // ) -> Result<Digest, RegistryError> {
    //     if repo.starts_with(PROXY_DIR) {
    //         return Err(RegistryError::InvalidName(format!(
    //             "Cannot upload manifest for proxied repo {repo}"
    //         )));
    //     }
    //     let _digest = Digest::digest_sha256(&mut raw_manifest.clone().reader()).unwrap();
    //     let _manifest: OCIManifest =
    //         serde_json::from_slice(&raw_manifest).map_err(|_| RegistryError::InvalidManifest)?;
    //     // entity::manifest::ActiveModel {
    //     //     digest: digest.to_string(),
    //     //     repo: repo,
    //     //     size: ""

    //     // }

    //     // entity::Manifest::insert()

    //     self.storage
    //         .write_image_manifest(raw_manifest.clone(), repo, reference)
    //         .await
    //         .map_err(|e| {
    //             event!(Level::ERROR, "Could not write manifest: {e}");
    //             RegistryError::Internal
    //         })?;

    //     Ok(Digest::digest_sha256(raw_manifest.reader()).unwrap())
    // }

    // pub async fn delete_manifest(
    //     &self,
    //     repo_name: &str,
    //     digest: &str,
    // ) -> Result<(), RegistryError> {
    //     event!(Level::WARN, "Manifest deletion is not correctly handled !");

    //     self.storage
    //         .delete_manifest(repo_name, digest)
    //         .await
    //         .map_err(|e| {
    //             event!(Level::ERROR, "Failed to delete manifest: {e}");
    //             RegistryError::Internal
    //         })
    // }

    pub async fn get_blob(
        &self,
        repo_name: &str,
        digest: &str,
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
        Ok(BlobReader::new(digest.to_string(), stream).await)
    }

    pub async fn store_blob_chunk<'a>(
        &self,
        name: &str,
        upload_uuid: &uuid::Uuid,
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

    // pub async fn complete_and_verify_blob_upload(
    //     &self,
    //     db: &impl ConnectionTrait,
    //     _repo_name: &str,
    //     session_id: &uuid::Uuid,
    //     digest: &str,
    // ) -> Result<(), RegistryError> {
    //     let upload = entity::blob_upload::Entity::find_by_id(*session_id)
    //         .one(db)
    //         .await?
    //         .ok_or(RegistryError::NotFound)?;

    //     self.storage
    //         .complete_blob_write(&session_id, digest)
    //         .await?;
    //     let blob_size = upload.offset;
    //     upload.delete(db).await?;
    //     entity::blob::ActiveModel {
    //         digest: Set(digest.to_string()),
    //         size: Set(blob_size),
    //         last_accessed: NotSet,
    //         ..Default::default()
    //     }
    //     .insert(db)
    //     .await?;

    //     Ok(())
    // }

    /**
     * TODO: check if blob referenced by manifests. If so, refuse to delete.
     */
    pub async fn delete_blob(&self, name: &str, digest: &str) -> Result<BlobDeleted, Status> {
        // if !is_digest(digest) {
        //     return Err(Status::InvalidArgument(format!(
        //         "Invalid digest: {}",
        //         digest
        //     )));
        // }
        match self.storage.delete_blob(name, digest).await {
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
