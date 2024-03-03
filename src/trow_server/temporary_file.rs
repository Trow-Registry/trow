use std::path::{Path, PathBuf};

use anyhow::Result;
use bytes::Bytes;
use futures::stream::Stream;
use futures::StreamExt;
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
    pub async fn open_for_writing(path: PathBuf) -> io::Result<TemporaryFile> {
        let res = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .await;
        let file = match res {
            Ok(f) => f,
            Err(e) => return Err(e),
        };
        Ok(TemporaryFile { file, path })
    }

    pub async fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.file.write_all(buf).await?;
        self.file.flush().await
    }

    pub async fn write_stream<S, E>(&mut self, mut stream: S) -> io::Result<()>
    where
        S: Stream<Item = Result<Bytes, E>> + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| io::Error::new(io::ErrorKind::UnexpectedEof, e))?;
            self.file.write_all(&chunk).await?;
        }
        self.file.flush().await?;
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub async fn rename(self, new_path: &Path) -> io::Result<()> {
        tokio::fs::rename(&self.path, new_path).await
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
    use tempfile::tempdir;

    use super::*;

    #[tokio::test]
    async fn test_temporary_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");
        let mut file = TemporaryFile::open_for_writing(path.clone()).await.unwrap();
        assert!(
            TemporaryFile::open_for_writing(path.clone())
                .await
                .err()
                .unwrap()
                .kind()
                == io::ErrorKind::AlreadyExists,
            "The same file cannot be opened for writing twice !"
        );
        file.write_all(b"hello").await.unwrap();
        assert_eq!(file.path(), path);
        drop(file);
        assert!(!path.exists(), "File should have been deleted");
    }

    /*
    #[tokio::test]
    async fn test_temporary_file_async_cancellation() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");

        let futures = (0..2).map(|_| {
            async {
                let mut file = match TemporaryFile::open_for_writing(path.clone()).await.unwrap() {
                    Some(f) => f,
                    None => return Err(()) as Result<(), ()>,
                };
                file.write_all(b"hello").await.unwrap();
                // Sleep to ensure that the future stay active long enough to be cancelled
                sleep(Duration::from_millis(500)).await;
                // Ensure `file` isn't dropped before the sleep
                drop(file);
                unreachable!();
            }
        });

        let res = try_join_all(futures).await;
        assert!(
            res.is_err(),
            "The same file cannot be opened for writing twice !"
        );
        assert!(!path.exists(), "File should have been deleted");
    }
    */

    #[tokio::test]
    async fn test_write() {
        const DUMMY_DATA: &[u8] = b"0123456789ABCDEF";
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        let mut file = TemporaryFile::open_for_writing(file_path.clone())
            .await
            .unwrap();
        file.write_all(DUMMY_DATA).await.unwrap();
        assert_eq!(fs::read(file.path()).await.unwrap(), DUMMY_DATA);
        drop(file);

        let mut file = TemporaryFile::open_for_writing(file_path.clone())
            .await
            .unwrap();
        let dummy_stream = futures::stream::iter(DUMMY_DATA.chunks(4).map(|b| Ok(Bytes::from(b))));
        file.write_stream::<_, reqwest::Error>(dummy_stream)
            .await
            .unwrap();
        assert_eq!(fs::read(file.path()).await.unwrap(), DUMMY_DATA);
        drop(file);
    }
}
