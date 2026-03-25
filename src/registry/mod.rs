pub(crate) mod api_types;
pub mod garbage_collect;
mod proxy;
pub mod server;
mod storage;

pub use api_types::{BlobReader, ContentInfo};
pub use proxy::{DownloadRemoteImageError, RemoteImage};
pub use server::TrowServer;
pub use storage::StorageBackendError;
use thiserror::Error;

use crate::configuration::SingleRegistryProxyConfig;
use crate::utils::digest::Digest;

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
