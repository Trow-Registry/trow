//! This file should peferably be empty.
//! The use of this file is unclear, but it does at the very least
//! clean out other sections of code.
use std::net::ToSocketAddrs;
use std::io::{Error, ErrorKind};

use capnp::message::{Builder, HeapAllocator};
use capnp_rpc::{RpcSystem, twoparty, rpc_twoparty_capnp};
use http_capnp::lycaon;
use rocket;
use tokio_core::reactor;
use tokio_io::AsyncRead;
use futures::Future;

use config;

pub struct CapnpConnection {
    pub proxy: lycaon::layer_interface::Client,
    pub core: reactor::Core,
    pub builder: Builder<HeapAllocator>,
}

pub fn connect_backend(
    config: &rocket::State<config::Config>,
) -> Result<CapnpConnection, Error> {

    let address = format!("localhost:{}", config.console_port);
    let mut core = reactor::Core::new().unwrap();
    let handle = core.handle();

    let addr = address.to_socket_addrs().and_then(|mut addr| {
        let err = Err("could not parse address".to_string());
        // The below piece of code is actually handled by using
        // `.or_ok()`, but it is not a solution until I can find a
        // proper error handler.
        match addr.next() {
            Some(x) => Ok(x),
            // TODO: This is a hack and will actually cause the code to panic when trying to unwrap.
            // A proper fix needs to be done for this, but it does make the type-checker happy...
            // This is a duplicate of some code in the state/mod.rs file.
            None => Err(err.unwrap()),
        }
    });
    debug!("Connecting to address: {}", address);
    let stream = addr.and_then(|addr| {
        core.run(::tokio_core::net::TcpStream::connect(&addr, &handle))
    });

    if let Ok(stream) = stream {
        stream.set_nodelay(true).expect("could not set nodelay");
        let (reader, writer) = stream.split();

        let rpc_network = Box::new(twoparty::VatNetwork::new(
            reader,
            writer,
            rpc_twoparty_capnp::Side::Client,
            Default::default(),
        ));

        let mut rpc_system = RpcSystem::new(rpc_network, None);
        let lycaon_proxy: lycaon::Client = rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);
        let interface = lycaon_proxy.get_layer_interface_request().send();
        let proxy = interface.pipeline.get_if();


        handle.spawn(rpc_system.map_err(|_e| ()));

        let mut builder = ::capnp::message::Builder::new(::capnp::message::HeapAllocator::new());
        Ok(CapnpConnection {
            proxy,
            core,
            builder,
        })
    } else {
        Err(Error::new(ErrorKind::Other, "could not connect to Backend"))
    }

}
