use orset::ORSet;

use types::{Digest, Layer};

impl Layer {
}

type LayerSet = ORSet<Digest>;
/// Backend functions for layer-based operations.
pub struct LayerImpl {
    layers: LayerSet,
}
impl LayerImpl {
    pub fn new(layers: LayerSet) -> LayerImpl {
        LayerImpl { layers }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen, QuickCheck, TestResult};

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

            Layer::new( digest, name, repo )
        }
    }


    #[test]
    #[ignore]
    fn test_process_layer() {
        fn inner(layer: Layer) -> TestResult {
                TestResult::failed()

        }
        QuickCheck::new().tests(100).max_tests(1000).quickcheck(
            inner as fn(Layer) -> TestResult,
        );
    }
}
