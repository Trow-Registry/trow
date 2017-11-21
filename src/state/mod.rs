use failure;
use capnp_rpc::{RpcSystem, twoparty, rpc_twoparty_capnp};
use capnp::capability::Promise;
use capnp::Error;
use futures::{Future, Stream};
use http_capnp::lycaon;
use orset::ORSet;
use rocket;
use tokio_core::reactor;
use tokio_io::AsyncRead;

use config;

/// Export Module layers
mod layers;
pub(crate) mod uuid;

type LayerImplType = lycaon::layer_interface::Client;
type UuidImplType = lycaon::uuid_interface::Client;
struct LycaonRPC {
    layerimpl: LayerImplType,
    uuidimpl: UuidImplType,
}
impl LycaonRPC {
    fn new(layerimpl: LayerImplType, uuidimpl: UuidImplType) -> LycaonRPC {
        LycaonRPC {
            layerimpl,
            uuidimpl,
        }
    }
}

impl lycaon::Server for LycaonRPC {
    fn get_layer_interface(
        &mut self,
        _params: lycaon::GetLayerInterfaceParams,
        mut results: lycaon::GetLayerInterfaceResults,
    ) -> Promise<(), Error> {
        debug!("returning the layer interface");
        results.get().set_if(self.layerimpl.clone());
        Promise::ok(())
    }
    fn get_uuid_interface(
        &mut self,
        _params: lycaon::GetUuidInterfaceParams,
        mut results: lycaon::GetUuidInterfaceResults,
    ) -> Promise<(), Error> {
        debug!("returning the layer interface");
        results.get().set_if(self.uuidimpl.clone());
        Promise::ok(())
    }
}

// TODO: merge this into the Config struct in config.rs
pub struct ConsoleConfig {
    pub console_port: i64,
}
impl ConsoleConfig {
    fn default() -> ConsoleConfig {
        ConsoleConfig { console_port: 29999 }
    }
}

fn get_config() -> ConsoleConfig {
    let rkt = rocket::Rocket::ignite();
    let cfg = rkt.config();

    ConsoleConfig {
        // TODO: This is currently duplicated in the config.rs file (where it should be).
        console_port: match cfg.get_int("console_port") {

            Ok(x) => x,
            Err(_) => ConsoleConfig::default().console_port,
        },
    }
}

pub fn main(handler: config::SocketHandler) -> Result<(), failure::Error> {

    // TODO: Strip out the rest of this functionality
    use std::thread;
    thread::spawn(move || loop {
        debug!("Listening...");

        let _ = handler
            .rx()
            .recv()
            .map_err(|e| failure::Error::from(e))
            .and_then(|val| {
                let req = config::BackendMessage::Frontend(config::Frontend::TestResponse);
                debug!("{:?}", val);
                handler.tx().send(req).map_err(|e| failure::Error::from(e))
            })
            .map_err(|e| {
                warn!("{}", e);
                e
            });
    });

    let cfg = get_config();
    use std::net::ToSocketAddrs;

    let address = format!("localhost:{}", cfg.console_port);
    reactor::Core::new().and_then(move |mut core| {
        let handle = core.handle();

        let addr = address.to_socket_addrs().and_then(|mut addr| {
            let err = Err("could not parse address".to_string());

            match addr.next() {
                Some(x) => Ok(x),
                // TODO: This is a hack and will actually cause the code to panic when trying to unwrap.
                // A proper fix needs to be done for this, but it does make the type-checker happy...
                None => Err(err.unwrap()),
            }
        });

        let socket = addr.and_then(|addr| ::tokio_core::net::TcpListener::bind(&addr, &handle))
            .expect("could not bind socket to address");

        let layers = ORSet::new("layers".to_string());
        let layerimpl = layers::LayerImpl::new(layers);
        let layerimpl = lycaon::layer_interface::ToClient::new(layerimpl)
            .from_server::<::capnp_rpc::Server>();

        let uuidimpl = uuid::UuidImpl::new();
        let uuidimpl = lycaon::uuid_interface::ToClient::new(uuidimpl)
            .from_server::<::capnp_rpc::Server>();

        let proxy = lycaon::ToClient::new(LycaonRPC::new(layerimpl, uuidimpl)).from_server::<::capnp_rpc::Server>();

        let handle1 = handle.clone();
        let done = socket.incoming().for_each(move |(socket, _addr)| {
            try!(socket.set_nodelay(true));
            let (reader, writer) = socket.split();
            let handle = handle1.clone();

            let network = twoparty::VatNetwork::new(
                reader,
                writer,
                rpc_twoparty_capnp::Side::Server,
                Default::default(),
            );

            let rpc_system = RpcSystem::new(Box::new(network), Some(proxy.clone().client));

            handle.spawn(rpc_system.map_err(|_| ()));
            Ok(())
        });


        info!("Starting Console on address: {}", address);
        core.run(done)
    }).map_err(|e| e.into())
}
