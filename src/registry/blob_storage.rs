use futures::AsyncRead;

use super::digest::Digest;
use crate::types::BoundedStream;

pub struct ContentInfo {
    pub length: u64,
    pub range: (u64, u64),
}

#[allow(dead_code)]
pub struct UploadInfo {
    name: String,
    session_id: String,
    uploaded: u32,
    size: u32,
}

pub struct BlobReader<S: AsyncRead + ?Sized + Send> {
    digest: Digest,
    reader: Box<S>,
    size: u64,
}
pub struct Stored {
    pub total_stored: u64,
    pub chunk: u64,
}

impl<S: futures::AsyncRead + Send> BlobReader<S> {
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
