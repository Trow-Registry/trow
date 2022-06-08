pub mod digest;
pub mod manifest;
mod metrics;
mod server;
mod temporary_file;
mod validate;

use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::future::Future;
use tokio::runtime::Runtime;
use tonic::transport::Server;

use server::trow_server::admission_controller_server::AdmissionControllerServer;
use server::trow_server::registry_server::RegistryServer;
use server::TrowServer;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegistryProxyConfig {
    pub alias: String,
    pub host: String,
    username: Option<String>,
    password: Option<String>,
}

pub struct TrowServerBuilder {
    data_path: String,
    listen_addr: std::net::SocketAddr,
    proxy_registry_config: Vec<RegistryProxyConfig>,
    allow_prefixes: Vec<String>,
    allow_images: Vec<String>,
    deny_prefixes: Vec<String>,
    deny_images: Vec<String>,
    tls_cert: Option<Vec<u8>>,
    tls_key: Option<Vec<u8>>,
    root_key: Option<Vec<u8>>,
}

pub fn build_server(
    data_path: &str,
    listen_addr: std::net::SocketAddr,
    proxy_registry_config: Vec<RegistryProxyConfig>,
    allow_prefixes: Vec<String>,
    allow_images: Vec<String>,
    deny_prefixes: Vec<String>,
    deny_images: Vec<String>,
) -> TrowServerBuilder {
    TrowServerBuilder {
        data_path: data_path.to_string(),
        listen_addr,
        proxy_registry_config,
        allow_prefixes,
        allow_images,
        deny_prefixes,
        deny_images,
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

    pub fn start_trow_sync(self) {
        let server = self.get_server_future();
        let rt = Runtime::new().expect("Failed to start Tokio runtime");

        debug!("Trow backend service running");

        match rt.block_on(server) {
            Ok(()) => {
                warn!("Trow backend shutting down");
            }
            Err(e) => {
                eprintln!("Failure in Trow server: {:?}", e);
                std::process::exit(1);
            }
        }
    }

    pub fn get_server_future(self) -> impl Future<Output = Result<(), tonic::transport::Error>> {
        let ts = TrowServer::new(
            &self.data_path,
            self.proxy_registry_config,
            self.allow_prefixes,
            self.allow_images,
            self.deny_prefixes,
            self.deny_images,
        )
        .expect("Failure configuring Trow Server");

        let future = Server::builder()
            .add_service(RegistryServer::new(ts.clone()))
            .add_service(AdmissionControllerServer::new(ts))
            .serve(self.listen_addr);
        future
    }
}
