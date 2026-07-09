use std::sync::Arc;

use axum::body::Body;

use crate::repositories::Repositories;
use crate::services::error::Error;
use crate::services::proxy_service::ProxyService;
use crate::types::{ManifestDeleted, VerifiedManifest};
use crate::utils::digest::Digest;
use crate::utils::manifest::{OCIManifest, REGEX_TAG, layer_is_distributable, manifest_media_type};
use crate::utils::resolve_reference::parse_reference;
use crate::{PROXY_DIR, TrowConfig};

pub struct ManifestPayload {
    pub bytes: bytes::Bytes,
    pub digest: String,
    pub content_type: String,
}

#[derive(Debug)]
pub struct ManifestService {
    repos: Arc<Repositories>,
    config: Arc<TrowConfig>,
    proxy: Arc<ProxyService>,
}

impl ManifestService {
    pub fn new(
        repos: Arc<Repositories>,
        config: Arc<TrowConfig>,
        proxy: Arc<ProxyService>,
    ) -> Self {
        Self {
            repos,
            config,
            proxy,
        }
    }

    pub async fn get_manifest(
        &self,
        repo: String,
        raw_reference: String,
        namespace: Option<&str>,
    ) -> Result<ManifestPayload, Error> {
        let image = parse_reference(&repo, &raw_reference, namespace)?;

        let digest = if image.registry() != "localhost" {
            let proxy_config = self
                .config
                .config_file
                .registry_proxies
                .registries
                .get_for(image.registry(), image.repository());
            self.proxy.download_image(&image, proxy_config).await?
        } else {
            let digest = if let Some(tag) = image.tag() {
                let tdigest = self.repos.tag.find_manifest_digest(&repo, tag).await?;
                match tdigest {
                    Some(d) => d,
                    None => {
                        return Err(Error::ManifestUnknown(format!(
                            "Unknown tag: {raw_reference}"
                        )));
                    }
                }
            } else if let Some(digest) = image.digest() {
                digest.to_string()
            } else {
                return Err(Error::ManifestUnknown(format!(
                    "Invalid reference: {raw_reference}"
                )));
            };

            if !self
                .repos
                .repo_blob_assoc
                .manifest_belongs_to_repo(&repo, &digest)
                .await?
            {
                return Err(Error::ManifestUnknown(format!("Unknown digest {digest}")));
            }
            digest
        };

        let res = self.repos.manifest.find(&digest).await?;
        let content_type = match res.media_type.as_ref() {
            Some(mt) => mt.clone(),
            None => determine_content_type(&res.blob)?,
        };

        Ok(ManifestPayload {
            bytes: res.blob.into(),
            digest,
            content_type,
        })
    }

    pub async fn put_manifest(
        &self,
        repo_name: String,
        reference: String,
        host: String,
        body: Body,
    ) -> Result<VerifiedManifest, Error> {
        if repo_name.starts_with(PROXY_DIR) {
            return Err(Error::UnsupportedForProxiedRepo);
        }
        let is_tag = REGEX_TAG.is_match(&reference);
        const MANIFEST_BODY_SIZE_LIMIT_MB: usize = 4;
        let manifest_bytes = axum::body::to_bytes(body, MANIFEST_BODY_SIZE_LIMIT_MB * 1024 * 1024)
            .await
            .map_err(|_| {
                Error::ManifestInvalid(format!(
                    "Manifest is bigger than limit of {MANIFEST_BODY_SIZE_LIMIT_MB}MiB"
                ))
            })?
            .to_vec();
        let manifest_parsed = serde_json::from_slice::<'_, OCIManifest>(&manifest_bytes)
            .map_err(|e| Error::ManifestInvalid(format!("{e}")))?;

        match &manifest_parsed {
            OCIManifest::List(m) => {
                let assets = m
                    .manifests()
                    .iter()
                    .filter(|l| layer_is_distributable(l.media_type()))
                    .map(|m| m.digest().as_ref());
                for digest in assets {
                    if !self
                        .repos
                        .repo_blob_assoc
                        .manifest_belongs_to_repo(&repo_name, digest)
                        .await?
                    {
                        return Err(Error::ManifestInvalid(format!(
                            "Manifest asset not found: {digest}"
                        )));
                    }
                }
            }
            OCIManifest::V2(m) => {
                let assets = m
                    .layers()
                    .iter()
                    .filter(|l| layer_is_distributable(l.media_type()))
                    .map(|l| l.digest().as_ref());
                for digest in assets {
                    if !self
                        .repos
                        .repo_blob_assoc
                        .blob_belongs_to_repo(digest, &repo_name)
                        .await?
                    {
                        return Err(Error::ManifestInvalid(format!(
                            "Blob asset not found: {digest}"
                        )));
                    }
                }
            }
        }
        let computed_digest = Digest::digest_sha256_slice(&manifest_bytes);
        let computed_digest_str = computed_digest.as_str();
        if !is_tag && computed_digest_str != reference {
            return Err(Error::ManifestInvalid(
                "Given digest does not match".to_string(),
            ));
        }

        self.repos
            .manifest
            .insert_or_ignore(computed_digest_str, &manifest_bytes)
            .await?;
        self.repos
            .repo_blob_assoc
            .insert_manifest_assoc(&repo_name, computed_digest_str)
            .await?;

        if is_tag {
            self.repos
                .tag
                .upsert(&reference, &repo_name, computed_digest_str)
                .await?;
        }

        let subject = manifest_parsed.subject().map(|s| s.digest().to_string());

        Ok(VerifiedManifest::new(
            Some(host),
            repo_name,
            computed_digest,
            reference,
            subject,
        ))
    }

    pub async fn delete_manifest(
        &self,
        repo: String,
        reference: String,
    ) -> Result<ManifestDeleted, Error> {
        if repo.starts_with(PROXY_DIR) {
            return Err(Error::UnsupportedForProxiedRepo);
        }
        if REGEX_TAG.is_match(&reference) {
            self.repos.tag.delete(&repo, &reference).await?;
        } else {
            let digest = Digest::try_from_raw(&reference)?;
            let digest_str = digest.as_str();
            self.repos
                .repo_blob_assoc
                .delete_manifest_assoc(&repo, digest_str)
                .await?;
            let num_repo_assoc = self
                .repos
                .repo_blob_assoc
                .count_manifest_assoc(digest_str)
                .await?;
            if num_repo_assoc == 0 {
                self.repos.manifest.delete(digest_str).await?;
            }
        }

        Ok(ManifestDeleted {})
    }
}

pub(crate) fn determine_content_type(manifest_bytes: &[u8]) -> Result<String, Error> {
    let manifest: OCIManifest = serde_json::from_slice(manifest_bytes)
        .map_err(|e| Error::ManifestInvalid(format!("Invalid manifest JSON: {}", e)))?;

    let content_type = match &manifest {
        OCIManifest::List(_) => manifest_media_type::OCI_INDEX,
        OCIManifest::V2(_) => manifest_media_type::OCI_V1,
    };

    Ok(content_type.to_string())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::TrowConfig;
    use crate::file_storage::FileStorage;
    use crate::services::error::Error;
    use crate::services::manifest_service::{ManifestService, determine_content_type};
    use crate::services::proxy_service::ProxyService;
    use crate::test_utilities::repos_in_memory;

    fn setup_service(
        repos: Arc<super::super::super::repositories::Repositories>,
    ) -> ManifestService {
        let dir = test_temp_dir::test_temp_dir!();
        let storage = Arc::new(FileStorage::new(dir.as_path_untracked().to_owned()).unwrap());
        let proxy = Arc::new(ProxyService::new(repos.clone(), storage));
        let config = Arc::new(TrowConfig::new());
        ManifestService::new(repos, config, proxy)
    }

    fn minimal_v2_manifest_json() -> &'static str {
        r#"{
            "schemaVersion": 2,
            "mediaType": "application/vnd.oci.image.manifest.v1+json",
            "config": {
                "mediaType": "application/vnd.oci.image.config.v1+json",
                "digest": "sha256:4d3c246dfef2edb11eccb051b47d896d0db8f1c4563c0cce9f6274b9abd9ac74",
                "size": 702
            },
            "layers": []
        }"#
    }

    #[tokio::test]
    async fn put_manifest_rejects_proxied_repo() {
        let repos = repos_in_memory().await;
        let svc = setup_service(repos);

        let result = svc
            .put_manifest(
                "f/docker.io/library/alpine".to_string(),
                "latest".to_string(),
                "localhost".to_string(),
                axum::body::Body::empty(),
            )
            .await;
        assert!(matches!(result, Err(Error::UnsupportedForProxiedRepo)));
    }

    #[tokio::test]
    async fn put_manifest_rejects_invalid_json() {
        let repos = repos_in_memory().await;
        let svc = setup_service(repos);

        let result = svc
            .put_manifest(
                "myrepo".to_string(),
                "latest".to_string(),
                "localhost".to_string(),
                axum::body::Body::from("not-json"),
            )
            .await;
        assert!(matches!(result, Err(Error::ManifestInvalid(_))));
    }

    #[tokio::test]
    async fn put_manifest_with_tag_succeeds() {
        let repos = repos_in_memory().await;
        let svc = setup_service(repos.clone());

        let manifest_json = minimal_v2_manifest_json();
        let result = svc
            .put_manifest(
                "myrepo".to_string(),
                "latest".to_string(),
                "localhost".to_string(),
                axum::body::Body::from(manifest_json),
            )
            .await
            .unwrap();

        assert_eq!(result.tag(), "latest");
        assert_eq!(result.repo_name(), "myrepo");

        // Verify tag was created
        let tag_digest = sqlx::query_scalar!(
            "SELECT manifest_digest FROM tag WHERE repo = ? AND tag = ?",
            "myrepo",
            "latest"
        )
        .fetch_one(repos.db_ro())
        .await
        .unwrap();
        assert_eq!(tag_digest, result.digest());
    }

    #[tokio::test]
    async fn put_manifest_with_digest_requires_matching() {
        let repos = repos_in_memory().await;
        let svc = setup_service(repos);

        let manifest_json = minimal_v2_manifest_json();
        let result = svc
            .put_manifest(
                "myrepo".to_string(),
                "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                    .to_string(),
                "localhost".to_string(),
                axum::body::Body::from(manifest_json),
            )
            .await;
        assert!(matches!(result, Err(Error::ManifestInvalid(_))));
    }

    #[tokio::test]
    async fn put_manifest_requires_layers_in_repo() {
        let repos = repos_in_memory().await;
        let svc = setup_service(repos);

        // Manifest with a layer that doesn't exist in repo
        let manifest_with_layer = r#"{
            "schemaVersion": 2,
            "mediaType": "application/vnd.oci.image.manifest.v1+json",
            "config": {
                "mediaType": "application/vnd.oci.image.config.v1+json",
                "digest": "sha256:4d3c246dfef2edb11eccb051b47d896d0db8f1c4563c0cce9f6274b9abd9ac74",
                "size": 702
            },
            "layers": [{
                "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
                "digest": "sha256:9d48c3bd43c520dc2784e868a780e976b207cbf493eaff8c6596eb871cbd9609",
                "size": 100
            }]
        }"#;

        let result = svc
            .put_manifest(
                "myrepo".to_string(),
                "latest".to_string(),
                "localhost".to_string(),
                axum::body::Body::from(manifest_with_layer),
            )
            .await;
        assert!(matches!(result, Err(Error::ManifestInvalid(_))));
    }

    #[tokio::test]
    async fn delete_manifest_rejects_proxied_repo() {
        let repos = repos_in_memory().await;
        let svc = setup_service(repos);

        let result = svc
            .delete_manifest(
                "f/docker.io/library/alpine".to_string(),
                "latest".to_string(),
            )
            .await;
        assert!(matches!(result, Err(Error::UnsupportedForProxiedRepo)));
    }

    #[tokio::test]
    async fn delete_manifest_by_tag() {
        let repos = repos_in_memory().await;
        let svc = setup_service(repos.clone());

        let digest = "sha256:abc123def456789012345678901234567890123456789012345678901234567";
        // tag has FK to manifest, so insert manifest first
        let json_bytes: &[u8] = b"{}";
        sqlx::query!(
            "INSERT INTO manifest (digest, json, blob) VALUES (?, ?, ?)",
            digest,
            json_bytes,
            json_bytes
        )
        .execute(repos.db_rw())
        .await
        .unwrap();
        sqlx::query!(
            "INSERT INTO tag (tag, repo, manifest_digest) VALUES (?, ?, ?)",
            "latest",
            "myrepo",
            digest
        )
        .execute(repos.db_rw())
        .await
        .unwrap();

        svc.delete_manifest("myrepo".to_string(), "latest".to_string())
            .await
            .unwrap();

        // Verify tag was deleted
        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM tag WHERE repo = ? AND tag = ?",
            "myrepo",
            "latest"
        )
        .fetch_one(repos.db_ro())
        .await
        .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn delete_manifest_by_digest_removes_manifest() {
        let repos = repos_in_memory().await;
        let svc = setup_service(repos.clone());

        let digest = "sha256:abc123def456789012345678901234567890123456789012345678901234567";
        // Setup: create manifest and association
        let json_bytes: &[u8] = b"{}";
        let blob_bytes: &[u8] = b"";
        sqlx::query!(
            "INSERT INTO manifest (digest, json, blob) VALUES (?, ?, ?)",
            digest,
            json_bytes,
            blob_bytes
        )
        .execute(repos.db_rw())
        .await
        .unwrap();
        sqlx::query!(
            "INSERT INTO repo_blob_assoc (repo_name, blob_digest, manifest_digest) VALUES (?, NULL, ?)",
            "myrepo", digest
        )
        .execute(repos.db_rw())
        .await
        .unwrap();

        svc.delete_manifest("myrepo".to_string(), digest.to_string())
            .await
            .unwrap();

        // Association deleted
        let assoc_count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM repo_blob_assoc WHERE manifest_digest = ?",
            digest
        )
        .fetch_one(repos.db_ro())
        .await
        .unwrap();
        assert_eq!(assoc_count, 0);

        // Manifest deleted (no more associations)
        let manifest_count: i64 =
            sqlx::query_scalar!("SELECT COUNT(*) FROM manifest WHERE digest = ?", digest)
                .fetch_one(repos.db_ro())
                .await
                .unwrap();
        assert_eq!(manifest_count, 0);
    }

    #[tokio::test]
    async fn get_manifest_by_tag() {
        let repos = repos_in_memory().await;
        let svc = setup_service(repos.clone());

        let digest = "sha256:abc123def456789012345678901234567890123456789012345678901234567";
        let manifest_bytes = minimal_v2_manifest_json().as_bytes();

        // Setup: manifest first (FK target), then tag, then association
        sqlx::query!(
            "INSERT INTO manifest (digest, json, blob) VALUES (?, ?, ?)",
            digest,
            manifest_bytes,
            manifest_bytes
        )
        .execute(repos.db_rw())
        .await
        .unwrap();
        sqlx::query!(
            "INSERT INTO tag (tag, repo, manifest_digest) VALUES (?, ?, ?)",
            "latest",
            "myrepo",
            digest
        )
        .execute(repos.db_rw())
        .await
        .unwrap();
        sqlx::query!(
            "INSERT INTO repo_blob_assoc (repo_name, blob_digest, manifest_digest) VALUES (?, NULL, ?)",
            "myrepo", digest
        )
        .execute(repos.db_rw())
        .await
        .unwrap();

        let result = svc
            .get_manifest("myrepo".to_string(), "latest".to_string(), None)
            .await
            .unwrap();
        assert_eq!(result.digest, digest);
        assert_eq!(
            result.content_type,
            "application/vnd.oci.image.manifest.v1+json"
        );
    }

    #[tokio::test]
    async fn get_manifest_unknown_tag() {
        let repos = repos_in_memory().await;
        let svc = setup_service(repos);

        let result = svc
            .get_manifest("myrepo".to_string(), "nonexistent".to_string(), None)
            .await;
        assert!(matches!(result, Err(Error::ManifestUnknown(_))));
    }

    #[tokio::test]
    async fn get_manifest_unknown_digest() {
        let repos = repos_in_memory().await;
        let svc = setup_service(repos);

        let result = svc
            .get_manifest(
                "myrepo".to_string(),
                "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                    .to_string(),
                None,
            )
            .await;
        assert!(matches!(result, Err(Error::ManifestUnknown(_))));
    }

    #[test]
    fn determine_content_type_v2_manifest() {
        let manifest = r#"{
            "schemaVersion": 2,
            "mediaType": "application/vnd.oci.image.manifest.v1+json",
            "config": {
                "mediaType": "application/vnd.oci.image.config.v1+json",
                "digest": "sha256:4d3c246dfef2edb11eccb051b47d896d0db8f1c4563c0cce9f6274b9abd9ac74",
                "size": 702
            },
            "layers": []
        }"#;
        let ct = determine_content_type(manifest.as_bytes()).unwrap();
        assert_eq!(ct, "application/vnd.oci.image.manifest.v1+json");
    }

    #[test]
    fn determine_content_type_index() {
        let manifest = r#"{"schemaVersion":2,"mediaType":"application/vnd.oci.image.index.v1+json","manifests":[]}"#;
        let ct = determine_content_type(manifest.as_bytes()).unwrap();
        assert_eq!(ct, "application/vnd.oci.image.index.v1+json");
    }
}
