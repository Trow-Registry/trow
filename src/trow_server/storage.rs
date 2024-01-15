use std::borrow::Cow;
use std::fs::{self, DirEntry, File};
use std::path::{Path, PathBuf};
use std::{io, str};
use futures::stream;

use crate::trow_server::server::is_digest;
use anyhow::{anyhow, Context, Result};
use bytes::Buf;
use bytes::Bytes;
use chrono::prelude::*;
use futures::stream::StreamExt;
use futures::Stream;
use reqwest;
use tokio::io::AsyncWriteExt;
use tokio::time::Duration;
use tracing::{event, Level};
use walkdir::WalkDir;

use super::api_types::*;
use super::digest;
use super::manifest::Manifest;

// Storage Driver Error
#[derive(thiserror::Error, Debug)]
pub enum StorageBackendError {
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

static MANIFESTS_DIR: &str = "manifests";
static BLOBS_DIR: &str = "blobs";
static UPLOADS_DIR: &str = "scratch";

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

    async fn get_manifest_digest(&self, repo_name: &str, reference: &str) -> Result<String> {
        let path = self
            .path
            .join(MANIFESTS_DIR)
            .join(repo_name)
            .join(reference);
        let manifest_history_bytes = tokio::fs::read(&path).await?;
        let latest_entry_bytes = manifest_history_bytes
            .split(|b| *b == b'\n')
            .last()
            .ok_or(anyhow!("Empty manifest history"))?;
        let latest_entry: ManifestHistoryEntry = serde_json::from_slice(latest_entry_bytes)?;

        Ok(latest_entry.digest)
    }

    async fn get_manifest(&self, repo_name: &str, reference: &str) -> Result<Manifest> {
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
        let manifest: Manifest = serde_json::from_slice(&manifest_bytes)?;
        Ok(manifest)
    }

    pub async fn write_blob_stream<S>(&self, digest: &str, mut stream: S) -> Result<PathBuf>
    where
        S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
    {
        let tmp_location = self.path.join(UPLOADS_DIR).join(digest);
        let location = self.path.join(BLOBS_DIR).join(digest);
        if location.exists() {
            event!(Level::INFO, "Blob already exists");
            return Ok(location);
        }
        let tmp_file = tokio::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&tmp_location)
            .await;

        let mut tmp_file = match tmp_file {
            // All good
            Ok(tmpf) => tmpf,
            // Special case: blob is being concurrently fetched
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                event!(Level::INFO, "Waiting for concurently fetched blob");
                while tmp_location.exists() {
                    // wait for download to be done (temp file to be moved)
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
                return Ok(location);
            }
            Err(e) => {
                return Err(anyhow!("Could not open tmp file: {}", e));
            }
        };

        while let Some(chunk) = stream.next().await {
            let chunk = catch_err_tmp_file(chunk, &tmp_location)?;
            catch_err_tmp_file(tmp_file.write_all(&chunk).await, &tmp_location)?;
        }
        catch_err_tmp_file(tmp_file.flush().await, &tmp_location)?;
        catch_err_tmp_file(
            tokio::fs::rename(&tmp_location, &location).await,
            &tmp_location,
        )?;
        Ok(location)
    }

    pub async fn write_blob_part<S>(&self, digest: &str, mut stream: S) -> Result<PathBuf>
    where
        S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
    {
        let tmp_location = self.path.join(UPLOADS_DIR).join(digest);
        let location = self.path.join(BLOBS_DIR).join(digest);
        if location.exists() {
            event!(Level::INFO, "Blob already exists");
            return Ok(location);
        }
        let tmp_file = tokio::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&tmp_location)
            .await;

        let mut tmp_file = match tmp_file {
            // All good
            Ok(tmpf) => tmpf,
            // Special case: blob is being concurrently fetched
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                event!(Level::INFO, "Waiting for concurently fetched blob");
                while tmp_location.exists() {
                    // wait for download to be done (temp file to be moved)
                    tokio::time::sleep(Duration::from_millis(200)).await;
                }
                return Ok(location);
            }
            Err(e) => {
                return Err(anyhow!("Could not open tmp file: {}", e));
            }
        };

        while let Some(chunk) = stream.next().await {
            let chunk = catch_err_tmp_file(chunk, &tmp_location)?;
            catch_err_tmp_file(tmp_file.write_all(&chunk).await, &tmp_location)?;
        }
        catch_err_tmp_file(tmp_file.flush().await, &tmp_location)?;
        catch_err_tmp_file(tokio::fs::rename(&tmp_location, &location).await, &tmp_location)?;
        Ok(location)
    }

    pub async fn write_image_manifest(
        &self,
        manifest: Bytes,
        repo_name: &str,
        tag: &str,
    ) -> Result<PathBuf> {
        let digest = digest::sha256_digest(manifest.as_ref().reader())?;
        let location = self.write_blob_stream(&digest, stream::once(Ok(manifest))).await?;

        // save link tag -> manifest
        let entry = ManifestHistoryEntry {
            digest: digest.clone(),
            timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true),
        };
        let entry_str = serde_json::to_string(&entry)?;
        let manifest_history_loc = self.path.join(MANIFESTS_DIR).join(repo_name).join(tag);
        let mut manifest_history_file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&manifest_history_loc)
            .await?;
        manifest_history_file.write_all(entry_str.as_bytes()).await?;
        manifest_history_file.flush().await?;

        Ok(location)
    }

    pub async fn get_manifest_history(
        &self,
        repo_name: &str,
        tag: &str,
    ) -> Result<Vec<ManifestHistoryEntry>> {
        let manifest_history_loc = self.path.join(MANIFESTS_DIR).join(repo_name).join(tag);
        let history_raw = tokio::fs::read(manifest_history_loc)
            .await
            .context("Could not read manifest history")?;
        let history =
            serde_json::from_slice(&history_raw).context("Could not parse manifest history")?;
        Ok(history)
    }

    // pub async fn list_repos(&self) -> Result<Vec<String>> {
    //     let manifest_dir = self.path.join(MANIFESTS_DIR);
    //     // let dirs = tokio::fs::read_dir(manifest_dir);
    //     let manifests = WalkDir::new(manifest_dir).into_iter().filter_map(|entry| {
    //         let entry = entry.ok()?;

    //         if entry.file_type().is_file() {
    //             let path = entry.path();
    //             let repo = path.strip_prefix(manifest_dir).ok()?;
    //             Some(repo.to_string_lossy())
    //         } else {
    //             None
    //         }
    //     }).collect();
    // }

    pub async fn list_repo_tags(&self, repo: &str) -> Result<Vec<String>> {
        let repo_manifest_dir = self.path.join(MANIFESTS_DIR).join(repo);
        let rmd = &repo_manifest_dir;
        let tags = WalkDir::new(rmd)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                if entry.file_type().is_file() {
                    let path = entry.path();
                    let repo = path.strip_prefix(rmd).ok()?;
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

fn does_manifest_match_digest(manifest: &DirEntry, digest: &str) -> bool {
    digest
        == match get_digest_from_manifest_path(manifest.path()) {
            Ok(test_digest) => test_digest,
            Err(e) => {
                event!(Level::WARN, "Failure reading repo {:?}", e);
                "NO_MATCH".to_string()
            }
        }
}

fn is_path_writable(path: &PathBuf) -> io::Result<bool> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    let permissions = metadata.permissions();
    Ok(!permissions.readonly())
}

fn get_digest_from_manifest_path<P: AsRef<Path>>(path: P) -> Result<String> {
    let manifest = fs::read_to_string(path)?;
    let latest_digest_line = manifest
        .lines()
        .last()
        .ok_or_else(|| anyhow!("Empty manifest"))?;
    // Each line is `{digest} {date}`
    let latest_digest = latest_digest_line
        .split(' ')
        .next()
        .ok_or_else(|| anyhow!("Invalid manifest line: `{}`", latest_digest_line))?;

    Ok(latest_digest.to_string())
}



/// Transform any result into an anyhow Result & removes the tmp file
fn catch_err_tmp_file<T, E: std::fmt::Display>(
    result: Result<T, E>,
    tmp_location: &Path,
) -> Result<T> {
    match result {
        Ok(t) => Ok(t),
        Err(e) => {
            fs::remove_file(tmp_location)
                .map_err(|e| event!(Level::WARN, "Could not remove tmp file: {:?}", e));
            return Err(anyhow!("Error downloading blob: {}", e));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::tempdir;

    #[test]
    fn build_object_store() {
        let dir = tempdir().unwrap();
        let url = format!("file://{}", dir.path().to_str().unwrap());
        let store = TrowStorageBackend::new(url.to_string()).unwrap();
    }
}
