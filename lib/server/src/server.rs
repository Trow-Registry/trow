use std;
use std::sync::{Arc, Mutex};

use futures::Future;
use grpcio::{self, RpcStatus, RpcStatusCode};
use trow_protobuf;
use trow_protobuf::server::{WriteLocation, UploadRequest, UploadDetails, BlobRef};
use uuid::Uuid;

/// Struct implementing callbacks for the Frontend
///
/// _uploads_: a HashSet of all uuids that are currently being tracked
#[derive(Clone)]
pub struct BackendService {
    uploads: Arc<Mutex<std::collections::HashSet<Layer>>>,
}

impl BackendService {
    pub fn new() -> Self {
        BackendService {
            uploads: Arc::new(Mutex::new(std::collections::HashSet::new())),
        }
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
struct Layer {
    repo_name: String,
    digest: String,
}

//TODO: fix
fn get_path_for_uuid(uuid: &str) -> String {
    format!("data/scratch/{}", uuid)
}

impl trow_protobuf::server_grpc::Backend for BackendService {
    fn get_write_location_for_blob(
        &self,
        ctx: grpcio::RpcContext,
        blob_ref: BlobRef,
        resp: grpcio::UnarySink<WriteLocation>,
    ) {
        let set = self.uploads.lock().unwrap();
        //LAYER MUST DIE!
        let layer = Layer {
            repo_name: blob_ref.get_repo_name().to_owned(),
            digest: blob_ref.get_uuid().to_owned(),
        };

        if set.contains(&layer) {
            let path = get_path_for_uuid(blob_ref.get_uuid());
            let mut w = WriteLocation::new();
            w.set_path(path);
            let f = resp
                .success(w)
                .map_err(move |e| warn!("Failed sending to client {:?}", e));
            ctx.spawn(f);
        } else {
            let f = resp
                .fail(RpcStatus::new(RpcStatusCode::Unknown, Some("UUID Not Known".to_string())))
                .map_err(|e| warn!("Received request for unknown UUID {:?}", e));
            ctx.spawn(f);
        }
    }

    fn request_upload(
        &self,
        ctx: grpcio::RpcContext,
        req: UploadRequest,
        sink: grpcio::UnarySink<UploadDetails>,
    ) {
        let mut resp = UploadDetails::new();
        let layer = Layer {
            repo_name: req.get_repo_name().to_owned(),
            //WTF?!
            digest: Uuid::new_v4().to_string(),
        };
        {
            self.uploads.lock().unwrap().insert(layer.clone());
            debug!("Hash Table: {:?}", self.uploads);
        }
        resp.set_uuid(layer.digest.to_owned());
        let f = sink
            .success(resp)
            .map_err(move |e| warn!("failed to reply! {:?}", e));
        ctx.spawn(f);
    }

}