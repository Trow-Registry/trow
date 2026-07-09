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
            WHERE bu.updated_at < strftime('%s', 'now', '-1 day')
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
}
