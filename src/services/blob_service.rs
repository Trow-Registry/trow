use std::pin::Pin;
use std::sync::Arc;

use tokio::io::AsyncRead;

use crate::file_storage::FileStorage;
use crate::repositories::Repositories;
use crate::services::Error;
use crate::types::BoundedStream;
use crate::utils::digest::Digest;
use crate::utils::resolve_reference::parse_reference;

pub struct BlobReader<S: AsyncRead + ?Sized + Send> {
    digest: Digest,
    reader: Box<S>,
    size: u64,
}

impl<S: tokio::io::AsyncRead + Send> BlobReader<S> {
    pub async fn new(digest: Digest, file: BoundedStream<S>) -> Self {
        let file_size = file.size() as u64;
        Self {
            digest,
            reader: Box::new(file.reader()),
            size: file_size,
        }
    }

    pub fn get_reader(self) -> Box<S> {
        self.reader
    }

    pub fn digest(&self) -> &Digest {
        &self.digest
    }

    pub fn blob_size(&self) -> u64 {
        self.size
    }
}

impl BlobReader<Pin<Box<dyn AsyncRead + Send>>> {
    pub fn new_boxed(digest: Digest, size: usize, reader: Pin<Box<dyn AsyncRead + Send>>) -> Self {
        Self {
            digest,
            reader: Box::new(reader),
            size: size as u64,
        }
    }
}

#[derive(Debug)]
pub struct BlobService {
    repos: Arc<Repositories>,
    storage: Arc<FileStorage>,
}

impl BlobService {
    pub fn new(repos: Arc<Repositories>, storage: Arc<FileStorage>) -> Self {
        Self { repos, storage }
    }

    pub async fn get_blob(
        &self,
        mut repo: String,
        digest: Digest,
        namespace: Option<&str>,
    ) -> Result<BlobReader<Pin<Box<dyn AsyncRead + Send>>>, Error> {
        let digest_str = digest.as_str();
        let blob = parse_reference(&repo, digest_str, namespace)?;

        if blob.registry() != "localhost" {
            repo = format!("f/{}/{}", blob.registry(), blob.repository());
        }

        self.repos
            .blob
            .touch_last_accessed(digest_str, &repo)
            .await?;

        let bounded = self.storage.get_blob_stream(&repo, digest_str).await?;
        let size = bounded.size();
        let reader: Pin<Box<dyn AsyncRead + Send>> = Box::pin(bounded.reader());
        Ok(BlobReader::new_boxed(digest, size, reader))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::file_storage::FileStorage;
    use crate::services::blob_service::BlobService;
    use crate::test_utilities::repos_in_memory;
    use crate::utils::digest::Digest;

    fn setup_storage(dir: &test_temp_dir::TestTempDir) -> (Arc<FileStorage>, std::path::PathBuf) {
        let storage = Arc::new(FileStorage::new(dir.as_path_untracked().to_owned()).unwrap());
        let blobs_dir = dir.as_path_untracked().join("blobs");
        (storage, blobs_dir)
    }

    #[tokio::test]
    async fn get_blob_not_found_when_no_db_entry() {
        let repos = repos_in_memory().await;
        let dir = test_temp_dir::test_temp_dir!();
        let (storage, _blobs_dir) = setup_storage(&dir);
        let svc = BlobService::new(repos, storage);

        let digest = Digest::try_from_raw(
            "sha256:abc123def456789012345678901234567890123456789012345678901234567",
        )
        .unwrap();
        let result = svc.get_blob("myrepo".to_string(), digest, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn get_blob_returns_data() {
        let repos = repos_in_memory().await;
        let dir = test_temp_dir::test_temp_dir!();
        let (storage, blobs_dir) = setup_storage(&dir);

        let digest_str = "sha256:abc123def456789012345678901234567890123456789012345678901234567";
        // Insert blob record
        sqlx::query!(
            "INSERT INTO blob (digest, size) VALUES (?, ?)",
            digest_str,
            4i64
        )
        .execute(repos.db_rw())
        .await
        .unwrap();
        // Insert blob association
        sqlx::query!(
            "INSERT INTO repo_blob_assoc (repo_name, blob_digest, manifest_digest) VALUES (?, ?, NULL)",
            "myrepo", digest_str
        )
        .execute(repos.db_rw())
        .await
        .unwrap();
        // Write blob file to storage
        let blob_path = blobs_dir.join(digest_str);
        tokio::fs::write(&blob_path, b"test").await.unwrap();

        let svc = BlobService::new(repos.clone(), storage);
        let digest = Digest::try_from_raw(digest_str).unwrap();
        let result = svc
            .get_blob("myrepo".to_string(), digest, None)
            .await
            .unwrap();

        assert_eq!(result.blob_size(), 4);
    }
}
