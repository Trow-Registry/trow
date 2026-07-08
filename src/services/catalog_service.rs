use std::sync::Arc;

use oci_spec::distribution::{RepositoryList, RepositoryListBuilder, TagList, TagListBuilder};

use crate::repositories::Repositories;
use crate::services::Error;

#[derive(Debug)]
pub struct CatalogService {
    repos: Arc<Repositories>,
}

impl CatalogService {
    pub fn new(repos: Arc<Repositories>) -> Self {
        Self { repos }
    }

    pub async fn list_repositories(
        &self,
        last: Option<&str>,
        limit: Option<u64>,
    ) -> Result<RepositoryList, Error> {
        let last = last.unwrap_or("");
        let limit = limit.unwrap_or(i64::MAX as u64) as i64;
        let repos = self.repos.repo_blob_assoc.list_repos(last, limit).await?;
        Ok(RepositoryListBuilder::default()
            .repositories(repos)
            .build()
            .unwrap())
    }

    pub async fn list_tags(
        &self,
        repo_name: &str,
        last: Option<&str>,
        limit: Option<u64>,
    ) -> Result<TagList, Error> {
        let last = last.unwrap_or("");
        let limit = limit.unwrap_or(i64::MAX as u64) as i64;
        let tags = self.repos.tag.list(repo_name, last, limit).await?;
        Ok(TagListBuilder::default()
            .name(repo_name.to_string())
            .tags(tags)
            .build()
            .unwrap())
    }
}

#[cfg(test)]
mod tests {
    use crate::services::catalog_service::CatalogService;
    use crate::test_utilities::repos_in_memory;

    fn fake_digest() -> &'static str {
        "sha256:0000000000000000000000000000000000000000000000000000000000000000"
    }

    #[tokio::test]
    async fn list_repositories_empty() {
        let repos = repos_in_memory().await;
        let svc = CatalogService::new(repos);
        let result = svc.list_repositories(None, None).await.unwrap();
        assert!(result.repositories().is_empty());
    }

    #[tokio::test]
    async fn list_repositories_returns_sorted_distinct_names() {
        let repos = repos_in_memory().await;
        let digest = fake_digest();
        // repo_blob_assoc has FK to blob, so insert blob first
        sqlx::query!(
            "INSERT INTO blob (digest, size) VALUES (?, ?)",
            digest,
            100i64
        )
        .execute(repos.db_rw())
        .await
        .unwrap();
        sqlx::query!(
            "INSERT INTO repo_blob_assoc (repo_name, blob_digest, manifest_digest) VALUES (?, ?, NULL), (?, ?, NULL), (?, ?, NULL)",
            "z/repo", digest, "a/repo", digest, "a/repo", digest
        )
        .execute(repos.db_rw())
        .await
        .unwrap();

        let svc = CatalogService::new(repos);
        let result = svc.list_repositories(None, None).await.unwrap();
        assert_eq!(result.repositories(), &["a/repo", "z/repo"]);
    }

    #[tokio::test]
    async fn list_repositories_respects_pagination() {
        let repos = repos_in_memory().await;
        let digest = fake_digest();
        sqlx::query!(
            "INSERT INTO blob (digest, size) VALUES (?, ?)",
            digest,
            100i64
        )
        .execute(repos.db_rw())
        .await
        .unwrap();
        sqlx::query!(
            "INSERT INTO repo_blob_assoc (repo_name, blob_digest, manifest_digest) VALUES (?, ?, NULL), (?, ?, NULL), (?, ?, NULL)",
            "alpha", digest, "beta", digest, "gamma", digest
        )
        .execute(repos.db_rw())
        .await
        .unwrap();

        let svc = CatalogService::new(repos);
        let result = svc.list_repositories(Some("alpha"), Some(1)).await.unwrap();
        assert_eq!(result.repositories(), &["beta"]);
    }

    #[tokio::test]
    async fn list_tags_empty_repo() {
        let repos = repos_in_memory().await;
        let svc = CatalogService::new(repos);
        let result = svc.list_tags("nonexistent", None, None).await.unwrap();
        assert_eq!(result.name(), "nonexistent");
        assert!(result.tags().is_empty());
    }

    #[tokio::test]
    async fn list_tags_returns_tags_for_repo() {
        let repos = repos_in_memory().await;
        // tag has FK to manifest, so we need a manifest row first
        let digest = fake_digest();
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
            "INSERT INTO tag (tag, repo, manifest_digest) VALUES (?, ?, ?), (?, ?, ?)",
            "v2.0",
            "myrepo",
            digest,
            "v1.0",
            "myrepo",
            digest
        )
        .execute(repos.db_rw())
        .await
        .unwrap();

        let svc = CatalogService::new(repos);
        let result = svc.list_tags("myrepo", None, None).await.unwrap();
        assert_eq!(result.tags(), &["v1.0", "v2.0"]);
    }

    #[tokio::test]
    async fn list_tags_respects_pagination() {
        let repos = repos_in_memory().await;
        let digest = fake_digest();
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
            "INSERT INTO tag (tag, repo, manifest_digest) VALUES (?, ?, ?), (?, ?, ?), (?, ?, ?)",
            "v3.0",
            "myrepo",
            digest,
            "v1.0",
            "myrepo",
            digest,
            "v2.0",
            "myrepo",
            digest
        )
        .execute(repos.db_rw())
        .await
        .unwrap();

        let svc = CatalogService::new(repos);
        let result = svc
            .list_tags("myrepo", Some("v1.0"), Some(1))
            .await
            .unwrap();
        assert_eq!(result.tags(), &["v2.0"]);
    }
}
