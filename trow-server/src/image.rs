use std::fmt;

use anyhow::{anyhow, Result};
use const_format::formatcp;
use http::uri::Scheme;
use lazy_static::lazy_static;

/// The regex validates an image reference.
/// It returns `name`, `tag` and `digest`.
///
/// Built from:
/// https://github.com/distribution/distribution/blob/91f33cb5c01ac8eecf4bc721994bcdbb9fb63ae7/reference/regexp.go
/// https://github.com/distribution/distribution/blob/b5e2f3f33dbc80d2c40b5d550541467477d5d36e/reference/reference.go
/// With addition of `scheme`
const fn get_image_ref_regex() -> &'static str {
    const SEPARATOR: &str = "(?:[._]|__|[-]*)";
    const ALPHANUMERIC: &str = "[a-z0-9]+";
    const DOMAIN_COMPONENT: &str = "(?:[a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9-]*[a-zA-Z0-9])";
    const NAME_COMPONENT: &str = formatcp!("{ALPHANUMERIC}(?:{SEPARATOR}{ALPHANUMERIC})*");
    const DOMAIN: &str =
        formatcp!("(?P<scheme>https?://)?{DOMAIN_COMPONENT}(?:[.]{DOMAIN_COMPONENT})*(?::[0-9]+)?");
    const DIGEST: &str = "[A-Za-z][A-Za-z0-9]*(?:[-_+.][A-Za-z][A-Za-z0-9]*)*[:][[:xdigit:]]{32,}";
    const TAG: &str = r#"[\w][\w.-]{0,127}"#;
    const NAME: &str = formatcp!("(?:{DOMAIN}/)?{NAME_COMPONENT}(/{NAME_COMPONENT})*");

    formatcp!("^(?P<name>{NAME})(?::(?P<tag>{TAG}))?(?:@(?P<digest>{DIGEST}))?$")
}

#[derive(Clone, Debug, PartialEq)]
pub struct Image {
    scheme: Scheme,
    host: String,    // Including port, docker.io by default
    repo: String,    // Between host and : including any /s
    pub tag: String, // Bit after the :, latest by default (can also be a digest)
}

impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tag_sep = if self.tag.contains(':') { ":" } else { "@" };
        write!(
            f,
            "{}://{}/{}{tag_sep}{}",
            self.scheme, self.host, self.repo, self.tag
        )
    }
}

impl Image {
    pub fn new(mut host: &str, mut repo: String, tag: String) -> Self {
        if host.ends_with("docker.io") {
            // The real docker registry is `registry-1.docker.io`, not `docker.io`.
            host = "registry-1.docker.io";
            if !repo.contains('/') {
                // handle images like "nginx:latest" that are actually library/nginx:latest
                repo = format!("library/{}", repo)
            }
        }

        // Handle http:// and https:// in the repo uri
        let scheme = if host.starts_with("http://") {
            host = host.trim_start_matches("http://");
            Scheme::HTTP
        } else {
            host = host.trim_start_matches("https://");
            Scheme::HTTPS
        };

        Image {
            scheme,
            host: host.to_string(),
            repo,
            tag,
        }
    }

    pub fn get_host(&self) -> &str {
        &self.host
    }

    /// Example return value: `https://registry-1.docker.io/v2/library/nginx`
    pub fn get_base_uri(&self) -> String {
        format!("{}://{}/v2/{}", self.scheme, self.host, self.repo)
    }

    pub fn get_manifest_url(&self) -> String {
        format!(
            "{}://{}/v2/{}/manifests/{}",
            self.scheme, self.host, self.repo, self.tag
        )
    }

    /// Example return value: `registry-1.docker.io/library/nginx@sha256:12345`
    pub fn get_ref(&self) -> String {
        let tag_sep = if self.tag.contains(':') { "@" } else { ":" };
        format!("{}/{}{tag_sep}{}", self.host, self.repo, self.tag)
    }

    pub fn get_repo(&self) -> &str {
        &self.repo
    }

    pub fn try_from_str(image_ref: &str) -> Result<Self> {
        lazy_static! {
            static ref RE: regex::Regex = regex::RegexBuilder::new(get_image_ref_regex())
                .size_limit(1024*1024) // 1MB limit (complex regex can explode in size)
                .unicode(false) // unicode is not allowed in image references
                .build().unwrap();
        };

        let captures = RE
            .captures(image_ref)
            .ok_or_else(|| anyhow!("Invalid image ref: `{}`", image_ref))?;

        let scheme = captures
            .name("scheme")
            .map(|s| s.as_str())
            .unwrap_or("https");
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
            host = "registry-1.docker.io";
            repo = name.to_string();
        } else {
            host = &name[..i];
            repo = name[i + 1..].to_string();
        }

        let tag = match captures.name("digest") {
            Some(match_) => match_.as_str(),
            None => match captures.name("tag") {
                Some(match_) => match_.as_str(),
                None => "latest",
            },
        };
        let host = format!("{scheme}{host}");
        Ok(Self::new(&host, repo, tag.to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        let mut ret = Image::try_from_str("debian").unwrap();
        assert_eq!(
            ret,
            Image {
                scheme: Scheme::HTTPS,
                host: "registry-1.docker.io".to_string(),
                repo: "library/debian".to_string(),
                tag: "latest".to_string(),
            }
        );
        ret = Image::try_from_str("amouat/network-utils").unwrap();
        assert_eq!(
            ret,
            Image {
                scheme: Scheme::HTTPS,
                host: "registry-1.docker.io".to_string(),
                repo: "amouat/network-utils".to_string(),
                tag: "latest".to_string(),
            }
        );
        ret = Image::try_from_str("amouat/network-utils:beta").unwrap();
        assert_eq!(
            ret,
            Image {
                scheme: Scheme::HTTPS,
                host: "registry-1.docker.io".to_string(),
                repo: "amouat/network-utils".to_string(),
                tag: "beta".to_string(),
            }
        );
        ret = Image::try_from_str("registry-1.docker.io/mandy").unwrap();
        assert_eq!(
            ret,
            Image {
                scheme: Scheme::HTTPS,
                host: "registry-1.docker.io".to_string(),
                repo: "library/mandy".to_string(),
                tag: "latest".to_string(),
            }
        );
        ret = Image::try_from_str("http://localhost:8080/myimage:test").unwrap();
        assert_eq!(
            ret,
            Image {
                scheme: Scheme::HTTP,
                host: "localhost:8080".to_string(),
                repo: "myimage".to_string(),
                tag: "test".to_string(),
            }
        );
        ret = Image::try_from_str("localhost:8080/mydir/myimage:test").unwrap();
        assert_eq!(
            ret,
            Image {
                scheme: Scheme::HTTPS,
                host: "localhost:8080".to_string(),
                repo: "mydir/myimage".to_string(),
                tag: "test".to_string(),
            }
        );

        ret = Image::try_from_str("quay.io/mydir/another/myimage:test").unwrap();
        assert_eq!(
            ret,
            Image {
                scheme: Scheme::HTTPS,
                host: "quay.io".to_string(),
                repo: "mydir/another/myimage".to_string(),
                tag: "test".to_string(),
            }
        );

        ret = Image::try_from_str("quay.io:99/myimage:heh@sha256:1e428d8e87bcc9cd156539c5afeb60075a518b20d2d4657db962df90e6552fa5").unwrap();
        assert_eq!(
            ret,
            Image {
                scheme: Scheme::HTTPS,
                host: "quay.io:99".to_string(),
                repo: "myimage".to_string(),
                tag: "sha256:1e428d8e87bcc9cd156539c5afeb60075a518b20d2d4657db962df90e6552fa5"
                    .to_string(),
            }
        );
    }
}