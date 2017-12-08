use std::cell::Cell;

use futures::Future;
use grpcio;
use grpc;
use grpc::peer_grpc::{Peer, PeerClient};

use std::sync::Arc;

#[derive(Clone)]
pub struct PeerService {
    counter: Cell<u64>,
    peers: Arc<Vec<PeerClient>>
}
impl PeerService {
    pub fn new() -> PeerService {
        PeerService {
            counter: Cell::new(0),
            peers: Arc::new(vec![]),
        }
    }
}

impl Peer for PeerService {
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
