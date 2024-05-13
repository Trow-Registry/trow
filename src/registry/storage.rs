use core::ops::Range;
use std::borrow::Cow;
use std::fs::{self, File};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::pin::pin;
use std::{io, str};

use bytes::{Buf, Bytes};
use chrono::prelude::*;
use futures::{AsyncReadExt, Stream};
use tokio::io::AsyncWriteExt;
use tokio::time::Duration;
use tokio_util::compat::TokioAsyncReadCompatExt;
use tracing::{event, Level};
use walkdir::WalkDir;

use super::manifest::{Manifest, ManifestError};
use super::server::{PROXY_DIR, SUPPORTED_DIGESTS};
use crate::registry::blob_storage::Stored;
use crate::registry::catalog_operations::HistoryEntry;
use crate::registry::temporary_file::TemporaryFile;
use crate::registry::Digest;
use crate::types::BoundedStream;

// Storage Driver Error
#[derive(thiserror::Error, Debug)]
pub enum StorageBackendError {
    #[error("the name `{0}` is not valid")]
    InvalidName(String),
    #[error("Manifest is not valid ({0:?})")]
    InvalidManifest(#[from] ManifestError),
    #[error("Blob not found:{0}")]
    BlobNotFound(PathBuf),
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
    fn init_create_path(root: &Path, dir: &str) -> Result<(), StorageBackendError> {
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
                Err(StorageBackendError::Io(e))
            }
        }
    }

    pub fn new(path: PathBuf) -> Result<Self, StorageBackendError> {
        Self::init_create_path(&path, MANIFESTS_DIR)?;
        Self::init_create_path(&path, BLOBS_DIR)?;
        Self::init_create_path(&path, UPLOADS_DIR)?;

        Ok(Self { path })
    }

    // TODO: replace this by DB
    pub async fn get_manifest_digest(
        &self,
        repo_name: &str,
        tag: &str,
    ) -> Result<Digest, StorageBackendError> {
        event!(Level::DEBUG, "Get manifest digest {repo_name}:{tag}");
        let path = self.path.join(MANIFESTS_DIR).join(repo_name).join(tag);
        let manifest_history_bytes = tokio::fs::read(&path).await?;
        let latest_entry_bytes =
            manifest_history_bytes
                .lines()
                .last()
                .ok_or(StorageBackendError::Internal(Cow::Borrowed(
                    "Empty manifest history",
                )))??;
        let latest_entry: HistoryEntry =
            serde_json::from_str(&latest_entry_bytes).map_err(|e| {
                StorageBackendError::Internal(Cow::Owned(format!("Invalid manifest entry: {e}")))
            })?;

        Ok(Digest::try_from_raw(&latest_entry.digest).unwrap())
    }

    pub async fn get_manifest(
        &self,
        repo_name: &str,
        reference: &str,
    ) -> Result<Manifest, StorageBackendError> {
        event!(Level::DEBUG, "Get manifest {repo_name}:{reference}");
        let digest = match Digest::try_from_raw(reference) {
            Ok(d) => d,
            Err(_) => self.get_manifest_digest(repo_name, reference).await?,
        };
        let manifest_stream = self.get_blob_stream(repo_name, &digest).await?;
        let mut manifest_bytes = Vec::new();
        manifest_stream
            .reader()
            .read_to_end(&mut manifest_bytes)
            .await?;
        // let manifest_bytes = tokio::fs::read(&path).await?;
        let manifest = Manifest::from_vec(manifest_bytes)?;
        Ok(manifest)
    }

    pub async fn get_blob_stream<'a>(
        &'a self,
        _repo_name: &str,
        digest: &Digest,
    ) -> Result<BoundedStream<impl futures::AsyncRead>, StorageBackendError> {
        event!(Level::DEBUG, "Get blob {_repo_name}:{digest}");
        let path = self
            .path
            .join(BLOBS_DIR)
            .join(digest.algo_str())
            .join(&digest.hash);
        let file = tokio::fs::File::open(&path).await.map_err(|e| {
            event!(Level::ERROR, "Could not open blob: {}", e);
            StorageBackendError::BlobNotFound(path)
        })?;
        let size = file.metadata().await?.len() as usize;
        Ok(BoundedStream::new(size, file.compat()))
    }

    pub async fn write_blob_stream<'a, S, E>(
        &'a self,
        digest: &Digest,
        stream: S,
        verify: bool,
    ) -> Result<PathBuf, StorageBackendError>
    where
        S: Stream<Item = Result<Bytes, E>> + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        event!(Level::DEBUG, "Write blob {digest}");
        let tmp_location = self.path.join(UPLOADS_DIR).join(digest.to_string());
        let location = self
            .path
            .join(BLOBS_DIR)
            .join(digest.algo_str())
            .join(&digest.hash);
        if location.exists() {
            event!(Level::INFO, "Blob already exists");
            return Ok(location);
        }
        tokio::fs::create_dir_all(location.parent().unwrap()).await?;
        let mut tmp_file = match TemporaryFile::new(tmp_location.clone()).await {
            // All good
            Ok(tmpf) => tmpf,
            // Special case: blob is being concurrently fetched
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                event!(Level::INFO, "Waiting for concurrently fetched blob");
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
        if verify {
            let reader = std::fs::File::open(tmp_file.path()).map_err(|e| {
                StorageBackendError::Internal(Cow::Owned(format!("Could not open tmp file: {e}")))
            })?;
            let tmp_digest = Digest::try_sha256(reader).map_err(|e| {
                StorageBackendError::Internal(Cow::Owned(format!(
                    "Could not calculate digest of blob: {e}"
                )))
            })?;
            if &tmp_digest != digest {
                return Err(StorageBackendError::InvalidDigest);
            }
        }

        tmp_file.rename(&location).await?;
        Ok(location)
    }

    /// Requests to start a resumable upload for the given repository.
    /// Returns a session identifier for the upload.
    pub async fn request_blob_upload(
        &self,
        repo_name: &str,
    ) -> Result<String, StorageBackendError> {
        event!(Level::DEBUG, "Request blob upload for {repo_name}");
        if is_proxy_repo(repo_name) {
            return Err(StorageBackendError::InvalidName(
                "Name reserved for proxied repos".into(),
            ));
        }
        let upload_id = uuid::Uuid::new_v4().to_string();
        let tmp_location = self.path.join(UPLOADS_DIR).join(&upload_id);
        TemporaryFile::new(tmp_location).await?;
        Ok(upload_id)
    }

    /// Writes part of a blob to disk.
    /// Upload then needs to be "completed"
    pub async fn write_blob_part_stream<'a, S, E>(
        &'a self,
        upload_id: &str,
        stream: S,
        range: Option<Range<u64>>,
    ) -> Result<Stored, WriteBlobRangeError>
    where
        S: Stream<Item = Result<Bytes, E>> + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        event!(Level::DEBUG, "Write blob part {upload_id} ({range:?})");
        let tmp_location = self.path.join(UPLOADS_DIR).join(upload_id);
        let (mut tmp_file, seek_pos) = TemporaryFile::append(tmp_location).await.map_err(|e| {
            event!(Level::ERROR, "Could not open tmp file: {}", e);
            match e.kind() {
                io::ErrorKind::NotFound => WriteBlobRangeError::NotFound,
                io::ErrorKind::AlreadyExists => WriteBlobRangeError::InvalidContentRange,
                _ => WriteBlobRangeError::Internal,
            }
        })?;
        let range_len = range.as_ref().map(|r| r.end - r.start);

        if let Some(range) = &range {
            if range.start != seek_pos {
                return Err(WriteBlobRangeError::InvalidContentRange);
            }
        }
        let bytes_written = tmp_file
            .write_stream(stream)
            .await
            .map_err(|_e| WriteBlobRangeError::Internal)? as u64;

        if matches!(range_len, Some(len) if len != bytes_written) {
            return Err(WriteBlobRangeError::InvalidContentRange);
        }
        tmp_file.untrack();

        Ok(Stored {
            total_stored: bytes_written + seek_pos,
            chunk: bytes_written,
        })
    }

    pub async fn complete_blob_write(
        &self,
        upload_id: &str,
        user_digest: &Digest,
    ) -> Result<(), StorageBackendError> {
        event!(Level::DEBUG, "Complete blob write {upload_id}");
        let tmp_location = self.path.join(UPLOADS_DIR).join(upload_id);
        let final_location = self
            .path
            .join(BLOBS_DIR)
            .join(user_digest.algo_str())
            .join(&user_digest.hash);
        let f = File::open(&tmp_location)?; // ERRRR
        let calculated_digest = Digest::try_sha256(f)?;
        if &calculated_digest != user_digest {
            event!(
                Level::ERROR,
                "Upload did not match given digest. Was given {} but got {}",
                user_digest,
                calculated_digest
            );
            return Err(StorageBackendError::InvalidDigest);
        }
        std::fs::create_dir_all(final_location.parent().unwrap()).unwrap();
        std::fs::rename(tmp_location, final_location).expect("Error moving blob to final location");
        Ok(())
    }

    pub async fn write_tag(
        &self,
        repo_name: &str,
        tag: &str,
        digest: &Digest,
    ) -> Result<(), StorageBackendError> {
        event!(Level::DEBUG, "Write tag {repo_name}:{tag}");
        let entry = HistoryEntry {
            digest: digest.to_string(),
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
        Ok(())
    }

    pub async fn write_image_manifest(
        &self,
        manifest: Bytes,
        repo_name: &str,
        tag: &str,
        verify_assets: bool,
    ) -> Result<PathBuf, StorageBackendError> {
        event!(Level::DEBUG, "Write image manifest {repo_name}:{tag}");
        let mani_digest = Digest::try_sha256(manifest.as_ref().reader()).map_err(|e| {
            StorageBackendError::Internal(Cow::Owned(format!(
                "Could not calculate digest of manifest: {e}"
            )))
        })?;
        if verify_assets {
            let manifest = Manifest::from_bytes(manifest.clone())?;
            for digest in manifest.get_local_asset_digests()? {
                let blob_path = self
                    .path
                    .join(BLOBS_DIR)
                    .join(digest.algo_str())
                    .join(&digest.hash);
                if !blob_path.exists() {
                    return Err(StorageBackendError::BlobNotFound(blob_path));
                }
            }
        }
        let manifest_stream = bytes_to_stream(manifest);
        if tag.starts_with("sha") {
            assert_eq!(mani_digest.to_string(), tag);
        }
        let location = self
            .write_blob_stream(&mani_digest, pin!(manifest_stream), true)
            .await?;

        // save link tag -> manifest
        self.write_tag(repo_name, tag, &mani_digest).await?;

        Ok(location)
    }

    pub async fn get_manifest_history(
        &self,
        repo_name: &str,
        tag: &str,
    ) -> Result<Vec<HistoryEntry>, StorageBackendError> {
        let manifest_history_loc = self.path.join(MANIFESTS_DIR).join(repo_name).join(tag);
        let history_raw = tokio::fs::read(manifest_history_loc).await?;
        io::Cursor::new(history_raw)
            .lines()
            .map(|man| serde_json::from_slice(man.unwrap().as_bytes()))
            .collect::<Result<Vec<HistoryEntry>, _>>()
            .map_err(|e| {
                StorageBackendError::Internal(Cow::Owned(format!(
                    "Could not parse manifest history ({repo_name}:{tag}): {e}"
                )))
            })
    }

    pub async fn list_repos(&self) -> Result<Vec<String>, StorageBackendError> {
        let manifest_dir = self.path.join(MANIFESTS_DIR);
        // let dirs = tokio::fs::read_dir(manifest_dir);
        let manifests = WalkDir::new(&manifest_dir)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                if entry.file_type().is_file() {
                    let path = entry.path();
                    let repo = path.parent()?.strip_prefix(&manifest_dir).ok()?;
                    Some(repo.to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .fold(Vec::new(), |mut acc, e| {
                if acc.last() != Some(&e) {
                    acc.push(e);
                }
                acc
            });

        Ok(manifests)
    }

    pub async fn list_repo_tags(&self, repo: &str) -> Result<Vec<String>, StorageBackendError> {
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

    pub async fn delete_blob(&self, digest: &Digest) -> Result<(), StorageBackendError> {
        let blob_path = self
            .path
            .join(BLOBS_DIR)
            .join(digest.algo_str())
            .join(&digest.hash);
        tokio::fs::remove_file(blob_path).await?;
        Ok(())
    }

    pub async fn delete_manifest(
        &self,
        repo_name: &str,
        digest: &Digest,
    ) -> Result<(), StorageBackendError> {
        let path = self
            .path
            .join(BLOBS_DIR)
            .join(digest.algo_str())
            .join(&digest.hash);
        if let Err(e) = tokio::fs::remove_file(path).await {
            event!(Level::WARN, "Could not delete manifest file: {}", e);
        }
        let tags = self.list_repo_tags(repo_name).await?;
        for t in tags {
            let manifest_history_loc = self.path.join(MANIFESTS_DIR).join(repo_name).join(t);
            let history_raw = tokio::fs::read(&manifest_history_loc).await?;
            let old_history = String::from_utf8(history_raw).unwrap();
            let new_history: String = old_history
                .lines()
                .filter(|l| !l.contains(&digest.to_string()))
                .collect();
            if new_history.is_empty() {
                tokio::fs::remove_file(&manifest_history_loc).await?;
            } else if new_history.len() != old_history.len() {
                tokio::fs::write(&manifest_history_loc, new_history).await?;
            }
        }
        Ok(())
    }

    pub async fn is_ready(&self) -> Result<(), StorageBackendError> {
        let path = self.path.join("fs-ready");
        let mut file = File::open(path)?;
        let size = file.write(b"Hello World")?;
        if size != 11 {
            return Err(StorageBackendError::Internal(
                "Could not write to file".into(),
            ));
        }
        file.flush()?;
        let metadata = file.metadata()?;
        let permissions = metadata.permissions();
        if permissions.readonly() {
            // impossible ?
            return Err(StorageBackendError::Internal(
                "Read only file system".into(),
            ));
        }
        Ok(())
    }
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

fn is_proxy_repo(repo_name: &str) -> bool {
    repo_name.starts_with(PROXY_DIR)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use crate::registry::manifest;

    #[test]
    fn trow_storage_backend_new() {
        let dir = test_temp_dir::test_temp_dir!();
        let store = TrowStorageBackend::new(dir.as_path_untracked().to_owned()).unwrap();
        assert!(store.path.join("blobs").exists());
        assert!(store.path.join("manifests").exists());
    }

    #[tokio::test]
    async fn trow_storage_backend_write_blob_stream() {
        let dir = test_temp_dir::test_temp_dir!();
        let store = TrowStorageBackend::new(dir.as_path_untracked().to_owned()).unwrap();
        let stream = pin!(bytes_to_stream(Bytes::from("test")));
        let digest = Digest::try_from_raw("sha256:1234").unwrap();
        let location = store
            .write_blob_stream(&digest, stream, false)
            .await
            .unwrap();
        assert!(location.exists());
        assert!(location == store.path.join("blobs").join("sha256").join("1234"));
    }

    #[tokio::test]
    async fn trow_storage_backend_write_image_manifest() {
        let dir = test_temp_dir::test_temp_dir!();
        let store = TrowStorageBackend::new(dir.as_path_untracked().to_owned()).unwrap();
        let mut manifest = manifest::OCIManifest::V2(manifest::OCIManifestV2 {
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
            manifest::OCIManifest::V2(ref mut m) => {
                m.layers.push(manifest::Object {
                    media_type: "application/vnd.docker.image.rootfs.diff.tar.gzip".to_string(),
                    size: Some(7027),
                    digest: "sha256:3b4e5a".to_string(),
                });
                let stream = pin!(bytes_to_stream(Bytes::from("test")));
                let digest = Digest::try_from_raw("sha256:3b4e5a").unwrap();
                store
                    .write_blob_stream(&digest, stream, false)
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
        let dir = test_temp_dir::test_temp_dir!();
        let store = TrowStorageBackend::new(dir.as_path_untracked().to_owned()).unwrap();

        let fd = store.path.join("manifests").join("zozo").join("image");
        fs::create_dir_all(&fd).unwrap();
        let mut file = File::create(fd.join("latest")).unwrap();
        let entry = HistoryEntry {
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
        assert!(digest.to_string() == "sha256:1234");
    }

    #[tokio::test]
    async fn trow_storage_backend_get_manifest_history() {
        let dir = test_temp_dir::test_temp_dir!();
        let store = TrowStorageBackend::new(dir.as_path_untracked().to_owned()).unwrap();

        let fd = store.path.join("manifests").join("zozo").join("image");
        fs::create_dir_all(&fd).unwrap();
        let mut file = File::create(fd.join("latest")).unwrap();
        let entry = HistoryEntry {
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
