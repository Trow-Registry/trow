use std::str::FromStr;
use std::sync::Arc;

use oci_spec::image::{Descriptor, ImageIndex, MediaType};

use crate::PROXY_DIR;
use crate::repositories::Repositories;
use crate::services::Error;
use crate::utils::digest::Digest;

#[derive(Debug)]
pub struct ReferrersService {
    repos: Arc<Repositories>,
}

impl ReferrersService {
    pub fn new(repos: Arc<Repositories>) -> Self {
        Self { repos }
    }

    pub async fn list_referrers(&self, repo: String, digest: String) -> Result<ImageIndex, Error> {
        if repo.starts_with(PROXY_DIR) {
            return Err(Error::UnsupportedForProxiedRepo);
        }
        let _ = Digest::try_from_raw(&digest)?;
        let rows = self.repos.manifest.list_referrers(&repo, &digest).await?;

        let mut descriptors = vec![];
        for row in rows {
            let parsed_manifest = row.content.0;

            let mediatype = parsed_manifest
                .media_type()
                .clone()
                .unwrap_or(MediaType::ImageConfig);

            let mut descriptor = Descriptor::new(
                mediatype,
                row.size as u64,
                oci_spec::image::Digest::from_str(&row.digest).unwrap(),
            );
            descriptor.set_artifact_type(parsed_manifest.artifact_type());
            descriptor.set_annotations(parsed_manifest.annotations().clone());
            descriptors.push(descriptor);
        }

        let mut response_manifest = ImageIndex::default();
        response_manifest.set_manifests(descriptors);
        response_manifest.set_media_type(Some(MediaType::ImageIndex));
        Ok(response_manifest)
    }
}

#[cfg(test)]
mod tests {
    use crate::services::error::Error;
    use crate::services::referrers_service::ReferrersService;
    use crate::test_utilities::repos_in_memory;

    #[tokio::test]
    async fn rejects_proxied_repo() {
        let repos = repos_in_memory().await;
        let svc = ReferrersService::new(repos);
        let result = svc
            .list_referrers(
                "f/docker.io/library/alpine".to_string(),
                "sha256:abc123".to_string(),
            )
            .await;
        assert!(matches!(result, Err(Error::UnsupportedForProxiedRepo)));
    }

    #[tokio::test]
    async fn rejects_invalid_digest() {
        let repos = repos_in_memory().await;
        let svc = ReferrersService::new(repos);
        let result = svc
            .list_referrers("myrepo".to_string(), "not-a-digest".to_string())
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn returns_empty_when_no_referrers() {
        let repos = repos_in_memory().await;
        let svc = ReferrersService::new(repos);
        let result = svc
            .list_referrers(
                "myrepo".to_string(),
                "sha256:abc123def456789012345678901234567890123456789012345678901234567"
                    .to_string(),
            )
            .await
            .unwrap();
        assert!(result.manifests().is_empty());
    }
}
