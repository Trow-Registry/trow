use std::sync::Arc;

use tokio::time::{self, Duration};

use super::StorageBackendError;
use crate::TrowServerState;

#[derive(Debug, thiserror::Error)]
pub enum GcError {
    #[error("Could not reclaim enough space")]
    CouldNotReclaimEnoughSpace,
    #[error("DB error: {0}")]
    DbError(#[from] sqlx::Error),
    #[error("Storage backend error: {0}")]
    StorageError(#[from] StorageBackendError),
}

pub async fn watchdog(state: Arc<TrowServerState>) -> Result<(), GcError> {
    let mut interval = time::interval(Duration::from_secs(120)); // 2 minutes
    loop {
        interval.tick().await;

        let space_to_reclaim = if let Some(Some(limit)) = state
            .config
            .config_file
            .as_ref()
            .map(|f| f.registry_proxies.max_size)
        {
            let blobs_size = sqlx::query_scalar!(r#"SELECT SUM(b.size) as "size!" FROM blob b"#)
                .fetch_one(&state.db_ro)
                .await?;
            let uploads_size =
                sqlx::query_scalar!(r#"SELECT SUM(u.offset) as "size!" FROM blob_upload u"#)
                    .fetch_one(&state.db_ro)
                    .await?;
            let space_taken = (blobs_size + uploads_size) as usize;
            let space_available = (limit.bytes() as f64 * 0.8) as usize;
            let space_needed = space_taken.saturating_sub(space_available);
            if space_needed > 0 {
                Some(space_needed)
            } else {
                None
            }
        } else {
            None
        };

        make_room(&state, space_to_reclaim).await?;
    }
}

async fn make_room(state: &TrowServerState, space_needed: Option<usize>) -> Result<usize, GcError> {
    let mut space_reclaimed = 0;

    space_reclaimed += delete_stale_uploads(state).await?;
    space_reclaimed += delete_orphan_blobs(state).await?;
    if let Some(space_required) = space_needed {
        space_reclaimed +=
            delete_old_proxied_images(state, space_required - space_reclaimed).await?;
    }

    Ok(space_reclaimed)
}

/// (fast)
pub async fn delete_stale_uploads(state: &TrowServerState) -> Result<usize, GcError> {
    let mut bytes_reclaimed = 0;

    let stale_uploads = sqlx::query!(
        r#"
            SELECT uuid, offset FROM blob_upload bu
            WHERE bu.updated_at < strftime('%s', 'now', '-1 day')
        "#
    )
    .fetch_all(&state.db_ro)
    .await?;

    for upload in stale_uploads {
        sqlx::query!("DELETE FROM blob_upload WHERE uuid = $1", upload.uuid)
            .execute(&state.db_rw)
            .await?;
        state.registry.storage.delete_upload(&upload.uuid).await?;
        bytes_reclaimed += upload.offset as usize;
    }

    Ok(bytes_reclaimed)
}

/// Delete blobs not referenced by manifest (slow)
async fn delete_orphan_blobs(state: &TrowServerState) -> Result<usize, GcError> {
    let mut bytes_reclaimed = 0;
    // String like is more straightforward than m.json -> config -> digest, ...
    let blobs_to_delete = sqlx::query!(
        r#"
        SELECT digest, size
        FROM blob b
        WHERE NOT EXISTS (
            SELECT 1
            FROM manifest m
            WHERE m.blob LIKE '%' || b.digest || '%'
        );
        "#,
    )
    .fetch_all(&state.db_ro)
    .await?;
    for blob in blobs_to_delete {
        sqlx::query!(
            r#"DELETE FROM repo_blob_association WHERE blob_digest = $1"#,
            blob.digest
        )
        .execute(&state.db_rw)
        .await?;
        sqlx::query!(r#"DELETE FROM blob WHERE digest = $1"#, blob.digest)
            .execute(&state.db_rw)
            .await?;
        state.registry.storage.delete_blob(&blob.digest).await?;
        bytes_reclaimed += blob.size as usize;
    }
    Ok(bytes_reclaimed)
}

/// (slow)
async fn delete_old_proxied_images(
    state: &TrowServerState,
    space_needed: usize,
) -> Result<usize, GcError> {
    let mut bytes_reclaimed = 0;
    let mut proxied_blobs = sqlx::query!(
        r#"
        SELECT digest, size
        FROM "blob" b
        WHERE NOT EXISTS (
            SELECT 1
            FROM "repo_blob_association" rba
            WHERE rba.blob_digest = b.digest
            AND rba.repo_name NOT LIKE 'f/%'
        )
        ORDER BY b.last_accessed DESC
        LIMIT 500;
        "#
    )
    .fetch_all(&state.db_ro)
    .await?;

    while bytes_reclaimed < space_needed {
        tracing::info!(proxied_blobs=?proxied_blobs, "protu");
        let blob_to_delete = match proxied_blobs.pop() {
            Some(b) => b,
            None => return Err(GcError::CouldNotReclaimEnoughSpace),
        };

        let manifests_to_delete = sqlx::query!(
            r#"SELECT digest FROM manifest WHERE blob LIKE '%' || $1 || '%'"#,
            blob_to_delete.digest
        )
        .fetch_all(&state.db_rw)
        .await?;
        for manifest_digest in manifests_to_delete {
            sqlx::query!(
                r#"
                DELETE FROM tag WHERE manifest_digest = $1;
                DELETE FROM manifest WHERE digest = $2;
                "#,
                manifest_digest.digest,
                manifest_digest.digest
            )
            .execute(&state.db_rw)
            .await?;
        }
        sqlx::query!(
            r#"
            DELETE FROM repo_blob_association WHERE blob_digest = $1;
            DELETE FROM blob WHERE digest = $2;
            "#,
            blob_to_delete.digest,
            blob_to_delete.digest
        )
        .execute(&state.db_rw)
        .await?;
        state
            .registry
            .storage
            .delete_blob(&blob_to_delete.digest)
            .await?;
        bytes_reclaimed += blob_to_delete.size as usize;
    }
    Ok(bytes_reclaimed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities;
    use crate::test_utilities::test_temp_dir;

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn test_delete_old_proxied_images() {
        let dir = test_temp_dir!();
        let (state, _router) = test_utilities::trow_router(|_| {}, &dir).await;

        // Ingest dummy data
        sqlx::query!(
            r#"
            INSERT INTO blob (digest, size, last_accessed)
            VALUES ('sha256:test1', 100, strftime('%s', 'now', '-2 days')),
                   ('sha256:test2', 175, strftime('%s', 'now', '-1 day')),
                   ('sha256:test3', 300, strftime('%s', 'now'))
            "#
        )
        .execute(&state.db_rw)
        .await
        .unwrap();

        sqlx::query!(
            r#"
            INSERT INTO repo_blob_association (repo_name, blob_digest)
            VALUES ('f/test_repo1', 'sha256:test1'),
                   ('f/test_repo3', 'sha256:test3')
            "#
        )
        .execute(&state.db_rw)
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
        .execute(&state.db_rw)
        .await
        .unwrap();

        // Test the function
        let space_needed = 250; // Request to reclaim 250 bytes
        let result = delete_old_proxied_images(&state, space_needed).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 275); // Expect to clean the 2 oldest blobs

        let repo_blob_associations =
            sqlx::query_scalar!(r#"SELECT repo_name FROM repo_blob_association"#)
                .fetch_all(&state.db_ro)
                .await
                .unwrap();
        assert_eq!(&repo_blob_associations, &["f/test_repo3"]);

        let manifests = sqlx::query_scalar!(r#"SELECT digest FROM manifest"#)
            .fetch_all(&state.db_ro)
            .await
            .unwrap();
        assert!(manifests.is_empty());
    }

    #[tracing_test::traced_test]
    #[tokio::test]
    async fn test_delete_orphan_blobs() {
        let dir = test_temp_dir!();
        let (state, _router) = test_utilities::trow_router(|_| {}, &dir).await;

        // Ingest dummy data
        sqlx::query!(
            r#"
            INSERT INTO blob (digest, size)
            VALUES ('sha256:test1', 28),
                   ('sha256:test2', 200),
                   ('sha256:test3', 155)
            "#
        )
        .execute(&state.db_rw)
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
        .execute(&state.db_rw)
        .await
        .unwrap();

        // Test the function
        let result = delete_orphan_blobs(&state).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 200); // Expect to clean the orphan blob 'sha256:test2'

        let blobs = sqlx::query_scalar!(r#"SELECT digest FROM blob"#)
            .fetch_all(&state.db_ro)
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

        // Ingest dummy data
        sqlx::query!(
            r#"
            INSERT INTO blob_upload (uuid, offset, updated_at, repo)
            VALUES ('test-uuid-1', 100, strftime('%s', 'now', '-2 days'), 'testrepo'),
                   ('test-uuid-2', 200, strftime('%s', 'now', '-5 hours'), 'testrepo'),
                   ('test-uuid-3', 150, strftime('%s', 'now', '-9 days'), 'testrepo')
            "#
        )
        .execute(&state.db_rw)
        .await
        .unwrap();

        // Test the function
        let result = delete_stale_uploads(&state).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 250); // Expect to clean stale uploads 'test-uuid-1' and 'test-uuid-3'

        let uploads = sqlx::query_scalar!(r#"SELECT uuid FROM blob_upload"#)
            .fetch_all(&state.db_ro)
            .await
            .unwrap();
        assert_eq!(uploads.len(), 1);
        assert_eq!(uploads[0], "test-uuid-2");
    }
}
