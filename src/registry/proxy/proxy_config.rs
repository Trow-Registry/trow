use aws_sdk_ecr::config::http::HttpResponse;
use aws_sdk_ecr::error::SdkError;
use aws_sdk_ecr::operation::get_authorization_token::GetAuthorizationTokenError;
use serde::{Deserialize, Serialize};

use crate::registry::digest::DigestError;
use crate::registry::manifest::ManifestReference;
use crate::registry::proxy::remote_image::RemoteImage;
use crate::registry::server::PROXY_DIR;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegistryProxiesConfig {
    pub registries: Vec<SingleRegistryProxyConfig>,
    #[serde(default)]
    pub offline: bool,
    #[serde(default)]
    pub max_size: Option<size::Size>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct SingleRegistryProxyConfig {
    /// What containerd calls "namespace" (ghcr.io, docker.io, ...)
    /// This can be empty !!
    pub host: String,
    /// TODO: insecure currently means "use HTTP", we should also support self-signed TLS
    pub insecure: bool,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Default for RegistryProxiesConfig {
    fn default() -> Self {
        RegistryProxiesConfig {
            registries: Vec::new(),
            offline: true,
            max_size: None,
        }
    }
}

impl RegistryProxiesConfig {
    pub async fn get_proxied_image<'a>(
        &'a self,
        repo_name: &str,
        reference: &ManifestReference,
        ns: Option<String>,
    ) -> Option<RemoteImage<'a>> {
        // All proxies are under "f/"
        let (host, repo) = if let Some(upstream) = ns {
            (upstream, repo_name.to_string())
        } else if repo_name.starts_with(PROXY_DIR) {
            let segments = repo_name.splitn(3, '/').collect::<Vec<_>>();
            debug_assert_eq!(segments[0], "f");
            (segments[1].to_string(), segments[2].to_string())
        } else {
            return None;
        };

        for proxy in self.registries.iter() {
            if proxy.host == host {
                let image = RemoteImage::new(host, repo, reference.clone(), Some(proxy));
                return Some(image);
            }
        }
        Some(RemoteImage::new(host, repo, reference.clone(), None))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DownloadRemoteImageError {
    #[error("DatabaseError: {0}")]
    DbError(#[from] sqlx::Error),
    #[error("Invalid digest: {0}")]
    InvalidDigest(#[from] DigestError),
    #[error("Failed to download image")]
    DownloadAttemptsFailed,
    #[error("Manifest JSON is not canonicalized")]
    ManifestNotCanonicalized,
    #[error("OCI client error: {0}")]
    OciClientError(#[from] oci_client::errors::OciDistributionError),
    #[error("Storage backend error: {0}")]
    StorageError(#[from] crate::registry::storage::StorageBackendError),
    #[error("Could not deserialize manifest: {0}")]
    ManifestDeserializationError(#[from] serde_json::Error),
    #[error("Could not get AWS ECR password: {0}")]
    EcrLoginError(#[from] EcrPasswordError),
}

#[derive(thiserror::Error, Debug)]
pub enum EcrPasswordError {
    #[error("Could not parse region from ECR URL")]
    InvalidRegion,
    #[error("Could not decode ECR token: {0}")]
    Base64DecodeError(#[from] base64::DecodeError),
    #[error("Could not convert ECR token to UTF8: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Could not get AWS token: {0}")]
    AWSError(#[from] SdkError<GetAuthorizationTokenError, HttpResponse>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::manifest::ManifestReference;

    fn manifest_ref() -> ManifestReference {
        ManifestReference::Tag("napoleon".to_string())
    }

    #[tokio::test]
    async fn test_get_proxied_image_with_proxy_cfg() {
        let config = RegistryProxiesConfig {
            registries: vec![SingleRegistryProxyConfig {
                host: "docker.io".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let remote_image = config
            .get_proxied_image("f/docker.io/nginx", &manifest_ref(), None)
            .await;
        let remote_image = remote_image.unwrap();
        assert_eq!(remote_image.get_proxy_cfg().unwrap(), &config.registries[0]);
        assert_eq!(remote_image.get_host(), "registry-1.docker.io");
        assert_eq!(remote_image.get_repo(), "library/nginx");
    }

    #[tokio::test]
    async fn test_get_proxied_image_with_namespace() {
        let config = RegistryProxiesConfig {
            ..Default::default()
        };
        let remote_image = config
            .get_proxied_image("nginx", &manifest_ref(), Some("docker.io".to_string()))
            .await;
        let remote_image = remote_image.unwrap();
        assert_eq!(*remote_image.get_proxy_cfg(), None);
        assert_eq!(remote_image.get_host(), "registry-1.docker.io");
        assert_eq!(remote_image.get_repo(), "library/nginx");
    }
}
