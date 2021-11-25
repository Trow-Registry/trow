use rocket::tokio::io::{AsyncRead, AsyncSeek};
use thiserror::Error;

pub use blob_storage::{BlobReader, BlobStorage, ContentInfo, UploadInfo};
pub use catalog_operations::{CatalogOperations, ManifestHistory};
pub use digest::{Digest, DigestAlgorithm};
pub use manifest_storage::{ManifestReader, ManifestStorage};
pub use metrics::{Metrics, MetricsError, MetricsResponse};
pub use validation::{AdmissionRequest, AdmissionResponse, Validation, ValidationError};

pub mod blob_storage;
pub mod catalog_operations;
#[allow(dead_code)]
pub mod digest;
pub mod manifest_storage;
pub mod metrics;
pub mod validation;

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
impl AsyncSeekRead for rocket::tokio::fs::File {}

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
