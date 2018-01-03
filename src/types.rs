pub type Digest = String;

#[derive(Debug, Clone)]
pub struct Layer {
    pub digest: Digest,
    pub name: String,
    pub repo: String,
}
impl Layer {
    pub fn new(name: String, repo: String, digest: Digest) -> Layer {
        Layer { digest, name, repo }
    }

    pub fn digest(&self) -> Digest {
        self.digest.clone()
    }
}
