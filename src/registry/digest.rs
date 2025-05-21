use std::{fmt, io};

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::digest::OutputSizeUser;
use sha2::{Digest as ShaDigest, Sha256};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt};

// Buffer size for SHA2 hashing
const BUFFER_SIZE: usize = 1024 * 1024;

// These regex are used to do a simple validation of the tag fields
lazy_static! {
    static ref REGEX_DIGEST: Regex = Regex::new(r"^[A-Fa-f0-9]{32,}$").unwrap();
}

#[derive(Error, Debug)]
pub enum DigestError {
    #[error("`{0}`")]
    InvalidDigest(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Digest(String);

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<String> for Digest {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl AsRef<str> for Digest {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Digest {
    pub fn try_from_raw(digest_str: &str) -> Result<Digest, DigestError> {
        let sep_loc = match digest_str.find(':') {
            Some(loc) => loc,
            None => {
                return Err(DigestError::InvalidDigest(
                    "separator ':' not found".to_owned(),
                ));
            }
        };
        let digest_str = digest_str.to_lowercase();
        let algo = &digest_str[..sep_loc];
        let hash = &digest_str[sep_loc + 1..];

        if !REGEX_DIGEST.is_match(hash) {
            return Err(DigestError::InvalidDigest(format!("Invalid hash: {hash}")));
        }
        if !["sha256", "sha512"].contains(&algo) {
            return Err(DigestError::InvalidDigest(format!(
                "Invalid algo: `{algo}`"
            )));
        }

        Ok(Self(digest_str))
    }

    pub fn algo_str(&self) -> &str {
        &self.0[..6]
    }

    pub fn hash(&self) -> &str {
        &self.0[7..]
    }

    pub async fn digest_sha256<R: AsyncRead + Unpin>(mut reader: R) -> io::Result<Digest> {
        Self::digest::<Sha256, _>(&mut reader).await
    }

    pub fn digest_sha256_slice(slice: &[u8]) -> Digest {
        let hash = hex::encode(Sha256::digest(slice));
        Self(format!("sha256:{hash}"))
    }

    #[allow(clippy::self_named_constructors)]
    pub async fn digest<D: ShaDigest + Default, R: AsyncRead + Unpin>(
        reader: &mut R,
    ) -> io::Result<Digest> {
        let mut digest = D::default();
        let mut buffer = vec![0u8; BUFFER_SIZE];
        let mut n = BUFFER_SIZE;
        while n == BUFFER_SIZE {
            n = reader.read(&mut buffer).await?;
            digest.update(&buffer[..n]);
        }
        let hash = hex::encode(digest.finalize());
        let algo = match <D as OutputSizeUser>::output_size() {
            32 => Ok("sha256"),
            64 => Ok("sha512"),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid digest size",
            )),
        }?;
        Ok(Self(format!("{algo}:{hash}")))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

#[cfg(test)]
mod test {

    use crate::registry::digest::Digest;

    #[test]
    fn sha256_digest_test() {
        let result = Digest::digest_sha256_slice("hello world".as_bytes());
        assert_eq!(
            &result.0,
            "sha256:b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn sha256_digest_empty_test() {
        let result = Digest::digest_sha256_slice("".as_bytes());
        assert_eq!(
            result.0,
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string()
        );
    }

    #[test]
    fn sha256_digest_brown_fox_test() {
        let result =
            Digest::digest_sha256_slice("the quick brown fox jumps over the lazy dog".as_bytes());
        assert_eq!(
            result.0,
            "sha256:05c6e08f1d9fdafa03147fcb8f82f124c76d2f70e3d989dc8aadb5e7d7450bec".to_string()
        );
    }

    #[test]
    fn digest_to_string() {
        let digest = Digest::try_from_raw(
            "sha256:05c6e08f1d9fdafa03147fcb8f82f124c76d2f70e3d989dc8aadb5e7d7450bec",
        )
        .unwrap();
        assert_eq!(
            digest.hash(),
            "05c6e08f1d9fdafa03147fcb8f82f124c76d2f70e3d989dc8aadb5e7d7450bec"
        );
        assert_eq!(digest.algo_str(), "sha256");
        assert_eq!(
            digest.to_string(),
            "sha256:05c6e08f1d9fdafa03147fcb8f82f124c76d2f70e3d989dc8aadb5e7d7450bec"
        );
    }
}
