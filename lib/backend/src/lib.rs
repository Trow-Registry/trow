extern crate failure;
extern crate futures;
extern crate grpcio;
extern crate protobuf;
extern crate uuid;

#[macro_use(log, warn, info, debug)]
extern crate log;
extern crate trow_protobuf;

mod peer;
mod backend;
use std::thread;
use peer::PeerService;
use backend::BackendService;
use futures::Future;
use grpcio::{Environment, ServerBuilder};

pub fn server(listen_addr: &str, listen_port: u16, bootstrap_addr: &str, bootstrap_port: u16) {
    let mut server = server_async(listen_addr, listen_port, bootstrap_addr, bootstrap_port);
    thread::park();
    let _ = server.shutdown().wait();
    warn!("GRPC Server shutdown!");
}

pub fn server_async(
    listen_addr: &str,
    listen_port: u16,
    bootstrap_addr: &str,
    bootstrap_port: u16,
) -> grpcio::Server {
    use std::sync::Arc;

    debug!("Setting up backend server");
    let env = Arc::new(Environment::new(1));
    let backend_service = trow_protobuf::backend_grpc::create_backend(BackendService::new());
    let peer_service =
        trow_protobuf::peer_grpc::create_peer(PeerService::new(bootstrap_addr, bootstrap_port));
    let mut server = ServerBuilder::new(env)
        .register_service(peer_service)
        .register_service(backend_service)
        .bind(listen_addr, listen_port)
        .build()
        .unwrap();
    server.start();
    for &(ref host, port) in server.bind_addrs() {
        info!("listening on {}:{}", host, port);
    }
    server
}

