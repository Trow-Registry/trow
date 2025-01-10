use anyhow::{anyhow, Result};
use futures::future::Remote;
use oci_client::client::ClientProtocol;
use oci_client::secrets::RegistryAuth;
use oci_client::RegistryOperation;
use serde::{Deserialize, Serialize};
use tracing::{event, Level};
use sqlx::Sqlite;

use crate::registry::manifest::ManifestReference;
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
        reference: &ManifestReference,
    ) -> Option<(&'a SingleRegistryProxyConfig, RemoteImage)> {
        // All proxies are under "f_"
        if repo_name.starts_with(PROXY_DIR) {
            let segments = repo_name.splitn(3, '/').collect::<Vec<_>>();
            debug_assert_eq!(segments[0], "f");
            let proxy_alias = segments[1].to_string();
            let repo = segments[2].to_string();

            for proxy in self.registries.iter() {
                if proxy.alias == proxy_alias {
                    let image = RemoteImage::new(&proxy.host, repo, reference.clone());
                    return Some((proxy, image));
                }
            }
        }
        None
    }
}

impl SingleRegistryProxyConfig {
    /// returns the downloaded digest
    /// TO RE-DO !!!
    pub async fn download_remote_image<C>(
        &self,
        image: &RemoteImage,
        registry: &TrowServer,
        db: C
    ) -> Result<Digest>
    where
        for<'e> &'e mut C: sqlx::Executor<'e, Database = Sqlite>, {
        // Replace eg f/docker/alpine by f/docker/library/alpine
        let repo_name = format!("f/{}/{}", self.alias, image.get_repo());

        let mut client_config = oci_client::client::ClientConfig::default();
        if image.get_host().starts_with("http://") {
            client_config.protocol = ClientProtocol::Http;
        }
        let client = oci_client::Client::new(client_config);
        let auth = match self.username {
          Some(u) if u == "AWS" && self.host.contains(".dkr.ecr.") => {
              let passwd = get_aws_ecr_password_from_env(&self.host)
                  .await?;
              RegistryAuth::Basic(u, passwd)
          },
          Some(u) => {
              RegistryAuth::Basic(u, self.password.unwrap_or_default())
          },
          None => RegistryAuth::Anonymous
        };
        client.auth(&image.clone().into(), &auth, RegistryOperation::Pull).await.ok();

        let remote_img_ref_digest = image.reference.digest();
        let (local_digest, latest_digest) = match remote_img_ref_digest {
            // The ref is a digest, no need to map tg to digest
            Some(digest) => (Some(digest.clone()), None),
            // The ref is a tag, let's search for the digest
            None => {
                // let local_digest = registry
                //     .storage
                //     .get_manifest_digest(&repo_name, &image.reference.to_string())
                //     .await
                //     .ok();
                let local_digest = None;
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
                let img_ref_as_digest = image.reference.digest();
                let mut image_to_dl = image.clone();
                if img_ref_as_digest.is_none() {
                    image_to_dl.reference =
                        ManifestReference::Digest(cl.get_digest_from_remote().await.unwrap());
                }
                let manifest_download = cl
                    .download_manifest_and_layers(registry, &image_to_dl, &repo_name)
                    .await;
                match (manifest_download, img_ref_as_digest) {
                    (Err(e), _) => {
                        event!(Level::WARN, "Failed to download proxied image: {}", e)
                    }
                    (Ok(()), None) => {
                        // let write_tag = registry
                        //     .storage
                        //     .write_tag(&repo_name, &image.reference.to_string(), &mani_digest)
                        //     .await;
                        let write_tag: Result<(), ()> = Err(());
                        match write_tag {
                            Ok(_) => return Ok(mani_digest),
                            Err(_) => event!(
                                Level::DEBUG,
                                "Internal error updating tag for proxied image ({})",
                                "unimplemented"
                            ),
                        }
                    }
                    (Ok(()), Some(_)) => return Ok(mani_digest),
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


/// Fetches AWS ECR credentials.
/// We use the [rusoto ChainProvider](https://docs.rs/rusoto_credential/0.48.0/rusoto_credential/struct.ChainProvider.html)
/// to fetch AWS credentials.
async fn get_aws_ecr_password_from_env(ecr_host: &str) -> Result<String> {
    let region = ecr_host
        .split('.')
        .nth(3)
        .ok_or_else(|| anyhow!("Could not parse region from ECR URL"))?
        .to_owned();
    let region = aws_types::region::Region::new(region);
    let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
        .region(region)
        .load()
        .await;
    let ecr_clt = aws_sdk_ecr::Client::new(&config);
    let token_response = ecr_clt.get_authorization_token().send().await?;
    let token = token_response
        .authorization_data
        .unwrap()
        .into_iter()
        .next()
        .unwrap()
        .authorization_token
        .unwrap();

    // The token is base64(username:password). Here, username is "AWS".
    // To get the password, we trim "AWS:" from the decoded token.
    let mut auth_str = general_purpose::STANDARD.decode(token)?;
    auth_str.drain(0..4);

    String::from_utf8(auth_str).context("Could not convert ECR token to valid password")
}
