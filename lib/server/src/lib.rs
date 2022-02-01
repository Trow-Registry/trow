pub mod digest;

use tonic::transport::Server;
mod metrics;
mod server;
mod temporary_file;
mod validate;
use log::{debug, warn};
use server::trow_server::admission_controller_server::AdmissionControllerServer;
use server::trow_server::registry_server::RegistryServer;
use server::TrowServer;
use std::future::Future;
use tokio::runtime::Runtime;

pub mod manifest;

pub struct TrowServerBuilder {
    data_path: String,
    listen_addr: std::net::SocketAddr,
    proxy_hub: bool,
    hub_user: Option<String>,
    hub_pass: Option<String>,
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
    proxy_hub: bool,
    hub_user: Option<String>,
    hub_pass: Option<String>,
    allow_prefixes: Vec<String>,
    allow_images: Vec<String>,
    deny_prefixes: Vec<String>,
    deny_images: Vec<String>,
) -> TrowServerBuilder {
    TrowServerBuilder {
        data_path: data_path.to_string(),
        listen_addr,
        proxy_hub,
        hub_user,
        hub_pass,
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
            self.proxy_hub,
            self.hub_user,
            self.hub_pass,
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
