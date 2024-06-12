use std::borrow::Cow;

use anyhow::Result;
use lazy_static::lazy_static;
use oci_spec::image::{ImageIndex, ImageManifest, MediaType};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::registry::digest::{Digest, DigestError};
use crate::registry::RegistryError;

lazy_static! {
    pub static ref REGEX_TAG: Regex = Regex::new("^[a-zA-Z0-9_][a-zA-Z0-9._-]{0,127}$").unwrap();
}

#[derive(thiserror::Error, Debug)]
pub enum ManifestError {
    #[error("Could not serialize manifest: {0}")]
    DeserializeError(#[from] serde_json::Error),
    #[error("Manifest contains invalid digest: {0}")]
    InvalidDigest(#[from] DigestError),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ManifestReference {
    Tag(String),
    Digest(String),
}

impl std::fmt::Display for ManifestReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Tag(t) => Cow::Borrowed(t),
            Self::Digest(d) => Cow::Owned(d.to_string()),
        };
        write!(f, "{}", s)
    }
}

impl TryFrom<&str> for ManifestReference {
    type Error = RegistryError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_str(value)
    }
}

impl ManifestReference {
    pub fn try_from_str(reference: &str) -> Result<Self, RegistryError> {
        if reference.contains(':') {
            match Digest::try_from_raw(reference) {
                Ok(_) => Ok(Self::Digest(reference.to_string())),
                Err(_) => Err(RegistryError::InvalidDigest),
            }
        } else if REGEX_TAG.is_match(reference) {
            Ok(Self::Tag(reference.to_string()))
        } else {
            Err(RegistryError::InvalidName(String::new()))
        }
    }

    pub fn tag(&self) -> Option<&str> {
        match self {
            Self::Tag(t) => Some(t),
            Self::Digest(_) => None,
        }
    }

    pub fn digest(&self) -> Option<&str> {
        match self {
            Self::Tag(_) => None,
            Self::Digest(d) => Some(d),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum OCIManifest {
    List(ImageIndex),
    V2(ImageManifest),
}

// #[derive(Error, Debug)]
// #[error("Invalid Manifest: {err:?}")]
// pub struct InvalidManifest {
//     err: String,
// }

// TODO: Consider changing this to enum with as_str() impl?
pub mod manifest_media_type {
    pub const DOCKER_V1: &str = "application/vnd.docker.distribution.manifest.v1+json";
    pub const DOCKER_V2: &str = "application/vnd.docker.distribution.manifest.v2+json";
    pub const OCI_V1: &str = "application/vnd.oci.image.manifest.v1+json";
    pub const DOCKER_LIST: &str = "application/vnd.docker.distribution.manifest.list.v2+json";
    pub const OCI_INDEX: &str = "application/vnd.oci.image.index.v1+json";

    // Weirdly the media type is optional in the JSON, so assume OCI_V1.
    // TODO: Check if we should be falling back to mime type
    pub const DEFAULT: &str = OCI_V1;
}

impl OCIManifest {
    /// Returns a Vector of the digests of all assets referenced in the Manifest
    /// With the exception of digests for "foreign blobs"
    pub fn get_local_asset_digests(&self) -> Vec<String> {
        let digests = match self {
            OCIManifest::V2(ref m2) => {
                let mut digests: Vec<String> = m2
                    .layers()
                    .iter()
                    .filter(|l| {
                        l.media_type()
                            != &MediaType::Other(
                                "application/vnd.docker.image.rootfs.foreign.diff.tar.gzip"
                                    .to_string(),
                            )
                    })
                    .map(|x| x.digest().to_string())
                    .collect();
                digests.push(m2.config().digest().to_string());
                digests
            }
            OCIManifest::List(ref list) => {
                // Just return the manifest digests.
                // We could recurse into the manifests, but they should have been checked already.
                list.manifests()
                    .iter()
                    .map(|x| x.digest().to_string())
                    .collect()
            }
        };
        digests
    }

    pub fn media_type(&self) -> &Option<MediaType> {
        match &self {
            OCIManifest::V2(m2) => m2.media_type(),
            OCIManifest::List(list) => list.media_type(),
        }
    }
}

#[cfg(test)]
mod test {

    use super::OCIManifest;

    #[test]
    fn ocimanifest_parse_valid_v2_2() {
        let data = r#"{
  "schemaVersion": 2,
  "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
  "config": {
    "mediaType": "application/vnd.docker.container.image.v1+json",
    "digest": "sha256:4d3c246dfef2edb11eccb051b47d896d0db8f1c4563c0cce9f6274b9abd9ac74",
    "size": 0
  },
  "layers": [
    {
      "mediaType": "application/vnd.docker.image.rootfs.diff.tar.gzip",
      "size": 2789670,
      "digest": "sha256:9d48c3bd43c520dc2784e868a780e976b207cbf493eaff8c6596eb871cbd9609"
    },
    {
      "mediaType": "application/vnd.docker.image.rootfs.diff.tar.gzip",
      "size": 5876721,
      "digest": "sha256:1ae95a11626f76a9bd496d4666276e4495508be864c894ce25602c0baff06826"
    },
    {
      "mediaType": "application/vnd.docker.image.rootfs.foreign.diff.tar.gzip",
      "size": 1612893008,
      "digest": "sha256:9038b92872bc268d5c975e84dd94e69848564b222ad116ee652c62e0c2f894b2",
      "urls": [
        "https://mcr.microsoft.com/v2/windows/servercore/blobs/sha256:9038b92872bc268d5c975e84dd94e69848564b222ad116ee652c62e0c2f894b2"
      ]
    }
  ]
}"#;
        let mani: OCIManifest = serde_json::from_str(data).unwrap();
        let m_v2 = match mani {
            OCIManifest::V2(ref m2) => m2,
            OCIManifest::List(_) => unreachable!(),
        };

        // oci-spec serializes docker mediatypes as `Other`
        // assert_eq!(m_v2.media_type(), &Some(MediaType::ImageManifest));
        assert_eq!(m_v2.schema_version(), 2);
        // assert_eq!(m_v2.config().media_type(), &MediaType::ImageConfig);
        assert_eq!(m_v2.config().size(), 0);
        assert_eq!(
            m_v2.config().digest(),
            "sha256:4d3c246dfef2edb11eccb051b47d896d0db8f1c4563c0cce9f6274b9abd9ac74"
        );
        // assert_eq!(m_v2.layers()[0].media_type(), &MediaType::ImageLayerGzip);
        assert_eq!(m_v2.layers()[0].size(), 2789670);
        assert_eq!(
            m_v2.layers()[0].digest(),
            "sha256:9d48c3bd43c520dc2784e868a780e976b207cbf493eaff8c6596eb871cbd9609"
        );
        assert_eq!(m_v2.layers().len(), 3);
        let digests_str: Vec<_> = mani.get_local_asset_digests();

        assert_eq!(digests_str.len(), 3);
        assert!(digests_str.contains(
            &"sha256:9d48c3bd43c520dc2784e868a780e976b207cbf493eaff8c6596eb871cbd9609".to_string()
        ));
        assert!(digests_str.contains(
            &"sha256:1ae95a11626f76a9bd496d4666276e4495508be864c894ce25602c0baff06826".to_string()
        ));
    }

    #[test]
    fn ocimanifest_parse_valid_v2() {
        let data = r#"{
     "schemaVersion": 2,
     "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
     "config": {
        "mediaType": "application/vnd.docker.container.image.v1+json",
        "size": 1278,
        "digest": "sha256:4a415e3663882fbc554ee830889c68a33b3585503892cc718a4698e91ef2a526"
     },
     "layers": [
        {
           "mediaType": "application/vnd.docker.image.rootfs.diff.tar.gzip",
           "size": 1967949,
           "digest": "sha256:1e76f742da490c8d7c921e811e5233def206e76683ee28d735397ec2231f131d"
        }
     ]
   }"#;
        let mani: OCIManifest = serde_json::from_str(data).unwrap();
        let m_v2 = match mani {
            OCIManifest::V2(ref m2) => m2,
            OCIManifest::List(_) => unreachable!(),
        };

        // assert_eq!(m_v2.media_type(), &Some(MediaType::ImageManifest));
        assert_eq!(m_v2.schema_version(), 2);
        // assert_eq!(m_v2.config().media_type(), &MediaType::ImageConfig);
        assert_eq!(m_v2.config().size(), 1278);
        assert_eq!(
            m_v2.config().digest(),
            "sha256:4a415e3663882fbc554ee830889c68a33b3585503892cc718a4698e91ef2a526"
        );
        // assert_eq!(m_v2.layers()[0].media_type(), &MediaType::ImageLayerGzip);
        assert_eq!(m_v2.layers()[0].size(), 1967949);
        assert_eq!(
            m_v2.layers()[0].digest(),
            "sha256:1e76f742da490c8d7c921e811e5233def206e76683ee28d735397ec2231f131d"
        );

        let digests_str: Vec<_> = mani.get_local_asset_digests();
        assert_eq!(digests_str.len(), 2);
        assert!(digests_str.contains(
            &"sha256:1e76f742da490c8d7c921e811e5233def206e76683ee28d735397ec2231f131d".to_string()
        ));
        assert!(digests_str.contains(
            &"sha256:4a415e3663882fbc554ee830889c68a33b3585503892cc718a4698e91ef2a526".to_string()
        ));
    }
}
