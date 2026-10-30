//! Results of the blob upload workflow.
//!
//! A blob upload moves through "open" and "completed"; these types are what
//! `BlobUploadService` returns and what the routes layer renders.

use uuid::Uuid;

use crate::utils::digest::Digest;

/// An open upload, positioned at the last accepted byte.
#[derive(Debug)]
pub struct UploadInfo {
    uuid: String,
    repo_name: String,
    range: (u64, u64),
}

impl UploadInfo {
    pub fn new(uuid: String, repo_name: String, range: (u64, u64)) -> Self {
        Self {
            uuid,
            repo_name,
            range,
        }
    }

    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    pub fn repo_name(&self) -> &String {
        &self.repo_name
    }

    pub fn range(&self) -> (u64, u64) {
        self.range
    }
}

/// A finished upload: the blob is now stored and associated with the repo.
#[derive(Debug)]
pub struct AcceptedUpload {
    digest: Digest,
    repo_name: String,
    uuid: Uuid,
    range: (u64, u64),
}

impl AcceptedUpload {
    pub fn new(digest: Digest, repo_name: String, uuid: Uuid, range: (u64, u64)) -> Self {
        Self {
            digest,
            repo_name,
            uuid,
            range,
        }
    }

    pub fn digest(&self) -> &Digest {
        &self.digest
    }

    pub fn repo_name(&self) -> &String {
        &self.repo_name
    }

    pub fn range(&self) -> (u64, u64) {
        self.range
    }

    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }
}

/// Outcome of `POST /v2/<name>/blobs/uploads/`: either the upload was
/// completed in a single request, or it is left open for PATCH.
#[derive(Debug)]
pub enum Upload {
    Accepted(AcceptedUpload),
    Info(UploadInfo),
}

/// Position of an in-progress upload, used by `GET /v2/.../uploads/<uuid>`.
#[derive(Debug)]
pub struct UploadStatus {
    pub uuid: Uuid,
    pub repo: String,
    pub offset: i64,
}
