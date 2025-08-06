//! types for the trow <=> trow-server interface

use serde_derive::{Deserialize, Serialize};
use tokio::io::AsyncRead;

use super::Digest;
use crate::types::BoundedStream;

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MetricsResponse {
    pub metrics: String,
}

// These types are largely stripped down versions of the Kubernetes types.
// In future, we could directly use k8s types, but I'd rather leave that to a higher level.

pub struct ContentInfo {
    pub length: u64,
    pub range: (u64, u64),
}

pub struct BlobReader<S: AsyncRead + ?Sized + Send> {
    digest: Digest,
    reader: Box<S>,
    size: u64,
}
pub struct Stored {
    pub total_stored: u64,
    pub chunk: u64,
}

impl<S: tokio::io::AsyncRead + Send> BlobReader<S> {
    pub async fn new(digest: Digest, file: BoundedStream<S>) -> Self {
        let file_size = file.size() as u64;
        Self {
            digest,
            reader: Box::new(file.reader()),
            size: file_size,
        }
    }

    pub fn get_reader(self) -> Box<S> {
        self.reader
    }

    pub fn digest(&self) -> &Digest {
        &self.digest
    }

    pub fn blob_size(&self) -> u64 {
        self.size
    }
}
