use aws_config::BehaviorVersion;
use aws_sdk_ecr::config::http::HttpResponse;
use aws_sdk_ecr::error::SdkError;
use aws_sdk_ecr::operation::get_authorization_token::GetAuthorizationTokenError;
use base64::Engine;
use futures::future::try_join_all;
use oci_client::client::ClientProtocol;
use oci_client::secrets::RegistryAuth;
use oci_client::Reference;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::registry::manifest::{ManifestReference, OCIManifest};
use crate::registry::proxy::remote_image::RemoteImage;
use crate::registry::server::PROXY_DIR;
use crate::registry::storage::TrowStorageBackend;
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
use crate::registry::digest::DigestError;

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

impl SingleRegistryProxyConfig {
    async fn setup_client(
        &self,
        insecure: bool,
    ) -> Result<(oci_client::Client, RegistryAuth), DownloadRemoteImageError> {
        let mut client_config = oci_client::client::ClientConfig::default();
        if insecure {
            client_config.protocol = ClientProtocol::Http;
        }
        let client = oci_client::Client::new(client_config);
        let auth = match self.username.as_deref() {
            Some(u @ "AWS") if self.host.contains(".dkr.ecr.") => {
                let passwd = get_aws_ecr_password_from_env(&self.host).await?;
                RegistryAuth::Basic(u.to_string(), passwd)
            }
            Some(u) => {
                RegistryAuth::Basic(u.to_string(), self.password.clone().unwrap_or_default())
            }
            None => RegistryAuth::Anonymous,
        };
        // client.auth(&image.clone().into(), &auth, RegistryOperation::Pull).await?;
        Ok((client, auth))
    }

    /// returns the downloaded digest
    pub async fn download_remote_image(
        &self,
        image: &RemoteImage,
        registry: &TrowServer,
        db_rw: &SqlitePool,
    ) -> Result<String, DownloadRemoteImageError> {
        // Replace eg f/docker/alpine by f/docker/library/alpine
        let repo_name = format!("f/{}/{}", self.alias, image.get_repo());
        tracing::debug!("Downloading proxied image {}", repo_name);

        let image_ref: Reference = image.clone().into();
        let try_cl = self.setup_client(image.scheme == "http").await.ok();

        // digests is a list of posstible digests for the given reference
        let digests = match &image.reference {
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
                .fetch_optional(db_rw)
                .await?;
                if let Some((cl, auth)) = &try_cl {
                    match cl.fetch_manifest_digest(&image_ref, auth).await {
                        Ok(d) => {
                            if Some(&d) != local_digest.as_ref() {
                                digests.push(Digest::try_from_raw(&d)?);
                            }
                        }
                        Err(e) => tracing::warn!("Failed to fetch remote tag digest: {}", e),
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
            let has_manifest = sqlx::query_scalar!(
                r#"SELECT EXISTS(SELECT 1 FROM manifest WHERE digest = $1)"#,
                mani_digest_str
            )
            .fetch_one(db_rw)
            .await?;
            if has_manifest == 1 {
                return Ok(mani_digest.to_string());
            }
            if let Some((cl, auth)) = &try_cl {
                let ref_to_dl = image_ref.clone_with_digest(mani_digest.to_string());

                let manifest_download = download_manifest_and_layers(
                    cl,
                    auth,
                    db_rw.clone(),
                    &registry.storage,
                    &ref_to_dl,
                    &repo_name,
                )
                .await;

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
                            .execute(db_rw)
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
    let engine = base64::engine::general_purpose::STANDARD;
    let mut auth_str = engine.decode(token)?;
    auth_str.drain(0..4);

    Ok(String::from_utf8(auth_str)?)
}

/// `ref_` MUST reference a digest (_not_ a tag)
async fn download_manifest_and_layers(
    cl: &oci_client::Client,
    auth: &RegistryAuth,
    db_rw: SqlitePool,
    storage: &TrowStorageBackend,
    ref_: &Reference,
    local_repo_name: &str,
) -> Result<(), DownloadRemoteImageError> {
    async fn download_blob(
        cl: &oci_client::Client,
        db_rw: SqlitePool,
        storage: &TrowStorageBackend,
        ref_: &Reference,
        layer_digest: &str,
        local_repo_name: &str,
    ) -> Result<(), DownloadRemoteImageError> {
        tracing::trace!("Downloading blob {}", layer_digest);
        let already_has_blob = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM blob WHERE digest = $1);",
            layer_digest,
        )
        .fetch_one(&db_rw)
        .await?
            == 1;

        if !already_has_blob {
            let stream = cl.pull_blob_stream(ref_, layer_digest).await?;
            let path = storage
                .write_blob_stream(&Digest::try_from_raw(layer_digest).unwrap(), stream, true)
                .await?;
            let size = path.metadata().unwrap().len() as i64;
            sqlx::query!(
                "INSERT INTO blob (digest, size) VALUES ($1, $2) ON CONFLICT DO NOTHING;",
                layer_digest,
                size
            )
            .execute(&db_rw)
            .await?;
        }
        sqlx::query!(
            "INSERT INTO repo_blob_association (repo_name, blob_digest) VALUES ($1, $2) ON CONFLICT DO NOTHING;",
            local_repo_name,
            layer_digest
        )
        .execute(&db_rw)
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

    match &manifest {
        OCIManifest::List(_) => {
            let images_to_dl = manifest
                .get_local_asset_digests()
                .iter()
                .map(|digest| ref_.clone_with_digest(digest.to_string()))
                .collect::<Vec<_>>();
            let futures = images_to_dl.iter().map(|img| {
                download_manifest_and_layers(cl, auth, db_rw.clone(), storage, img, local_repo_name)
            });
            try_join_all(futures).await?;
        }
        OCIManifest::V2(_) => {
            let layer_digests = manifest.get_local_asset_digests();
            let futures = layer_digests
                .iter()
                .map(|l| download_blob(cl, db_rw.clone(), storage, ref_, l, local_repo_name));
            try_join_all(futures).await?;
        }
    }

    sqlx::query!(
        r"INSERT INTO manifest (digest, json, blob) VALUES ($1, jsonb($2), $2) ON CONFLICT DO NOTHING;
        INSERT INTO repo_blob_association (repo_name, blob_digest) VALUES ($3, $4) ON CONFLICT DO NOTHING;",
        digest,
        raw_manifest,
        local_repo_name,
        digest
    )
    .execute(&db_rw)
    .await?;

    Ok(())
}
