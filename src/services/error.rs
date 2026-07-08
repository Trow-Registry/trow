use crate::file_storage::StorageBackendError;
use crate::services::proxy_service::errors::DownloadRemoteImageError;
use crate::utils::digest::DigestError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not found")]
    NotFound,
    #[error("invalid input: {0}")]
    Invalid(String),
    #[error("unsupported for proxied repo")]
    UnsupportedForProxiedRepo,
    #[error("manifest invalid: {0}")]
    ManifestInvalid(String),
    #[error("manifest unknown: {0}")]
    ManifestUnknown(String),
    #[error("blob upload unknown")]
    BlobUploadUnknown,
    #[error("db error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("storage error: {0}")]
    Storage(#[from] StorageBackendError),
    #[error("digest error: {0}")]
    Digest(#[from] DigestError),
    #[error("proxy download error: {0}")]
    Proxy(Box<DownloadRemoteImageError>),
}

impl From<DownloadRemoteImageError> for Error {
    fn from(err: DownloadRemoteImageError) -> Self {
        Error::Proxy(Box::new(err))
    }
}
