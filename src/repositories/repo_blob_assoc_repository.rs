use sqlx::SqlitePool;

pub struct RepoBlobAssocRepository {
    db_ro: SqlitePool,
    db_rw: SqlitePool,
}

impl std::fmt::Debug for RepoBlobAssocRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RepoBlobAssocRepository")
            .finish_non_exhaustive()
    }
}

impl RepoBlobAssocRepository {
    pub fn new(db_ro: SqlitePool, db_rw: SqlitePool) -> Self {
        Self { db_ro, db_rw }
    }

    /// SELECT rba.manifest_digest FROM repo_blob_assoc rba WHERE rba.manifest_digest = $1 AND rba.repo_name = $2
    pub async fn manifest_belongs_to_repo(
        &self,
        repo_name: &str,
        manifest_digest: &str,
    ) -> Result<Option<Option<String>>, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
            SELECT rba.manifest_digest
            FROM repo_blob_assoc rba
            WHERE rba.manifest_digest = $1 AND rba.repo_name = $2
            "#,
            manifest_digest,
            repo_name
        )
        .fetch_optional(&self.db_ro)
        .await
    }

    /// SELECT rba.blob_digest FROM repo_blob_assoc rba WHERE rba.blob_digest = $1 AND rba.repo_name = $2
    pub async fn blob_belongs_to_repo(
        &self,
        blob_digest: &str,
        repo_name: &str,
    ) -> Result<Option<Option<String>>, sqlx::Error> {
        sqlx::query_scalar!(
            r"SELECT rba.blob_digest FROM repo_blob_assoc rba
            WHERE rba.blob_digest = $1 AND rba.repo_name = $2",
            blob_digest,
            repo_name
        )
        .fetch_optional(&self.db_ro)
        .await
    }

    /// SELECT EXISTS(SELECT 1 FROM repo_blob_assoc WHERE manifest_digest = $1 AND repo_name = $2)
    pub async fn manifest_exists_in_repo(
        &self,
        manifest_digest: &str,
        repo_name: &str,
    ) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar!(
            r#"SELECT EXISTS(
                    SELECT 1 FROM repo_blob_assoc WHERE manifest_digest = $1 AND repo_name = $2
                )"#,
            manifest_digest,
            repo_name
        )
        .fetch_one(&self.db_rw)
        .await
    }

    /// INSERT INTO repo_blob_assoc VALUES ($1, $2, NULL) ON CONFLICT DO NOTHING (blob assoc)
    pub async fn insert_blob_assoc(
        &self,
        repo_name: &str,
        blob_digest: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO repo_blob_assoc VALUES ($1, $2, NULL) ON CONFLICT DO NOTHING",
            repo_name,
            blob_digest
        )
        .execute(&self.db_rw)
        .await?;
        Ok(())
    }

    /// INSERT INTO repo_blob_assoc (repo_name, blob_digest) VALUES ($1, $2) ON CONFLICT DO NOTHING
    pub async fn insert_blob_assoc_safe(
        &self,
        repo_name: &str,
        blob_digest: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO repo_blob_assoc (repo_name, blob_digest) VALUES ($1, $2) ON CONFLICT DO NOTHING;",
            repo_name,
            blob_digest
        )
        .execute(&self.db_rw)
        .await?;
        Ok(())
    }

    /// INSERT INTO repo_blob_assoc VALUES ($1, NULL, $2) ON CONFLICT ... (manifest assoc)
    pub async fn insert_manifest_assoc(
        &self,
        repo_name: &str,
        manifest_digest: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO repo_blob_assoc
            VALUES ($1, NULL, $2)
            ON CONFLICT (repo_name, blob_digest, manifest_digest) DO NOTHING
            "#,
            repo_name,
            manifest_digest
        )
        .execute(&self.db_rw)
        .await?;
        Ok(())
    }

    /// INSERT INTO repo_blob_assoc (repo_name, manifest_digest) VALUES ($1, $2) ON CONFLICT DO NOTHING
    pub async fn insert_manifest_assoc_safe(
        &self,
        repo_name: &str,
        manifest_digest: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "INSERT INTO repo_blob_assoc (repo_name, manifest_digest) VALUES ($1, $2) ON CONFLICT DO NOTHING;",
            repo_name,
            manifest_digest
        )
        .execute(&self.db_rw)
        .await?;
        Ok(())
    }

    /// DELETE FROM repo_blob_assoc WHERE repo_name = $1 AND manifest_digest = $2
    pub async fn delete_manifest_assoc(
        &self,
        repo_name: &str,
        manifest_digest: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM repo_blob_assoc WHERE repo_name = $1 AND manifest_digest = $2",
            repo_name,
            manifest_digest
        )
        .execute(&self.db_rw)
        .await?;
        Ok(())
    }

    /// SELECT COUNT(*) FROM repo_blob_assoc WHERE manifest_digest = $1
    pub async fn count_manifest_assoc(&self, manifest_digest: &str) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar!(
            "SELECT COUNT(*) FROM repo_blob_assoc WHERE manifest_digest = $1",
            manifest_digest
        )
        .fetch_one(&self.db_ro)
        .await
    }

    /// SELECT DISTINCT rba.repo_name FROM repo_blob_assoc rba WHERE rba.repo_name > $1 ORDER BY ... LIMIT $2
    pub async fn list_repos(
        &self,
        last_name: &str,
        limit: i64,
    ) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
            SELECT DISTINCT rba.repo_name
            FROM repo_blob_assoc rba
            WHERE rba.repo_name > $1
            ORDER BY rba.repo_name ASC
            LIMIT $2
            "#,
            last_name,
            limit
        )
        .fetch_all(&self.db_ro)
        .await
    }
}
