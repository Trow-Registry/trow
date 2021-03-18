use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};

// We want to use enums in the manifest and we want to serialize and deserialize them to and from strings
// Layer
const LAYER_TAR_STR: &str = "application/vnd.oci.image.layer.v1.tar";
const LAYER_TAR_GZ_STR: &str = "application/vnd.oci.image.layer.v1.tar+gzip";
const LAYER_NON_DIST_TAR_STR: &str = "application/vnd.oci.image.layer.nondistributable.v1.tar";
const LAYER_NON_DIST_TAR_GZ_STR: &str =
    "application/vnd.oci.image.layer.nondistributable.v1.tar+gzip";

// Manifest
const MANIFEST_STR: &str = "application/vnd.oci.image.manifest.v1+json";
const MANIFEST_CONFIG_STR: &str = "application/vnd.oci.image.config.v1+json";
const MANIFEST_CONFIG_UNKNOWN_STR: &str = "application/vnd.unknown.config.v1+json";
const MANIFEST_VULNERABILITIES_REPORT_STR: &str = "application/vnd.oci.vuln.report.v1+json";
// Index
const MANIFEST_INDEX_STR: &str = "application/vnd.oci.image.index.v1+json";

// Docker
const MANIFEST_DOCKER_V1_STR: &str = "application/vnd.docker.distribution.manifest.v1+prettyjws";
const MANIFEST_DOCKER_V2_STR: &str = "application/vnd.docker.distribution.manifest.v2+json";
const MANIFEST_DOCKER_CONFIG_STR: &str = "application/vnd.docker.container.image.v1+json";
const LAYER_DOCKER_TAR_GZ_STR: &str = "application/vnd.docker.image.rootfs.diff.tar.gzip";

// Definition of all the MimeTypes
// If you need more mime types add them here
#[derive(Debug, Clone, PartialEq)]
pub enum MediaType {
    LayerTar, // application/vnd.oci.image.layer.v1.tar

    LayerTarGz, // application/vnd.oci.image.layer.v1.tar+gzip

    LayerNonDistTar, // application/vnd.oci.image.layer.nondistributable.v1.tar

    LayerNonDistTarGz, // application/vnd.oci.image.layer.nondistributable.v1.tar+gzip

    Manifest, // "application/vnd.oci.image.manifest.v1+json";

    ManifestConfig, // "application/vnd.oci.image.config.v1+json"

    ManifestConfigUnknown, // vnd.unknown.config.v1+json

    ManifestIndex, // application/vnd.oci.image.index.v1+json

    ManifestVulnerabilitiesReport, // application/vnd.oci.vuln.report.v1+json

    // Docker media types
    ManifestDockerV1, // application/vnd.docker.distribution.manifest.v1+prettyjws

    ManifestDockerV2, // application/vnd.docker.distribution.manifest.v2+json

    ManifestDockerConfig,

    LayerDockerTarGz,
}

// Implemented custom deserializer from string to Enum
impl<'de> Deserialize<'de> for MediaType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?.to_lowercase();
        let state = match s.as_str() {
            MANIFEST_DOCKER_V1_STR => MediaType::ManifestDockerV1,
            MANIFEST_DOCKER_V2_STR => MediaType::ManifestDockerV2,
            MANIFEST_DOCKER_CONFIG_STR => MediaType::ManifestDockerConfig,
            LAYER_DOCKER_TAR_GZ_STR => MediaType::LayerDockerTarGz,
            LAYER_TAR_STR => MediaType::LayerTar,
            LAYER_TAR_GZ_STR => MediaType::LayerTarGz,
            LAYER_NON_DIST_TAR_STR => MediaType::LayerNonDistTar,
            LAYER_NON_DIST_TAR_GZ_STR => MediaType::LayerNonDistTarGz,
            MANIFEST_STR => MediaType::Manifest,
            MANIFEST_CONFIG_STR => MediaType::ManifestConfig,
            MANIFEST_CONFIG_UNKNOWN_STR => MediaType::ManifestConfigUnknown,
            MANIFEST_VULNERABILITIES_REPORT_STR => MediaType::ManifestVulnerabilitiesReport,
            MANIFEST_INDEX_STR => MediaType::ManifestIndex,
            other => {
                return Err(D::Error::custom(format!("Invalid MediaType '{}'", other)));
            }
        };
        Ok(state)
    }
}

// Implemented custom serializer from Enum to String
impl Serialize for MediaType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match *self {
            MediaType::ManifestDockerV1 => MANIFEST_DOCKER_V1_STR,
            MediaType::ManifestDockerV2 => MANIFEST_DOCKER_V2_STR,
            MediaType::ManifestDockerConfig => MANIFEST_DOCKER_CONFIG_STR,
            MediaType::LayerDockerTarGz => LAYER_DOCKER_TAR_GZ_STR,
            MediaType::LayerTar => LAYER_TAR_STR,
            MediaType::LayerTarGz => LAYER_TAR_GZ_STR,
            MediaType::LayerNonDistTar => LAYER_NON_DIST_TAR_STR,
            MediaType::LayerNonDistTarGz => LAYER_NON_DIST_TAR_GZ_STR,
            MediaType::Manifest => MANIFEST_STR,
            MediaType::ManifestConfig => MANIFEST_CONFIG_STR,
            MediaType::ManifestConfigUnknown => MANIFEST_CONFIG_UNKNOWN_STR,
            MediaType::ManifestVulnerabilitiesReport => MANIFEST_VULNERABILITIES_REPORT_STR,
            MediaType::ManifestIndex => MANIFEST_INDEX_STR,
        })
    }
}

impl std::fmt::Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaType::ManifestDockerV1 => write!(f, "{}", MANIFEST_DOCKER_V1_STR),
            MediaType::ManifestDockerV2 => write!(f, "{}", MANIFEST_DOCKER_V2_STR),
            MediaType::ManifestDockerConfig => write!(f, "{}", MANIFEST_DOCKER_CONFIG_STR),
            MediaType::LayerDockerTarGz => write!(f, "{}", LAYER_DOCKER_TAR_GZ_STR),
            MediaType::LayerTar => write!(f, "{}", LAYER_TAR_STR),
            MediaType::LayerTarGz => write!(f, "{}", LAYER_TAR_GZ_STR),
            MediaType::LayerNonDistTar => write!(f, "{}", LAYER_NON_DIST_TAR_STR),
            MediaType::LayerNonDistTarGz => write!(f, "{}", LAYER_NON_DIST_TAR_GZ_STR),
            MediaType::Manifest => write!(f, "{}", MANIFEST_STR),
            MediaType::ManifestConfig => write!(f, "{}", MANIFEST_CONFIG_STR),
            MediaType::ManifestConfigUnknown => write!(f, "{}", MANIFEST_CONFIG_UNKNOWN_STR),
            MediaType::ManifestVulnerabilitiesReport => {
                write!(f, "{}", MANIFEST_VULNERABILITIES_REPORT_STR)
            }
            MediaType::ManifestIndex => write!(f, "{}", MANIFEST_INDEX_STR),
        }
    }
}

impl std::str::FromStr for MediaType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            MANIFEST_DOCKER_V1_STR => Ok(MediaType::ManifestDockerV1),
            MANIFEST_DOCKER_V2_STR => Ok(MediaType::ManifestDockerV2),
            MANIFEST_DOCKER_CONFIG_STR => Ok(MediaType::ManifestDockerConfig),
            LAYER_DOCKER_TAR_GZ_STR => Ok(MediaType::LayerDockerTarGz),
            LAYER_TAR_STR => Ok(MediaType::LayerTar),
            LAYER_TAR_GZ_STR => Ok(MediaType::LayerTarGz),
            LAYER_NON_DIST_TAR_STR => Ok(MediaType::LayerNonDistTar),
            LAYER_NON_DIST_TAR_GZ_STR => Ok(MediaType::LayerNonDistTarGz),
            MANIFEST_STR => Ok(MediaType::Manifest),
            MANIFEST_CONFIG_STR => Ok(MediaType::ManifestConfig),
            MANIFEST_CONFIG_UNKNOWN_STR => Ok(MediaType::ManifestConfigUnknown),
            MANIFEST_VULNERABILITIES_REPORT_STR => Ok(MediaType::ManifestVulnerabilitiesReport),
            MANIFEST_INDEX_STR => Ok(MediaType::ManifestIndex),
            _ => Err(format!(
                "'{}' is not a valid or supported MediaType Content-Type header",
                s
            )),
        }
    }
}

/// Byb default we set the MediaType to Manifest with is the OCI manifest
impl Default for MediaType {
    fn default() -> Self {
        MediaType::Manifest
    }
}
