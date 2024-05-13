mod admission;
pub mod api_types;
mod image;
pub mod manifest;
mod metrics;
mod proxy_auth;
mod server;
pub mod storage;
mod temporary_file;

use std::path::PathBuf;

pub use admission::ImageValidationConfig;
use anyhow::Result;
pub use proxy_auth::{RegistryProxiesConfig, SingleRegistryProxyConfig};
pub use server::TrowServer;


pub use blob_storage::{BlobReader, BlobStorage, ContentInfo, UploadInfo};
pub use catalog_operations::{CatalogOperations, ManifestHistory};
pub use digest::{Digest, DigestAlgorithm};
pub use manifest_storage::{ManifestReader, ManifestStorage};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncSeek};

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
}

//If there's a better solution, please let me know.
//I'd much rather not have to write an impl for every class :(
pub trait AsyncSeekRead: AsyncRead + AsyncSeek + Send {}
impl AsyncSeekRead for tokio::fs::File {}

// Super trait
pub trait RegistryStorage: ManifestStorage + BlobStorage + CatalogOperations {
    /// Whether the specific name(space) exists
    fn exists(&self, name: &str) -> Result<bool, StorageDriverError>;

    /// Whether the driver supports processing of data chunks in a streaming mode
    /// For example when the client uploads chunks of data, instead of buffering them
    /// in memory and then passing the full data, the driver can process single chunks
    /// individually. This significantly decrease the memory usage of the registry
    fn support_streaming(&self) -> bool;
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
