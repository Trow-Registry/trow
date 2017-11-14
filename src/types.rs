pub type Digest = String;

#[derive(Debug, Clone)]
pub struct Layer {
    digest: Digest,
    name: String,
    repo: String,
}
impl Layer {
    pub fn new(digest: Digest, name: String, repo: String) -> Layer {
        Layer { digest, name, repo }
    }

    pub fn digest(&self) -> Digest {
        self.digest.clone()
    }
}
