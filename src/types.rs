use std::ops::Deref;

use serde::{Deserialize, Serialize};
use tokio::io::AsyncRead;

use crate::registry::Digest;

// TODO: Kill this file. Move types and methods to where they're used.

pub struct BoundedStream<S: AsyncRead> {
    size: usize,
    stream: S,
}

impl<S: AsyncRead> Deref for BoundedStream<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl<S: AsyncRead> BoundedStream<S> {
    pub fn new(size: usize, stream: S) -> Self {
        BoundedStream { size, stream }
    }

    pub fn size(&self) -> usize {
        self.size
    }
    pub fn reader(self) -> S {
        self.stream
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct OptionalDigestQuery {
    pub digest: Option<Digest>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DigestQuery {
    pub digest: Digest,
}

#[derive(Debug, Serialize)]
pub struct UploadInfo {
    uuid: String,
    repo_name: String,
    range: (u64, u64),
}

pub struct BlobDeleted {}

pub struct ManifestDeleted {}

impl UploadInfo {
    pub fn new(uuid: String, repo_name: String, range: (u64, u64)) -> Self {
        Self {
            uuid,
            repo_name,
            range,
        }
    }

    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    pub fn repo_name(&self) -> &String {
        &self.repo_name
    }

    pub fn range(&self) -> (u64, u64) {
        self.range
    }
}

#[derive(Debug, Serialize)]
pub struct AcceptedUpload {
    digest: Digest,
    repo_name: String,
    uuid: uuid::Uuid,
    range: (u64, u64),
}

impl AcceptedUpload {
    pub fn new(digest: Digest, repo_name: String, uuid: uuid::Uuid, range: (u64, u64)) -> Self {
        Self {
            digest,
            repo_name,
            uuid,
            range,
        }
    }

    pub fn digest(&self) -> &Digest {
        &self.digest
    }

    pub fn repo_name(&self) -> &String {
        &self.repo_name
    }

    pub fn range(&self) -> (u64, u64) {
        self.range
    }

    pub fn uuid(&self) -> &uuid::Uuid {
        &self.uuid
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
    repo_name: String,
    digest: Digest,
    tag: String,
    subject: Option<String>,
}

impl VerifiedManifest {
    pub fn new(
        base_url: Option<String>,
        repo_name: String,
        digest: Digest,
        tag: String,
        subject: Option<String>,
    ) -> Self {
        Self {
            base_url,
            repo_name,
            digest,
            tag,
            subject,
        }
    }

    pub fn subject(&self) -> Option<&String> {
        self.subject.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_str()
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn repo_name(&self) -> &String {
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
