//! Results of the manifest workflow.

use crate::utils::digest::Digest;

/// A manifest that has been validated, stored and tagged.
#[derive(Debug)]
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

/// Marker returned from a successful manifest deletion.
#[derive(Debug)]
pub struct ManifestDeleted {}
