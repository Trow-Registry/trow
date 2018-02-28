use std;
use std::path::Path;
use std::sync::{Arc, Mutex};

use grpcio;
use grpc;

use failure::Error;
use futures::Future;
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
        BackendService { uploads: Arc::new(Mutex::new(std::collections::HashSet::new())) }
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
struct Layer {
    name: String,
    repo: String,
    digest: String,
}

/// given a layer, it returns it's size on disk
fn get_size(layer: Layer) -> Result<u64, Error> {
    let path = construct_absolute_path(&layer)?;
    std::fs::metadata(path.as_os_str())?;
    debug!("Getting length of file {:?}", path.as_os_str());
    let file = std::fs::File::open(path)?;
    file.metadata()
        .and_then(|metadata| Ok(metadata.len()))
        .map_err(|e| e.into())
}

/// Delete a file by uuid.
fn delete_blob_by_uuid(uuid: &str) -> bool {
    use std::fs;
    let path = format!("data/scratch/{}", uuid);

    fs::remove_file(path).map(|_| true).unwrap_or(false)
}

/// Takes the digest, and constructs an absolute pathstring to the digest.
fn construct_absolute_path(layer: &Layer) -> Result<Box<Path>, Error> {
    std::env::current_dir()
        .map(|cwd| {
            let absolute_dir = cwd.join(format!(
                "data/layers/{}/{}/{}",
                layer.name,
                layer.repo,
                layer.digest
            ));
            debug!("Absolute Path: {:?}", absolute_dir);
            absolute_dir.into_boxed_path()
        })
        .map_err(|e| e.into())
}

impl grpc::backend_grpc::Backend for BackendService {
    fn layer_exists(
        &self,
        ctx: grpcio::RpcContext,
        req: grpc::backend::Layer,
        sink: grpcio::UnarySink<grpc::backend::LayerExistsResult>,
    ) {
        let layer = Layer {
            name: req.get_name().to_owned(),
            repo: req.get_repo().to_owned(),
            digest: req.get_digest().to_owned(),
        };

        let mut resp = grpc::backend::LayerExistsResult::new();
        let _ = get_size(layer)
            .map(|length| {
                debug!("Success, building return object");
                resp.set_success(true);
                resp.set_length(length);
            })
            .map_err(|_| {
                debug!("Failure, building return object");
                resp.set_success(false);
            });

        let req = req.clone();
        let f = sink.success(resp).map_err(move |e| {
            warn!("failed to reply! {:?}, {:?}", req, e)
        });
        ctx.spawn(f);
    }

    fn gen_uuid(
        &self,
        ctx: grpcio::RpcContext,
        req: grpc::backend::Layer,
        sink: grpcio::UnarySink<grpc::backend::GenUuidResult>,
    ) {
        let mut resp = grpc::backend::GenUuidResult::new();
        let layer = Layer {
            name: req.get_name().to_owned(),
            repo: req.get_repo().to_owned(),
            digest: Uuid::new_v4().to_string(),
        };
        {
            self.uploads.lock().unwrap().insert(layer.clone());
            debug!("Hash Table: {:?}", self.uploads);
        }
        resp.set_uuid(layer.digest.to_owned());
        let f = sink.success(resp).map_err(
            move |e| warn!("failed to reply! {:?}", e),
        );
        ctx.spawn(f);
    }

    fn uuid_exists(
        &self,
        ctx: grpcio::RpcContext,
        req: grpc::backend::Layer,
        sink: grpcio::UnarySink<grpc::backend::Result>,
    ) {
        let mut resp = grpc::backend::Result::new();
        let set = self.uploads.lock().unwrap();
        let layer = Layer {
            name: req.get_name().to_owned(),
            repo: req.get_repo().to_owned(),
            digest: req.get_digest().to_owned(),
        };
        resp.set_success(set.contains(&layer));

        let f = sink.success(resp).map_err(
            move |e| warn!("failed to reply! {:?}", e),
        );
        ctx.spawn(f);
    }

    fn cancel_upload(
        &self,
        ctx: grpcio::RpcContext,
        req: grpc::backend::Layer,
        sink: grpcio::UnarySink<grpc::backend::Result>,
    ) {
        let mut resp = grpc::backend::Result::new();
        let mut set = self.uploads.lock().unwrap();
        let layer = Layer {
            name: req.get_name().to_owned(),
            repo: req.get_repo().to_owned(),
            digest: req.get_digest().to_owned(),
        };
        let _ = delete_blob_by_uuid(&layer.digest);
        resp.set_success(set.remove(&layer));

        let f = sink.success(resp).map_err(
            move |e| warn!("failed to reply! {:?}", e),
        );
        ctx.spawn(f);
    }

    fn delete_uuid(
        &self,
        ctx: grpcio::RpcContext,
        req: grpc::backend::Layer,
        sink: grpcio::UnarySink<grpc::backend::Result>,
    ) {
        let layer = Layer {
            name: req.get_name().to_owned(),
            repo: req.get_repo().to_owned(),
            digest: req.get_digest().to_owned(),
        };
        let mut set = self.uploads.lock().unwrap();

        let mut resp = grpc::backend::Result::new();
        debug!("Before Delete: {:?}", self.uploads);
        resp.set_success(set.remove(&layer));
        debug!("After Delete: {:?}", self.uploads);

        let f = sink.success(resp).map_err(
            move |e| warn!("failed to reply! {:?}", e),
        );
        ctx.spawn(f);
    }

    fn upload_manifest(
        &self,
        ctx: grpcio::RpcContext,
        _req: grpc::backend::Manifest,
        sink: grpcio::UnarySink<grpc::backend::Result>,
    ) {
        warn!("upload manifest not implemented");
        let mut resp = grpc::backend::Result::new();
        resp.set_success(false);

        let f = sink.success(resp).map_err(
            move |e| warn!("failed to reply! {:?}", e),
        );
        ctx.spawn(f);
    }

    // ---------- Admin calls ----------------

    fn get_uuids(
        &self,
        ctx: grpcio::RpcContext,
        _req: grpc::backend::Empty,
        sink: grpcio::UnarySink<grpc::backend::UuidList>,
    ) {
        let mut resp = grpc::backend::UuidList::new();
        {
            use protobuf;
            use std::iter::FromIterator;
            let set = self.uploads.lock().unwrap();
            let set = set.clone().into_iter().map(|x| {
                let mut val = grpc::backend::GenUuidResult::new();
                val.set_uuid(x.digest);
                val
            });
            resp.set_uuids(protobuf::RepeatedField::from_iter(set));
        }
        let f = sink.success(resp).map_err(
            move |e| warn!("failed to reply! {:?}", e),
        );
        ctx.spawn(f);
    }
}


#[cfg(test)]
mod test {
    // 1. start up a listening backend service
    // 2. test the exposed service
    use super::*;
    use server_async;
    use config::{LycaonBackendConfig, Service};
    use grpc::backend_grpc::BackendClient;
    use grpc::backend;
    use std::sync::Arc;
    use grpcio::{ChannelBuilder, EnvBuilder};

    macro_rules! setup_grpc {
        ($v:ident) => {
            let config = default_config();
            let $v = client(&config);
            let _server = server_async(config);
        }
    }

    // test grpc interface ////////////////////////////////////////////////////
    static mut COUNTER: u16 = 30000;

    fn default_config() -> LycaonBackendConfig {
        let counter;
        unsafe {
            counter = COUNTER;
            COUNTER += 1;
        }
        let listen = Service {
            host: "localhost".to_owned(),
            port: counter,
        };
        let bootstrap = Service {
            host: "localhost".to_owned(),
            port: 1024,
        };
        LycaonBackendConfig { listen, bootstrap }
    }

    fn client(config: &LycaonBackendConfig) -> BackendClient {
        // configure client
        let env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(env).connect(&format!(
            "{}:{}",
            config.listen.host(),
            config.listen.port()
        ));
        let client = BackendClient::new(ch);
        client
    }

    #[test]
    fn test_generated_uuid_in_struct() {
        setup_grpc!(client);

        let empty = backend::Empty::new();
        let layer = backend::Layer::new();

        // gen uuid
        let uuid = client.gen_uuid(layer).unwrap();
        let uuid = uuid.get_uuid();

        // check existence
        let uuids = client.get_uuids(empty).unwrap();
        let uuids = uuids.get_uuids().iter().map(|wrapper| {
            wrapper.get_uuid().to_owned()
        }).collect::<Vec<String>>();

        assert!(uuids.contains(&uuid.to_owned()));
        assert_eq!(uuids.len(), 1);
    }

    #[test]
    fn test_generated_uuid_accessible() {
        setup_grpc!(client);
        let layer = backend::Layer::new();

        // gen uuid
        let uuid = client.gen_uuid(layer.clone()).unwrap();
        let uuid = uuid.get_uuid();

        // check existence with invalid uuid
        let result = client.uuid_exists(layer).unwrap();

        assert!(!result.get_success());

        // check existence with valid uuid
        let mut layer = backend::Layer::new();
        layer.set_digest(uuid.to_owned());
        let result = client.uuid_exists(layer).unwrap();
        assert!(result.get_success());
    }

    #[test]
    fn test_layer_exists() {
        setup_grpc!(client);


        // test valid layer
        let mut layer = backend::Layer::new();

        layer.set_name("test".to_owned());
        layer.set_repo("test".to_owned());
        layer.set_digest("test_layer".to_owned());

        let result = client.layer_exists(layer).unwrap();

        assert!(result.get_success());

        // test invalid layer
        let mut layer = backend::Layer::new();

        layer.set_name("test".to_owned());
        layer.set_repo("test".to_owned());
        layer.set_digest("invalid_layer".to_owned());

        let result = client.layer_exists(layer).unwrap();

        assert!(!result.get_success());
    }

    #[test]
    fn test_cancel_upload() {
        setup_grpc!(client);
        // test non-existent uuid
        let layer = backend::Layer::new();

        let result = client.cancel_upload(layer).unwrap();

        assert!(!result.get_success());

        // test invalid uuid
        let mut layer = backend::Layer::new();

        layer.set_digest("invalid".to_owned());

        let result = client.cancel_upload(layer).unwrap();

        assert!(!result.get_success());

        // test valid uuid
        let layer = backend::Layer::new();
        let uuid_result = client.gen_uuid(layer).unwrap();
        let uuid = uuid_result.get_uuid();

        let mut layer = backend::Layer::new();
        layer.set_digest(uuid.to_owned());
        let result = client.cancel_upload(layer).unwrap();
        assert!(result.get_success());
    }


    // This function is the same as the above `test_cancel_upload` function
    #[test]
    fn test_delete_uuid() {
        setup_grpc!(client);
        // test non-existent uuid
        let layer = backend::Layer::new();

        let result = client.delete_uuid(layer).unwrap();

        assert!(!result.get_success());

        // test invalid uuid
        let mut layer = backend::Layer::new();

        layer.set_digest("invalid".to_owned());

        let result = client.delete_uuid(layer).unwrap();

        assert!(!result.get_success());

        // test valid uuid
        let layer = backend::Layer::new();
        let uuid_result = client.gen_uuid(layer).unwrap();
        let uuid = uuid_result.get_uuid();

        let mut layer = backend::Layer::new();
        layer.set_digest(uuid.to_owned());
        let result = client.delete_uuid(layer).unwrap();
        assert!(result.get_success());
    }

    #[test]
    fn test_upload_manifest() {
        setup_grpc!(client);
        let manifest = backend::Manifest::new();

        let result = client.upload_manifest(manifest).unwrap();

        assert!(!result.get_success());
    }
    // end test grpc interface ////////////////////////////////////////////////

    fn gen_layer() -> Layer {
        Layer {
            name: "test".to_owned(),
            repo: "test".to_owned(),
            digest: "test_layer".to_owned(),
        }
    }

    #[test]
    fn test_get_size() {

        // non-existing file
        let mut layer = gen_layer();
        layer.digest = "invalid_digest".to_owned();

        let result = get_size(layer);

        match result {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }

        // existing file
        let layer = gen_layer();

        let result = get_size(layer);

        match result {
            Ok(val) => assert_eq!(0, val),
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn test_construct_absolute_path() {
        let layer = gen_layer();

        let path = construct_absolute_path(&layer);

        match path {
            Ok(path) => {
                assert!(path.is_absolute());
                assert!(path.ends_with(layer.digest));
            },
            Err(_) => assert!(false),
        }
    }
}
