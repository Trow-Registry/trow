use chrono::{DateTime, Utc};
use derive_more::Display;
use std::io::{Read, Seek};

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

pub struct ContentInfo {
    pub length: u64,
    pub range: (u64, u64),
}

pub struct BlobDeleted {}

pub struct ManifestDeleted {}

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
    uuid: Uuid,
    range: (u32, u32),
}

pub fn create_accepted_upload(
    digest: Digest,
    repo_name: RepoName,
    uuid: Uuid,
    range: (u32, u32),
) -> AcceptedUpload {
    AcceptedUpload {
        digest,
        repo_name,
        uuid,
        range,
    }
}

impl AcceptedUpload {
    pub fn digest(&self) -> &Digest {
        &self.digest
    }

    pub fn repo_name(&self) -> &RepoName {
        &self.repo_name
    }

    pub fn range(&self) -> (u32, u32) {
        self.range
    }
}

#[derive(Responder)]
pub enum Upload {
    Accepted(AcceptedUpload),
    Info(UploadInfo),
}

#[derive(Debug, Serialize)]
pub struct VerifiedManifest {
    repo_name: RepoName,
    digest: Digest,
    tag: String,
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
}

pub fn create_verified_manifest(
    repo_name: RepoName,
    digest: Digest,
    tag: String,
) -> VerifiedManifest {
    VerifiedManifest {
        repo_name,
        digest,
        tag,
    }
}

//If there's a better solution, please let me know.
//I'd much rather not have to write an impl for every class :(
pub trait SeekRead: Read + Seek {}
impl SeekRead for std::fs::File {}
pub struct ManifestReader {
    content_type: String,
    digest: Digest,
    reader: Box<dyn SeekRead>,
}

impl ManifestReader {
    pub fn get_reader(self) -> Box<dyn SeekRead> {
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
    reader: Box<dyn SeekRead>,
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
    reader: Box<dyn SeekRead>,
}

impl BlobReader {
    pub fn get_reader(self) -> Box<dyn SeekRead> {
        self.reader
    }

    pub fn digest(&self) -> &Digest {
        &self.digest
    }
}

pub fn create_blob_reader(reader: Box<dyn SeekRead>, digest: Digest) -> BlobReader {
    BlobReader { reader, digest }
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct RepoCatalog {
    #[serde(rename = "repositories")]
    catalog: Vec<String>,
}

impl RepoCatalog {
    pub fn new() -> RepoCatalog {
        RepoCatalog {
            catalog: Vec::new(),
        }
    }

    pub fn insert(&mut self, rn: String) {
        self.catalog.push(rn);
        self.catalog.sort();
    }

    pub fn catalog(&self) -> &Vec<String> {
        &self.catalog
    }

    pub fn raw(self) -> Vec<String> {
        self.catalog
    }
}

impl From<Vec<String>> for RepoCatalog {
    fn from(cat: Vec<String>) -> Self {
        RepoCatalog { catalog: cat }
    }
}

mod history_date_format {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d %H:%M:%S%.f %Z";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct HistoryEntry {
    pub digest: String,
    #[serde(with = "history_date_format")]
    pub date: DateTime<Utc>,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct ManifestHistory {
    #[serde(rename = "image")]
    tag: String,
    history: Vec<HistoryEntry>,
}

impl ManifestHistory {
    pub fn new(tag: String) -> ManifestHistory {
        ManifestHistory {
            tag,
            history: Vec::new(),
        }
    }

    pub fn insert(&mut self, digest: String, date: DateTime<Utc>) {
        self.history.push(HistoryEntry { digest, date });
    }

    pub fn catalog(&self) -> &Vec<HistoryEntry> {
        &self.history
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TagList {
    #[serde(rename = "name")]
    repo: String,
    #[serde(rename = "tags")]
    list: Vec<String>,
}

impl TagList {
    pub fn new(repo_name: String) -> TagList {
        TagList {
            repo: repo_name,
            list: Vec::new(),
        }
    }

    pub fn new_filled(repo: String, list: Vec<String>) -> TagList {
        TagList { repo, list }
    }

    pub fn insert(&mut self, tag: String) {
        self.list.push(tag);
    }

    pub fn repo_name(&self) -> &str {
        &self.repo
    }

    pub fn list(&self) -> &Vec<String> {
        &self.list
    }

    pub fn raw(self) -> Vec<String> {
        self.list
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

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct HealthResponse {
    pub message: String,
    pub is_healthy: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReadinessResponse {
    pub message: String,
    pub is_ready: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MetricsResponse {
    pub metrics: String,
}
