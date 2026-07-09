use std::sync::Arc;

use tokio::time::{self, Duration};

use crate::TrowConfig;
use crate::file_storage::FileStorage;
use crate::repositories::Repositories;
use crate::services::Error;

#[derive(Debug)]
pub struct GcService {
    repos: Arc<Repositories>,
    storage: Arc<FileStorage>,
    config: Arc<TrowConfig>,
}

impl GcService {
    pub fn new(
        repos: Arc<Repositories>,
        storage: Arc<FileStorage>,
        config: Arc<TrowConfig>,
    ) -> Self {
        Self {
            repos,
            storage,
            config,
        }
    }

    /// Blocks forever, running the GC loop on a 10-minute interval.
    pub async fn watchdog(self: Arc<Self>) {
        let mut interval = time::interval(Duration::from_secs(600));
        loop {
            interval.tick().await;
            if let Err(e) = self.run_once().await {
                tracing::error!("Could not make room: {e}");
            }
        }
    }

    /// Runs one GC pass; safe to call manually (used in tests).
    pub async fn run_once(&self) -> Result<(), Error> {
        let space_to_reclaim = self.compute_space_to_reclaim().await?;

        let mut space_reclaimed = 0;
        space_reclaimed += self.delete_stale_uploads().await?;
        space_reclaimed += self.delete_orphan_blobs().await?;
        if let Some(space_required) = space_to_reclaim {
            space_reclaimed += self
                .delete_old_proxied_images(space_required.saturating_sub(space_reclaimed))
                .await?;
            if space_reclaimed < space_required {
                tracing::warn!(
                    needed = bytes_humanstring(space_required),
                    "Could not reclaim enough space"
                )
            }
        }
        if space_reclaimed > 0 {
            tracing::info!(
                reclaimed = bytes_humanstring(space_reclaimed),
                "Total space reclaimed"
            );
        }
        Ok(())
    }

    async fn compute_space_to_reclaim(&self) -> Result<Option<usize>, Error> {
        let Some(limit) = self.config.config_file.registry_proxies.max_size else {
            return Ok(None);
        };
        let blobs = self.repos.blob.sum_size().await?;
        let uploads = self.repos.blob_upload.sum_offset().await?;
        let space_taken = blobs + uploads;
        let space_available = (limit.bytes() as f64 * 0.8) as usize;
        let needed = space_taken.saturating_sub(space_available);
        Ok((needed > 0).then_some(needed))
    }

    pub async fn delete_stale_uploads(&self) -> Result<usize, Error> {
        let mut bytes_reclaimed = 0;
        let stale = self.repos.blob_upload.list_stale_older_than_days().await?;
        for upload in stale {
            self.repos.blob_upload.delete(&upload.uuid).await?;
            self.storage.delete_upload(&upload.uuid).await?;
            bytes_reclaimed += upload.offset as usize;
        }
        if bytes_reclaimed > 0 {
            tracing::info!(
                reclaimed = bytes_humanstring(bytes_reclaimed),
                "Reclaimed space by deleting stale uploads"
            )
        }
        Ok(bytes_reclaimed)
    }

    pub async fn delete_orphan_blobs(&self) -> Result<usize, Error> {
        let mut bytes_reclaimed = 0;
        let blobs = self.repos.blob.list_orphaned_older_than_days().await?;
        for blob in blobs {
            self.repos.blob.delete(&blob.digest).await?;
            self.storage.delete_blob(&blob.digest).await?;
            bytes_reclaimed += blob.size as usize;
        }
        if bytes_reclaimed > 0 {
            tracing::info!(
                reclaimed = bytes_humanstring(bytes_reclaimed),
                "Reclaimed space by deleting orphaned blobs"
            )
        }
        Ok(bytes_reclaimed)
    }

    pub async fn delete_old_proxied_images(&self, space_needed: usize) -> Result<usize, Error> {
        let mut bytes_reclaimed = 0;
        let mut proxied = self.repos.blob.list_proxied_older_than_days().await?;

        while bytes_reclaimed < space_needed {
            let Some(blob) = proxied.pop() else {
                return Ok(bytes_reclaimed);
            };

            let manifests = self
                .repos
                .manifest
                .list_manifests_using_blob(&blob.digest)
                .await?;
            for md in manifests {
                self.repos.manifest.delete(&md).await?;
            }
            self.repos.blob.delete(&blob.digest).await?;
            self.storage.delete_blob(&blob.digest).await?;
            bytes_reclaimed += blob.size as usize;
        }
        if bytes_reclaimed > 0 {
            tracing::info!(
                reclaimed = bytes_humanstring(bytes_reclaimed),
                "Reclaimed space by deleting proxied blobs"
            )
        }
        Ok(bytes_reclaimed)
    }
}

fn bytes_humanstring(bytes: usize) -> String {
    size::Size::from_bytes(bytes).to_string()
}

#[cfg(test)]
mod tests {
    use crate::test_utilities;
    use crate::test_utilities::test_temp_dir;

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn test_delete_old_proxied_images() {
        let dir = test_temp_dir!();
        let (state, _router) = test_utilities::trow_router(|_| {}, &dir).await;

        sqlx::query!(
            r#"
            INSERT INTO blob (digest, size, last_accessed)
            VALUES ('sha256:test1', 100, strftime('%s', 'now', '-3 days')),
                   ('sha256:test2', 175, strftime('%s', 'now', '-3 day')),
                   ('sha256:test3', 300, strftime('%s', 'now', '-2 days'))
            "#
        )
        .execute(state.services.repos().db_rw())
        .await
        .unwrap();

        sqlx::query!(
            r#"
            INSERT INTO repo_blob_assoc (repo_name, blob_digest)
            VALUES ('f/test_repo1', 'sha256:test1'),
                   ('f/test_repo3', 'sha256:test3')
            "#
        )
        .execute(state.services.repos().db_rw())
        .await
        .unwrap();

        let dummy_manifest =
            r#"{"config":{"digest":"sha256:test2"},"layers":[{"digest":"sha256:test3"}]}"#
                .as_bytes();
        sqlx::query!(
            r#"
            INSERT INTO manifest (digest, blob, json)
            VALUES ('sha256:test_manifest', $1, jsonb($1))
            "#,
            dummy_manifest
        )
        .execute(state.services.repos().db_rw())
        .await
        .unwrap();

        let space_needed = 250;
        let result = state
            .services
            .gc
            .delete_old_proxied_images(space_needed)
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 275);

        let repo_blob_assocs = sqlx::query_scalar!(r#"SELECT repo_name FROM repo_blob_assoc"#)
            .fetch_all(state.services.repos().db_ro())
            .await
            .unwrap();
        assert_eq!(&repo_blob_assocs, &["f/test_repo3"]);

        let manifests = sqlx::query_scalar!(r#"SELECT digest FROM manifest"#)
            .fetch_all(state.services.repos().db_ro())
            .await
            .unwrap();
        assert!(manifests.is_empty());
    }

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn test_delete_orphan_blobs() {
        let dir = test_temp_dir!();
        let (state, _router) = test_utilities::trow_router(|_| {}, &dir).await;

        sqlx::query!(
            r#"
            INSERT INTO blob (digest, size, last_accessed)
            VALUES ('sha256:test1', 28, strftime('%s', 'now', '-3 days')),
                   ('sha256:test2', 200, strftime('%s', 'now', '-3 days')),
                   ('sha256:test3', 155, strftime('%s', 'now', '-3 days'))
            "#
        )
        .execute(state.services.repos().db_rw())
        .await
        .unwrap();

        let dummy_manifest =
            r#"{"config":{"digest":"sha256:test1"},"layers":[{"digest":"sha256:test3"}]}"#
                .as_bytes();
        sqlx::query!(
            r#"
            INSERT INTO manifest (digest, blob, json)
            VALUES ('sha256:test_manifest1', $1, jsonb($1))
            "#,
            dummy_manifest
        )
        .execute(state.services.repos().db_rw())
        .await
        .unwrap();

        let result = state.services.gc.delete_orphan_blobs().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 200);

        let blobs = sqlx::query_scalar!(r#"SELECT digest FROM blob"#)
            .fetch_all(state.services.repos().db_ro())
            .await
            .unwrap();
        assert_eq!(blobs.len(), 2);
        assert!(blobs.contains(&"sha256:test1".to_string()));
        assert!(blobs.contains(&"sha256:test3".to_string()));
    }

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn test_delete_stale_uploads() {
        let dir = test_temp_dir!();
        let (state, _router) = test_utilities::trow_router(|_| {}, &dir).await;

        sqlx::query!(
            r#"
            INSERT INTO blob_upload (uuid, offset, updated_at, repo)
            VALUES ('test-uuid-1', 100, strftime('%s', 'now', '-2 days'), 'testrepo'),
                   ('test-uuid-2', 200, strftime('%s', 'now', '-5 hours'), 'testrepo'),
                   ('test-uuid-3', 150, strftime('%s', 'now', '-9 days'), 'testrepo')
            "#
        )
        .execute(state.services.repos().db_rw())
        .await
        .unwrap();

        let result = state.services.gc.delete_stale_uploads().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 250);

        let uploads = sqlx::query_scalar!(r#"SELECT uuid FROM blob_upload"#)
            .fetch_all(state.services.repos().db_ro())
            .await
            .unwrap();
        assert_eq!(uploads.len(), 1);
        assert_eq!(uploads[0], "test-uuid-2");
    }
}
