use core::ops::Range;
use std::borrow::Cow;
use std::fs::{self, File};
use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::pin::pin;
use std::{io, str};

use anyhow::Result;
use bytes::{Buf, Bytes};
use chrono::prelude::*;
use futures::stream::StreamExt;
use futures::Stream;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::time::Duration;
use tracing::{event, Level};
use walkdir::WalkDir;

use super::api_types::*;
use super::manifest::{FromJson, Manifest};
use super::server::SUPPORTED_DIGESTS;
use crate::trow_server::digest;
use crate::trow_server::temporary_file::TemporaryFile;

// Storage Driver Error
#[derive(thiserror::Error, Debug)]
pub enum StorageBackendError {
    #[error("the name `{0}` is not valid")]
    InvalidName(String),
    #[error("Manifest is not valid ({0:?})")]
    InvalidManifest(Option<String>),
    #[error("Digest did not match content")]
    InvalidDigest,
    #[error("Unsupported Operation")]
    Unsupported,
    #[error("Requested index does not match actual")]
    InvalidContentRange,
    #[error("Internal error: {0}")]
    Internal(Cow<'static, str>),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum WriteBlobRangeError {
    #[error("Invalid content range")]
    InvalidContentRange,
    #[error("Not found")]
    NotFound,
    #[error("Internal storage error")]
    Internal,
    #[error("Could not read from stream")]
    InvalidStream,
}

static MANIFESTS_DIR: &str = "manifests";
static BLOBS_DIR: &str = "blobs";
static UPLOADS_DIR: &str = "scratch";

/// Current storage structure:
/// - /blobs/sha256/<digest>: is a blob (manifests are treated as blobs)
/// - /manifests/<image-name..>/<tag>: file containing a list of manifest digests
/// - /scratch/<uuid>: is a blob being uploaded
///
/// TODO future structure:
/// - /blobs/sha256/<digest>: contains blobs
/// - /uploads/<uuid>: is a blob being uploaded
/// - /repositories/<image-name..>/tags/<tag>: is a file with a manifest digest
/// - /repositories/<image-name..>/revisions/sha256/<digest>: is a manifest
#[derive(Clone, Debug)]
pub struct TrowStorageBackend {
    path: PathBuf,
}

impl TrowStorageBackend {
    fn init_create_path(root: &Path, dir: &str) -> Result<()> {
        let path = root.join(dir);
        match fs::create_dir_all(&path) {
            Ok(_) => Ok(()),
            Err(e) => {
                event!(
                    Level::ERROR,
                    r#"
                    Failed to create directory required by trow {:?}
                    Please check the parent directory is writable by the trow user.
                    {:?}"#,
                    path,
                    e
                );
                Err(e.into())
            }
        }
    }

    pub fn new(path: PathBuf) -> Result<Self> {
        Self::init_create_path(&path, MANIFESTS_DIR)?;
        Self::init_create_path(&path, BLOBS_DIR)?;
        Self::init_create_path(&path, UPLOADS_DIR)?;

        Ok(Self { path })
    }

    pub async fn get_manifest_digest(
        &self,
        repo_name: &str,
        tag: &str,
    ) -> Result<String, StorageBackendError> {
        let path = self.path.join(MANIFESTS_DIR).join(repo_name).join(tag);
        let manifest_history_bytes = tokio::fs::read(&path).await?;
        let latest_entry_bytes = manifest_history_bytes.split(|b| *b == b'\n').last().ok_or(
            StorageBackendError::Internal(Cow::Borrowed("Empty manifest history")),
        )?;
        let latest_entry: ManifestHistoryEntry = serde_json::from_slice(latest_entry_bytes)
            .map_err(|e| {
                StorageBackendError::Internal(Cow::Owned(format!("Invalid manifest entry: {e}")))
            })?;

        Ok(latest_entry.digest)
    }

    pub async fn get_manifest(
        &self,
        repo_name: &str,
        reference: &str,
    ) -> Result<Manifest, StorageBackendError> {
        let digest = if is_digest(reference) {
            Cow::Borrowed(reference)
        } else {
            Cow::Owned(self.get_manifest_digest(repo_name, reference).await?)
        };
        let path = self
            .path
            .join(BLOBS_DIR)
            .join("sha256")
            .join(digest.as_ref());
        let manifest_bytes = tokio::fs::read(&path).await?;
        let manifest_json: serde_json::Value =
            serde_json::from_slice(&manifest_bytes).map_err(|e| {
                StorageBackendError::Internal(Cow::Owned(format!(
                    "Manifest is not valid JSON: {e}"
                )))
            })?;
        let manifest = Manifest::from_json(&manifest_json).map_err(|e| {
            StorageBackendError::Internal(Cow::Owned(format!(
                "Manifest does not respect schema: {e}"
            )))
        })?;
        Ok(manifest)
    }

    pub async fn write_blob_stream<'a, S, E>(
        &'a self,
        digest: &str,
        stream: S,
    ) -> Result<PathBuf, StorageBackendError>
    where
        S: Stream<Item = Result<Bytes, E>> + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        let (alg, hash) = match is_digest2(digest) {
            Some((alg, hash)) => (alg, hash),
            None => return Err(StorageBackendError::InvalidDigest),
        };

        let tmp_location = self.path.join(UPLOADS_DIR).join(digest);
        let location = self.path.join(BLOBS_DIR).join(alg).join(hash);
        if location.exists() {
            event!(Level::INFO, "Blob already exists");
            return Ok(location);
        }
        tokio::fs::create_dir_all(location.parent().unwrap()).await?;
        let mut tmp_file = match TemporaryFile::open_for_writing(tmp_location.clone()).await {
            // All good
            Ok(tmpf) => tmpf,
            // Special case: blob is being concurrently fetched
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                event!(Level::INFO, "Waiting for concurently fetched blob");
                while tmp_location.exists() {
                    // wait for download to be done (temp file to be moved)
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
                // TODO
                return Ok(location);
            }
            Err(e) => {
                return Err(e.into());
            }
        };
        tmp_file.write_stream(stream).await?;
        tmp_file.rename(&location).await?;
        Ok(location)
    }

    /// Writes part of a blob to disk.
    pub async fn write_blob_part_stream<S>(
        &self,
        upload_id: &str,
        mut stream: S,
        range: Option<Range<u64>>,
    ) -> Result<PathBuf, WriteBlobRangeError>
    where
        S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
    {
        // invalid range
        // does not exist
        // could not read stream

        let tmp_location = self.path.join(UPLOADS_DIR).join(upload_id);
        let mut open_options = tokio::fs::OpenOptions::new();
        open_options.write(true);
        if range.is_none() || range.as_ref().unwrap().start == 0 {
            open_options.create_new(true);
        }

        let mut tmp_file = open_options.open(&tmp_location).await.map_err(|e| {
            event!(Level::ERROR, "Could not open tmp file: {}", e);
            match e.kind() {
                io::ErrorKind::NotFound => WriteBlobRangeError::NotFound,
                io::ErrorKind::AlreadyExists => WriteBlobRangeError::InvalidContentRange,
                _ => WriteBlobRangeError::Internal,
            }
        })?;
        let range_start_len = match &range {
            None => None,
            Some(r) => Some((r.start, r.end - r.start)),
        };

        fn err_catcher(err: impl std::error::Error, tmp_location: &Path) -> WriteBlobRangeError {
            let res = fs::remove_file(tmp_location);
            if let Err(e) = res {
                event!(Level::WARN, "Could not remove tmp file: {:?}", e);
            }
            event!(Level::INFO, "Could not write blob stream: {:?}", err);
            WriteBlobRangeError::Internal
        }
        if let Some(range) = &range {
            tmp_file
                .seek(io::SeekFrom::Start(range.start))
                .await
                .map_err(|e| err_catcher(e, &tmp_location))?;
        }
        let mut stream_size = 0;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| err_catcher(e, &tmp_location))?;
            stream_size += chunk.len();
            if range_start_len.is_some() && stream_size > range_start_len.unwrap().1 as usize {
                return Err(WriteBlobRangeError::InvalidContentRange);
            }
            tmp_file
                .write_all(&chunk)
                .await
                .map_err(|e| err_catcher(e, &tmp_location))?;
        }
        if stream_size != range_start_len.unwrap().1 as usize {
            return Err(WriteBlobRangeError::InvalidContentRange);
        }
        tmp_file
            .flush()
            .await
            .map_err(|e| err_catcher(e, &tmp_location))?;
        Ok(tmp_location)
    }

    pub async fn write_image_manifest(
        &self,
        manifest: Bytes,
        repo_name: &str,
        tag: &str,
        verify: bool,
    ) -> Result<PathBuf, StorageBackendError> {
        let digest = digest::sha256_tag_digest(manifest.as_ref().reader()).map_err(|e| {
            StorageBackendError::Internal(Cow::Owned(format!(
                "Could not calculate digest of manifest: {e}"
            )))
        })?;
        if verify {
            let manifest: Manifest = serde_json::from_slice(&manifest)
                .map_err(|_| StorageBackendError::InvalidManifest(None))?;
            for digest in manifest.get_local_asset_digests() {
                let (alg, hash) = is_digest2(digest).ok_or_else(|| {
                    StorageBackendError::InvalidManifest(Some(format!(
                        "Invalid digest in manifest: {}",
                        digest
                    )))
                })?;
                let blob_path = self.path.join(BLOBS_DIR).join(alg).join(hash);
                if !blob_path.exists() {
                    return Err(StorageBackendError::InvalidManifest(Some(format!(
                        "Blob not found: {:?}",
                        blob_path
                    ))));
                }
            }
        }
        let manifest_stream = bytes_to_stream(manifest);
        let location = self
            .write_blob_stream(&digest, pin!(manifest_stream))
            .await?;

        // save link tag -> manifest
        let entry = ManifestHistoryEntry {
            digest: digest.clone(),
            date: Utc::now(),
        };
        let mut entry_str = serde_json::to_vec(&entry).map_err(|_| {
            StorageBackendError::Internal(Cow::Borrowed("Invalid manifest tag entry"))
        })?;
        entry_str.push(b'\n');
        let manifest_history_loc = self.path.join(MANIFESTS_DIR).join(repo_name).join(tag);
        tokio::fs::create_dir_all(manifest_history_loc.parent().unwrap()).await?;
        let mut manifest_history_file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&manifest_history_loc)
            .await?;
        manifest_history_file.write_all(&entry_str).await?;
        manifest_history_file.flush().await?;

        Ok(location)
    }

    pub async fn get_manifest_history(
        &self,
        repo_name: &str,
        tag: &str,
    ) -> Result<Vec<ManifestHistoryEntry>, StorageBackendError> {
        let manifest_history_loc = self.path.join(MANIFESTS_DIR).join(repo_name).join(tag);
        let history_raw = tokio::fs::read(manifest_history_loc).await?;
        io::Cursor::new(history_raw)
            .lines()
            .map(|man| serde_json::from_slice(man.unwrap().as_bytes()))
            .collect::<Result<Vec<ManifestHistoryEntry>, _>>()
            .map_err(|e| {
                StorageBackendError::Internal(Cow::Owned(format!(
                    "Could not parse manifest history ({repo_name}:{tag}): {e}"
                )))
            })
    }

    pub async fn list_repos(&self) -> Result<Vec<String>> {
        let manifest_dir = self.path.join(MANIFESTS_DIR);
        // let dirs = tokio::fs::read_dir(manifest_dir);
        let manifests = WalkDir::new(&manifest_dir)
            .into_iter()
            .filter_map(|entry| {
                let entry = entry.ok()?;

                if entry.file_type().is_file() {
                    let path = entry.path();
                    let repo = path.strip_prefix(&manifest_dir).ok()?;
                    Some(repo.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(manifests)
    }

    pub async fn list_repo_tags(&self, repo: &str) -> Result<Vec<String>> {
        let repo_manifest_dir = self.path.join(MANIFESTS_DIR).join(repo);
        let tags = WalkDir::new(&repo_manifest_dir)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                if entry.file_type().is_file() {
                    let path = entry.path();
                    let repo = path.strip_prefix(&repo_manifest_dir).ok()?;
                    Some(repo.to_string_lossy().into_owned())
                } else {
                    None
                }
            })
            .collect();

        Ok(tags)
    }

    pub async fn delete_blob(&self, digest: &str) -> Result<()> {
        let blob_path = self.path.join(BLOBS_DIR).join(digest);
        tokio::fs::remove_file(blob_path).await?;
        Ok(())
    }
}

pub fn is_path_writable(path: &PathBuf) -> io::Result<bool> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    let permissions = metadata.permissions();
    Ok(!permissions.readonly())
}

pub fn is_digest(maybe_digest: &str) -> bool {
    for alg in &SUPPORTED_DIGESTS {
        if maybe_digest.starts_with(&format!("{}:", alg)) {
            return true;
        }
    }

    false
}

pub fn is_digest2(maybe_digest: &str) -> Option<(&str, &str)> {
    for alg in &SUPPORTED_DIGESTS {
        if maybe_digest.starts_with(&format!("{}:", alg)) {
            let parts: Vec<&str> = maybe_digest.splitn(2, ':').collect();
            return Some((alg, parts[1]));
        }
    }

    None
}

fn bytes_to_stream(bytes: Bytes) -> impl Stream<Item = Result<Bytes, reqwest::Error>> {
    futures::stream::once(async move { Ok(bytes) })
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::tempdir;

    use super::*;
    use crate::trow_server::manifest;

    #[test]
    fn trow_storage_backend_new() {
        let dir = tempdir().unwrap();
        let store = TrowStorageBackend::new(dir.into_path()).unwrap();
        assert!(store.path.join("blobs").exists());
        assert!(store.path.join("manifests").exists());
    }

    #[tokio::test]
    async fn trow_storage_backend_write_blob_stream() {
        let store = TrowStorageBackend::new(tempdir().unwrap().into_path()).unwrap();
        let stream = pin!(bytes_to_stream(Bytes::from("test")));
        let location = store
            .write_blob_stream("sha256:1234", stream)
            .await
            .unwrap();
        assert!(location.exists());
        assert!(location == store.path.join("blobs").join("sha256").join("1234"));
    }

    #[tokio::test]
    async fn trow_storage_backend_write_image_manifest() {
        let store = TrowStorageBackend::new(tempdir().unwrap().into_path()).unwrap();
        let mut manifest = Manifest::V2(manifest::ManifestV2 {
            schema_version: 2,
            media_type: Some("application/vnd.docker.distribution.manifest.v2+json".to_string()),
            config: manifest::Object {
                media_type: "application/vnd.docker.container.image.v1+json".to_string(),
                size: Some(7027),
                digest: "sha256:3b4e5a".to_string(),
            },
            layers: vec![],
        });
        let manifest_bytes = serde_json::to_vec(&manifest).unwrap();
        let location = store
            .write_image_manifest(Bytes::from(manifest_bytes), "zozo/image", "latest", false)
            .await
            .unwrap();
        fs::remove_file(location).unwrap();
        // Now let's test verification
        match manifest {
            Manifest::V2(ref mut m) => {
                m.layers.push(manifest::Object {
                    media_type: "application/vnd.docker.image.rootfs.diff.tar.gzip".to_string(),
                    size: Some(7027),
                    digest: "sha256:3b4e5a".to_string(),
                });
                let stream = pin!(bytes_to_stream(Bytes::from("test")));
                store
                    .write_blob_stream("sha256:3b4e5a", stream)
                    .await
                    .unwrap();
            }
            _ => unreachable!(),
        }
        let manifest_bytes = serde_json::to_vec(&manifest).unwrap();
        store
            .write_image_manifest(Bytes::from(manifest_bytes), "zozo/image", "latest", true)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn trow_storage_backend_get_manifest_digest() {
        let store = TrowStorageBackend::new(tempdir().unwrap().into_path()).unwrap();

        let fd = store.path.join("manifests").join("zozo").join("image");
        fs::create_dir_all(&fd).unwrap();
        let mut file = File::create(fd.join("latest")).unwrap();
        let entry = ManifestHistoryEntry {
            digest: "sha256:1234".to_string(),
            date: Utc::now(),
        };
        file.write_all(serde_json::to_string(&entry).unwrap().as_bytes())
            .unwrap();
        file.flush().unwrap();

        let digest = store
            .get_manifest_digest("zozo/image", "latest")
            .await
            .unwrap();
        assert!(digest == "sha256:1234");
    }

    #[tokio::test]
    async fn trow_storage_backend_get_manifest_history() {
        let store = TrowStorageBackend::new(tempdir().unwrap().into_path()).unwrap();

        let fd = store.path.join("manifests").join("zozo").join("image");
        fs::create_dir_all(&fd).unwrap();
        let mut file = File::create(fd.join("latest")).unwrap();
        let entry = ManifestHistoryEntry {
            digest: "sha256:1234".to_string(),
            date: Utc::now(),
        };
        file.write_all(&serde_json::to_vec(&entry).unwrap())
            .unwrap();
        drop(file);
        // file.flush().unwrap();

        let history = store
            .get_manifest_history("zozo/image", "latest")
            .await
            .unwrap();
        assert!(history.len() == 1);
        assert!(history[0].digest == "sha256:1234");
    }
}
