//! This file should peferably be empty.
//! The use of this file is unclear, but it does at the very least
//! clean out other sections of code.
use std::net::ToSocketAddrs;

use capnp::message::{Builder, HeapAllocator};
use capnp_rpc::{RpcSystem, twoparty, rpc_twoparty_capnp};
use failure::Error;
use http_capnp::lycaon;
use rocket;
use tokio_core::reactor;
use tokio_io::AsyncRead;
use futures::Future;

use config;
use errors;

pub struct CapnpConnection {
    pub proxy: lycaon::layer_interface::Client,
    pub core: Result<reactor::Core, Error>,
    pub builder: Builder<HeapAllocator>,
}

pub fn connect_backend(config: &rocket::State<config::Config>) -> Result<CapnpConnection, Error> {

    let mut core = reactor::Core::new()?;
    let handle = core.handle();

    format!("localhost:{}", config.console_port)
        .to_socket_addrs()
        .map_err(|e| Error::from(e))
        .and_then(|mut addr| {
            addr.next()
                .ok_or(errors::Server::Invalid("address format").into())
        })
        .and_then(|addr| {
            debug!("Connecting to address: {}", addr);
            core.run(::tokio_core::net::TcpStream::connect(&addr, &handle))
                .map_err(|e| e.into())
        })
        .and_then(|stream| {
            stream.set_nodelay(true)
                .map_err(|e| e.into())
                .map(|_| {
                let (reader, writer) = stream.split();
                Box::new(twoparty::VatNetwork::new(
                    reader,
                    writer,
                    rpc_twoparty_capnp::Side::Client,
                    Default::default(),
                ))
            })
        })
        .map(|rpc_network| {
            let mut rpc_system = RpcSystem::new(rpc_network, None);

            let lycaon_proxy: lycaon::Client =
                rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);
            let interface = lycaon_proxy.get_layer_interface_request().send();
            let proxy = interface.pipeline.get_if();

            handle.spawn(rpc_system.map_err(|_| ()));

            let builder = ::capnp::message::Builder::new(::capnp::message::HeapAllocator::new());
            CapnpConnection {
                proxy,
                core: Ok(core),
                builder,
            }
        })
}
