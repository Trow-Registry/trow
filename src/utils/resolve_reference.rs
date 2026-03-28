use oci_spec::distribution::Reference;

use crate::registry::PROXY_DIR;
use crate::routes::Error;
use crate::utils::manifest::REGEX_TAG;

pub fn parse_reference(
    name: &str,
    version: &str,
    namespace: Option<&str>,
) -> Result<Reference, Error> {
    let (upstream, mut name) = if let Some(upstream) = namespace {
        (upstream.to_string(), name.to_string())
    } else if name.starts_with(PROXY_DIR) {
        let segments = name.splitn(3, '/').collect::<Vec<_>>();
        debug_assert_eq!(segments[0], "f");
        (segments[1].to_string(), segments[2].to_string())
    } else {
        ("localhost".to_string(), name.to_string())
    };
    if upstream == "docker.io" && name.find('/').is_none() {
        name = format!("library/{}", name);
    }

    let version = resolve_version(version)?;
    Ok(match version {
        BlobVersion::Tag(tag) => Reference::with_tag(upstream, name, tag.to_owned()),
        BlobVersion::Digest(digest) => Reference::with_digest(upstream, name, digest.to_owned()),
    })
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
    fn test_parse_reference() {
        let reference = parse_reference("library/ubuntu", "latest", Some("docker.io")).unwrap();
        assert_eq!(reference.whole(), "docker.io/library/ubuntu:latest");
    }

    #[test]
    fn test_parse_reference_docker_library() {
        let reference = parse_reference("ubuntu", "latest", Some("docker.io")).unwrap();
        assert_eq!(reference.whole(), "docker.io/library/ubuntu:latest");
    }
}
