use oci_client::errors::OciDistributionError;

use crate::file_storage::StorageBackendError;
use crate::services::proxy_service::errors::DownloadRemoteImageError;
use crate::utils::digest::DigestError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("internal error: {0}")]
    Internal(String),
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

impl Error {
    /// The human readable part of an error, used to fill OCI error `detail`.
    pub fn detail(&self) -> String {
        match self {
            Error::Invalid(s) | Error::ManifestInvalid(s) | Error::ManifestUnknown(s) => s.clone(),
            Error::NotFound => "not found".to_string(),
            Error::Unauthorized => "unauthorized".to_string(),
            Error::Internal(s) => s.clone(),
            Error::UnsupportedForProxiedRepo => "unsupported for proxied repo".to_string(),
            Error::BlobUploadUnknown => "blob upload unknown".to_string(),
            Error::Db(e) => e.to_string(),
            Error::Storage(e) => e.to_string(),
            Error::Digest(e) => e.to_string(),
            Error::Proxy(e) => e.to_string(),
        }
    }
}

impl Error {
    /// Attaches detail to an error that carries none, keeping the variant so
    /// classification is unaffected.
    pub fn with_cause(self, cause: impl std::fmt::Display) -> Self {
        match self {
            Error::Unauthorized => {
                tracing::warn!("{cause}");
                Error::Unauthorized
            }
            other => other,
        }
    }
}

impl From<DownloadRemoteImageError> for Error {
    fn from(err: DownloadRemoteImageError) -> Self {
        Error::Proxy(Box::new(err))
    }
}

/// Transport-independent classification of a service error.
///
/// The routes layer maps this onto OCI error codes; it lets that mapping stay
/// in `src/routes/response/errors.rs` without importing storage or digest
/// internals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    NotFound,
    Unauthorized,
    InvalidInput,
    UnsupportedForProxiedRepo,
    ManifestInvalid,
    ManifestUnknown,
    BlobUploadUnknown,
    DigestInvalid,
    BlobNotFound,
    UnsatisfiableRange,
    Internal,
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        match self {
            Error::NotFound => ErrorKind::NotFound,
            Error::Unauthorized => ErrorKind::Unauthorized,
            Error::Internal(_) => ErrorKind::Internal,
            Error::Invalid(_) => ErrorKind::InvalidInput,
            Error::UnsupportedForProxiedRepo => ErrorKind::UnsupportedForProxiedRepo,
            Error::ManifestInvalid(_) => ErrorKind::ManifestInvalid,
            Error::ManifestUnknown(_) => ErrorKind::ManifestUnknown,
            Error::BlobUploadUnknown => ErrorKind::BlobUploadUnknown,
            Error::Db(sqlx::Error::RowNotFound) => ErrorKind::NotFound,
            Error::Db(_) => ErrorKind::Internal,
            Error::Storage(StorageBackendError::BlobNotFound(_)) => ErrorKind::BlobNotFound,
            Error::Storage(StorageBackendError::InvalidContentRange) => {
                ErrorKind::UnsatisfiableRange
            }
            Error::Storage(_) => ErrorKind::Internal,
            Error::Digest(DigestError::InvalidDigest(_)) => ErrorKind::DigestInvalid,
            Error::Proxy(e) => match e.as_ref() {
                DownloadRemoteImageError::OciClientError(
                    OciDistributionError::ImageManifestNotFoundError(_),
                ) => ErrorKind::ManifestUnknown,
                _ => ErrorKind::Internal,
            },
        }
    }
}
