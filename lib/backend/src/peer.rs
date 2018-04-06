use std::cell::Cell;

use futures::Future;
use grpcio;
use grpc::peer;
use grpc::peer_grpc::{Peer, PeerClient};
use grpcio::{ChannelBuilder, EnvBuilder};

use std::sync::Arc;

/// Struct implementing callbacks for Peers
///
/// _peers_: a Vector of all known clients, will be populated from
/// dns records in the K8s cluster
#[derive(Clone)]
pub struct PeerService {
    counter: Cell<u64>,
    peers: Arc<Vec<PeerClient>>
}
impl PeerService {

    pub fn new(host: &str, port: u16) -> PeerService {

        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(&format!("{}:{}", host, port));
        let client = PeerClient::new(ch);

        PeerService {
            counter: Cell::new(0),
            peers: Arc::new(vec![client]),
        }
    }
}

impl Peer for PeerService {
    fn heartbeat (
        &self,
        ctx: grpcio::RpcContext,
        _req: peer::Heartbeat,
        sink: grpcio::UnarySink<peer::Heartbeat>,
    ) {
        {
            for _ in self.peers.iter() {
                print!("Hello there!");
            }
        }
        debug!("Heartbeat received!");
        let f = sink
            .success(peer::Heartbeat::new())
            .map_err(move |e| warn!("failed to reply! {:?}", e));
        ctx.spawn(f);
    }

    fn delta_sync(
        &self,
        ctx: grpcio::RpcContext,
        req: peer::ORSetDelta,
        sink: grpcio::UnarySink<peer::ORSetDeltaReply>,
    ) {
        self.counter.set(self.counter.get() + 1);
        debug!("Counter: {:?}", self.counter);
        let mut resp = peer::ORSetDeltaReply::new();
        let deltatype = req.get_deltatype();
        resp.set_deltatype(deltatype);
        resp.set_element("server".to_owned());
        let f = sink.success(resp).map_err(move |e| {
            warn!("failed to reply! {:?}, {:?}", req, e)
        });
        ctx.spawn(f);
    }
}
