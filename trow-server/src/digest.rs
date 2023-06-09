use std::io::Read;

use anyhow::{Error, Result};
// Crypto and crypto related imports
use sha2::{Digest, Sha256};

// Buffer size for SHA2 hashing
const BUFFER_SIZE: usize = 1024;

fn digest<D: Digest + Default, R: Read>(reader: &mut R) -> Result<String> {
    let mut sh = D::default();
    let mut buffer = [0u8; BUFFER_SIZE];
    loop {
        let n = match reader.read(&mut buffer) {
            Ok(n) => n,
            Err(err) => return Err(Error::from(err)),
        };
        sh.update(&buffer[..n]);
        if n == 0 || n < BUFFER_SIZE {
            break;
        }
    }
    Ok(hex::encode(sh.finalize()))
}

fn sha256_digest<R: Read>(mut reader: R) -> Result<String> {
    digest::<Sha256, _>(&mut reader)
}

pub fn sha256_tag_digest<R: Read>(mut reader: R) -> Result<String> {
    let digest = sha256_digest(&mut reader)?;
    Ok(format!("sha256:{}", digest))
}

#[cfg(test)]
mod test {
    use std::io::BufReader;

    use crate::digest::{sha256_digest, sha256_tag_digest};

    #[test]
    fn sha256_digest_test() {
        let result = sha256_digest(BufReader::new("hello world".as_bytes())).unwrap();
        assert_eq!(
            result,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn sha256_tag_digest_test() {
        let result = sha256_tag_digest(BufReader::new("hello world".as_bytes())).unwrap();
        assert_eq!(
            result,
            "sha256:b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn sha256_digest_empty_test() {
        let result = sha256_digest(BufReader::new("".as_bytes())).unwrap();
        assert_eq!(
            result,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn sha256_tag_empty_digest_test() {
        let result = sha256_tag_digest(BufReader::new("".as_bytes())).unwrap();
        assert_eq!(
            result,
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn sha256_digest_brown_fox_test() {
        let result = sha256_digest(BufReader::new(
            "the quick brown fox jumps over the lazy dog".as_bytes(),
        ))
        .unwrap();
        assert_eq!(
            result,
            "05c6e08f1d9fdafa03147fcb8f82f124c76d2f70e3d989dc8aadb5e7d7450bec"
        );
    }

    #[test]
    fn sha256_tag_brown_fox_digest_test() {
        let result = sha256_tag_digest(BufReader::new(
            "the quick brown fox jumps over the lazy dog".as_bytes(),
        ))
        .unwrap();
        assert_eq!(
            result,
            "sha256:05c6e08f1d9fdafa03147fcb8f82f124c76d2f70e3d989dc8aadb5e7d7450bec"
        );
    }
}
