use aws_config::BehaviorVersion;
use base64::Engine;
use futures::future::try_join_all;
use lazy_static::lazy_static;
use oci_client::Reference;
use oci_client::client::ClientProtocol;
use oci_client::secrets::RegistryAuth;
use regex::Regex;

use crate::TrowServerState;
use crate::registry::digest::Digest;
use crate::registry::manifest::{ManifestReference, OCIManifest};
use crate::registry::proxy::proxy_config::{DownloadRemoteImageError, EcrPasswordError};
use crate::registry::{RemoteImage, SingleRegistryProxyConfig};

impl<'a> RemoteImage<'a> {
    /// returns the downloaded digest
    pub async fn download(
        &self,
        state: &TrowServerState,
    ) -> Result<String, DownloadRemoteImageError> {
        // Replace eg f/docker.io/alpine by f/docker.io/library/alpine
        let repo_name = format!("f/{}/{}", self.get_host(), self.get_repo());
        tracing::debug!("Downloading proxied image {}", repo_name);

        let image_ref: Reference = self.clone().into();
        let try_cl = match get_oci_client(&self.host, self.proxy_config).await {
            Ok(cl) => Some(cl),
            Err(e) => {
                tracing::warn!("Could not get an OCI client: {e}");
                None
            }
        };

        // digests is a list of posstible digests for the given reference
        let digests = match &self.reference {
            ManifestReference::Digest(d) => vec![d.clone()],
            ManifestReference::Tag(t) => {
                let mut digests = vec![];
                let local_digest = sqlx::query_scalar!(
                    r#"
                    SELECT manifest_digest
                    FROM tag
                    WHERE repo = $1
                    AND tag = $2
                    "#,
                    repo_name,
                    t
                )
                .fetch_optional(&state.db_rw)
                .await?;
                if let Some((cl, auth)) = &try_cl {
                    match cl.fetch_manifest_digest(&image_ref, auth).await {
                        Ok(d) => {
                            if Some(&d) != local_digest.as_ref() {
                                digests.push(Digest::try_from_raw(&d)?);
                            }
                        }
                        Err(e) => tracing::warn!("Failed to fetch remote tag digest: {e}"),
                    }
                }
                if let Some(local_digest) = local_digest {
                    digests.push(Digest::try_from_raw(&local_digest)?);
                }
                digests
            }
        };

        for mani_digest in digests.into_iter() {
            let mani_digest_str = mani_digest.as_str();
            // In order to just support querying the manifest digest we need logic to create the necessary repo_blob_assoc entries
            let has_manifest = sqlx::query_scalar!(
                r#"SELECT EXISTS(
                    SELECT 1 FROM repo_blob_assoc WHERE manifest_digest = $1 AND repo_name = $2
                )"#,
                mani_digest_str,
                repo_name
            )
            .fetch_one(&state.db_rw)
            .await?;
            if has_manifest == 1 {
                return Ok(mani_digest.to_string());
            }
            if let Some((cl, auth)) = &try_cl {
                let ref_to_dl = image_ref.clone_with_digest(mani_digest.to_string());

                let manifest_download =
                    download_manifest_and_layers(cl, auth, state, &ref_to_dl, &repo_name).await;

                match manifest_download {
                    Err(e) => tracing::warn!("Failed to download proxied image: {}", e),
                    Ok(()) => {
                        if let Some(tag) = image_ref.tag() {
                            sqlx::query!(
                                r#"INSERT INTO tag (repo, tag, manifest_digest)
                                VALUES ($1, $2, $3)
                                ON CONFLICT (repo, tag) DO UPDATE SET manifest_digest = $3"#,
                                repo_name,
                                tag,
                                mani_digest_str
                            )
                            .execute(&state.db_rw)
                            .await?;
                        }
                        return Ok(mani_digest.to_string());
                    }
                }
            }
        }

        Err(DownloadRemoteImageError::DownloadAttemptsFailed)
    }
}

async fn get_oci_client(
    host: &str,
    cfg: Option<&SingleRegistryProxyConfig>,
) -> Result<(oci_client::Client, RegistryAuth), DownloadRemoteImageError> {
    lazy_static! {
        static ref REGEX_PRIVATE_ECR: Regex =
            Regex::new(r"^[0-9]+\.dkr\.ecr\.[a-z0-9-]+\.amazonaws.com$").unwrap();
    }

    let mut client_config = oci_client::client::ClientConfig::default();
    if cfg.is_some_and(|c| c.insecure) {
        client_config.protocol = ClientProtocol::Http;
    }
    let client = oci_client::Client::new(client_config);
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

    // client.auth(&image.clone().into(), &auth, RegistryOperation::Pull).await?;
    Ok((client, auth))
}

/// `ref_` MUST reference a digest (_not_ a tag)
async fn download_manifest_and_layers(
    cl: &oci_client::Client,
    auth: &RegistryAuth,
    state: &TrowServerState,
    ref_: &Reference,
    local_repo_name: &str,
) -> Result<(), DownloadRemoteImageError> {
    async fn download_blob(
        cl: &oci_client::Client,
        state: &TrowServerState,
        ref_: &Reference,
        layer_digest: &str,
        local_repo_name: &str,
    ) -> Result<(), DownloadRemoteImageError> {
        tracing::trace!("Downloading blob {}", layer_digest);
        let already_has_blob = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM blob WHERE digest = $1);",
            layer_digest,
        )
        .fetch_one(&state.db_rw)
        .await?
            == 1;

        if !already_has_blob {
            let stream = cl.pull_blob_stream(ref_, layer_digest).await?;
            let path = state
                .registry
                .storage
                .write_blob_stream(layer_digest, stream, true)
                .await?;
            let size = path.metadata().unwrap().len() as i64;
            sqlx::query!(
                "INSERT INTO blob (digest, size) VALUES ($1, $2) ON CONFLICT DO NOTHING;",
                layer_digest,
                size
            )
            .execute(&state.db_rw)
            .await?;
        }
        sqlx::query!(
            "INSERT INTO repo_blob_assoc (repo_name, blob_digest) VALUES ($1, $2) ON CONFLICT DO NOTHING;",
            local_repo_name,
            layer_digest
        )
        .execute(&state.db_rw)
        .await?;

        Ok(())
    }

    const MIME_TYPES_DISTRIBUTION_MANIFEST: &[&str] = &[
        oci_client::manifest::IMAGE_MANIFEST_MEDIA_TYPE,
        oci_client::manifest::IMAGE_MANIFEST_LIST_MEDIA_TYPE,
        oci_client::manifest::OCI_IMAGE_MEDIA_TYPE,
        oci_client::manifest::OCI_IMAGE_INDEX_MEDIA_TYPE,
    ];

    tracing::debug!("Downloading manifest + layers for {}", ref_);

    let (raw_manifest, digest) = cl
        .pull_manifest_raw(ref_, auth, MIME_TYPES_DISTRIBUTION_MANIFEST)
        .await?;
    let manifest: OCIManifest = serde_json::from_slice(&raw_manifest).map_err(|e| {
        oci_client::errors::OciDistributionError::ManifestParsingError(e.to_string())
    })?;

    let blobs = manifest.get_local_blob_digests();
    let futures = blobs
        .iter()
        .map(|l| download_blob(cl, state, ref_, l, local_repo_name));
    try_join_all(futures).await?;

    sqlx::query!(
        r"INSERT INTO manifest (digest, json, blob) VALUES ($1, jsonb($2), $2) ON CONFLICT DO NOTHING;
        INSERT INTO repo_blob_assoc (repo_name, manifest_digest) VALUES ($3, $4) ON CONFLICT DO NOTHING;",
        digest,
        raw_manifest,
        local_repo_name,
        digest
    )
    .execute(&state.db_rw)
    .await?;

    Ok(())
}

/// Fetches AWS ECR credentials.
/// We use the [rusoto ChainProvider](https://docs.rs/rusoto_credential/0.48.0/rusoto_credential/struct.ChainProvider.html)
/// to fetch AWS credentials.
async fn get_aws_ecr_password_from_env(ecr_host: &str) -> Result<String, EcrPasswordError> {
    let region = ecr_host
        .split('.')
        .nth(3)
        .ok_or(EcrPasswordError::InvalidRegion)?
        .to_owned();
    let region = aws_types::region::Region::new(region);
    let config = aws_config::defaults(BehaviorVersion::v2025_01_17())
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
    let engine = base64::engine::general_purpose::STANDARD;
    let mut auth_str = engine.decode(token)?;
    auth_str.drain(0..4);

    Ok(String::from_utf8(auth_str)?)
}

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
