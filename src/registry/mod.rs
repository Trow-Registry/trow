mod admission;
pub mod api_types;
pub mod manifest;
mod proxy;
pub mod server;
mod storage;
mod temporary_file;

pub use admission::ImageValidationConfig;
pub use blob_storage::{BlobReader, ContentInfo, UploadInfo};
pub use digest::Digest;
pub use manifest_storage::ManifestReader;
pub use proxy::{RegistryProxiesConfig, SingleRegistryProxyConfig};
pub use server::TrowServer;
pub use storage::StorageBackendError;
use thiserror::Error;

pub mod blob_storage;
#[allow(dead_code)]
pub mod digest;
pub mod manifest_storage;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ConfigFile {
    #[serde(default)]
    pub registry_proxies: RegistryProxiesConfig,
    pub image_validation: Option<ImageValidationConfig>,
}

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

impl From<sqlx::Error> for RegistryError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {err:?}");
        Self::Internal
    }
}

impl From<StorageBackendError> for RegistryError {
    fn from(err: StorageBackendError) -> Self {
        match err {
            StorageBackendError::BlobNotFound(_) => Self::NotFound,
            StorageBackendError::Internal(e) => {
                tracing::error!("Internal storage error: {e}");
                Self::Internal
            }
            StorageBackendError::InvalidContentRange => Self::InvalidContentRange,
            StorageBackendError::InvalidDigest => Self::InvalidDigest,
            StorageBackendError::InvalidName(name) => Self::InvalidName(name),
            StorageBackendError::Io(e) => {
                tracing::error!("Internal IO error: {e:?}");
                Self::Internal
            }
            StorageBackendError::Unsupported => Self::Unsupported,
        }
    }
}
