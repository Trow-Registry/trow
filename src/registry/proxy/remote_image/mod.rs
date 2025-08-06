mod download;

use std::borrow::Cow;
use std::fmt;

use const_format::formatcp;
use lazy_static::lazy_static;

use crate::registry::SingleRegistryProxyConfig;
use crate::registry::digest::{Digest, DigestError};
use crate::registry::manifest::ManifestReference;

#[derive(thiserror::Error, Debug)]
pub enum RemoteImageError {
    #[error("Invalid image ref: {0}")]
    InvalidImageReference(String),
    #[error("Invalid digest: {0}")]
    InvalidDigest(#[from] DigestError),
}

/// The regex validates an image reference.
/// It returns `name`, `tag` and `digest`.
///
/// Built from:
/// https://github.com/distribution/reference/blob/727f80d42224f6696b8e1ad16b06aadf2c6b833b/regexp.go
const fn get_image_ref_regex() -> &'static str {
    const SEPARATOR: &str = "(?:[._]|__|[-]+)";
    const ALPHANUMERIC: &str = "[a-z0-9]+";
    const DOMAIN_COMPONENT: &str = "(?:[a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9-]*[a-zA-Z0-9])";
    const NAME_COMPONENT: &str = formatcp!("{ALPHANUMERIC}(?:{SEPARATOR}{ALPHANUMERIC})*");
    const DOMAIN_NAME: &str = formatcp!("{DOMAIN_COMPONENT}(?:[.]{DOMAIN_COMPONENT})*");
    const IPV6_ADDR: &str = r"\[(?:[a-fA-F0-9:]+)\]";
    const DOMAIN: &str = formatcp!("(?:{DOMAIN_NAME}|{IPV6_ADDR})(?::[0-9]+)?");
    const DIGEST: &str = "[A-Za-z][A-Za-z0-9]*(?:[-_+.][A-Za-z][A-Za-z0-9]*)*[:][[:xdigit:]]{32,}";
    const TAG: &str = r"[\w][\w.-]{0,127}";
    const NAME: &str = formatcp!("(?:{DOMAIN}/)?{NAME_COMPONENT}(/{NAME_COMPONENT})*");

    formatcp!("^(?P<name>{NAME})(?::(?P<tag>{TAG}))?(?:@(?P<digest>{DIGEST}))?$")
}

#[derive(Clone, Debug, PartialEq)]
pub struct RemoteImage<'a> {
    proxy_config: Option<&'a SingleRegistryProxyConfig>,
    /// `http` or `https`
    pub scheme: &'static str,
    /// Including port, docker.io by default
    host: String,
    repo: String,
    /// Tag or digest, `latest` by default
    pub reference: ManifestReference,
}

impl<'a> std::default::Default for RemoteImage<'a> {
    fn default() -> Self {
        Self {
            scheme: "https",
            host: "docker.io".to_string(),
            repo: "(none)".to_string(),
            reference: ManifestReference::Tag("latest".to_string()),
            proxy_config: None,
        }
    }
}

impl<'a> From<RemoteImage<'a>> for oci_client::Reference {
    fn from(val: RemoteImage) -> Self {
        match val.reference {
            ManifestReference::Digest(d) => {
                oci_client::Reference::with_digest(val.host, val.repo, d.to_string())
            }
            ManifestReference::Tag(t) => oci_client::Reference::with_tag(val.host, val.repo, t),
        }
    }
}

impl<'a> fmt::Display for RemoteImage<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_ref())
    }
}

impl<'a> RemoteImage<'a> {
    pub fn new(
        host: String,
        mut repo: String,
        reference: ManifestReference,
        proxy_config: Option<&'a SingleRegistryProxyConfig>,
    ) -> Self {
        if host.ends_with("docker.io") && !repo.contains('/') {
            // handle images like "nginx:latest" that are actually library/nginx:latest
            repo = format!("library/{repo}")
        }
        let insecure = matches!(
            proxy_config,
            Some(SingleRegistryProxyConfig { insecure: true, .. })
        );
        let scheme = if insecure { "http" } else { "https" };

        RemoteImage {
            host,
            repo,
            reference,
            scheme,
            proxy_config,
        }
    }

    pub fn get_host(&self) -> &str {
        &self.host
    }

    /// Example return value: `https://docker.io/v2/library/nginx`
    pub fn get_base_uri(&self) -> String {
        format!("{}://{}/v2/{}", self.scheme, self.host, self.repo)
    }

    pub fn get_manifest_url(&self) -> String {
        format!("{}/manifests/{}", self.get_base_uri(), self.reference)
    }

    /// Example return value: `docker.io/library/nginx@sha256:12345`
    pub fn get_ref(&self) -> String {
        let (sep, ref_) = match &self.reference {
            ManifestReference::Digest(d) => ("@", Cow::Owned(d.to_string())),
            ManifestReference::Tag(t) => (":", Cow::Borrowed(t)),
        };
        format!("{}/{}{sep}{ref_}", self.host, self.repo)
    }

    pub fn get_repo(&self) -> &str {
        &self.repo
    }

    pub fn get_proxy_cfg(&self) -> &Option<&'a SingleRegistryProxyConfig> {
        &self.proxy_config
    }

    /// Note: this uses the same validation rules as the Docker engine.
    pub fn try_from_str(image_ref: &str) -> Result<Self, RemoteImageError> {
        lazy_static! {
            static ref RE: regex::Regex = regex::RegexBuilder::new(get_image_ref_regex())
                .size_limit(1024*1024) // 1MB limit (complex regex can explode in size)
                .unicode(false) // unicode is not allowed in image references
                .build().unwrap();
        };

        let captures = RE
            .captures(image_ref)
            .ok_or_else(|| RemoteImageError::InvalidImageReference(image_ref.to_string()))?;

        let name = captures.name("name").unwrap().as_str();

        let host;
        let repo;
        // https://github.com/distribution/distribution/blob/6affafd1f030087d88f88841bf66a8abe2bf4d24/reference/normalize.go#L90
        let i = name.find('/').unwrap_or(0);
        if i == 0
            || (!name[..i].contains(['.', ':'])
                && &name[..i] != "localhost"
                && name[..i].to_lowercase() == name[..i])
        {
            host = "docker.io";
            repo = name.to_string();
        } else {
            host = &name[..i];
            repo = name[i + 1..].to_string();
        }

        let ref_ = if let Some(match_digest) = captures.name("digest") {
            let digest = Digest::try_from_raw(match_digest.as_str())?;
            ManifestReference::Digest(digest)
        } else if let Some(match_tag) = captures.name("tag") {
            ManifestReference::Tag(match_tag.as_str().to_owned())
        } else {
            ManifestReference::Tag("latest".to_string())
        };

        Ok(Self::new(host.to_string(), repo, ref_, None))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_valid() {
        let mut ret = RemoteImage::try_from_str("debian").unwrap();
        assert_eq!(
            ret,
            RemoteImage {
                host: "docker.io".to_string(),
                repo: "library/debian".to_string(),
                ..Default::default()
            }
        );
        ret = RemoteImage::try_from_str("amouat/network-utils").unwrap();
        assert_eq!(
            ret,
            RemoteImage {
                host: "docker.io".to_string(),
                repo: "amouat/network-utils".to_string(),
                ..Default::default()
            }
        );
        ret = RemoteImage::try_from_str("amouat/network-utils:beta").unwrap();
        assert_eq!(
            ret,
            RemoteImage {
                host: "docker.io".to_string(),
                repo: "amouat/network-utils".to_string(),
                reference: ManifestReference::Tag("beta".to_string()),
                ..Default::default()
            }
        );
        ret = RemoteImage::try_from_str("docker.io/mandy").unwrap();
        assert_eq!(
            ret,
            RemoteImage {
                host: "docker.io".to_string(),
                repo: "library/mandy".to_string(),
                ..Default::default()
            }
        );
        ret = RemoteImage::try_from_str("localhost:8080/myimage:test").unwrap();
        assert_eq!(
            ret,
            RemoteImage {
                host: "localhost:8080".to_string(),
                repo: "myimage".to_string(),
                reference: ManifestReference::try_from_str("test").unwrap(),
                ..Default::default()
            }
        );
        ret = RemoteImage::try_from_str("localhost:8080/mydir/myimage:test").unwrap();
        assert_eq!(
            ret,
            RemoteImage {
                host: "localhost:8080".to_string(),
                repo: "mydir/myimage".to_string(),
                reference: ManifestReference::try_from_str("test").unwrap(),
                ..Default::default()
            }
        );

        ret = RemoteImage::try_from_str("quay.io/mydir/another/myimage:test").unwrap();
        assert_eq!(
            ret,
            RemoteImage {
                host: "quay.io".to_string(),
                repo: "mydir/another/myimage".to_string(),
                reference: ManifestReference::try_from_str("test").unwrap(),
                ..Default::default()
            }
        );

        ret = RemoteImage::try_from_str("quay.io:99/myimage:heh@sha256:1e428d8e87bcc9cd156539c5afeb60075a518b20d2d4657db962df90e6552fa5").unwrap();
        assert_eq!(
            ret,
            RemoteImage {
                host: "quay.io:99".to_string(),
                repo: "myimage".to_string(),
                reference: ManifestReference::try_from_str(
                    "sha256:1e428d8e87bcc9cd156539c5afeb60075a518b20d2d4657db962df90e6552fa5"
                )
                .unwrap(),
                ..Default::default()
            }
        );

        ret = RemoteImage::try_from_str("[::1]:3409/mydir/another/myimage:test").unwrap();
        assert_eq!(
            ret,
            RemoteImage {
                host: "[::1]:3409".to_string(),
                repo: "mydir/another/myimage".to_string(),
                reference: ManifestReference::try_from_str("test").unwrap(),
                ..Default::default()
            }
        );
    }

    #[test]
    fn test_parse_invalid() {
        let invalid_images = [
            "http://docker.io/amouat/myimage:test",
            "https://docker.io/amouat/myimage:test",
            "example.com:floppotron",
            "example.com/harlots:floppotron@saucisse",
        ];

        for i in invalid_images.iter() {
            let ret = RemoteImage::try_from_str("http://docker.io/amouat/myimage:test");
            if let Ok(img) = ret {
                panic!("Invalid image ref `{i}` parsed as `{img}`");
            }
        }
    }

    #[test]
    fn test_get_uri() {
        let img = RemoteImage::new(
            "docker.io".to_string(),
            "debian".to_string(),
            ManifestReference::try_from_str("funky").unwrap(),
            None,
        );
        assert_eq!(img.get_base_uri(), "https://docker.io/v2/library/debian");
        assert_eq!(
            img.get_manifest_url(),
            "https://docker.io/v2/library/debian/manifests/funky"
        );

        let proxy_cfg = SingleRegistryProxyConfig {
            insecure: true,
            ..Default::default()
        };
        let img = RemoteImage::new(
            "cia.gov".to_string(),
            "not-watching".to_string(),
            ManifestReference::try_from_str("i-swear").unwrap(),
            Some(&proxy_cfg),
        );
        assert_eq!(img.get_base_uri(), "http://cia.gov/v2/not-watching");
        assert_eq!(
            img.get_manifest_url(),
            "http://cia.gov/v2/not-watching/manifests/i-swear"
        );

        let img = RemoteImage::try_from_str("spy:v3.1.0-cia-INTERNAL").unwrap();
        assert_eq!(img.get_base_uri(), "https://docker.io/v2/library/spy");
        assert_eq!(
            img.get_manifest_url(),
            "https://docker.io/v2/library/spy/manifests/v3.1.0-cia-INTERNAL"
        );
    }
}
