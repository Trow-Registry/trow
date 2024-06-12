use bytes::Bytes;

pub struct ManifestReader {
    content_type: String,
    digest: String,
    contents: Bytes,
    size: u64,
}

impl ManifestReader {
    pub async fn new(content_type: String, digest: String, contents: Bytes) -> Self {
        let size = contents.len() as u64;
        Self {
            content_type,
            digest,
            contents,
            size,
        }
    }

    pub fn get_contents(self) -> Bytes {
        self.contents
    }

    pub fn content_type(&self) -> &str {
        &self.content_type
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn size(&self) -> u64 {
        self.size
    }
}
