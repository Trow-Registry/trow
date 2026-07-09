use sqlx::SqlitePool;

pub struct TagRepository {
    db_ro: SqlitePool,
    db_rw: SqlitePool,
}

impl std::fmt::Debug for TagRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TagRepository").finish_non_exhaustive()
    }
}

impl TagRepository {
    pub fn new(db_ro: SqlitePool, db_rw: SqlitePool) -> Self {
        Self { db_ro, db_rw }
    }

    /// SELECT t.manifest_digest FROM tag t WHERE t.repo = $1 AND t.tag = $2
    pub async fn find_manifest_digest(
        &self,
        repo: &str,
        tag: &str,
    ) -> Result<Option<String>, sqlx::Error> {
        sqlx::query_scalar!(
            "SELECT t.manifest_digest FROM tag t WHERE t.repo = $1 AND t.tag = $2",
            repo,
            tag
        )
        .fetch_optional(&self.db_ro)
        .await
    }

    /// SELECT t.tag FROM tag t WHERE t.repo = $1 AND t.tag > $2 ORDER BY ... LIMIT $3
    pub async fn list(
        &self,
        repo: &str,
        last_tag: &str,
        limit: i64,
    ) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
            SELECT t.tag
            FROM tag t
            WHERE t.repo = $1
                AND t.tag COLLATE NOCASE > $2
            ORDER BY t.tag COLLATE NOCASE ASC
            LIMIT $3
            "#,
            repo,
            last_tag,
            limit
        )
        .fetch_all(&self.db_ro)
        .await
    }

    /// INSERT INTO tag VALUES ($1, $2, $3) ON CONFLICT (repo, tag) DO UPDATE SET manifest_digest = EXCLUDED.manifest_digest
    /// Note: first param is tag name, second is repo, third is digest (matches original SQL order)
    pub async fn upsert(
        &self,
        tag: &str,
        repo: &str,
        manifest_digest: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO tag
            VALUES ($1, $2, $3)
            ON CONFLICT (repo, tag) DO UPDATE
                SET manifest_digest = EXCLUDED.manifest_digest
            "#,
            tag,
            repo,
            manifest_digest
        )
        .execute(&self.db_rw)
        .await?;
        Ok(())
    }

    /// DELETE FROM tag WHERE repo = $1 AND tag = $2
    pub async fn delete(&self, repo: &str, tag: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(r#"DELETE FROM tag WHERE repo = $1 AND tag = $2"#, repo, tag)
            .execute(&self.db_rw)
            .await?;
        Ok(())
    }
}
