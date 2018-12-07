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

pub fn start_server(data_path: &str, listen_addr: &str, listen_port: u16) {
    match server_async(data_path, listen_addr, listen_port) {
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

pub fn server_async(
    data_path: &str,
    listen_addr: &str,
    listen_port: u16,
) -> Result<grpcio::Server, Error> {
    use std::sync::Arc;

    debug!("Setting up Trow server");
    let env = Arc::new(Environment::new(1));

    //It's a bit weird but cloning the service should be fine
    //Internally it uses Arcs to point to the communal data
    let ts = TrowService::new(data_path)?;
    let registry_service = trow_protobuf::server_grpc::create_registry(ts.clone());
    let admission_service = trow_protobuf::server_grpc::create_admission_controller(ts);

    let mut server = ServerBuilder::new(env)
        .register_service(registry_service)
        .register_service(admission_service)
        .bind(listen_addr, listen_port)
        .build()?;
    server.start();
    for &(ref host, port) in server.bind_addrs() {
        info!("listening on {}:{}", host, port);
    }
    Ok(server)
}
