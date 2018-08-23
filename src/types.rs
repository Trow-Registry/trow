use std::io::Read;

#[derive(Clone, Debug, Display, Serialize)]
#[display(fmt = "{}", _0)]
pub struct Uuid(pub String);

#[derive(Clone, Debug, Display, Serialize)]
#[display(fmt = "{}", _0)]
pub struct RepoName(pub String);

#[derive(Clone, Debug, Display, Serialize)]
#[display(fmt = "{}", _0)]
pub struct Digest(pub String);

#[derive(Debug, Serialize)]
pub struct UploadInfo {
    uuid: Uuid,
    repo_name: RepoName,
    range: (u32, u32),
}

impl UploadInfo {
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn repo_name(&self) -> &RepoName {
        &self.repo_name
    }

    pub fn range(&self) -> (u32, u32) {
        self.range
    }
}

pub fn create_upload_info(uuid: Uuid, repo_name: RepoName, range: (u32, u32)) -> UploadInfo {
    UploadInfo {
        uuid,
        repo_name,
        range,
    }
}

#[derive(Debug, Serialize)]
pub struct AcceptedUpload {
    digest: Digest,
    repo_name: RepoName,
}

pub fn create_accepted_upload(digest: Digest, repo_name: RepoName) -> AcceptedUpload {
    AcceptedUpload { digest, repo_name }
}
impl AcceptedUpload {
    pub fn digest(&self) -> &Digest {
        &self.digest
    }

    pub fn repo_name(&self) -> &RepoName {
        &self.repo_name
    }
}

#[derive(Debug, Serialize)]
pub struct VerifiedManifest {
    repo_name: RepoName,
    digest: Digest,
    tag: String,
    content_type: String,
}

impl VerifiedManifest {
    pub fn digest(&self) -> &Digest {
        &self.digest
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn repo_name(&self) -> &RepoName {
        &self.repo_name
    }

    pub fn content_type(&self) -> &str {
        &self.content_type
    }
}

pub fn create_verified_manifest(
    repo_name: RepoName,
    digest: Digest,
    tag: String,
    content_type: String,
) -> VerifiedManifest {
    VerifiedManifest {
        repo_name,
        digest,
        tag,
        content_type,
    }
}

pub struct ManifestReader {
    content_type: String,
    digest: Digest,
    reader: Box<Read>,
}

impl ManifestReader {
    pub fn get_reader(self) -> Box<Read> {
        self.reader
    }

    pub fn content_type(&self) -> &str {
        &self.content_type
    }

    pub fn digest(&self) -> &Digest {
        &self.digest
    }
}

pub fn create_manifest_reader(
    reader: Box<Read>,
    content_type: String,
    digest: Digest,
) -> ManifestReader {
    ManifestReader {
        reader,
        content_type,
        digest,
    }
}

pub struct BlobReader {
    digest: Digest,
    reader: Box<Read>,
}

impl BlobReader {
    pub fn get_reader(self) -> Box<Read> {
        self.reader
    }

    pub fn digest(&self) -> &Digest {
        &self.digest
    }
}

pub fn create_blob_reader(reader: Box<Read>, digest: Digest) -> BlobReader {
    BlobReader { reader, digest }
}
