use crate::types::{
    AdmissionRequest, AdmissionResponse, BlobReader, ContentInfo, ManifestHistory, ManifestReader,
    MetricsResponse,
};
use digest::{Digest, DigestAlgorithm};
use std::io::Read;
use thiserror::Error;

#[allow(dead_code)]
pub mod digest;

// TODO: implement the 'mount' and 'multi-arch' OCI spec

//==================================================================================================\
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

//==================================================================================================
#[allow(dead_code)]
pub struct UploadInfo {
    name: String,
    session_id: String,
    uploaded: u32,
    size: u32,
}

// Super trait
pub trait RegistryStorage: ManifestStorage + BlobStorage + CatalogOperations {
    /// Whether the specific name(space) exists
    fn exists(&self, name: &String) -> Result<bool, StorageDriverError>;

    /// Whether the driver supports processing of data chunks in a streaming mode
    /// For example when the client uploads chunks of data, instead of buffering them
    /// in memory and then passing the full data, the driver can process single chunks
    /// individually. This significantly decrease the memory usage of the registry
    fn support_streaming(&self) -> bool;
}

// This trait handles all the necessary Manifest Operations (get, save delete)
pub trait ManifestStorage {
    /// Fetch the manifest identified by name and reference where reference can be a tag or digest.
    /// A HEAD request can also be issued to this endpoint to obtain resource information without receiving all data.
    /// GET: /v2/<name>/manifests/<reference>
    /// HEAD: /v2/<name>/manifests/<reference>
    fn get_manifest(&self, name: &str, tag: &str) -> Result<ManifestReader, StorageDriverError>;

    // Stores should take a reader that has the data, possibly a second method that returns byte array

    /// Put the manifest identified by name and tag. (Note that manifests cannot be pushed by digest)
    /// data is a link to reader for supplying the bytes of the manifest.
    /// Returns digest of the manifest.
    /// PUT: /v2/<name>/manifests/<tag>
    fn store_manifest(
        &self,
        name: &str,
        tag: &str,
        data: &mut Box<dyn Read>,
    ) -> Result<Digest, StorageDriverError>;

    // Store a manifest via Writer trait for drivers which support it
    // AM: I think this was just for Trow, so we can remove, right?
    //fn store_manifest_with_writer(&self, name: &str, tag: &str) -> Result<Box<dyn Write>>;

    /// Delete the manifest identified by name and reference. Note that a manifest can only be deleted by digest.
    /// DELETE: /v2/<name>/manifests/<reference>
    fn delete_manifest(&self, name: &str, digest: &Digest) -> Result<(), StorageDriverError>;

    /// Whether the specific manifest exists
    fn has_manifest(&self, name: &str, algo: &DigestAlgorithm, reference: &str) -> bool;
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

pub trait CatalogOperations {
    /// Returns a vec of all repository names in the registry
    /// Can optionally be given a start value and maximum number of results to return.
    fn get_catalog(
        &self,
        start_value: Option<&str>,
        num_results: Option<u32>,
    ) -> Result<Vec<String>, StorageDriverError>;

    /// Returns a vec of all tags under the given repository
    /// Start value and num_results used to control number of returned results
    /// Allows for some optimisations.
    fn get_tags(
        &self,
        repo: &str,
        start_value: Option<&str>,
        num_results: Option<u32>,
    ) -> Result<Vec<String>, StorageDriverError>;

    /// Returns the history for a given tag (what digests it has pointed to)
    fn get_history(
        &self,
        repo: &str,
        name: &str,
        start_value: Option<&str>,
        num_results: Option<u32>,
    ) -> Result<ManifestHistory, StorageDriverError>;
}

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Internal validation error")]
    Internal,
}
pub trait Validation {
    // This function signature is very tied to the implementation.
    // If you develop a new front-end and have problems here, we should change it.
    fn validate_admission(
        &self,
        admission_req: &AdmissionRequest,
        host_names: &Vec<String>,
    ) -> Result<AdmissionResponse, ValidationError>;
}

#[derive(Error, Debug)]
pub enum MetricsError {
    #[error("Internal metrics error")]
    Internal,
}

pub trait Metrics {
    fn is_healthy(&self) -> bool;
    fn is_ready(&self) -> bool;
    fn get_metrics(&self) -> Result<MetricsResponse, MetricsError>;
}
