use std;

use http_capnp::lycaon;

use capnp::capability::Promise;
use capnp::Error;

struct Layer {
    digest: String,
    name: String,
    repo: String,
}
impl Layer {
    fn from_params(layer: lycaon::layer::Reader) -> Layer {
        Layer {
            digest: layer.get_digest().unwrap().to_string(),
            name: layer.get_name().unwrap().to_string(),
            repo: layer.get_repo().unwrap().to_string(),
        }
    }
}

fn construct_absolute_path(layer: Layer) -> String {
    format!("data/layers/{}", layer.digest)
}

fn file_length(file: std::fs::File) -> Result<u64, std::io::Error> {
    file.metadata().and_then(|metadata| Ok(metadata.len()))
}

fn do_thing(layer: Layer, ret: &mut lycaon::layer_result::Builder) {
    let path = construct_absolute_path(layer);
    match std::path::Path::new(&path).exists() {
        true => {
            ret.set_exists(true);
            let res = std::fs::File::open(path).and_then(|file| {
                file_length(file).and_then(|length| {
                    ret.set_length(length);
                    Ok(())
                })
            });
            if let Err(res) = res {
                warn!("could not open file");
            }
        }
        false => {
            ret.set_exists(false);
            ret.set_length(0);
        }
    };
}

/// Backend functions for layer-based operations.
pub struct LayerImpl;
impl lycaon::layer_interface::Server for LayerImpl {
    /// Check if a layer exists on the file-system
    ///
    /// Returns a Struct containing a boolean flag and the length of
    /// the file (if exists).
    fn layer_exists(
        &mut self,
        params: lycaon::layer_interface::LayerExistsParams,
        mut results: lycaon::layer_interface::LayerExistsResults,
    ) -> Promise<(), Error> {
        let layer = params.get().and_then(|args| {
            args.get_layer().and_then(
                |layer| Ok(Layer::from_params(layer)),
            )
        });
        let _ = layer
            .and_then(|layer| {
                let mut builder =
                    ::capnp::message::Builder::new(::capnp::message::HeapAllocator::new());
                let mut ret = builder.init_root::<lycaon::layer_result::Builder>();
                do_thing(layer, &mut ret);
                results.get().set_result(ret.as_reader())
            })
            .or_else(|e| {
                warn!("Error building LayerExists");
                warn!("{}", e);
                Err(e)
            });
        Promise::ok(())
    }
}
