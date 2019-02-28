use std::collections::HashSet;
use std::io::Read;

#[derive(Clone, Debug, Display, Serialize)]
#[display(fmt = "{}", _0)]
pub struct Uuid(pub String);

#[derive(Clone, Debug, Display, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct RepoCatalog {
    #[serde(rename = "repositories")]
    catalog: HashSet<RepoName>,
}

impl RepoCatalog {
    pub fn new() -> RepoCatalog {
        RepoCatalog {
            catalog: HashSet::new(),
        }
    }

    pub fn insert(&mut self, rn: RepoName) {
        self.catalog.insert(rn);
    }

    pub fn catalog(&self) -> &HashSet<RepoName> {
        &self.catalog
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TagList {
    #[serde(rename = "name")]
    repo: RepoName,
    #[serde(rename = "tags")]
    list: HashSet<String>,
}

impl TagList {
    pub fn new(repo_name: RepoName) -> TagList {
        TagList {
            repo: repo_name,
            list: HashSet::new(),
        }
    }

    pub fn insert(&mut self, tag: String) {
        self.list.insert(tag);
    }

    pub fn repo_name(&self) -> &RepoName {
        &self.repo
    }

    pub fn list(&self) -> &HashSet<String> {
        &self.list
    }
}


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Status {
    pub status: String,          //"Success" or "Failure". TODO: use proper type.
    pub message: Option<String>, //Human readable description. Shown in kubectl output.
    /*
    pub reason: String, //Machine readable description of "failure". Not sure where this goes.
    pub details: ?, // Data associated with reason field
    */
    pub code: Option<i32>, // Suggested http return code, 0 if not set
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AdmissionRequest {
    pub uid: String,
    pub object: serde_json::Value,
    pub namespace: String,
    pub operation: String, //CREATE, UPDATE, DELETE, CONNECT
                           //probably want user info as well, but normally it's the service account :(
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AdmissionResponse {
    pub uid: String,
    pub allowed: bool,
    pub status: Option<Status>,
    /* Not yet implemented, Patch, PatchType & AuditAnnotations. */
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AdmissionReview {
    //TODO: Get rid of stringly typing
    pub api_version: String,
    pub kind: String,
    pub request: Option<AdmissionRequest>,
    pub response: Option<AdmissionResponse>,
}
