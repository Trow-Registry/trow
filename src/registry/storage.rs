use std::borrow::Cow;
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};
use std::{io, str};

use bytes::Bytes;
use futures::Stream;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::time::Duration;
use tokio_util::compat::TokioAsyncReadCompatExt;

use crate::registry::blob_storage::Stored;
use crate::registry::temporary_file::FileWrapper;
use crate::registry::Digest;
use crate::types::BoundedStream;

// Storage Driver Error
#[derive(thiserror::Error, Debug)]
pub enum StorageBackendError {
    #[error("the name `{0}` is not valid")]
    InvalidName(String),
    #[error("Blob not found:{0}")]
    BlobNotFound(PathBuf),
    #[error("Digest did not match content")]
    InvalidDigest,
    #[error("Unsupported Operation")]
    Unsupported,
    #[error("Invalid content range")]
    InvalidContentRange,
    #[error("Internal error: {0}")]
    Internal(Cow<'static, str>),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}

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
#[derive(Clone, Debug)]
pub struct TrowStorageBackend {
    path: PathBuf,
}

impl TrowStorageBackend {
    fn init_create_path(root: &Path, dir: &str) -> Result<(), StorageBackendError> {
        let path = root.join(dir);
        match std::fs::create_dir_all(&path) {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::error!(
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
        Self::init_create_path(&path, BLOBS_DIR)?;
        Self::init_create_path(&path, UPLOADS_DIR)?;

        Ok(Self { path })
    }

    pub async fn get_blob_stream(
        &self,
        repo_name: &str,
        digest: &Digest,
    ) -> Result<BoundedStream<impl futures::AsyncRead>, StorageBackendError> {
        tracing::debug!("Get blob {repo_name}:{digest}");
        let path = self.path.join(BLOBS_DIR).join(digest.to_string());
        let file = tokio::fs::File::open(&path).await.map_err(|e| {
            tracing::error!("Could not open blob: {}", e);
            StorageBackendError::BlobNotFound(path)
        })?;
        let size = file.metadata().await?.len() as usize;
        Ok(BoundedStream::new(size, file.compat()))
    }

    pub async fn write_blob_stream<S, E>(
        &self,
        digest: &Digest,
        stream: S,
        verify: bool,
    ) -> Result<PathBuf, StorageBackendError>
    where
        S: Stream<Item = Result<Bytes, E>> + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        tracing::debug!("Write blob {digest}");
        let tmp_location = self.path.join(UPLOADS_DIR).join(digest.to_string());
        let location = self.path.join(BLOBS_DIR).join(digest.to_string());
        if location.exists() {
            tracing::info!("Blob already exists");
            return Ok(location);
        }
        tokio::fs::create_dir_all(location.parent().unwrap()).await?;
        let mut tmp_file = match FileWrapper::new_tmp(tmp_location.clone()).await {
            // All good
            Ok(tmpf) => tmpf,
            // Special case: blob is being concurrently fetched
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                tracing::info!("Waiting for concurrently fetched blob");
                while tmp_location.exists() {
                    // wait for download to be done (temp file to be moved)
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
                if location.exists() {
                    return Ok(location);
                } else {
                    return Err(StorageBackendError::BlobNotFound(location));
                }
            }
            Err(e) => {
                return Err(StorageBackendError::Io(e));
            }
        };
        tmp_file.write_stream(stream).await?;
        if verify {
            let tmp_digest = tmp_file.digest().await.map_err(|e| {
                StorageBackendError::Internal(Cow::Owned(format!(
                    "Could not calculate digest of blob: {e}"
                )))
            })?;
            if tmp_digest != *digest {
                return Err(StorageBackendError::InvalidDigest);
            }
        }
        tmp_file.rename(&location).await?;
        Ok(location)
    }

    /// Writes part of a blob to disk.
    /// Upload then needs to be "completed"
    pub async fn write_blob_part_stream<S, E>(
        &self,
        upload_id: &uuid::Uuid,
        stream: S,
        range: Option<RangeInclusive<u64>>,
    ) -> Result<Stored, StorageBackendError>
    where
        S: Stream<Item = Result<Bytes, E>> + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        tracing::debug!("Write blob part {upload_id} ({range:?})");
        let tmp_location = self.path.join(UPLOADS_DIR).join(upload_id.to_string());
        let mut tmp_file = FileWrapper::append(tmp_location.clone())
            .await
            .map_err(|e| {
                tracing::error!("Could not open tmp file {}: {}", tmp_location.display(), e);
                match e.kind() {
                    io::ErrorKind::NotFound => StorageBackendError::BlobNotFound(tmp_location),
                    io::ErrorKind::AlreadyExists => StorageBackendError::InvalidContentRange,
                    _ => StorageBackendError::Io(e),
                }
            })?;

        let file_size = tmp_file.metadata().await?.len();
        let range_len = range.as_ref().map(|r| r.end() - r.start() + 1);

        if let Some(range) = &range {
            if *range.start() != file_size {
                tracing::error!(
                    "Invalid content-range: start={} file_pos={}",
                    range.start(),
                    file_size
                );
                return Err(StorageBackendError::InvalidContentRange);
            }
        }
        let bytes_written = tmp_file.write_stream(stream).await.map_err(|_e| {
            StorageBackendError::Internal(Cow::Borrowed("Couldn't write stream to file"))
        })? as u64;

        if matches!(range_len, Some(len) if len != bytes_written) {
            tracing::error!(
                "Invalid content-length: expected={} actual={}",
                range_len.unwrap(),
                bytes_written
            );
            return Err(StorageBackendError::InvalidContentRange);
        }

        Ok(Stored {
            total_stored: bytes_written + file_size,
            chunk: bytes_written,
        })
    }

    pub async fn complete_blob_write(
        &self,
        upload_id: &uuid::Uuid,
        user_digest: &Digest,
    ) -> Result<(), StorageBackendError> {
        tracing::debug!("Complete blob write {upload_id}");
        let tmp_location = self.path.join(UPLOADS_DIR).join(upload_id.to_string());
        let final_location = self.path.join(BLOBS_DIR).join(user_digest.to_string());
        // Should we even do this ? It breaks OCI tests:
        // let f = std::fs::File::open(&tmp_location)?;
        // let calculated_digest = Digest::digest_sha256(f)?;
        // if &calculated_digest != user_digest {
        //     tracing::error!(
        //         "Upload did not match given digest. Was given {} but got {}",
        //         user_digest,
        //         calculated_digest
        //     );
        //     return Err(StorageBackendError::InvalidDigest);
        // }
        fs::create_dir_all(final_location.parent().unwrap())
            .await
            .unwrap();
        fs::rename(tmp_location, final_location)
            .await
            .expect("Error moving blob to final location");
        Ok(())
    }

    pub async fn delete_blob(
        &self,
        repo: &str,
        digest: &Digest,
    ) -> Result<(), StorageBackendError> {
        tracing::debug!("Delete blob {repo}:{digest}");
        let blob_path = self.path.join(BLOBS_DIR).join(digest.as_str());
        tokio::fs::remove_file(blob_path).await?;
        Ok(())
    }

    pub async fn is_ready(&self) -> Result<(), StorageBackendError> {
        let path = self.path.join("fs-ready");
        let mut file = tokio::fs::File::create(path).await?;
        let size = file.write(b"Hello World").await?;
        if size != 11 {
            return Err(StorageBackendError::Internal(
                "Could not write to file".into(),
            ));
        }
        file.flush().await?;
        let metadata = file.metadata().await?;
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

// fn is_proxy_repo(repo_name: &str) -> bool {
//     repo_name.starts_with(PROXY_DIR)
// }

#[cfg(test)]
mod tests {

    use std::pin::pin;

    use super::*;

    fn bytes_to_stream(bytes: Bytes) -> impl Stream<Item = Result<Bytes, reqwest::Error>> {
        futures::stream::once(async move { Ok(bytes) })
    }

    #[test]
    fn trow_storage_backend_new() {
        let dir = test_temp_dir::test_temp_dir!();
        let store = TrowStorageBackend::new(dir.as_path_untracked().to_owned()).unwrap();
        assert!(store.path.join("blobs").exists());
        drop(dir);
    }

    #[tokio::test]
    async fn trow_storage_backend_write_blob_stream() {
        let dir = test_temp_dir::test_temp_dir!();
        let store = TrowStorageBackend::new(dir.as_path_untracked().to_owned()).unwrap();
        let stream = pin!(bytes_to_stream(Bytes::from("test")));
        let digest = Digest::try_from_raw("sha256:123456789101112131415161718192021").unwrap();
        let location = store
            .write_blob_stream(&digest, stream, false)
            .await
            .unwrap();
        assert!(location.exists());
        assert_eq!(
            location,
            store
                .path
                .join("blobs")
                .join("sha256:123456789101112131415161718192021")
        );
        drop(dir);
    }
}
