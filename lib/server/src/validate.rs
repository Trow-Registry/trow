use grpcio::{self, RpcStatus, RpcStatusCode, WriteFlags};
use server::TrowService;

use trow_protobuf;
use trow_protobuf::server::*;

use futures::{stream, Future, Sink};

impl trow_protobuf::server_grpc::AdmissionController for TrowService {
    fn validate_admission(
        &self,
        ctx: grpcio::RpcContext,
        ar: AdmissionRequest,
        sink: grpcio::UnarySink<AdmissionResponse>,
    ) {
        let mut resp = AdmissionResponse::new();
        resp.set_is_allowed(false);
        resp.set_reason("It smells of elderberries".to_string());

        let f = sink
            .success(resp)
            .map_err(|e| warn!("failed to reply! {:?}", e));
        ctx.spawn(f);

        /*
        if enforce image exists && image doesn't exist {
            //TODO: add check that enforce exists is on
            fail
        }



        if not allowed registry {
            //only makes sense if enforce exists isn't enabled
            fail
        }

        if not allowed image {
            fail
        }
        */
    }
}
