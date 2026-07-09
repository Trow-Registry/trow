//! Domain models returned by repositories. Field names and order match SQL
//! columns for zero-friction hand-off with `sqlx::query_as!`.

use sqlx::FromRow;
use sqlx::types::Json;

use crate::utils::manifest::OCIManifest;

#[derive(Debug, Clone, FromRow)]
pub struct Blob {
    pub digest: String,
    pub size: i64,
    pub last_accessed: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct BlobUpload {
    pub uuid: String,
    pub repo: String,
    pub offset: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct Manifest {
    pub blob: Vec<u8>,
    /// The `mediaType` extracted from the manifest JSON, if present.
    pub media_type: Option<String>,
    pub digest: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct Tag {
    pub repo: String,
    pub tag: String,
    pub manifest_digest: String,
}

#[derive(Debug, FromRow)]
pub struct ManifestReferrer {
    pub content: Json<OCIManifest>,
    pub digest: String,
    pub size: i64,
}
