#[macro_use] extern crate failure;
#[macro_use] extern crate failure_derive;
extern crate futures;
extern crate grpcio;
extern crate protobuf;
extern crate uuid;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate rustc_serialize;
extern crate crypto;

#[macro_use(log, warn, info, debug)]
extern crate log;
extern crate trow_protobuf;

mod server;
pub mod manifest;
use std::thread;
use server::TrowService;
use futures::Future;
use grpcio::{Environment, ServerBuilder};

pub fn start_server(_data_path: &str, listen_addr: &str, listen_port: u16) {
    let mut server = server_async(listen_addr, listen_port);
    thread::park();
    let _ = server.shutdown().wait();
    warn!("Trow Server shutdown!");
}

pub fn server_async(
    listen_addr: &str,
    listen_port: u16,
) -> grpcio::Server {
    use std::sync::Arc;

    debug!("Setting up backend server");
    let env = Arc::new(Environment::new(1));
    let trow_service = trow_protobuf::server_grpc::create_backend(TrowService::new());

    let mut server = ServerBuilder::new(env)
        .register_service(trow_service)
        .bind(listen_addr, listen_port)
        .build()
        .unwrap();
    server.start();
    for &(ref host, port) in server.bind_addrs() {
        info!("listening on {}:{}", host, port);
    }
    server
}

