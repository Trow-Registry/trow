use std::ops::RangeInclusive;
use std::sync::Arc;

use bytes::Bytes;
use futures::stream::Stream;
use uuid::Uuid;

use crate::file_storage::FileStorage;
use crate::repositories::Repositories;
use crate::services::Error;
use crate::services::reference::ensure_writable;
use crate::types::{AcceptedUpload, Upload, UploadInfo};
use crate::utils::digest::Digest;

#[derive(Debug)]
pub struct UploadStatus {
    pub uuid: Uuid,
    pub repo: String,
    pub offset: i64,
}

#[derive(Debug)]
pub struct BlobUploadService {
    repos: Arc<Repositories>,
    storage: Arc<FileStorage>,
}

impl BlobUploadService {
    pub fn new(repos: Arc<Repositories>, storage: Arc<FileStorage>) -> Self {
        Self { repos, storage }
    }

    pub async fn start_upload<S, E>(
        &self,
        repo_name: String,
        digest: Option<Digest>,
        data: S,
    ) -> Result<Upload, Error>
    where
        S: Stream<Item = Result<Bytes, E>> + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        ensure_writable(&repo_name)?;

        let upload_uuid = Uuid::new_v4().to_string();
        self.repos
            .blob_upload
            .create(&upload_uuid, &repo_name)
            .await?;

        if let Some(digest) = digest {
            let accepted = self
                .complete_upload(&repo_name, &upload_uuid, &digest, data, None)
                .await?;
            return Ok(Upload::Accepted(accepted));
        }

        Ok(Upload::Info(UploadInfo::new(
            upload_uuid,
            repo_name,
            (0, 0),
        )))
    }

    pub async fn patch_upload<S, E>(
        &self,
        repo_name: String,
        uuid: Uuid,
        range: Option<RangeInclusive<u64>>,
        data: S,
    ) -> Result<UploadInfo, Error>
    where
        S: Stream<Item = Result<Bytes, E>> + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        ensure_writable(&repo_name)?;
        let uuid_str = uuid.to_string();
        self.repos.blob_upload.exists(&uuid_str).await?;

        let size = self
            .storage
            .write_blob_part_stream(&uuid, data, range)
            .await?;
        let total_stored = size.total_stored as i64;
        self.repos
            .blob_upload
            .update_offset(&uuid_str, total_stored)
            .await?;

        Ok(UploadInfo::new(
            uuid_str,
            repo_name,
            (0, size.total_stored.saturating_sub(1)),
        ))
    }

    pub async fn complete_upload<S, E>(
        &self,
        repo_name: &str,
        uuid_str: &str,
        digest: &Digest,
        data: S,
        range: Option<RangeInclusive<u64>>,
    ) -> Result<AcceptedUpload, Error>
    where
        S: Stream<Item = Result<Bytes, E>> + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        let upload_row = self.repos.blob_upload.find(uuid_str).await?;
        if upload_row.repo != repo_name {
            return Err(Error::Invalid("Repository mismatch".to_string()));
        }
        let upload_id_bin = Uuid::parse_str(uuid_str).unwrap();

        let size = self
            .storage
            .write_blob_part_stream(&upload_id_bin, data, range)
            .await?;

        self.storage
            .complete_blob_write(&upload_id_bin, digest.as_str())
            .await?;

        self.repos.blob_upload.delete(&upload_row.uuid).await?;

        let digest_str = digest.as_str();
        let size_i64 = size.total_stored as i64;
        self.repos
            .blob
            .insert_or_ignore(digest_str, size_i64)
            .await?;

        self.repos
            .repo_blob_assoc
            .insert_blob_assoc(&upload_row.repo, digest_str)
            .await?;

        Ok(AcceptedUpload::new(
            digest.clone(),
            upload_row.repo,
            upload_id_bin,
            (0, size.total_stored.saturating_sub(1)),
        ))
    }

    pub async fn get_upload_status(
        &self,
        repo_name: String,
        uuid: Uuid,
    ) -> Result<UploadStatus, Error> {
        ensure_writable(&repo_name)?;
        let uuid_str = uuid.to_string();
        let offset = self
            .repos
            .blob_upload
            .find_offset_in_repo(&uuid_str, &repo_name)
            .await?;
        Ok(UploadStatus {
            uuid,
            repo: repo_name,
            offset,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bytes::Bytes;
    use uuid::Uuid;

    use crate::file_storage::FileStorage;
    use crate::services::blob_upload_service::BlobUploadService;
    use crate::services::error::Error;
    use crate::test_utilities::repos_in_memory;

    fn setup_storage(dir: &test_temp_dir::TestTempDir) -> Arc<FileStorage> {
        Arc::new(FileStorage::new(dir.as_path_untracked().to_owned()).unwrap())
    }

    #[tokio::test]
    async fn start_upload_rejects_proxied_repo() {
        let repos = repos_in_memory().await;
        let dir = test_temp_dir::test_temp_dir!();
        let storage = setup_storage(&dir);
        let svc = BlobUploadService::new(repos, storage);

        let result = svc
            .start_upload(
                "f/docker.io/library/alpine".to_string(),
                None,
                futures::stream::empty::<Result<Bytes, std::io::Error>>(),
            )
            .await;
        assert!(matches!(result, Err(Error::UnsupportedForProxiedRepo)));
    }

    #[tokio::test]
    async fn start_upload_creates_upload_record() {
        let repos = repos_in_memory().await;
        let dir = test_temp_dir::test_temp_dir!();
        let storage = setup_storage(&dir);
        let svc = BlobUploadService::new(repos.clone(), storage);

        let result = svc
            .start_upload(
                "myrepo".to_string(),
                None,
                futures::stream::empty::<Result<Bytes, std::io::Error>>(),
            )
            .await
            .unwrap();
        // Returns Upload::Info when no digest provided
        assert!(matches!(result, crate::types::Upload::Info(_)));

        // Verify upload was created in DB
        let count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM blob_upload")
            .fetch_one(repos.db_ro())
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn get_upload_status_rejects_proxied_repo() {
        let repos = repos_in_memory().await;
        let dir = test_temp_dir::test_temp_dir!();
        let storage = setup_storage(&dir);
        let svc = BlobUploadService::new(repos, storage);

        let uuid = Uuid::new_v4();
        let result = svc
            .get_upload_status("f/docker.io/library/alpine".to_string(), uuid)
            .await;
        assert!(matches!(result, Err(Error::UnsupportedForProxiedRepo)));
    }

    #[tokio::test]
    async fn get_upload_status_returns_offset() {
        let repos = repos_in_memory().await;
        let dir = test_temp_dir::test_temp_dir!();
        let storage = setup_storage(&dir);
        let svc = BlobUploadService::new(repos.clone(), storage);

        let uuid = Uuid::new_v4();
        let uuid_str = uuid.to_string();
        sqlx::query!(
            "INSERT INTO blob_upload (uuid, offset, repo) VALUES (?, ?, ?)",
            uuid_str,
            42i64,
            "myrepo"
        )
        .execute(repos.db_rw())
        .await
        .unwrap();

        let result = svc
            .get_upload_status("myrepo".to_string(), uuid)
            .await
            .unwrap();
        assert_eq!(result.uuid, uuid);
        assert_eq!(result.repo, "myrepo");
        assert_eq!(result.offset, 42);
    }

    #[tokio::test]
    async fn patch_upload_rejects_proxied_repo() {
        let repos = repos_in_memory().await;
        let dir = test_temp_dir::test_temp_dir!();
        let storage = setup_storage(&dir);
        let svc = BlobUploadService::new(repos, storage);

        let uuid = Uuid::new_v4();
        let result = svc
            .patch_upload(
                "f/docker.io/library/alpine".to_string(),
                uuid,
                None,
                futures::stream::empty::<Result<Bytes, std::io::Error>>(),
            )
            .await;
        assert!(matches!(result, Err(Error::UnsupportedForProxiedRepo)));
    }
}
