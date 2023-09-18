use axum::extract::BodyStream;

use super::digest::Digest;
use super::{AsyncSeekRead, StorageDriverError};

pub struct ContentInfo {
    pub length: u64,
    pub range: (u64, u64),
}

#[allow(dead_code)]
pub struct UploadInfo {
    name: String,
    session_id: String,
    uploaded: u32,
    size: u32,
}

pub struct BlobReader {
    digest: Digest,
    reader: tokio::fs::File,
    size: u64,
}
pub struct Stored {
    pub total_stored: u64,
    pub chunk: u64,
}

impl BlobReader {
    pub async fn new(digest: Digest, file: tokio::fs::File) -> Self {
        let file_size = file.metadata().await.unwrap().len();
        Self {
            digest,
            reader: file,
            size: file_size,
        }
    }

    pub fn get_reader(self) -> impl AsyncSeekRead {
        self.reader
    }

    pub fn digest(&self) -> &Digest {
        &self.digest
    }

    pub fn blob_size(&self) -> u64 {
        self.size
    }
}

#[axum::async_trait]
pub trait BlobStorage {
    /// Retrieve the blob from the registry identified by digest.
    /// A HEAD request can also be issued to this endpoint to obtain resource information without receiving all data.
    /// GET: /v2/<name>/blobs/<digest>
    async fn get_blob(&self, name: &str, digest: &Digest)
        -> Result<BlobReader, StorageDriverError>;

    /// Delete the blob identified by name and digest
    /// DELETE: /v2/<name>/blobs/<digest>
    async fn delete_blob(&self, name: &str, digest: &Digest) -> Result<(), StorageDriverError>;

    /// Requests to start a resumable upload for the given repository.
    /// Returns a session identifier for the upload.
    async fn start_blob_upload(&self, name: &str) -> Result<String, StorageDriverError>;

    /// Retrieve status of upload identified by session_id.
    /// The primary purpose of this endpoint is to resolve the current status of a resumable upload.
    /// GET: /v2/<name>/blobs/uploads/<session_id>
    async fn status_blob_upload(&self, name: &str, session_id: &str) -> UploadInfo;

    /// Upload a chunk of data for the specified upload.
    /// PATCH: /v2/<name>/blobs/uploads/<session_id>
    /// This method has the session_id as a parameter because at this point we don't know the final
    /// file name. So the data needs to be appended to a temporary file/location with the session_id
    /// as its identifier.
    /// Passed optional ContentInfo which describes range of data.
    /// Returns current size of blob, including any previous chunks
    ///
    /// TODO: Really should not be using Rocket specific DataStream object.
    /// It's used to determine if max transfer cap was hit.
    /// Solution may be to return method that gets write sink (see old code).
    async fn store_blob_chunk<'a>(
        &self,
        name: &str,
        session_id: &str,
        data_info: Option<ContentInfo>,
        data: BodyStream,
    ) -> Result<Stored, StorageDriverError>;

    /// Finalises the upload of the given session_id.
    /// Also verfies uploaded blob matches user digest
    async fn complete_and_verify_blob_upload(
        &self,
        name: &str,
        session_id: &str,
        digest: &Digest,
    ) -> Result<(), StorageDriverError>;

    /// Cancel outstanding upload processes, releasing associated resources.
    /// If this is not called, the unfinished uploads will eventually timeout.
    /// DELETE: /v2/<name>/blobs/uploads/<session_id>
    /// Here we need to delete the existing temporary file/location based on its identifier: the session_id
    async fn cancel_blob_upload(
        &self,
        name: &str,
        session_id: &str,
    ) -> Result<(), StorageDriverError>;

    /// Whether the specific blob exists
    /// AM: Assume this is for HEAD requests?
    async fn has_blob(&self, name: &str, digest: &Digest) -> bool;
}
