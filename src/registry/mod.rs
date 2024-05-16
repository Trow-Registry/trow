mod admission;
pub mod api_types;
mod image;
pub mod manifest;
mod proxy_auth;
mod server;
mod storage;
mod temporary_file;

use std::path::PathBuf;

pub use admission::ImageValidationConfig;
use anyhow::Result;
pub use blob_storage::{BlobReader, ContentInfo, UploadInfo};
pub use catalog_operations::ManifestHistory;
pub use digest::{Digest, DigestAlgorithm};
pub use manifest_storage::ManifestReader;
pub use proxy_auth::{RegistryProxiesConfig, SingleRegistryProxyConfig};
pub use server::TrowServer;
pub use storage::StorageBackendError;
use thiserror::Error;

pub mod blob_storage;
pub mod catalog_operations;
#[allow(dead_code)]
pub mod digest;
pub mod manifest_storage;

// Storage Driver Error
#[derive(Error, Debug)]
pub enum StorageDriverError {
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
    #[error("Internal storage error")]
    Internal,
    #[error("Not found")]
    NotFound,
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
    pub fn get_server(self) -> Result<TrowServer> {
        TrowServer::new(
            self.data_path,
            self.proxy_registry_config,
            self.image_validation_config,
        )
    }
}
