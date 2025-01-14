mod proxy_config;
mod remote_image;

use axum::http::HeaderValue;
use hyper::HeaderMap;
pub use proxy_config::{RegistryProxiesConfig, SingleRegistryProxyConfig};
pub use remote_image::RemoteImage;

use crate::registry::manifest::manifest_media_type;

pub fn create_accept_header() -> HeaderMap {
    const ACCEPT: [&str; 4] = [
        manifest_media_type::OCI_V1,
        manifest_media_type::DOCKER_V2,
        manifest_media_type::DOCKER_LIST,
        manifest_media_type::OCI_INDEX,
    ];

    let mut headers = HeaderMap::new();
    headers.insert(
        reqwest::header::ACCEPT,
        HeaderValue::from_str(&ACCEPT.join(", ")).unwrap(),
    );
    headers
}
