#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate futures;
extern crate grpcio;
extern crate protobuf;
extern crate uuid;
#[macro_use]
extern crate serde_derive;
extern crate crypto;
extern crate rustc_serialize;
extern crate serde_json;

#[macro_use(log, warn, info, debug)]
extern crate log;
extern crate trow_protobuf;

pub mod manifest;
mod server;
mod validate;
use failure::Error;
use futures::Future;
use grpcio::{Environment, ServerBuilder};
use server::TrowService;
use std::thread;

pub struct TrowServerBuilder {
    data_path: String,
    listen_addr: String,
    listen_port: u16,
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
    listen_addr: &str,
    listen_port: u16,
    allow_prefixes: Vec<String>,
    allow_images: Vec<String>,
    deny_prefixes: Vec<String>,
    deny_images: Vec<String>,
) -> TrowServerBuilder {
    TrowServerBuilder {
        data_path: data_path.to_string(),
        listen_addr: listen_addr.to_string(),
        listen_port,
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

    pub fn start_sync(self) {
        match self.server_async() {
            Ok(mut server) => {
                thread::park();
                let _ = server.shutdown().wait();
                warn!("Trow Server shutdown!");
            }
            Err(e) => {
                eprintln!("Failed to start Trow server: {:?}", e);
                std::process::exit(1);
            }
        }
    }

    pub fn server_async(self) -> Result<grpcio::Server, Error> {
        use std::sync::Arc;

        debug!("Setting up Trow server");
        let env = Arc::new(Environment::new(1));

        //It's a bit weird but cloning the service should be fine
        //Internally it uses Arcs to point to the communal data
        let ts = TrowService::new(
            &self.data_path,
            self.allow_prefixes,
            self.allow_images,
            self.deny_prefixes,
            self.deny_images,
        )?;
        let registry_service = trow_protobuf::server_grpc::create_registry(ts.clone());
        let admission_service = trow_protobuf::server_grpc::create_admission_controller(ts);

        let sb = ServerBuilder::new(env)
            .register_service(registry_service)
            .register_service(admission_service);

        /*
        //Once we can use the secure feature, this will enable TLS certs

        let sb = if self.tls_cert.is_some() && self.tls_key.is_some() {
            let scb = ServerCredentialsBuilder::new()
                .add_cert(self.tls_cert.unwrap(), self.tls_key.unwrap());
            let scb = if let Some(key) = self.root_key {
                scb.root_cert(key, true)
            } else {
                scb
            };
            let sc = scb.build();
            sb.bind_secure(self.listen_addr, self.listen_port, sc)
        } else {
            sb.bind(self.listen_addr, self.listen_port)
        };
        */
        let sb = sb.bind(self.listen_addr, self.listen_port);

        let mut server = sb.build()?;
        server.start();

        for &(ref host, port) in server.bind_addrs() {
            info!("listening on {}:{}", host, port);
        }
        Ok(server)
    }
}
