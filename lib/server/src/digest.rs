use failure::{self, Error};
use std::io::Read;

// Crypto and crypto related imports
use sha2::{Sha256, Digest};

// Buffer size for SHA2 hashing
const BUFFER_SIZE: usize = 1024;

fn digest<D: Digest + Default, R: Read>(reader: &mut R) -> Result<String, Error> {
    let mut sh = D::default();
    let mut buffer = [0u8; BUFFER_SIZE];
    loop {
        let n = match reader.read(&mut buffer) {
            Ok(n) => n,
            Err(err) => return Err(Error::from(err))
        };
        sh.update(&buffer[..n]);
        if n == 0 || n < BUFFER_SIZE {
            break;
        }
    }
    Ok(hex::encode(sh.finalize()))
}

fn sha256_digest<R: Read>(mut reader: R) -> Result<String, Error> {
    digest::<Sha256, _>(& mut reader)
}

pub fn sha256_tag_digest<R: Read>(mut reader: R) -> Result<String, Error> {
    let digest = sha256_digest(& mut reader)?;
    Ok(format!("sha256:{}", digest))
}

#[cfg(test)]
mod test {
    use crate::digest::{sha256_digest, sha256_tag_digest};
    use std::io::BufReader;

    #[test]
    fn sha256_digest_test() {
        let result = sha256_digest(BufReader::new("hello world".as_bytes())).unwrap();
        assert_eq!(result, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
    }

    #[test]
    fn sha256_tag_digest_test() {
        let result = sha256_tag_digest(BufReader::new("hello world".as_bytes())).unwrap();
        assert_eq!(result, "sha256:b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
    }

}