use sqlx::SqlitePool;

use super::models::BlobUpload;

pub struct BlobUploadRepository {
    db_ro: SqlitePool,
    db_rw: SqlitePool,
}

impl std::fmt::Debug for BlobUploadRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlobUploadRepository")
            .finish_non_exhaustive()
    }
}

impl BlobUploadRepository {
    pub fn new(db_ro: SqlitePool, db_rw: SqlitePool) -> Self {
        Self { db_ro, db_rw }
    }

    /// SELECT * FROM blob_upload WHERE uuid=$1
    pub async fn find(&self, uuid: &str) -> Result<BlobUpload, sqlx::Error> {
        sqlx::query_as!(
            BlobUpload,
            r#"
            SELECT uuid, repo, offset, updated_at
            FROM blob_upload
            WHERE uuid=$1
            "#,
            uuid
        )
        .fetch_one(&self.db_ro)
        .await
    }

    /// SELECT rowid FROM blob_upload WHERE uuid=$1 (existence check)
    pub async fn exists(&self, uuid: &str) -> Result<(), sqlx::Error> {
        sqlx::query_scalar!(r#"SELECT rowid FROM blob_upload WHERE uuid=$1"#, uuid)
            .fetch_one(&self.db_ro)
            .await?;
        Ok(())
    }

    /// DELETE FROM blob_upload WHERE uuid=$1
    pub async fn delete(&self, uuid: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM blob_upload
            WHERE uuid=$1
            "#,
            uuid
        )
        .execute(&self.db_rw)
        .await?;
        Ok(())
    }

    /// INSERT INTO blob_upload (uuid, repo, offset) VALUES ($1, $2, $3)
    pub async fn create(&self, uuid: &str, repo: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO blob_upload (uuid, repo, offset)
            VALUES ($1, $2, $3)
            "#,
            uuid,
            repo,
            0_i64
        )
        .execute(&self.db_rw)
        .await?;
        Ok(())
    }

    /// UPDATE blob_upload SET offset=$2, updated_at=unixepoch() WHERE uuid=$1
    pub async fn update_offset(&self, uuid: &str, offset: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE blob_upload SET offset=$2, updated_at=unixepoch() WHERE uuid=$1",
            uuid,
            offset
        )
        .execute(&self.db_rw)
        .await?;
        Ok(())
    }

    /// SELECT offset FROM blob_upload WHERE uuid = $1 AND repo = $2
    pub async fn find_offset_in_repo(&self, uuid: &str, repo: &str) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar!(
            "SELECT offset FROM blob_upload WHERE uuid = $1 AND repo = $2",
            uuid,
            repo
        )
        .fetch_one(&self.db_ro)
        .await
    }

    /// SELECT uuid, repo, offset, updated_at FROM blob_upload bu WHERE bu.updated_at < ...
    pub async fn list_stale_older_than_days(&self) -> Result<Vec<BlobUpload>, sqlx::Error> {
        sqlx::query_as!(
            BlobUpload,
            r#"
            SELECT uuid, repo, offset, updated_at
            FROM blob_upload bu
            WHERE bu.updated_at < unixepoch('now', '-1 day')
            "#
        )
        .fetch_all(&self.db_ro)
        .await
    }

    /// SELECT SUM(u.offset) FROM blob_upload u
    pub async fn sum_offset(&self) -> Result<usize, sqlx::Error> {
        let res = sqlx::query_scalar!(r#"SELECT SUM(u.offset) as "size!" FROM blob_upload u"#)
            .fetch_one(&self.db_ro)
            .await?;
        Ok(usize::try_from(res).unwrap_or(0))
    }

    /// INSERT INTO blob_upload (uuid, offset, repo) VALUES ($1, $2, $3)
    ///
    /// Creates an upload that already has bytes written, as a previous PATCH
    /// would have left it.
    #[doc(hidden)]
    pub async fn create_with_offset(
        &self,
        uuid: &str,
        repo: &str,
        offset: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO blob_upload (uuid, offset, repo)
            VALUES ($1, $2, $3)
            "#,
            uuid,
            offset,
            repo
        )
        .execute(&self.db_rw)
        .await?;
        Ok(())
    }

    /// SELECT COUNT(*) FROM blob_upload
    pub async fn count(&self) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!" FROM blob_upload"#)
            .fetch_one(&self.db_ro)
            .await
    }

    /// SELECT uuid FROM blob_upload
    pub async fn list_all_uuids(&self) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query_scalar!(r#"SELECT uuid FROM blob_upload"#)
            .fetch_all(&self.db_ro)
            .await
    }

    /// UPDATE blob_upload SET updated_at = unixepoch() - $2 WHERE uuid = $1
    ///
    /// Backdates an upload relative to the database clock so staleness tests
    /// place rows deterministically on either side of the cutoff.
    #[doc(hidden)]
    pub async fn set_age_secs(&self, uuid: &str, age_secs: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE blob_upload SET updated_at = unixepoch() - $2 WHERE uuid = $1"#,
            uuid,
            age_secs
        )
        .execute(&self.db_rw)
        .await?;
        Ok(())
    }

    /// INSERT INTO blob_upload (uuid, offset, updated_at, repo) VALUES ($1, $2, unixepoch() - $3, $4)
    ///
    /// Seeds a stale upload in one statement, for GC tests that need several.
    #[doc(hidden)]
    pub async fn create_aged(
        &self,
        uuid: &str,
        repo: &str,
        offset: i64,
        age_secs: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO blob_upload (uuid, offset, updated_at, repo)
            VALUES ($1, $2, unixepoch() - $3, $4)
            "#,
            uuid,
            offset,
            age_secs,
            repo
        )
        .execute(&self.db_rw)
        .await?;
        Ok(())
    }

    /// SELECT typeof(updated_at) FROM blob_upload WHERE uuid = $1
    ///
    /// Guards that the column's default stays an epoch integer rather than a
    /// `CURRENT_TIMESTAMP` string, which would never compare as stale.
    #[doc(hidden)]
    pub async fn updated_at_storage_class(&self, uuid: &str) -> Result<String, sqlx::Error> {
        sqlx::query_scalar!(
            r#"SELECT typeof(updated_at) AS "typeof!: String" FROM blob_upload WHERE uuid = $1"#,
            uuid
        )
        .fetch_one(&self.db_ro)
        .await
    }
}
