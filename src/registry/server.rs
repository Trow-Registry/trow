use std::path::PathBuf;
use std::str;

use anyhow::Result;

// use super::manifest::Manifest;
use super::proxy::RegistryProxiesConfig;
use super::storage::TrowStorageBackend;
use super::ImageValidationConfig;

pub static SUPPORTED_DIGESTS: [&str; 1] = ["sha256"];
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
    pub proxy_registry_config: RegistryProxiesConfig,
    pub image_validation_config: Option<ImageValidationConfig>,
}

impl TrowServer {
    pub fn new(
        data_path: PathBuf,
        proxy_registry_config: Option<RegistryProxiesConfig>,
        image_validation_config: Option<ImageValidationConfig>,
    ) -> Result<Self> {
        let proxy_registry_config = proxy_registry_config.unwrap_or_default();

        let svc = Self {
            proxy_registry_config,
            image_validation_config,
            storage: TrowStorageBackend::new(data_path)?,
        };
        Ok(svc)
    }

    // pub async fn get_blob(
    //     &self,
    //     repo_name: &str,
    //     digest: &Digest,
    // ) -> Result<BlobReader<impl AsyncRead>, RegistryError> {
    //     event!(
    //         Level::DEBUG,
    //         "Getting read location for blob {} in {}",
    //         digest,
    //         repo_name
    //     );
    //     let stream = match self.storage.get_blob_stream(repo_name, digest).await {
    //         Ok(stream) => stream,
    //         Err(StorageBackendError::BlobNotFound(_)) => return Err(RegistryError::NotFound),
    //         Err(_) => return Err(RegistryError::Internal),
    //     };
    //     Ok(BlobReader::new(digest.clone(), stream).await)
    // }

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
