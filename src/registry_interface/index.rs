use crate::registry_interface::media::MediaType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct Index {
    #[serde(default)]
    #[serde(rename = "schemaVersion")]
    pub schema_version: i8,

    #[serde(default)]
    #[serde(rename = "manifests")]
    pub manifests: Vec<IndexManifest>,

    #[serde(default)]
    #[serde(rename = "annotations")]
    pub annotations: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IndexManifest {
    #[serde(rename = "mediaType")]
    pub media_type: MediaType,

    #[serde(default)]
    #[serde(rename = "size")]
    pub size: u64,

    #[serde(default)]
    #[serde(rename = "digest")]
    pub digest: String,

    #[serde(default)]
    #[serde(rename = "urls")]
    pub urls: Vec<String>,

    #[serde(rename = "platform")]
    pub platform: IndexPlatform,

    #[serde(default)]
    #[serde(rename = "annotations")]
    pub annotations: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IndexPlatform {
    #[serde(default)]
    #[serde(rename = "architecture")]
    pub architecture: String,

    #[serde(default)]
    #[serde(rename = "os")]
    pub os: String,

    #[serde(default)]
    #[serde(rename = "os.version")]
    pub os_version: String,

    #[serde(default)]
    #[serde(rename = "os.features")]
    pub os_features: Vec<String>,

    #[serde(default)]
    #[serde(rename = "variant")]
    pub variant: String,
}

impl Default for IndexPlatform {
    fn default() -> Self {
        IndexPlatform {
            architecture: "".to_string(),
            os: "".to_string(),
            os_version: "".to_string(),
            os_features: vec![],
            variant: "".to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::registry_interface::index::Index;
    use crate::registry_interface::media::MediaType;
    use crate::serde_json::Result;

    #[test]
    fn index_basic() {
        // Parse a manifest JSON into a manifest
        let manifest_json = r#"
        {
  "schemaVersion": 2,
  "manifests": [
    {
      "mediaType": "application/vnd.oci.image.manifest.v1+json",
      "size": 7143,
      "digest": "sha256:e692418e4cbaf90ca69d05a66403747baa33ee08806650b51fab815ad7fc331f",
      "platform": {
        "architecture": "ppc64le",
        "os": "linux"
      }
    },
    {
      "mediaType": "application/vnd.oci.image.manifest.v1+json",
      "size": 7682,
      "digest": "sha256:5b0bcabd1ed22e9fb1310cf6c2dec7cdef19f0ad69efa1f392e94a4333501270",
      "platform": {
        "architecture": "amd64",
        "os": "linux"
      }
    }
  ],
  "annotations": {
    "com.example.key1": "value1",
    "com.example.key2": "value2"
  }
}"#;

        // Parse the string of data into serde_json::Value.
        let m: Result<Index> = crate::serde_json::from_str(manifest_json);
        assert!(m.is_ok());

        let m = m.unwrap();

        assert_eq!(2, m.schema_version);
        assert_eq!(2, m.annotations.len());
        assert_eq!(2, m.manifests.len());

        let manifest = m.manifests[0].clone();
        assert_eq!(7143, manifest.size);
        assert_eq!(MediaType::Manifest, manifest.media_type);
        assert_eq!(
            "sha256:e692418e4cbaf90ca69d05a66403747baa33ee08806650b51fab815ad7fc331f",
            manifest.digest
        );
        assert_eq!("linux", manifest.platform.os);
        assert_eq!("ppc64le", manifest.platform.architecture);

        let manifest = m.manifests[1].clone();
        assert_eq!(7682, manifest.size);
        assert_eq!(MediaType::Manifest, manifest.media_type);
        assert_eq!(
            "sha256:5b0bcabd1ed22e9fb1310cf6c2dec7cdef19f0ad69efa1f392e94a4333501270",
            manifest.digest
        );
        assert_eq!("linux", manifest.platform.os);
        assert_eq!("amd64", manifest.platform.architecture);
    }
}
