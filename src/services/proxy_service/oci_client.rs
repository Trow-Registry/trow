use ::oci_client::client::ClientProtocol;
use ::oci_client::secrets::RegistryAuth;
use aws_config::BehaviorVersion;
use base64::Engine;
use lazy_static::lazy_static;
use regex::Regex;

use super::errors::{DownloadRemoteImageError, EcrPasswordError};
use crate::configuration::SingleRegistryProxyConfig;

pub async fn get_oci_client(
    host: &str,
    cfg: Option<&SingleRegistryProxyConfig>,
) -> Result<(::oci_client::Client, RegistryAuth), DownloadRemoteImageError> {
    lazy_static! {
        static ref REGEX_PRIVATE_ECR: Regex =
            Regex::new(r"^[0-9]+\.dkr\.ecr\.[a-z0-9-]+\.amazonaws.com$").unwrap();
    }

    let mut client_config = ::oci_client::client::ClientConfig::default();
    if cfg.is_some_and(|c| c.insecure) {
        client_config.protocol = ClientProtocol::Http;
    }
    let client = ::oci_client::Client::new(client_config);
    let auth = match cfg.and_then(|c| c.username.as_deref()) {
        Some(u) => RegistryAuth::Basic(
            u.to_string(),
            cfg.map(|c| c.password.clone().unwrap_or_default())
                .unwrap_or_default(),
        ),
        None => {
            if REGEX_PRIVATE_ECR.is_match(host) {
                let passwd = get_aws_ecr_password_from_env(host).await?;
                RegistryAuth::Basic("AWS".to_string(), passwd)
            } else {
                RegistryAuth::Anonymous
            }
        }
    };

    Ok((client, auth))
}

/// Fetches AWS ECR credentials.
/// We use the [rusoto ChainProvider](https://docs.rs/rusoto_credential/0.48.0/rusoto_credential/struct.ChainProvider.html)
/// to fetch AWS credentials.
pub async fn get_aws_ecr_password_from_env(ecr_host: &str) -> Result<String, EcrPasswordError> {
    let region = ecr_host
        .split('.')
        .nth(3)
        .ok_or(EcrPasswordError::InvalidRegion)?
        .to_owned();
    let region = aws_types::region::Region::new(region);
    let config = aws_config::defaults(BehaviorVersion::v2026_01_12())
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

    let engine = base64::engine::general_purpose::STANDARD;
    let mut auth_str = engine.decode(token)?;
    auth_str.drain(0..4);

    Ok(String::from_utf8(auth_str)?)
}

pub const MIME_TYPES_DISTRIBUTION_MANIFEST: &[&str] = &[
    ::oci_client::manifest::IMAGE_MANIFEST_MEDIA_TYPE,
    ::oci_client::manifest::IMAGE_MANIFEST_LIST_MEDIA_TYPE,
    ::oci_client::manifest::OCI_IMAGE_MEDIA_TYPE,
    ::oci_client::manifest::OCI_IMAGE_INDEX_MEDIA_TYPE,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_oci_client_no_cfg() {
        let (_clt, auth) = get_oci_client("example.com", None).await.unwrap();
        assert!(matches!(auth, RegistryAuth::Anonymous));
    }
    #[tokio::test]
    async fn test_get_oci_client_no_cfg_ecr() {
        let err = get_oci_client("1234.dkr.ecr.mars-1.amazonaws.com", None).await;
        assert!(matches!(
            err,
            Err(DownloadRemoteImageError::EcrLoginError(_))
        ));
    }
    #[tokio::test]
    async fn test_get_oci_client_cfg_basic() {
        let proxy_cfg = SingleRegistryProxyConfig {
            username: Some("Jacky".to_string()),
            ..Default::default()
        };
        let (_clt, auth) = get_oci_client("prout.oups", Some(&proxy_cfg))
            .await
            .unwrap();
        assert_eq!(
            auth,
            RegistryAuth::Basic("Jacky".to_string(), String::new())
        );
    }
}
