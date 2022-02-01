use std::path::{Path, PathBuf};
use tokio::fs::{self, File};
use tokio::io::{self, AsyncWriteExt};

/// Designed for downloading files. The [`Drop`] implementation makes sure that
/// the underlying file is deleted in case of an error.
/// Intended use: create the [`TemporaryFile`], write to it, then move
/// the underlying file to its final destination.
pub struct TemporaryFile {
    file: File,
    path: PathBuf,
}

impl TemporaryFile {
    /// Returns `Ok(None)` if the file already exists.
    pub async fn open_for_writing(path: PathBuf) -> io::Result<Option<TemporaryFile>> {
        let res = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .await;
        let file = match res {
            Ok(f) => f,
            Err(e) => match e.kind() {
                std::io::ErrorKind::AlreadyExists => {
                    return Ok(None);
                }
                _ => return Err(e.into()),
            },
        };
        Ok(Some(TemporaryFile { file, path }))
    }
    pub async fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.file.write_all(buf).await
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Special drop to ensure that the file is removed
impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::future::try_join_all;
    use tempfile::tempdir;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_temporary_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");
        let mut file = TemporaryFile::open_for_writing(path.clone())
            .await
            .unwrap()
            .unwrap();
        assert!(
            TemporaryFile::open_for_writing(path.clone())
                .await
                .unwrap()
                .is_none(),
            "The same file cannot be opened for writing twice !"
        );
        file.write_all(b"hello").await.unwrap();
        assert_eq!(file.path(), path);
        drop(file);
        assert!(!path.exists(), "File should have been deleted");
    }

    #[tokio::test]
    async fn test_temporary_file_async_cancellation() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");

        let futures = (0..2).map(|_| {
            let path = path.clone();
            async move {
                let mut file = match TemporaryFile::open_for_writing(path.clone()).await.unwrap() {
                    Some(f) => f,
                    None => return Err(()),
                };
                file.write_all(b"hello").await.unwrap();
                sleep(Duration::from_millis(500)).await;
                drop(file);
                Ok(())
            }
        });

        let res = try_join_all(futures).await;
        assert!(
            res.is_err(),
            "The same file cannot be opened for writing twice !"
        );
        assert!(!path.exists(), "File should have been deleted");
    }
}
