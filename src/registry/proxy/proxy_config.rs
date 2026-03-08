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
    #[serde(default)]
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
    /// Optional path prefix for scoped credential matching.
    /// Allows different credentials for different projects on the same registry host.
    /// Example: "system" matches repos like "system/app", "system/worker".
    /// When multiple entries match the same host, the longest matching prefix wins.
    #[serde(default)]
    pub path_prefix: Option<String>,
    /// TODO: insecure currently means "use HTTP", we should also support self-signed TLS
    #[serde(default)]
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

        let mut best_match: Option<&SingleRegistryProxyConfig> = None;
        let mut best_prefix_len: usize = 0;

        for proxy in self.registries.iter() {
            if proxy.host != host {
                continue;
            }
            match &proxy.path_prefix {
                Some(prefix) if repo.starts_with(prefix.as_str()) => {
                    if prefix.len() > best_prefix_len {
                        best_match = Some(proxy);
                        best_prefix_len = prefix.len();
                    }
                }
                Some(_) => {} // prefix doesn't match this repo
                None => {
                    // Host-only match (no prefix) — fallback if no prefix matches
                    if best_prefix_len == 0 && best_match.is_none() {
                        best_match = Some(proxy);
                    }
                }
            }
        }

        Some(RemoteImage::new(host, repo, reference.clone(), best_match))
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
        assert_eq!(remote_image.get_host(), "docker.io");
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
        assert_eq!(remote_image.get_host(), "docker.io");
        assert_eq!(remote_image.get_repo(), "library/nginx");
    }

    #[tokio::test]
    async fn test_path_prefix_selects_correct_credentials() {
        let config = RegistryProxiesConfig {
            registries: vec![
                SingleRegistryProxyConfig {
                    host: "registry.example.com".to_string(),
                    path_prefix: Some("project-a".to_string()),
                    username: Some("project-a-token".to_string()),
                    ..Default::default()
                },
                SingleRegistryProxyConfig {
                    host: "registry.example.com".to_string(),
                    path_prefix: Some("project-b".to_string()),
                    username: Some("project-b-token".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        // "project-a/app" matches the project-a prefix
        let image = config
            .get_proxied_image("f/registry.example.com/project-a/app", &manifest_ref(), None)
            .await
            .unwrap();
        assert_eq!(
            image.get_proxy_cfg().unwrap().username,
            Some("project-a-token".to_string())
        );

        // "project-b/worker" matches the project-b prefix
        let image = config
            .get_proxied_image("f/registry.example.com/project-b/worker", &manifest_ref(), None)
            .await
            .unwrap();
        assert_eq!(
            image.get_proxy_cfg().unwrap().username,
            Some("project-b-token".to_string())
        );

        // "other/app" matches neither prefix — no credentials
        let image = config
            .get_proxied_image("f/registry.example.com/other/app", &manifest_ref(), None)
            .await
            .unwrap();
        assert_eq!(*image.get_proxy_cfg(), None);
    }

    #[tokio::test]
    async fn test_path_prefix_longest_match_wins() {
        let config = RegistryProxiesConfig {
            registries: vec![
                SingleRegistryProxyConfig {
                    host: "registry.example.com".to_string(),
                    path_prefix: Some("org".to_string()),
                    username: Some("org-token".to_string()),
                    ..Default::default()
                },
                SingleRegistryProxyConfig {
                    host: "registry.example.com".to_string(),
                    path_prefix: Some("org/sub".to_string()),
                    username: Some("org-sub-token".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        // "org/sub/app" matches both, but "org/sub" is longer
        let image = config
            .get_proxied_image("f/registry.example.com/org/sub/app", &manifest_ref(), None)
            .await
            .unwrap();
        assert_eq!(
            image.get_proxy_cfg().unwrap().username,
            Some("org-sub-token".to_string())
        );

        // "org/other" matches only "org"
        let image = config
            .get_proxied_image("f/registry.example.com/org/other", &manifest_ref(), None)
            .await
            .unwrap();
        assert_eq!(
            image.get_proxy_cfg().unwrap().username,
            Some("org-token".to_string())
        );
    }

    #[tokio::test]
    async fn test_path_prefix_host_only_fallback() {
        let config = RegistryProxiesConfig {
            registries: vec![
                SingleRegistryProxyConfig {
                    host: "registry.example.com".to_string(),
                    path_prefix: Some("special".to_string()),
                    username: Some("special-token".to_string()),
                    ..Default::default()
                },
                SingleRegistryProxyConfig {
                    host: "registry.example.com".to_string(),
                    // no path_prefix — fallback for this host
                    username: Some("default-token".to_string()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        // "special/app" matches the prefix entry
        let image = config
            .get_proxied_image("f/registry.example.com/special/app", &manifest_ref(), None)
            .await
            .unwrap();
        assert_eq!(
            image.get_proxy_cfg().unwrap().username,
            Some("special-token".to_string())
        );

        // "other/app" falls back to the host-only entry
        let image = config
            .get_proxied_image("f/registry.example.com/other/app", &manifest_ref(), None)
            .await
            .unwrap();
        assert_eq!(
            image.get_proxy_cfg().unwrap().username,
            Some("default-token".to_string())
        );
    }

    #[tokio::test]
    async fn test_no_path_prefix_backward_compatible() {
        // Existing config without path_prefix still works
        let config = RegistryProxiesConfig {
            registries: vec![SingleRegistryProxyConfig {
                host: "docker.io".to_string(),
                username: Some("old-user".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };
        let image = config
            .get_proxied_image("f/docker.io/nginx", &manifest_ref(), None)
            .await
            .unwrap();
        assert_eq!(image.get_proxy_cfg().unwrap(), &config.registries[0]);
        assert_eq!(
            image.get_proxy_cfg().unwrap().username,
            Some("old-user".to_string())
        );
    }
}
