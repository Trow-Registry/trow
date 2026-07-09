use aws_sdk_ecr::config::http::HttpResponse;
use aws_sdk_ecr::error::SdkError;
use aws_sdk_ecr::operation::get_authorization_token::GetAuthorizationTokenError;

use crate::utils::digest::DigestError;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, thiserror::Error)]
pub enum DownloadRemoteImageError {
    #[error("DatabaseError: {0}")]
    DbError(#[from] sqlx::Error),
    #[error("Invalid digest: {0}")]
    InvalidDigest(#[from] DigestError),
    #[error("Failed to download image")]
    DownloadAttemptsFailed,
    #[error("Manifest JSON is not canonicalized")]
    ManifestNotCanonicalized,
    #[error("OCI client error: {0}")]
    OciClientError(#[from] ::oci_client::errors::OciDistributionError),
    #[error("Storage backend error: {0}")]
    StorageError(#[from] crate::file_storage::StorageBackendError),
    #[error("Could not deserialize manifest: {0}")]
    ManifestDeserializationError(#[from] serde_json::Error),
    #[error("Could not get AWS ECR password: {0}")]
    EcrLoginError(#[from] EcrPasswordError),
}

#[allow(clippy::large_enum_variant)]
#[derive(thiserror::Error, Debug)]
pub enum EcrPasswordError {
    #[error("Could not parse region from ECR URL")]
    InvalidRegion,
    #[error("Could not decode ECR token: {0}")]
    Base64DecodeError(#[from] base64::DecodeError),
    #[error("Could not convert ECR token to UTF8: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Could not get AWS token: {0}")]
    AWSError(#[from] SdkError<GetAuthorizationTokenError, HttpResponse>),
}
