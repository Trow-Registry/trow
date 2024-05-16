use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tracing::{event, Level};

use crate::registry::proxy::proxy_client::ProxyClient;
use crate::registry::proxy::remote_image::RemoteImage;
use crate::registry::server::PROXY_DIR;
use crate::registry::{Digest, TrowServer};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegistryProxiesConfig {
    pub registries: Vec<SingleRegistryProxyConfig>,
    #[serde(default)]
    pub offline: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SingleRegistryProxyConfig {
    pub alias: String,
    /// This field is unvalidated and may contain a scheme or not.
    /// eg: `http://example.com` and `example.com`
    pub host: String,
    pub username: Option<String>,
    pub password: Option<String>,
    #[serde(default)]
    pub ignore_repos: Vec<String>,
}

impl Default for RegistryProxiesConfig {
    fn default() -> Self {
        RegistryProxiesConfig {
            registries: Vec::new(),
            offline: true,
        }
    }
}

impl RegistryProxiesConfig {
    pub async fn get_proxy_config<'a>(
        &'a self,
        repo_name: &str,
        reference: &str,
    ) -> Option<(&'a SingleRegistryProxyConfig, RemoteImage)> {
        // All proxies are under "f_"
        if repo_name.starts_with(PROXY_DIR) {
            let segments = repo_name.splitn(3, '/').collect::<Vec<_>>();
            debug_assert_eq!(segments[0], "f");
            let proxy_alias = segments[1].to_string();
            let repo = segments[2].to_string();

            for proxy in self.registries.iter() {
                if proxy.alias == proxy_alias {
                    let image = RemoteImage::new(&proxy.host, repo, reference.into());
                    return Some((proxy, image));
                }
            }
        }
        None
    }
}

impl SingleRegistryProxyConfig {
    /// returns the downloaded digest
    pub async fn download_remote_image(
        &self,
        image: &RemoteImage,
        registry: &TrowServer,
    ) -> Result<Digest> {
        // Replace eg f/docker/alpine by f/docker/library/alpine
        let repo_name = format!("f/{}/{}", self.alias, image.get_repo());

        let try_cl = match ProxyClient::try_new(self.clone(), image).await {
            Ok(cl) => Some(cl),
            Err(e) => {
                event!(
                    Level::WARN,
                    "Could not create client for proxied registry {}: {:?}",
                    self.host,
                    e
                );
                None
            }
        };
        let remote_img_ref_digest = Digest::try_from_raw(&image.reference);
        let (local_digest, latest_digest) = match remote_img_ref_digest {
            // The ref is a digest, no need to map tg to digest
            Ok(digest) => (Some(digest), None),
            // The ref is a tag, let's search for the digest
            Err(_) => {
                let local_digest = registry
                    .storage
                    .get_manifest_digest(&repo_name, &image.reference)
                    .await
                    .ok();
                let latest_digest = match &try_cl {
                    Some(cl) => cl.get_digest_from_remote().await,
                    _ => None,
                };
                if latest_digest == local_digest && local_digest.is_none() {
                    anyhow::bail!(
                        "Could not fetch digest for {}:{}",
                        repo_name,
                        image.reference
                    );
                }
                (local_digest, latest_digest)
            }
        };

        let manifest_digests = [latest_digest, local_digest].into_iter().flatten();
        for mani_digest in manifest_digests {
            let have_manifest = registry
                .storage
                .get_blob_stream("(fixme: none)", &mani_digest)
                .await
                .is_ok();
            if have_manifest {
                return Ok(mani_digest);
            }
            if try_cl.is_none() {
                event!(
                    Level::WARN,
                    "Missing manifest for proxied image, proxy client not available"
                );
            }
            if let Some(cl) = &try_cl {
                let img_ref_as_digest = Digest::try_from_raw(&image.reference);
                let manifest_download = cl
                    .download_manifest_and_layers(registry, &image, &repo_name)
                    .await;
                match (manifest_download, img_ref_as_digest) {
                    (Err(e), _) => {
                        event!(Level::WARN, "Failed to download proxied image: {}", e)
                    }
                    (Ok(()), Err(_)) => {
                        let write_tag = registry
                            .storage
                            .write_tag(&repo_name, &image.reference, &mani_digest)
                            .await;
                        match write_tag {
                            Ok(_) => return Ok(mani_digest),
                            Err(e) => event!(
                                Level::DEBUG,
                                "Internal error updating tag for proxied image ({})",
                                e
                            ),
                        }
                    }
                    (Ok(()), Ok(_)) => return Ok(mani_digest),
                }
            }
        }
        Err(anyhow!(
            "Could not fetch manifest for proxied image {}:{}",
            repo_name,
            image.reference
        ))
    }
}
