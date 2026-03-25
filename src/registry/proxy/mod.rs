mod proxy_config;
mod remote_image;

pub use proxy_config::{RegistryProxiesConfig, SingleRegistryProxyConfig, DownloadRemoteImageError};
pub use remote_image::RemoteImage;
