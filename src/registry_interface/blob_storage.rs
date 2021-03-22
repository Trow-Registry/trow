use super::digest::Digest;
use super::SeekRead;
use super::StorageDriverError;
use std::io::Read;

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
    pub digest: Digest,
    pub reader: Box<dyn SeekRead>,
}

impl BlobReader {
    pub fn get_reader(self) -> Box<dyn SeekRead> {
        self.reader
    }

    pub fn digest(&self) -> &Digest {
        &self.digest
    }
}

pub trait BlobStorage {
    /// Retrieve the blob from the registry identified by digest.
    /// A HEAD request can also be issued to this endpoint to obtain resource information without receiving all data.
    /// GET: /v2/<name>/blobs/<digest>
    fn get_blob(&self, name: &str, digest: &Digest) -> Result<BlobReader, StorageDriverError>;

    /// Delete the blob identified by name and digest
    /// DELETE: /v2/<name>/blobs/<digest>
    fn delete_blob(&self, name: &str, digest: &Digest) -> Result<(), StorageDriverError>;

    /// Requests to start a resumable upload for the given repository.
    /// Returns a session identifier for the upload.
    fn start_blob_upload(&self, name: &str) -> Result<String, StorageDriverError>;

    /// Retrieve status of upload identified by session_id.
    /// The primary purpose of this endpoint is to resolve the current status of a resumable upload.
    /// GET: /v2/<name>/blobs/uploads/<session_id>
    fn status_blob_upload(&self, name: &str, session_id: &str) -> UploadInfo;

    /// Upload a chunk of data for the specified upload.
    /// PATCH: /v2/<name>/blobs/uploads/<session_id>
    /// This method has the session_id as a parameter because at this point we don't know the final
    /// file name. So the data needs to be appended to a temporary file/location with the session_id
    /// as its identifier.
    /// Passed optional ContentInfo which describes range of data.
    /// Returns current size of blob, including any previous chunks
    fn store_blob_chunk(
        &self,
        name: &str,
        session_id: &str,
        data_info: Option<ContentInfo>,
        data: &mut Box<dyn Read>,
    ) -> Result<u64, StorageDriverError>;

    /// Finalises the upload of the given session_id.
    /// Also verfies uploaded blob matches user digest
    fn complete_and_verify_blob_upload(
        &self,
        name: &str,
        session_id: &str,
        digest: &Digest,
    ) -> Result<(), StorageDriverError>;

    /// Cancel outstanding upload processes, releasing associated resources.
    /// If this is not called, the unfinished uploads will eventually timeout.
    /// DELETE: /v2/<name>/blobs/uploads/<session_id>
    /// Here we need to delete the existing temporary file/location based on its identifier: the session_id
    fn cancel_blob_upload(&self, name: &str, session_id: &str) -> Result<(), StorageDriverError>;

    /// Whether the specific blob exists
    /// AM: Assume this is for HEAD requests?
    fn has_blob(&self, name: &str, digest: &Digest) -> bool;
}
