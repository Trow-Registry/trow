use crate::registry_interface::Digest;

use derive_more::Display;
use rocket::Responder;
use serde::{Deserialize, Serialize};

// TODO: Kill this file. Move types and methods to where they're used.

#[derive(Clone, Debug, Display, Serialize)]
#[display(fmt = "{}", _0)]
pub struct Uuid(pub String);

#[derive(Clone, Debug, Display, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[display(fmt = "{}", _0)]
pub struct RepoName(pub String);

#[derive(Debug, Serialize)]
pub struct UploadInfo {
    uuid: Uuid,
    repo_name: RepoName,
    range: (u32, u32),
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
