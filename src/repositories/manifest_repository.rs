use sqlx::SqlitePool;
use sqlx::types::Json;

use super::models::{Manifest, ManifestReferrer};
use crate::utils::manifest::OCIManifest;

pub struct ManifestRepository {
    db_ro: SqlitePool,
    db_rw: SqlitePool,
}

impl std::fmt::Debug for ManifestRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ManifestRepository").finish_non_exhaustive()
    }
}

impl ManifestRepository {
    pub fn new(db_ro: SqlitePool, db_rw: SqlitePool) -> Self {
        Self { db_ro, db_rw }
    }

    /// SELECT m.blob, m.json ->> 'mediaType', m.digest FROM manifest m WHERE m.digest = $1
    pub async fn find(&self, digest: &str) -> Result<Manifest, sqlx::Error> {
        sqlx::query_as!(
            Manifest,
            r#"
            SELECT m.blob, m.json ->> 'mediaType' as "media_type: String", m.digest
            FROM manifest m
            WHERE m.digest = $1
            "#,
            digest
        )
        .fetch_one(&self.db_ro)
        .await
    }

    /// INSERT INTO manifest (digest, json, blob) VALUES ($1, jsonb($2), $2) ON CONFLICT (digest) DO NOTHING
    pub async fn insert_or_ignore(&self, digest: &str, blob: &[u8]) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO manifest (digest, json, blob)
            VALUES ($1, jsonb($2), $2)
            ON CONFLICT (digest) DO NOTHING
            "#,
            digest,
            blob
        )
        .execute(&self.db_rw)
        .await?;
        Ok(())
    }

    /// DELETE FROM manifest where digest = $1
    pub async fn delete(&self, digest: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM manifest where digest = $1", digest)
            .execute(&self.db_rw)
            .await?;
        Ok(())
    }

    /// SELECT json(m.json), m.digest, length(m.blob) FROM manifest m JOIN repo_blob_assoc ...
    pub async fn list_referrers(
        &self,
        repo: &str,
        digest: &str,
    ) -> Result<Vec<ManifestReferrer>, sqlx::Error> {
        sqlx::query_as!(
            ManifestReferrer,
            r#"
            SELECT json(m.json) as "content!: Json<OCIManifest>",
                m.digest,
                length(m.blob) as "size!: i64"
            FROM manifest m
            INNER JOIN repo_blob_assoc rba ON rba.manifest_digest = m.digest
            WHERE rba.repo_name = $1
                AND (m.json -> 'subject' ->> 'digest') = $2
            "#,
            repo,
            digest
        )
        .fetch_all(&self.db_ro)
        .await
    }

    /// SELECT DISTINCT manifest_digest FROM manifest_blob_assoc WHERE blob_digest = $1
    pub async fn list_manifests_using_blob(
        &self,
        blob_digest: &str,
    ) -> Result<Vec<String>, sqlx::Error> {
        sqlx::query_scalar!(
            r#"SELECT DISTINCT manifest_digest FROM manifest_blob_assoc WHERE blob_digest = $1"#,
            blob_digest
        )
        .fetch_all(&self.db_ro)
        .await
    }
}
