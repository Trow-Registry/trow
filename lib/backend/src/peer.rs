use std::cell::Cell;

use futures::Future;
use grpcio;
use grpc;
use grpc::peer_grpc::{Peer, PeerClient};

use std::sync::Arc;

use config;

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
    pub fn empty() -> PeerService {
        PeerService {
            counter: Cell::new(0),
            peers: Arc::new(vec![]),
        }
    }

    pub fn new(service: config::Service) -> PeerService {
        use grpcio::{ChannelBuilder, EnvBuilder};
        use grpc::peer;
        use grpc::peer_grpc::PeerClient;

        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(&service.address());
        let client = PeerClient::new(ch);

        client.heartbeat(peer::Heartbeat::new())
            .expect("no heartbeat received");
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
        req: grpc::peer::Heartbeat,
        sink: grpcio::UnarySink<grpc::peer::Heartbeat>,
    ) {
        debug!("Heartbeat received!");
        let f = sink
            .success(grpc::peer::Heartbeat::new())
            .map_err(move |e| warn!("failed to reply! {:?}", e));
        ctx.spawn(f);
    }

    fn delta_sync(
        &self,
        ctx: grpcio::RpcContext,
        req: grpc::peer::ORSetDelta,
        sink: grpcio::UnarySink<grpc::peer::ORSetDeltaReply>,
    ) {
        self.counter.set(self.counter.get() + 1);
        debug!("Counter: {:?}", self.counter);
        let mut resp = grpc::peer::ORSetDeltaReply::new();
        let deltatype = req.get_deltatype();
        resp.set_deltatype(deltatype);
        resp.set_element("server".to_owned());
        let f = sink.success(resp).map_err(move |e| {
            warn!("failed to reply! {:?}, {:?}", req, e)
        });
        ctx.spawn(f);
    }
}
