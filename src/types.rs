pub type Digest = String;

#[derive(Debug, Clone)]
pub struct Layer {
    pub digest: Digest,
    pub repo_name: String,
}
impl Layer {
    pub fn new(repo_name: String, digest: Digest) -> Layer {
        Layer { digest, repo_name }
    }

    pub fn digest(&self) -> Digest {
        self.digest.clone()
    }
}
