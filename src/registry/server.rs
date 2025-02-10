use std::path::PathBuf;
use std::str;

// use super::manifest::Manifest;
use super::storage::TrowStorageBackend;
use super::{ConfigFile, StorageBackendError};

pub static PROXY_DIR: &str = "f/"; //Repositories starting with this are considered proxies

/* Struct implementing callbacks for the Frontend
 *
 * _manifests_path_: path to where the manifests are
 * _layers_path_: path to where blobs are stored
 * _scratch_path_: path to temporary storage for uploads
 *
 * Each "route" gets a clone of this struct.
 * The Arc makes sure they all point to the same data.
 */
#[derive(Clone, Debug)]
pub struct TrowServer {
    pub storage: TrowStorageBackend,
    pub config: ConfigFile,
}

impl TrowServer {
    pub fn new(
        data_path: PathBuf,
        config: Option<ConfigFile>,
    ) -> Result<Self, StorageBackendError> {
        let svc = Self {
            config: config.unwrap_or_default(),
            storage: TrowStorageBackend::new(data_path)?,
        };
        Ok(svc)
    }

    // Readiness check
    pub async fn is_ready(&self) -> bool {
        match self.storage.is_ready().await {
            Ok(()) => true,
            Err(e) => {
                tracing::error!("Storage backend not ready: {e}");
                false
            }
        }
    }

    pub async fn is_healthy(&self) -> bool {
        true
    }
}
