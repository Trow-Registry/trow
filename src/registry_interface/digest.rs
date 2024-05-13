use std::io::Read;
use std::str::FromStr;
use std::{fmt, io};

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest as ShaDigest, Sha256};
use thiserror::Error;

// Buffer size for SHA2 hashing
const BUFFER_SIZE: usize = 1024;

// These regex are used to do a simple validation of the tag fields
lazy_static! {
    static ref REGEX_ALGO: Regex = Regex::new(r"^[A-Za-z0-9_+.-]+$").unwrap();
    static ref REGEX_DIGEST: Regex = Regex::new(r"^[A-Fa-f0-9]+$").unwrap();
}

#[derive(Error, Debug)]
pub enum DigestError {
    #[error("`{0}`")]
    InvalidDigest(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default)]
pub enum DigestAlgorithm {
    #[default]
    Sha256,
    Sha512,
}

impl std::str::FromStr for DigestAlgorithm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sha256" => Ok(DigestAlgorithm::Sha256),
            "sha512" => Ok(DigestAlgorithm::Sha512),
            "SHA256" => Ok(DigestAlgorithm::Sha256),
            "SHA512" => Ok(DigestAlgorithm::Sha512),
            _ => Err(format!("'{}' is not a valid DigestAlgorithm", s)),
        }
    }
}

impl std::fmt::Display for DigestAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl DigestAlgorithm {
    pub fn as_str(&self) -> &'static str {
        match self {
            DigestAlgorithm::Sha256 => "sha256",
            DigestAlgorithm::Sha512 => "sha512",
        }
    }
}

// This contains the algorithm and the hashed value
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Digest {
    pub algo: DigestAlgorithm,
    pub hash: String,
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.algo, self.hash)
    }
}

impl Digest {
    pub fn try_from_raw(digest_str: &str) -> Result<Digest, DigestError> {
        let algo_digest = digest_str
            .split(':')
            .map(String::from)
            .collect::<Vec<String>>();

        // check that we have both parts: algo and digest
        if algo_digest.len() < 2 {
            return Err(DigestError::InvalidDigest(format!(
                "Component cannot be parsed into a digest: {}",
                &digest_str
            )));
        }

        // Do a simple verification
        let algo = String::from(&algo_digest[0]);
        let hash = String::from(&algo_digest[1]);

        if !REGEX_ALGO.is_match(&algo) {
            return Err(DigestError::InvalidDigest(format!(
                "Component cannot be parsed into a TAG wrong digest algorithm: {} - {}",
                &digest_str, &algo
            )));
        }

        if !REGEX_DIGEST.is_match(&hash) {
            return Err(DigestError::InvalidDigest(format!(
                "Component cannot be parsed into a TAG wrong digest format: {} - {}",
                &digest_str, &hash
            )));
        }

        let algo_enum =
            DigestAlgorithm::from_str(algo.as_str()).map_err(DigestError::InvalidDigest)?;

        Ok(Digest {
            algo: algo_enum,
            hash,
        })
    }

    pub fn algo_str(&self) -> &'static str {
        self.algo.as_str()
    }

    pub fn try_sha256<R: Read>(mut reader: R) -> io::Result<Self> {
        let d = digest::<Sha256, _>(&mut reader)?;
        Ok(Self {
            algo: DigestAlgorithm::Sha256,
            hash: d,
        })
    }
}

fn digest<D: ShaDigest + Default, R: Read>(reader: &mut R) -> io::Result<String> {
    let mut sh = D::default();
    let mut buffer = [0u8; BUFFER_SIZE];
    let mut n = BUFFER_SIZE;
    while n == BUFFER_SIZE {
        n = reader.read(&mut buffer)?;
        sh.update(&buffer[..n]);
    }
    Ok(hex::encode(sh.finalize()))
}

#[cfg(test)]
mod test {
    use std::io::BufReader;

    use crate::registry_interface::digest::{Digest, DigestAlgorithm};

    #[test]
    fn sha256_digest_test() {
        let result = Digest::try_sha256(BufReader::new("hello world".as_bytes())).unwrap();
        assert_eq!(
            result,
            Digest {
                algo: DigestAlgorithm::Sha256,
                hash: "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
                    .to_string()
            }
        );
    }

    #[test]
    fn sha256_digest_empty_test() {
        let result = Digest::try_sha256(BufReader::new("".as_bytes())).unwrap();
        assert_eq!(
            result,
            Digest {
                algo: DigestAlgorithm::Sha256,
                hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                    .to_string()
            }
        );
    }

    #[test]
    fn sha256_digest_brown_fox_test() {
        let result = Digest::try_sha256(BufReader::new(
            "the quick brown fox jumps over the lazy dog".as_bytes(),
        ))
        .unwrap();
        assert_eq!(
            result,
            Digest {
                algo: DigestAlgorithm::Sha256,
                hash: "05c6e08f1d9fdafa03147fcb8f82f124c76d2f70e3d989dc8aadb5e7d7450bec"
                    .to_string()
            }
        );
    }

    #[test]
    fn digest_to_string() {
        let digest = Digest {
            algo: DigestAlgorithm::Sha256,
            hash: "05c6e08f1d9fdafa03147fcb8f82f124c76d2f70e3d989dc8aadb5e7d7450bec".to_string(),
        };

        assert_eq!(
            digest.to_string(),
            "sha256:05c6e08f1d9fdafa03147fcb8f82f124c76d2f70e3d989dc8aadb5e7d7450bec"
        );
    }
}
