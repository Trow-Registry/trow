mod admission;
pub mod api_types;
pub mod manifest;
mod proxy;
pub mod server;
mod storage;
mod temporary_file;

use std::path::PathBuf;

pub use admission::ImageValidationConfig;
use anyhow::Result;
pub use blob_storage::{BlobReader, ContentInfo, UploadInfo};
pub use catalog_operations::ManifestHistory;
pub use digest::{Digest, DigestAlgorithm};
pub use manifest_storage::ManifestReader;
pub use proxy::{RegistryProxiesConfig, SingleRegistryProxyConfig};
pub use server::TrowServer;
pub use storage::StorageBackendError;
use thiserror::Error;
use tracing::{event, Level};

pub mod blob_storage;
pub mod catalog_operations;
#[allow(dead_code)]
pub mod digest;
pub mod manifest_storage;

// Storage Driver Error
#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("the name `{0}` is not valid")]
    InvalidName(String),
    #[error("manifest is not valid")]
    InvalidManifest,
    #[error("Digest did not match content")]
    InvalidDigest,
    #[error("Unsupported Operation")]
    Unsupported,
    #[error("Requested index does not match actual")]
    InvalidContentRange,
    #[error("Internal server error")]
    Internal,
    #[error("Not found")]
    NotFound,
}

impl From<sea_orm::DbErr> for RegistryError {
    fn from(err: sea_orm::DbErr) -> Self {
        event!(Level::ERROR, "Database error: {err:?}");
        Self::Internal
    }
}

impl From<StorageBackendError> for RegistryError {
    fn from(err: StorageBackendError) -> Self {
        match err {
            StorageBackendError::BlobNotFound(_) => Self::NotFound,
            StorageBackendError::Internal(e) => {
                event!(Level::ERROR, "Internal storage error: {e}");
                Self::Internal
            }
            StorageBackendError::InvalidContentRange => Self::InvalidContentRange,
            StorageBackendError::InvalidDigest => Self::InvalidDigest,
            StorageBackendError::InvalidManifest(_msg) => Self::InvalidManifest,
            StorageBackendError::InvalidName(name) => Self::InvalidName(name),
            StorageBackendError::Io(e) => {
                event!(Level::ERROR, "Internal IO error: {e:?}");
                Self::Internal
            }
            StorageBackendError::Unsupported => Self::Unsupported,
        }
    }
}

pub struct TrowServerBuilder {
    data_path: PathBuf,
    proxy_registry_config: Option<RegistryProxiesConfig>,
    image_validation_config: Option<ImageValidationConfig>,
}

pub fn build_server(
    data_path: PathBuf,
    proxy_registry_config: Option<RegistryProxiesConfig>,
    image_validation_config: Option<ImageValidationConfig>,
) -> TrowServerBuilder {
    TrowServerBuilder {
        data_path,
        proxy_registry_config,
        image_validation_config,
    }
}

impl TrowServerBuilder {
    pub async fn get_server(self) -> Result<TrowServer> {
        TrowServer::new(
            self.data_path,
            self.proxy_registry_config,
            self.image_validation_config,
        )
    }
}
