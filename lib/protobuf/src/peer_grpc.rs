// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

const METHOD_PEER_HEARTBEAT: ::grpcio::Method<super::peer::Heartbeat, super::peer::Heartbeat> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/lycaon.Peer/heartbeat",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_PEER_DELTA_SYNC: ::grpcio::Method<super::peer::ORSetDelta, super::peer::ORSetDeltaReply> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/lycaon.Peer/deltaSync",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

pub struct PeerClient {
    client: ::grpcio::Client,
}

impl PeerClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        PeerClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn heartbeat_opt(&self, req: super::peer::Heartbeat, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::peer::Heartbeat> {
        self.client.unary_call(&METHOD_PEER_HEARTBEAT, req, opt)
    }

    pub fn heartbeat(&self, req: super::peer::Heartbeat) -> ::grpcio::Result<super::peer::Heartbeat> {
        self.heartbeat_opt(req, ::grpcio::CallOption::default())
    }

    pub fn heartbeat_async_opt(&self, req: super::peer::Heartbeat, opt: ::grpcio::CallOption) -> ::grpcio::ClientUnaryReceiver<super::peer::Heartbeat> {
        self.client.unary_call_async(&METHOD_PEER_HEARTBEAT, req, opt)
    }

    pub fn heartbeat_async(&self, req: super::peer::Heartbeat) -> ::grpcio::ClientUnaryReceiver<super::peer::Heartbeat> {
        self.heartbeat_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn delta_sync_opt(&self, req: super::peer::ORSetDelta, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::peer::ORSetDeltaReply> {
        self.client.unary_call(&METHOD_PEER_DELTA_SYNC, req, opt)
    }

    pub fn delta_sync(&self, req: super::peer::ORSetDelta) -> ::grpcio::Result<super::peer::ORSetDeltaReply> {
        self.delta_sync_opt(req, ::grpcio::CallOption::default())
    }

    pub fn delta_sync_async_opt(&self, req: super::peer::ORSetDelta, opt: ::grpcio::CallOption) -> ::grpcio::ClientUnaryReceiver<super::peer::ORSetDeltaReply> {
        self.client.unary_call_async(&METHOD_PEER_DELTA_SYNC, req, opt)
    }

    pub fn delta_sync_async(&self, req: super::peer::ORSetDelta) -> ::grpcio::ClientUnaryReceiver<super::peer::ORSetDeltaReply> {
        self.delta_sync_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Item = (), Error = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait Peer {
    fn heartbeat(&self, ctx: ::grpcio::RpcContext, req: super::peer::Heartbeat, sink: ::grpcio::UnarySink<super::peer::Heartbeat>);
    fn delta_sync(&self, ctx: ::grpcio::RpcContext, req: super::peer::ORSetDelta, sink: ::grpcio::UnarySink<super::peer::ORSetDeltaReply>);
}

pub fn create_peer<S: Peer + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_PEER_HEARTBEAT, move |ctx, req, resp| {
        instance.heartbeat(ctx, req, resp)
    });
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_PEER_DELTA_SYNC, move |ctx, req, resp| {
        instance.delta_sync(ctx, req, resp)
    });
    builder.build()
}
