use std::io::Read;

pub type Digest = String;

#[derive(Debug, Serialize)]
pub struct UploadInfo {
    uuid: String,
    repo_name: String,
    range: (u32, u32),
}

impl UploadInfo {
    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    pub fn repo_name(&self) -> &str {
        &self.repo_name
    }

    pub fn range(&self) -> (u32, u32) {
        self.range
    }
}

pub fn create_upload_info(uuid: String, repo_name: String, range: (u32, u32)) -> UploadInfo {
    UploadInfo {
        uuid,
        repo_name,
        range,
    }
}

#[derive(Debug, Serialize)]
pub struct AcceptedUpload {
    digest: String,
    repo_name: String,
}

pub fn create_accepted_upload(digest: String, repo_name: String) -> AcceptedUpload {
    AcceptedUpload {
        digest,
        repo_name,
    }
}
impl AcceptedUpload {

    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn repo_name(&self) -> &str {
        &self.repo_name
    }
}

#[derive(Debug, Serialize)]
pub struct VerifiedManifest {
    location: String,
    digest: String,
    content_type: String
}

impl VerifiedManifest {
    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn location(&self) -> &str {
        &self.location
    }
}

pub fn create_verified_manifest(location: String, digest: String, content_type: String) -> VerifiedManifest {
    VerifiedManifest { location, digest, content_type }
}

pub struct ManifestReader {
    content_type: String,
    digest: String,
    reader: Box<Read>,
}

impl ManifestReader {
    pub fn get_reader(self) -> Box<Read> {
        self.reader
    }

    pub fn content_type(&self) -> &str {
        &self.content_type
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }
}

pub fn create_manifest_reader(reader: Box<Read>, content_type: String, digest: String) -> ManifestReader {
    ManifestReader { reader, content_type, digest }
}

pub struct BlobReader {
    digest: String,
    reader: Box<Read>,
}

impl BlobReader {
    pub fn get_reader(self) -> Box<Read> {
        self.reader
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }
}

pub fn create_blob_reader(reader: Box<Read>, digest: String) -> BlobReader {
    BlobReader { reader, digest }
}


//DIE MOFO
#[derive(Debug, Clone)]
pub struct Layer {
    pub digest: Digest,
    pub repo_name: String,
}
