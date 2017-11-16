//! This file should peferably be empty.
//! The use of this file is unclear, but it does at the very least
//! clean out other sections of code.
use std::net::ToSocketAddrs;

use capnp;
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
    pub proxy: Result<CapnpInterface, Error>,
    pub core: Result<reactor::Core, Error>,
    pub builder: Builder<HeapAllocator>,
}

pub enum CapnpInterface {
    Layer(lycaon::layer_interface::Client),
    Uuid(lycaon::uuid_interface::Client),
}

fn connect_backend(
    config: &rocket::State<config::Config>,
) -> Result<(lycaon::Client, reactor::Core), Error> {

    let mut core = reactor::Core::new()?;
    let handle = core.handle();

    let mut addr = format!("localhost:{}", config.console_port)
        .to_socket_addrs()?;
    let addr = addr.next().ok_or(errors::Server::Invalid("address format"))?;

    debug!("Connecting to address: {}", addr);
    let stream = core.run(
        ::tokio_core::net::TcpStream::connect(&addr, &handle),
    )?;
    let rpc_network = stream.set_nodelay(true).map(|_| {
        let (reader, writer) = stream.split();
        Box::new(twoparty::VatNetwork::new(
            reader,
            writer,
            rpc_twoparty_capnp::Side::Client,
            Default::default(),
        ))
    })?;
    let mut rpc_system = RpcSystem::new(rpc_network, None);
    let lycaon_proxy: lycaon::Client = rpc_system.bootstrap(rpc_twoparty_capnp::Side::Server);

    handle.spawn(rpc_system.map_err(|_| ()));

    Ok((lycaon_proxy, core))
}

fn builder() -> capnp::message::Builder<::capnp::message::HeapAllocator> {
    ::capnp::message::Builder::new(::capnp::message::HeapAllocator::new())
}

impl CapnpInterface {
    pub fn layer_interface(
        config: &rocket::State<config::Config>,
    ) -> Result<CapnpConnection, Error> {
        let (lycaon_proxy, core) = connect_backend(config)?;
        let interface = lycaon_proxy.get_layer_interface_request().send();
        let proxy = interface.pipeline.get_if();
        let proxy = CapnpInterface::Layer(proxy);
        Ok(CapnpConnection {
            proxy: Ok(proxy),
            core: Ok(core),
            builder: builder(),
        })
    }

    pub fn uuid_interface(
        config: &rocket::State<config::Config>,
    ) -> Result<CapnpConnection, Error> {
        let (lycaon_proxy, core) = connect_backend(config)?;
        let interface = lycaon_proxy.get_uuid_interface_request().send();
        let proxy = interface.pipeline.get_if();
        let proxy = CapnpInterface::Uuid(proxy);
        Ok(CapnpConnection {
            proxy: Ok(proxy),
            core: Ok(core),
            builder: builder(),
        })
    }
}
