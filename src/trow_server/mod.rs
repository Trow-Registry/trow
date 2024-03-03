mod admission;
pub mod api_types;
pub mod digest;
mod image;
pub mod manifest;
mod metrics;
mod proxy_auth;
mod server;
mod storage;
mod temporary_file;

pub use admission::ImageValidationConfig;
use anyhow::Result;
pub use proxy_auth::{RegistryProxiesConfig, SingleRegistryProxyConfig};
pub use server::TrowServer;

pub struct TrowServerBuilder {
    data_path: String,
    proxy_registry_config: Option<RegistryProxiesConfig>,
    image_validation_config: Option<ImageValidationConfig>,
}

pub fn build_server(
    data_path: &str,
    proxy_registry_config: Option<RegistryProxiesConfig>,
    image_validation_config: Option<ImageValidationConfig>,
) -> TrowServerBuilder {
    TrowServerBuilder {
        data_path: data_path.to_string(),
        proxy_registry_config,
        image_validation_config,
    }
}

impl TrowServerBuilder {
    pub fn get_server(self) -> Result<TrowServer> {
        TrowServer::new(
            &self.data_path,
            self.proxy_registry_config,
            self.image_validation_config,
        )
    }
}
