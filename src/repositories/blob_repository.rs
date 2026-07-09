use sqlx::SqlitePool;

use super::models::Blob;

pub struct BlobRepository {
    db_ro: SqlitePool,
    db_rw: SqlitePool,
}

impl std::fmt::Debug for BlobRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlobRepository").finish_non_exhaustive()
    }
}

impl BlobRepository {
    pub fn new(db_ro: SqlitePool, db_rw: SqlitePool) -> Self {
        Self { db_ro, db_rw }
    }

    #[doc(hidden)]
    pub fn db_rw(&self) -> &SqlitePool {
        &self.db_rw
    }

    #[doc(hidden)]
    pub fn db_ro(&self) -> &SqlitePool {
        &self.db_ro
    }

    /// UPDATE blob SET last_accessed=unixepoch() WHERE digest=$1 AND EXISTS (SELECT 1 FROM repo_blob_assoc WHERE blob_digest=$1 AND repo_name=$2)
    pub async fn touch_last_accessed(
        &self,
        digest: &str,
        repo_name: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE blob SET last_accessed=unixepoch()
            WHERE digest = $1
              AND EXISTS (
                  SELECT 1 FROM repo_blob_assoc
                  WHERE blob_digest = $1 AND repo_name = $2
              )
            "#,
            digest,
            repo_name
        )
        .execute(&self.db_rw)
        .await?;
        Ok(())
    }

    /// INSERT INTO blob (digest, size) VALUES ($1, $2) ON CONFLICT (digest) DO NOTHING
    pub async fn insert_or_ignore(&self, digest: &str, size: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO blob (digest, size)
            VALUES ($1, $2) ON CONFLICT (digest) DO NOTHING
            "#,
            digest,
            size
        )
        .execute(&self.db_rw)
        .await?;
        Ok(())
    }

    /// SELECT EXISTS(SELECT 1 FROM blob WHERE digest = $1)
    pub async fn exists(&self, digest: &str) -> Result<bool, sqlx::Error> {
        let res: i64 = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM blob WHERE digest = $1);",
            digest
        )
        .fetch_one(&self.db_ro)
        .await?;
        Ok(res == 1)
    }

    /// SELECT SUM(b.size) FROM blob b
    pub async fn sum_size(&self) -> Result<usize, sqlx::Error> {
        let res = sqlx::query_scalar!(r#"SELECT SUM(b.size) as "size!" FROM blob b"#)
            .fetch_one(&self.db_ro)
            .await?;
        Ok(usize::try_from(res).unwrap_or(0))
    }

    /// SELECT digest, size FROM blob b WHERE b.last_accessed < ... AND NOT EXISTS (...)
    pub async fn list_orphaned_older_than_days(&self) -> Result<Vec<Blob>, sqlx::Error> {
        sqlx::query_as!(
            Blob,
            r#"
            SELECT digest, size, last_accessed
            FROM blob b
            WHERE b.last_accessed < strftime('%s', 'now', '-1 day')
            AND NOT EXISTS (
                    SELECT 1
                    FROM manifest_blob_assoc mba
                    WHERE mba.blob_digest = b.digest
                );
            "#
        )
        .fetch_all(&self.db_ro)
        .await
    }

    /// SELECT digest, size FROM blob b WHERE b.last_accessed < ... AND NOT EXISTS (...) ORDER BY ... LIMIT 500
    pub async fn list_proxied_older_than_days(&self) -> Result<Vec<Blob>, sqlx::Error> {
        sqlx::query_as!(
            Blob,
            r#"
            SELECT digest, size, last_accessed
            FROM "blob" b
            WHERE b.last_accessed < strftime('%s', 'now', '-1 day')
            AND NOT EXISTS (
                    SELECT 1
                    FROM "repo_blob_assoc" rba
                    WHERE rba.blob_digest = b.digest
                    AND rba.repo_name NOT LIKE 'f/%'
                )
            ORDER BY b.last_accessed DESC
            LIMIT 500;
            "#
        )
        .fetch_all(&self.db_ro)
        .await
    }

    /// DELETE FROM blob WHERE digest = $1
    pub async fn delete(&self, digest: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(r#"DELETE FROM blob WHERE digest = $1"#, digest)
            .execute(&self.db_rw)
            .await?;
        Ok(())
    }
}
