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
pub struct FileWrapper {
    file: File,
    path: PathBuf,
    temporary: bool,
}

impl FileWrapper {
    pub async fn new_tmp(path: PathBuf) -> io::Result<Self> {
        let mut open_opt = fs::OpenOptions::new();
        let file = open_opt.create_new(true).write(true).open(&path).await?;

        Ok(FileWrapper {
            file,
            path,
            temporary: true,
        })
    }

    pub async fn append(path: PathBuf) -> io::Result<Self> {
        let mut open_opt = fs::OpenOptions::new();
        let file = open_opt.append(true).create(true).open(&path).await?;

        Ok(FileWrapper {
            file,
            path,
            temporary: false,
        })
    }

    #[allow(unused)]
    pub async fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.file.write_all(buf).await?;
        self.file.flush().await
    }

    /// Returns the number of bytes written
    pub async fn write_stream<S, E>(&mut self, mut stream: S) -> io::Result<usize>
    where
        S: Stream<Item = Result<Bytes, E>> + Unpin,
        E: std::error::Error + Send + Sync + 'static,
    {
        let mut len = 0;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| io::Error::new(io::ErrorKind::UnexpectedEof, e))?;
            len += chunk.len();
            self.file.write_all(&chunk).await?;
        }
        self.file.flush().await?;
        Ok(len)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub async fn metadata(&self) -> io::Result<std::fs::Metadata> {
        self.file.metadata().await
    }

    pub async fn rename(self, new_path: &Path) -> io::Result<()> {
        tokio::fs::rename(&self.path, new_path).await
    }
}

/// Special drop to ensure that the file is removed
impl Drop for FileWrapper {
    fn drop(&mut self) {
        if self.temporary {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use futures::future::try_join_all;
    use test_temp_dir::test_temp_dir;

    use super::*;

    #[tokio::test]
    async fn test_temporary_file() {
        let dir = test_temp_dir!();
        let path = dir.subdir_untracked("test.txt");
        let mut file = FileWrapper::new_tmp(path.clone()).await.unwrap();
        assert!(
            FileWrapper::new_tmp(path.clone())
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

    #[tokio::test]
    async fn test_temporary_file_async_cancellation() {
        let tmp_dir = test_temp_dir!();
        let tmp_path = tmp_dir.as_path_untracked();
        let path = tmp_path.join("test.txt");

        let futures = (0..2).map(|_| {
            async {
                let mut file = match FileWrapper::new_tmp(path.clone()).await {
                    Ok(f) => f,
                    Err(_) => return Err(()) as Result<(), ()>,
                };
                file.write_all(b"hello").await.unwrap();
                // Sleep to ensure that the future stay active long enough to be cancelled
                tokio::time::sleep(Duration::from_millis(500)).await;
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

    #[tokio::test]
    async fn test_write() {
        const DUMMY_DATA: &[u8] = b"0123456789ABCDEF";
        let tmp_dir = test_temp_dir!();
        let tmp_path = tmp_dir.as_path_untracked();
        let file_path = tmp_path.join("test.txt");

        let mut file = FileWrapper::new_tmp(file_path.clone()).await.unwrap();
        file.write_all(DUMMY_DATA).await.unwrap();
        assert_eq!(fs::read(file.path()).await.unwrap(), DUMMY_DATA);
        drop(file);

        let mut file = FileWrapper::new_tmp(file_path.clone()).await.unwrap();
        let dummy_stream = futures::stream::iter(DUMMY_DATA.chunks(4).map(|b| Ok(Bytes::from(b))));
        file.write_stream::<_, reqwest::Error>(dummy_stream)
            .await
            .unwrap();
        assert_eq!(fs::read(file.path()).await.unwrap(), DUMMY_DATA);
        drop(file);
    }
}
