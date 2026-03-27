use std::str::FromStr;

use oci_spec::distribution::{self, Reference};

use crate::registry::PROXY_DIR;
use crate::routes::Error;
use crate::utils::manifest::REGEX_TAG;

pub fn parse_reference(
    name: &str,
    version: &str,
    namespace: Option<&str>,
) -> Result<Reference, Error> {
    let (upstream, name) = if let Some(upstream) = namespace {
        (upstream.to_string(), name.to_string())
    } else if name.starts_with(PROXY_DIR) {
        let segments = name.splitn(3, '/').collect::<Vec<_>>();
        debug_assert_eq!(segments[0], "f");
        (segments[1].to_string(), segments[2].to_string())
    } else {
        ("localhost".to_string(), name.to_string())
    };

    let version = resolve_version(version)?;
    let str_reference = match version {
        BlobVersion::Tag(tag) => format!("{upstream}/{name}:{tag}"),
        BlobVersion::Digest(digest) => format!("{upstream}/{name}@{digest}"),
    };

    // Only from_str calls `split_domain`, which handles the docker "library" hack.
    match Reference::from_str(&str_reference) {
        Ok(reference) => Ok(reference),
        Err(distribution::ParseError::DigestInvalidLength) => Err(Error::BlobUnknown),
        Err(_) => Err(Error::NameInvalid(str_reference)),
    }
}

#[derive(Debug, Clone, PartialEq)]
enum BlobVersion<'a> {
    Tag(&'a str),
    Digest(&'a str),
}

fn resolve_version<'a>(version: &'a str) -> Result<BlobVersion<'a>, Error> {
    if version.contains(':') {
        Ok(BlobVersion::Digest(version))
    } else if REGEX_TAG.is_match(version) {
        Ok(BlobVersion::Tag(version))
    } else {
        Err(Error::NameInvalid(version.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_reference_docker_library() {
        let reference = parse_reference("ubuntu", "latest", Some("docker.io")).unwrap();
        assert_eq!(reference.whole(), "docker.io/library/ubuntu:latest");
    }
}
