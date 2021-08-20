use super::{AsyncSeekRead, StorageDriverError};
use super::{Digest, DigestAlgorithm};
use rocket::data::DataStream;
use std::pin::Pin;

pub struct ManifestReader {
    pub content_type: String,
    pub digest: Digest,
    pub reader: Pin<Box<dyn AsyncSeekRead>>,
}

impl ManifestReader {
    pub fn get_reader(self) -> Pin<Box<dyn AsyncSeekRead>> {
        self.reader
    }

    pub fn content_type(&self) -> &str {
        &self.content_type
    }

    pub fn digest(&self) -> &Digest {
        &self.digest
    }
}

// This trait handles all the necessary Manifest Operations (get, save delete)
#[rocket::async_trait]
pub trait ManifestStorage {
    /// Fetch the manifest identified by name and reference where reference can be a tag or digest.
    /// A HEAD request can also be issued to this endpoint to obtain resource information without receiving all data.
    /// GET: /v2/<name>/manifests/<reference>
    /// HEAD: /v2/<name>/manifests/<reference>
    async fn get_manifest(
        &self,
        name: &str,
        tag: &str,
    ) -> Result<ManifestReader, StorageDriverError>;

    // Stores should take a reader that has the data, possibly a second method that returns byte array

    /// TODO: DataStream is currently tied to Rocket implementation to handle transfers that get capped.
    /// Fixing this means either changing the interface to return a sink the route can write to or coming up with a
    /// new interface type.

    /// Put the manifest identified by name and tag. (Note that manifests cannot be pushed by digest)
    /// data is a link to reader for supplying the bytes of the manifest.
    /// Returns digest of the manifest.
    ///
    async fn store_manifest<'a>(
        &self,
        name: &str,
        tag: &str,
        data: DataStream<'a>,
    ) -> Result<Digest, StorageDriverError>;

    // Store a manifest via Writer trait for drivers which support it
    // AM: I think this was just for Trow, so we can remove, right?
    //fn store_manifest_with_writer(&self, name: &str, tag: &str) -> Result<Box<dyn Write>>;

    /// Delete the manifest identified by name and reference. Note that a manifest can only be deleted by digest.
    /// DELETE: /v2/<name>/manifests/<reference>
    async fn delete_manifest(&self, name: &str, digest: &Digest) -> Result<(), StorageDriverError>;

    /// Whether the specific manifest exists
    async fn has_manifest(&self, name: &str, algo: &DigestAlgorithm, reference: &str) -> bool;
}
