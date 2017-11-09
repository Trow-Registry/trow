use std;
use std::path::Path;

use http_capnp::lycaon;

use capnp::capability::Promise;
use capnp::Error;
use capnp::message;

#[derive(Debug, Clone)]
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

/// Takes the digest, and constructs an absolute pathstring to the digest.
fn construct_absolute_path(layer: Layer) -> Box<Path> {
    let cwd = std::env::current_dir().unwrap();
    let absolute_dir = cwd.join(format!("data/layers/{}", layer.digest));
    debug!("Absolute Path: {:?}", absolute_dir);
    absolute_dir.into_boxed_path()
}

fn file_length(file: std::fs::File) -> Result<u64, std::io::Error> {
    file.metadata().and_then(|metadata| Ok(metadata.len()))
}

/// Process the Incoming Request
///
/// Given a Layer and Builder, check for file existence and get length
fn process(
    layer: Layer,
    builder: &mut lycaon::layer_result::Builder,
) -> Result<(), std::io::Error> {
    let path = construct_absolute_path(layer);
    match path.exists() {
        true => {
            builder.set_exists(true);
            std::fs::File::open(path).and_then(|file| {
                file_length(file).and_then(|length| {
                    builder.set_length(length);
                    Ok(())
                })
            })
        }
        false => {
            builder.set_exists(false);
            builder.set_length(0);
            Ok(())
        }
    }
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
                let mut builder = message::Builder::new(message::HeapAllocator::new());
                let mut builder = builder.init_root::<lycaon::layer_result::Builder>();
                let _ = process(layer, &mut builder).or_else(|e| {
                    warn!("{}", e);
                    Err(e)
                });
                results.get().set_result(builder.as_reader())
            })
            .or_else(|e| {
                warn!("Error building LayerExists");
                warn!("{}", e);
                Err(e)
            });
        Promise::ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen, QuickCheck, TestResult};
    // use metric::{AggregationMethod, Telemetry};
    // use std::sync;

    impl Arbitrary for Layer {
        fn arbitrary<G>(g: &mut G) -> Self
        where
            G: Gen,
        {
            let digest_len = g.gen_range(1, 256);
            let name_len = g.gen_range(1, 256);
            let repo_len = g.gen_range(1, 256);

            let digest: String = g.gen_ascii_chars().take(digest_len).collect();
            let digest: String = format!("sha256:{}", digest);
            let name: String = g.gen_ascii_chars().take(name_len).collect();
            let repo: String = g.gen_ascii_chars().take(repo_len).collect();

            Layer { digest, name, repo }
        }
    }


    #[test]
    fn test_process_layer() {
        fn inner(layer: Layer) -> TestResult {
            let mut builder = message::Builder::new(message::HeapAllocator::new());
            let mut builder = builder.init_root::<lycaon::layer_result::Builder>();
            process(layer.clone(), &mut builder)
                .map(|_| {
                    let reader = builder.as_reader();
                    let length = reader.get_length();
                    let exists = reader.get_exists();

                    if length == 0 {
                        assert!(!exists);
                    }
                    TestResult::passed()
                })
                .map_err(|_| TestResult::failed())
                .unwrap()

        }
        QuickCheck::new().tests(100).max_tests(1000).quickcheck(
            inner as fn(Layer) -> TestResult,
        );
    }

    #[test]
    fn test_construct_absolute_path() {
        fn inner(layer: Layer) -> TestResult {
            let path = construct_absolute_path(layer);
            assert!(path.has_root());
            TestResult::passed()
        }
        QuickCheck::new().tests(100).max_tests(1000).quickcheck(
            inner as fn(Layer) -> TestResult,
        );
    }
}
