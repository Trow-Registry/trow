#[macro_use(log, warn)]
extern crate log;

use tonic::transport::Server;
mod server;
use server::trow_server::server::RegistryServer;
use server::TrowServer;

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

    pub async fn start_trow_sync(self) -> Result<(), Box<dyn std::error::Error>> {
        
        let ts = TrowServer {};

        match Server::builder()
            .add_service(RegistryServer::new(ts))
            .serve(self.listen_addr)
            .await
        {
            Ok(()) => {
                warn!("Server shutting down");
            }
            Err(e) => {
                eprintln!("Failure in Trow server: {:?}", e);
                std::process::exit(1);
            }
        }

        Ok(())
    }
}
