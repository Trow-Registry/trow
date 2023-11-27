use std::collections::HashSet;
use std::fs::{self, DirEntry, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::{io, str};

use crate::response::manifest_history;

use super::errors::{DigestValidationError, ProxyError};
use anyhow::{anyhow, Context, Result};
use async_recursion::async_recursion;
use bytes::Buf;
use bytes::Bytes;
use chrono::prelude::*;
use futures::future::try_join_all;
use futures::stream::StreamExt;
use futures::Stream;
use object_store::{path, GetOptions, ObjectStore};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{self, Method};
use tokio::io::AsyncWriteExt;
use tokio::time::Duration;
use tracing::{event, Level};
use uuid::Uuid;

use super::api_types::*;
use super::digest;
use super::image::RemoteImage;
use super::manifest::{manifest_media_type, FromJson, Manifest};
use super::proxy_auth::{ProxyClient, SingleRegistryProxyConfig};
use super::server::SUPPORTED_DIGESTS;
use super::temporary_file::TemporaryFile;
use super::{metrics, ImageValidationConfig, RegistryProxiesConfig};

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

#[derive(Clone, Debug)]
pub struct TrowStorageBackend {
    object_store: Arc<dyn ObjectStore>,
    // TODO:
    // - concurency locks for single server mode
    // - concurency locks using raft ?
    // - local cache for remote storage ?
}

impl TrowStorageBackend {
    pub fn new(url: String) -> Result<Self> {
        let url = reqwest::Url::from_str(&url).unwrap();
        let (store, path ) = object_store::parse_url(&url).unwrap();
        Ok(Self {
            object_store: Arc::from(store),
        })
    }

    pub fn get_object_store(&self) -> &dyn ObjectStore {
        &*self.object_store
    }

    async fn get_digest_from_manifest(&self, repo_name: &str, reference: &str) -> Result<String> {
        let path = path::Path::from_iter(["manifests", repo_name, reference]);
        let manifest_bytes = self.object_store.get(&path).await?.bytes().await?;
        let manifest = std::str::from_utf8(&manifest_bytes)?;

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

    async fn get_blob ()

    pub async fn write_blob_stream<S>(&self, digest: &str, mut stream: S) -> Result<()>
    where
        S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
    {
        let location = path::Path::from_iter(["blobs", digest]);
        let (multipart_id, mut writer) = self.object_store.put_multipart(&location).await?;

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(chunk) => {
                    writer.write_all(&chunk).await?;
                }
                Err(e) => {
                    let _err = self
                        .object_store
                        .abort_multipart(&location, &multipart_id)
                        .await?;
                    return Err(anyhow!("Error downloading blob: {}", e));
                }
            }
        }
        writer.shutdown().await?;
        Ok(())
    }

    pub async fn write_image_manifest(
        &self,
        manifest: Bytes,
        repo_name: &str,
        tag: &str,
    ) -> Result<()> {
        let digest = digest::sha256_digest(manifest.as_ref().reader())?;

        // write image manifest as a blob
        let manifest_blob_loc = path::Path::from_iter(["blobs", "sha256", &digest]);
        match self.object_store.head(&manifest_blob_loc).await {
            Ok(_) => {
                event!(Level::INFO, "Manifest already exists");
            }
            Err(object_store::Error::NotFound { .. }) => {
                event!(Level::INFO, "Writing manifest");
                self.object_store.put(&manifest_blob_loc, manifest).await?;
            }
            e @ Err(_) => {
                e.context("Could not HEAD manifest blob")?;
            }
        }

        // save link tag -> manifest
        let manifest_history_loc = path::Path::from_iter(["manifests", repo_name, tag]);
        let mut manifest_history: Vec<ManifestHistoryEntry> = {
            let get_object = self.object_store.get(&manifest_history_loc).await;
            match get_object {
                Ok(get_result) => {
                    let bytes = get_result.bytes().await?;
                    serde_json::from_slice(&bytes).unwrap_or_else(|_| {
                        event!(Level::WARN, "Could not parse manifest history");
                        Vec::new()
                    })
                }
                Err(object_store::Error::NotFound { .. }) => Vec::new(),
                Err(e) => return Err(anyhow!(e).context("Could not GET manifest history file")),
            }
        };
        manifest_history.push(ManifestHistoryEntry {
            digest: digest.clone(),
            timestamp: Utc::now().to_rfc3339_opts(SecondsFormat::Nanos, true),
        });
        let manifest_history_bytes = Bytes::from(serde_json::to_vec(&manifest_history)?);
        self.object_store
            .put(&manifest_history_loc, manifest_history_bytes)
            .await?;

        Ok(())
    }

    pub async fn get_manifest_history(
        &self,
        repo_name: &str,
        tag: &str,
    ) -> Result<Vec<ManifestHistoryEntry>> {
        let manifest_history_loc = path::Path::from_iter(["manifests", repo_name, tag]);
        let get_result = self
            .object_store
            .get(&manifest_history_loc)
            .await
            .context("Could not GET manifest history file")?;
        let bytes = get_result.bytes().await?;
        let history = serde_json::from_slice(&bytes).context("Could not parse manifest history")?;
        Ok(history)
    }

    pub async fn list_repos(&self) -> Result<Vec<String>> {
        let manifest_dir = path::Path::from_iter(["manifests"]);
        let repo_paths = self
            .object_store
            .list_with_delimiter(Some(&manifest_dir))
            .await?
            .common_prefixes;

        let entries = repo_paths
            .into_iter()
            .map(|p| {
                let p = p.to_string();
                let (_, repo) = p.split_once('/').unwrap();
                repo.to_owned()
            })
            .collect();

        Ok(entries)
    }

    pub async fn list_repo_tags(&self, repo: &str) -> Result<Vec<String>> {
        let mut manifests = Vec::new();
        let repo_dir = path::Path::from_iter(["manifests", repo]);
        let mut manifest_iter = self.object_store.list(Some(&repo_dir));
        while let Some(manifest) = manifest_iter.next().await {
            let manifest = manifest.context("Could not list manifest")?;
            let path = String::from(manifest.location);
            let invalid_manifest_path_err = || anyhow!("Invalid manifest path `{path}`");
            let (_, tag) = path
                .rsplit_once("/")
                .ok_or_else(invalid_manifest_path_err)?;
            manifests.push(tag.to_owned());
        }
        Ok(manifests)
    }
}

// pub fn get_catalog_path_for_blob(digest: &str) -> Result<PathBuf> {
//     let mut iter = digest.split(':');
//     let alg = iter
//         .next()
//         .ok_or_else(|| anyhow!("Digest '{digest}' did not contain alg component"))?;
//     if !SUPPORTED_DIGESTS.contains(&alg) {
//         return Err(anyhow!("Hash algorithm '{alg}' not supported"));
//     }
//     let val = iter
//         .next()
//         .ok_or_else(|| anyhow!("Digest '{digest}' did not contain value component"))?;
//     if let Some(val) = iter.next() {
//         return Err(anyhow!("Digest '{digest}' contains too many elements"));
//     }

//     Ok(PathBuf::from("blobs").join(alg).join(val))
// }

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

// /**
//  * Checks a file matches the given digest.
//  *
//  * TODO: should be able to use range of hashes.
//  * TODO: check if using a static for the hasher speeds things up.
//  */
// fn validate_digest(file: &PathBuf, digest: &str) -> Result<()> {
//     let f = File::open(file)?;
//     let reader = BufReader::new(f);

//     let calculated_digest = sha256_tag_digest(reader)?;

//     if calculated_digest != digest {
//         event!(
//             Level::ERROR,
//             "Upload did not match given digest. Was given {} but got {}",
//             digest,
//             calculated_digest
//         );
//         return Err(anyhow!(DigestValidationError {
//             user_digest: digest.to_string(),
//             actual_digest: calculated_digest,
//         }));
//     }

//     Ok(())
// }

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
