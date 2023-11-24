use std::collections::HashSet;
use std::fs::{self, DirEntry, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::{io, str};

use anyhow::{anyhow, Context, Result};
use async_recursion::async_recursion;
use chrono::prelude::*;
use futures::future::try_join_all;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{self, Method};
use super::errors::{DigestValidationError, ProxyError};
use tokio::time::Duration;
use tracing::{event, Level};
use uuid::Uuid;
use object_store::{ObjectStore, path};

use super::api_types::*;
use super::digest::sha256_tag_digest;
use super::image::RemoteImage;
use super::manifest::{manifest_media_type, FromJson, Manifest};
use super::proxy_auth::{ProxyClient, SingleRegistryProxyConfig};
use super::temporary_file::TemporaryFile;
use super::{metrics, ImageValidationConfig, RegistryProxiesConfig};
use super::server::{SUPPORTED_DIGESTS};

pub struct TrowStorageBackend {
    object_store: Arc<dyn ObjectStore>,
    // TODO:
    // - concurency locks for single server mode
    // - concurency locks using raft ?
    // - local cache for remote storage ?
}

impl TrowStorageBackend {
    pub fn get_object_store(&self) -> &dyn ObjectStore {
        &*self.object_store
    }

    pub fn get_catalog_path_for_blob(digest: &str) -> Result<PathBuf> {
        let mut iter = digest.split(':');
        let alg = iter
            .next()
            .ok_or_else(|| anyhow!("Digest '{digest}' did not contain alg component"))?;
        if !SUPPORTED_DIGESTS.contains(&alg) {
            return Err(anyhow!("Hash algorithm '{alg}' not supported"));
        }
        let val = iter
            .next()
            .ok_or_else(|| anyhow!("Digest '{digest}' did not contain value component"))?;
        if let Some(val) = iter.next() {
            return Err(anyhow!("Digest '{digest}' contains too many elements"));
        }

        Ok(PathBuf::from("blobs").join(alg).join(val))
    }

    // fn get_digest_from_manifest(&self, repo_name: &str, reference: &str) -> Result<String> {
    //     get_digest_from_manifest_path(self.manifests_path.join(repo_name).join(reference))
    // }

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



}
