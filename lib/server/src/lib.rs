mod digest;

#[macro_use(warn, debug, info, error)]
extern crate log;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate failure_derive;
#[macro_use]
extern crate failure;
extern crate chrono;

// crypto and crypto related crates
extern crate sha2;
extern crate hex;

use tonic::transport::Server;
mod server;
mod validate;
use server::trow_server::registry_server::RegistryServer;
use server::trow_server::admission_controller_server::AdmissionControllerServer;
use server::TrowServer;
use tokio::runtime::Runtime;

pub mod manifest;

pub struct TrowServerBuilder {
    data_path: String,
    listen_addr: std::net::SocketAddr,
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
    allow_prefixes: Vec<String>,
    allow_images: Vec<String>,
    deny_prefixes: Vec<String>,
    deny_images: Vec<String>,
) -> TrowServerBuilder {
    TrowServerBuilder {
        data_path: data_path.to_string(),
        listen_addr,
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

    pub fn start_trow_sync(self) -> () {

        let mut rt = Runtime::new().expect("Failed to start Tokio runtime");
        let ts = TrowServer::new(
            &self.data_path,
            self.allow_prefixes,
            self.allow_images,
            self.deny_prefixes,
            self.deny_images).expect("Failure configuring Trow Server");

        let server = Server::builder()
            .add_service(RegistryServer::new(ts.clone()))
            .add_service(AdmissionControllerServer::new(ts))
            .serve(self.listen_addr);

        debug!("Trow backend service running");

        match rt.block_on(server)
        {
            Ok(()) => {
                warn!("Trow backend shutting down");
            }
            Err(e) => {
                eprintln!("Failure in Trow server: {:?}", e);
                std::process::exit(1);
            }
        }
    }
}
