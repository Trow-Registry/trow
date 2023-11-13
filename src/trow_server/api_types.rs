//! types for the trow <=> trow-server interface

use serde_derive::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub struct Upload {
    pub repo_name: String,
    pub uuid: String,
}

#[derive(Clone, PartialEq)]
pub struct UploadRequest {
    /// e.g. "amouat/network-utils", "nginx", "my-org/my-team/my-repo"
    ///
    /// Expect some auth stuff as well later
    pub repo_name: String,
}

#[derive(Clone, PartialEq)]
pub struct UploadDetails {
    pub uuid: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct UploadRef {
    pub repo_name: String,
    pub uuid: String,
}

#[derive(Clone, PartialEq, Debug)]
pub struct BlobRef {
    pub repo_name: String,

    pub digest: String,
}

/// At the moment this will be a simple file path, but could evolve in future
#[derive(Clone, PartialEq)]
pub struct WriteLocation {
    pub path: String,
}

#[derive(Clone, PartialEq)]
pub struct ManifestWriteDetails {
    pub path: String,

    pub uuid: String,
}
/// Could have a single "Location", but this allows divergence in the future

#[derive(Clone, PartialEq)]
pub struct BlobReadLocation {
    pub path: String,
}
/// At the moment this will be a simple file path, but could evolve in future

#[derive(Clone, PartialEq)]
pub struct CompleteRequest {
    pub repo_name: String,

    pub uuid: String,

    pub user_digest: String,
}

#[derive(Clone, PartialEq)]
pub struct CompletedUpload {
    pub digest: String,
}

#[derive(Clone, PartialEq)]
pub struct ManifestRef {
    pub repo_name: String,
    /// Can be digest or tag
    pub reference: String,
}

#[derive(Clone, PartialEq)]
pub struct VerifyManifestRequest {
    pub manifest: Option<ManifestRef>,

    pub uuid: String,
}

#[derive(Clone, PartialEq)]
pub struct VerifiedManifest {
    pub digest: String,
    /// Version of manifest, used for media type return
    pub content_type: String,
}

#[derive(Clone, PartialEq)]
pub struct ManifestReadLocation {
    pub digest: String,
    /// For the moment path to file
    pub path: String,
    /// Version of manifest, used for media type return
    pub content_type: String,
}

#[derive(Clone, PartialEq)]
pub struct CatalogRequest {
    pub limit: u32,

    pub last_repo: String,
}

#[derive(Clone, PartialEq)]
pub struct ListTagsRequest {
    pub repo_name: String,

    pub limit: u32,

    pub last_tag: String,
}

#[derive(Clone, PartialEq)]
pub struct CatalogEntry {
    pub repo_name: String,
}

#[derive(Clone, PartialEq)]
pub struct Tag {
    pub tag: String,
}

#[derive(Clone, PartialEq)]
pub struct BlobDeleted {}

#[derive(Clone, PartialEq)]
pub struct ManifestDeleted {}

#[derive(Clone, PartialEq)]
pub struct ManifestHistoryRequest {
    pub repo_name: String,
    /// Always tag, not digest
    pub tag: String,
    /// For pagination can pass the last digest we saw and how many results we want
    pub limit: u32,

    pub last_digest: String,
}

#[derive(thiserror::Error, Debug)]
pub enum Status {
    #[error("Cancelled: {0}")]
    Cancelled(String),
    #[error("Unknown: {0}")]
    Unknown(String),
    #[error("InvalidArgument: {0}")]
    InvalidArgument(String),
    #[error("DeadlineExceeded: {0}")]
    DeadlineExceeded(String),
    #[error("NotFound: {0}")]
    NotFound(String),
    #[error("AlreadyExists: {0}")]
    AlreadyExists(String),
    #[error("FailedPrecondition: {0}")]
    FailedPrecondition(String),
    #[error("Internal: {0}")]
    Internal(String),
    #[error("Unavailable: {0}")]
    Unavailable(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Timestamp {
    pub seconds: i64,
    pub nanos: i32,
}

#[derive(Clone, PartialEq)]
pub struct ManifestHistoryEntry {
    pub digest: String,

    pub date: Option<Timestamp>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub message: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadyStatus {
    pub is_ready: bool,
    pub message: String,
}

#[derive(Clone, PartialEq)]
pub struct MetricsRequest {}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MetricsResponse {
    pub metrics: String,
}
/// These types are largely stripped down versions of the Kubernetes types.
/// In future, we could directly use k8s types, but I'd rather leave that to a higher level.

#[derive(Clone, PartialEq)]
pub struct AdmissionRequest {
    pub images: Vec<String>,

    pub namespace: String,
    /// Used by mutation webhook
    pub image_paths: Vec<String>,

    pub host_name: String,
}

#[derive(Clone, PartialEq)]
pub struct AdmissionResponse {
    pub is_allowed: bool,

    pub reason: String,
    /// only used for mutation
    pub patch: Option<Vec<u8>>,
}
