use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::registry_interface::Digest;

// TODO: Kill this file. Move types and methods to where they're used.

#[derive(Clone, Debug, Display, Serialize)]
#[display(fmt = "{}", _0)]
pub struct Uuid(pub String);

#[derive(Clone, Debug, Display, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[display(fmt = "{}", _0)]
pub struct RepoName(pub String);

#[derive(Deserialize, Debug)]
pub struct DigestQuery {
    pub digest: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UploadInfo {
    base_url: String,
    uuid: Uuid,
    repo_name: RepoName,
    range: (u32, u32),
}

pub struct BlobDeleted {}

pub struct ManifestDeleted {}

impl UploadInfo {
    pub fn new(base_url: String, uuid: Uuid, repo_name: RepoName, range: (u32, u32)) -> Self {
        Self {
            base_url,
            uuid,
            repo_name,
            range,
        }
    }

    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn repo_name(&self) -> &RepoName {
        &self.repo_name
    }

    pub fn range(&self) -> (u32, u32) {
        self.range
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

#[derive(Debug, Serialize)]
pub struct AcceptedUpload {
    base_url: String,
    digest: Digest,
    repo_name: RepoName,
    uuid: Uuid,
    range: (u32, u32),
}

impl AcceptedUpload {
    pub fn new(
        base_url: String,
        digest: Digest,
        repo_name: RepoName,
        uuid: Uuid,
        range: (u32, u32),
    ) -> Self {
        Self {
            base_url,
            digest,
            repo_name,
            uuid,
            range,
        }
    }

    pub fn digest(&self) -> &Digest {
        &self.digest
    }

    pub fn repo_name(&self) -> &RepoName {
        &self.repo_name
    }

    pub fn range(&self) -> (u32, u32) {
        self.range
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

#[derive(Serialize, Debug)]
pub enum Upload {
    Accepted(AcceptedUpload),
    Info(UploadInfo),
}

#[derive(Debug, Serialize)]
pub struct VerifiedManifest {
    base_url: Option<String>,
    repo_name: RepoName,
    digest: Digest,
    tag: String,
}

impl VerifiedManifest {
    pub fn new(base_url: Option<String>, repo_name: RepoName, digest: Digest, tag: String) -> Self {
        Self {
            base_url,
            repo_name,
            digest,
            tag,
        }
    }

    pub fn digest(&self) -> &Digest {
        &self.digest
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn repo_name(&self) -> &RepoName {
        &self.repo_name
    }
    pub fn base_url(&self) -> Option<&String> {
        self.base_url.as_ref()
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
