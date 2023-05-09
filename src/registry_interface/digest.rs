use lazy_static::lazy_static;
use regex::Regex;
use std::io::Read;

// Crypto and crypto related imports
use sha2::{Sha256, Sha512};

// We need to rename here!
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::Digest as ShaDigest;
use std::fmt;
use std::str::FromStr;
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

pub const SUPPORTED_DIGEST_ALGORITHMS: [DigestAlgorithm; 2] =
    [DigestAlgorithm::Sha256, DigestAlgorithm::Sha512];

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
        match self {
            DigestAlgorithm::Sha256 => write!(f, "sha256"),
            DigestAlgorithm::Sha512 => write!(f, "sha512"),
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

fn digest<D: ShaDigest + Default, R: Read>(reader: &mut R) -> Result<String> {
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

fn sha256_tag_digest<R: Read>(mut reader: R) -> Result<String> {
    let digest = sha256_digest(&mut reader)?;
    Ok(format!("{}:{}", DigestAlgorithm::Sha256, digest))
}

fn sha512_digest<R: Read>(mut reader: R) -> Result<String> {
    digest::<Sha512, _>(&mut reader)
}

fn sha512_tag_digest<R: Read>(mut reader: R) -> Result<String> {
    let digest = sha512_digest(&mut reader)?;
    Ok(format!("{}:{}", DigestAlgorithm::Sha512, digest))
}

/// Returns a hash in the form of: algo:hash
pub fn hash_tag<R: Read>(algo: &DigestAlgorithm, reader: R) -> Result<String> {
    match algo {
        DigestAlgorithm::Sha256 => sha256_tag_digest(reader),
        DigestAlgorithm::Sha512 => sha512_tag_digest(reader),
    }
}

/// Returns a hash in the form of: hash
pub fn hash_reference<R: Read>(algo: &DigestAlgorithm, reader: R) -> Result<String> {
    match algo {
        DigestAlgorithm::Sha256 => sha256_digest(reader),
        DigestAlgorithm::Sha512 => sha512_digest(reader),
    }
}

pub fn parse(component: &str) -> Result<Digest, DigestError> {
    let algo_digest = component
        .split(':')
        .map(String::from)
        .collect::<Vec<String>>();

    // check that we have both parts: algo and digest
    if algo_digest.len() < 2 {
        return Err(DigestError::InvalidDigest(format!(
            "Component cannot be parsed into a digest: {}",
            &component
        )));
    }

    // Do a simple verification
    let algo = String::from(&algo_digest[0]);
    let digest = String::from(&algo_digest[1]);

    if !REGEX_ALGO.is_match(&algo) {
        return Err(DigestError::InvalidDigest(format!(
            "Component cannot be parsed into a TAG wrong digest algorithm: {} - {}",
            &component, &algo
        )));
    }

    if !REGEX_DIGEST.is_match(&digest) {
        return Err(DigestError::InvalidDigest(format!(
            "Component cannot be parsed into a TAG wrong digest format: {} - {}",
            &component, &digest
        )));
    }

    let algo_enum = DigestAlgorithm::from_str(algo.as_str()).map_err(DigestError::InvalidDigest)?;

    Ok(Digest {
        algo: algo_enum,
        hash: digest,
    })
}

#[cfg(test)]
mod test {
    use crate::registry_interface::digest::{
        sha256_digest, sha256_tag_digest, Digest, DigestAlgorithm,
    };
    use std::io::BufReader;

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

    #[test]
    fn digest_test() {
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
