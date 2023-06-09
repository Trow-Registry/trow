use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use thiserror::Error;

pub trait FromJson {
    fn from_json(raw: &Value) -> Result<Self>
    where
        Self: std::marker::Sized;
}

/**
 *
 * This manifest format is a straight conversion from JSON to Rust for manipulation purposes.
 *
 * It should only be used for input/output; don't use it as an internal data structure.
 * (In other words we may have our own concept of manifests which are used to build these versions).
 *
 * Also note that image metadata may move to some sort of DB in the future for fast reliable searches etc.
 *
 * I'm not really sure this buys us much over the JSON deserialization though...
 *
 * ARG, mistake here, manifest should be responsible for schema vesion tag
 */
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Manifest {
    List(ManifestList),
    V2(ManifestV2),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestList {
    pub schema_version: u8,
    pub media_type: String, //TODO: make enum
    pub manifests: Vec<ManifestListEntry>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestListEntry {
    pub media_type: String, //TODO: make enum
    pub size: u32,
    pub digest: String,
    pub platform: Option<Platform>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Platform {
    pub architecture: String,
    pub os: String,
    #[serde(rename = "os.version")]
    pub os_version: Option<String>,
    #[serde(rename = "os.features")]
    pub os_features: Option<String>,
    pub variant: Option<String>,
    pub features: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestV2 {
    pub schema_version: u8,
    pub media_type: Option<String>, //TODO: make enum
    pub config: Object,
    pub layers: Vec<Object>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Object {
    pub media_type: String, //enum would be better
    pub size: Option<u64>,
    pub digest: String, //special type would be nice
}

#[derive(Error, Debug)]
#[error("Invalid Manifest: {err:?}")]
pub struct InvalidManifest {
    err: String,
}

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

fn schema_2(raw: &Value) -> Result<Manifest> {
    // According to the spec, manifests don't have to have a mediaType (?!).
    // Assume V2 if not present.
    let mt = raw["mediaType"]
        .as_str()
        .unwrap_or(manifest_media_type::DEFAULT);

    match mt {
        manifest_media_type::DOCKER_V2 | manifest_media_type::OCI_V1 => {
            Ok(Manifest::V2(serde_json::from_value(raw.clone())?))
        }

        manifest_media_type::DOCKER_LIST | manifest_media_type::OCI_INDEX => {
            Ok(Manifest::List(serde_json::from_value(raw.clone())?))
        }

        unknown => Err(InvalidManifest {
            err: format!("Media Type {} is not supported.", unknown),
        }
        .into()),
    }
}

impl FromJson for Manifest {
    fn from_json(raw: &Value) -> Result<Self> {
        let schema_version = raw["schemaVersion"].as_u64().ok_or(InvalidManifest {
            err: "schemaVersion is required".to_owned(),
        })?;
        match schema_version {
            1 => Err(InvalidManifest {
                err: "Manifest Schema version 1 is not supported. Please update.".to_owned(),
            }
            .into()),
            2 => schema_2(raw),
            n => Err(InvalidManifest {
                err: format!("Unsupported version: {}", n),
            }
            .into()),
        }
    }
}

impl Manifest {
    /// Returns a Vector of the digests of all assets referenced in the Manifest
    /// With the exception of digests for "foreign blobs"
    pub fn get_local_asset_digests(&self) -> Vec<&str> {
        match *self {
            Manifest::V2(ref m2) => {
                let mut digests: Vec<&str> = m2
                    .layers
                    .iter()
                    .filter(|x| {
                        x.media_type != "application/vnd.docker.image.rootfs.foreign.diff.tar.gzip"
                    })
                    .map(|x| x.digest.as_str())
                    .collect();
                digests.push(&m2.config.digest);
                digests
            }
            Manifest::List(ref list) => {
                // Just return the manifest digests.
                // We could recurse into the manifests, but they should have been checked already.

                list.manifests.iter().map(|x| x.digest.as_str()).collect()
            }
        }
    }

    // TODO: use proper enums and return &str
    pub fn get_media_type(&self) -> String {
        match *self {
            Manifest::V2(ref m2) => m2
                .media_type
                .as_ref()
                .unwrap_or(&manifest_media_type::DEFAULT.to_string())
                .to_string(),
            Manifest::List(ref list) => list.media_type.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::io::BufReader;

    use serde_json::{self, Value};

    use super::{FromJson, Manifest};
    use crate::digest::sha256_tag_digest;

    #[test]
    fn valid_v2_2() {
        let data = r#"{
   "schemaVersion": 2,
   "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
   "config": {
      "mediaType": "application/vnd.docker.container.image.v1+json",
      "digest": "sha256:4d3c246dfef2edb11eccb051b47d896d0db8f1c4563c0cce9f6274b9abd9ac74"
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

        //Pretty sure we should be able to do this directly
        let v: Value = serde_json::from_str(data).unwrap();
        let mani = Manifest::from_json(&v).unwrap();

        // There's probably an easier way to do this
        let m_v2 = match mani {
            Manifest::V2(ref m2) => m2,
            Manifest::List(_) => panic!(),
        };

        assert_eq!(
            m_v2.media_type,
            Some("application/vnd.docker.distribution.manifest.v2+json".to_string())
        );
        assert_eq!(m_v2.schema_version, 2);
        assert_eq!(
            m_v2.config.media_type,
            "application/vnd.docker.container.image.v1+json"
        );
        assert_eq!(m_v2.config.size, None);
        assert_eq!(
            m_v2.config.digest,
            "sha256:4d3c246dfef2edb11eccb051b47d896d0db8f1c4563c0cce9f6274b9abd9ac74"
        );
        assert_eq!(
            m_v2.layers[0].media_type,
            "application/vnd.docker.image.rootfs.diff.tar.gzip"
        );
        assert_eq!(m_v2.layers[0].size, Some(2789670));
        assert_eq!(
            m_v2.layers[0].digest,
            "sha256:9d48c3bd43c520dc2784e868a780e976b207cbf493eaff8c6596eb871cbd9609"
        );

        assert_eq!(mani.get_local_asset_digests().len(), 3);
        assert!(mani
            .get_local_asset_digests()
            .contains(&"sha256:9d48c3bd43c520dc2784e868a780e976b207cbf493eaff8c6596eb871cbd9609"));
        assert!(mani
            .get_local_asset_digests()
            .contains(&"sha256:1ae95a11626f76a9bd496d4666276e4495508be864c894ce25602c0baff06826"));
    }
    #[test]
    fn valid_v2() {
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
        //Pretty sure we should be able to do this directly
        let v: Value = serde_json::from_str(data).unwrap();
        let mani = Manifest::from_json(&v).unwrap();

        // There's probably an easier way to do this
        let m_v2 = match mani {
            Manifest::V2(ref m2) => m2,
            Manifest::List(_) => panic!(),
        };

        assert_eq!(
            m_v2.media_type,
            Some("application/vnd.docker.distribution.manifest.v2+json".to_string())
        );
        assert_eq!(m_v2.schema_version, 2);
        assert_eq!(
            m_v2.config.media_type,
            "application/vnd.docker.container.image.v1+json"
        );
        assert_eq!(m_v2.config.size.unwrap(), 1278);
        assert_eq!(
            m_v2.config.digest,
            "sha256:4a415e3663882fbc554ee830889c68a33b3585503892cc718a4698e91ef2a526"
        );
        assert_eq!(
            m_v2.layers[0].media_type,
            "application/vnd.docker.image.rootfs.diff.tar.gzip"
        );
        assert_eq!(m_v2.layers[0].size.unwrap(), 1967949);
        assert_eq!(
            m_v2.layers[0].digest,
            "sha256:1e76f742da490c8d7c921e811e5233def206e76683ee28d735397ec2231f131d"
        );

        assert_eq!(mani.get_local_asset_digests().len(), 2);
        assert!(mani
            .get_local_asset_digests()
            .contains(&"sha256:1e76f742da490c8d7c921e811e5233def206e76683ee28d735397ec2231f131d"));
        assert!(mani
            .get_local_asset_digests()
            .contains(&"sha256:4a415e3663882fbc554ee830889c68a33b3585503892cc718a4698e91ef2a526"));
    }

    #[test]
    fn valid_oci() {
        let config = "{}\n".as_bytes();
        let config_digest = sha256_tag_digest(BufReader::new(config)).unwrap();
        let data = format!(
            r#"{{ "config": {{ "digest": "{}",
                             "mediaType": "application/vnd.oci.image.config.v1+json",
                             "size": {} }},
                 "mediaType": "application/vnd.oci.image.manifest.v1+json",
                 "layers": [], "schemaVersion": 2 }}"#,
            config_digest,
            config.len()
        );

        let v: Value = serde_json::from_str(&data).unwrap();
        assert!(Manifest::from_json(&v).is_ok());
    }

    #[test]
    fn valid_manifest_list() {
        let data = r#"{
                    "schemaVersion": 2,
                    "mediaType": "application/vnd.docker.distribution.manifest.list.v2+json",
                    "manifests": [
                      {
                        "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
                        "size": 7143,
                        "digest": "sha256:e692418e4cbaf90ca69d05a66403747baa33ee08806650b51fab815ad7fc331f",
                        "platform": {
                          "architecture": "ppc64le",
                          "os": "linux"
                        }
                      },
                      {
                        "mediaType": "application/vnd.docker.distribution.manifest.v2+json",
                        "size": 7682,
                        "digest": "sha256:5b0bcabd1ed22e9fb1310cf6c2dec7cdef19f0ad69efa1f392e94a4333501270",
                        "platform": {
                          "architecture": "amd64",
                          "os": "linux",
                          "features": [
                            "sse4"
                          ]
                        }
                      }
                    ]
                  }
                  "#;

        let v: Value = serde_json::from_str(data).unwrap();
        assert!(Manifest::from_json(&v).is_ok());
    }
}
