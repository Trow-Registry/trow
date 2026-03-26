use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ImageValidationConfig {
    pub default: String,
    pub allow: Vec<String>,
    pub deny: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ConfigFile {
    #[serde(deserialize_with = "de_unwrap_or_default")]
    pub registry_proxies: RegistryProxiesConfig,
    pub image_validation: Option<ImageValidationConfig>,
}

fn de_unwrap_or_default<'de, T, D>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or_default())
}

#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct RegistryProxyConfigs(Vec<SingleRegistryProxyConfig>);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegistryProxiesConfig {
    #[serde(default)]
    pub registries: RegistryProxyConfigs,
    #[serde(default)]
    pub offline: bool,
    #[serde(default)]
    pub max_size: Option<size::Size>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct SingleRegistryProxyConfig {
    /// What containerd calls "namespace" (ghcr.io, docker.io, ...)
    /// This can be empty !!
    pub host: String,
    /// TODO: insecure currently means "use HTTP", we should also support self-signed TLS
    #[serde(default)]
    pub insecure: bool,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Default for RegistryProxiesConfig {
    fn default() -> Self {
        RegistryProxiesConfig {
            registries: RegistryProxyConfigs(Vec::new()),
            offline: true,
            max_size: None,
        }
    }
}

impl RegistryProxyConfigs {
    pub fn get_for<'a>(&'a self, registry_host: &str) -> Option<&'a SingleRegistryProxyConfig> {
        self.0
            .iter()
            .find(|&registry| registry.host == registry_host)
            .map(|v| v as _)
    }
}

impl From<Vec<SingleRegistryProxyConfig>> for RegistryProxyConfigs {
    fn from(vec: Vec<SingleRegistryProxyConfig>) -> Self {
        RegistryProxyConfigs(vec)
    }
}
