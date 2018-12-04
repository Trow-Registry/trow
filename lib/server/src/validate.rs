use grpcio::{self, RpcStatus, RpcStatusCode, WriteFlags};
use server::TrowService;

use trow_protobuf;
use trow_protobuf::server::*;

impl trow_protobuf::server_grpc::AdmissionController for TrowService {
    fn validate_admission(
        &self,
        ctx: grpcio::RpcContext,
        ar: AdmissionRequest,
        sink: grpcio::UnarySink<AdmissionResponse>,
    ) {
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
