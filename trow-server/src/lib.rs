mod admission;
pub mod digest;
mod image;
pub mod manifest;
mod metrics;
mod proxy_auth;
mod server;
mod temporary_file;

use std::future::Future;

pub use admission::ImageValidationConfig;
pub use proxy_auth::{RegistryProxiesConfig, SingleRegistryProxyConfig};
use server::trow_server::admission_controller_server::AdmissionControllerServer;
use server::trow_server::registry_server::RegistryServer;
use server::TrowServer;
use tonic::transport::Server;

pub struct TrowServerBuilder {
    data_path: String,
    listen_addr: std::net::SocketAddr,
    proxy_registry_config: Option<RegistryProxiesConfig>,
    image_validation_config: Option<ImageValidationConfig>,
    tls_cert: Option<Vec<u8>>,
    tls_key: Option<Vec<u8>>,
    root_key: Option<Vec<u8>>,
}

pub fn build_server(
    data_path: &str,
    listen_addr: std::net::SocketAddr,
    proxy_registry_config: Option<RegistryProxiesConfig>,
    image_validation_config: Option<ImageValidationConfig>,
) -> TrowServerBuilder {
    TrowServerBuilder {
        data_path: data_path.to_string(),
        listen_addr,
        proxy_registry_config,
        image_validation_config,
        tls_cert: None,
        tls_key: None,
        root_key: None,
    }
}

impl TrowServerBuilder {
    pub fn add_tls(mut self, tls_cert: Vec<u8>, tls_key: Vec<u8>) -> TrowServerBuilder {
        self.tls_cert = Some(tls_cert);
        self.tls_key = Some(tls_key);
        self
    }

    pub fn add_root_cert(mut self, root_key: Vec<u8>) -> TrowServerBuilder {
        self.root_key = Some(root_key);
        self
    }

    pub fn get_server_future(self) -> impl Future<Output = Result<(), tonic::transport::Error>> {
        let ts = TrowServer::new(
            &self.data_path,
            self.proxy_registry_config,
            self.image_validation_config,
        )
        .expect("Failure configuring Trow Server");

        Server::builder()
            .add_service(RegistryServer::new(ts.clone()))
            .add_service(AdmissionControllerServer::new(ts))
            .serve(self.listen_addr)
    }
}
