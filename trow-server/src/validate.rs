use anyhow::{anyhow, Result};
use const_format::formatcp;
use lazy_static::lazy_static;
use log::info;
use tonic::{Request, Response, Status};

use crate::server::trow_server::admission_controller_server::AdmissionController;
use crate::server::trow_server::{AdmissionRequest, AdmissionResponse};
use crate::server::{Image, TrowServer};

const DOCKER_HUB_HOSTNAME: &str = "docker.io";

/// The regex validates an image reference.
/// It returns `name`, `tag` and `digest`.
///
/// From:
/// https://github.com/distribution/distribution/blob/91f33cb5c01ac8eecf4bc721994bcdbb9fb63ae7/reference/regexp.go
/// https://github.com/distribution/distribution/blob/b5e2f3f33dbc80d2c40b5d550541467477d5d36e/reference/reference.go
const fn get_image_ref_regex() -> &'static str {
    const SEPARATOR: &str = "(?:[._]|__|[-]*)";
    const ALPHANUMERIC: &str = "[a-z0-9]+";
    const DOMAIN_COMPONENT: &str = "(?:[a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9-]*[a-zA-Z0-9])";
    const NAME_COMPONENT: &str = formatcp!("{ALPHANUMERIC}(?:{SEPARATOR}{ALPHANUMERIC})*");
    const DOMAIN: &str = formatcp!("{DOMAIN_COMPONENT}(?:[.]{DOMAIN_COMPONENT})*(?::[0-9]+)?");
    const DIGEST: &str = "[A-Za-z][A-Za-z0-9]*(?:[-_+.][A-Za-z][A-Za-z0-9]*)*[:][[:xdigit:]]{32,}";
    const TAG: &str = r#"[\w][\w.-]{0,127}"#;
    const NAME: &str = formatcp!("(?:{DOMAIN}/)?{NAME_COMPONENT}(/{NAME_COMPONENT})*");

    formatcp!("^(?P<name>{NAME})(?::(?P<tag>{TAG}))?(?:@(?P<digest>{DIGEST}))?$")
}

/*
 * Current function is based on old Docker code to parse image names. There is a newer
 * regex based solution, but this will take some porting. At the moment invalid image names
 * are not rejected.
 *
 * The complexity is a bit unfortunate, a mixture of Docker wanting to control the
 * "default namespace", the official images and evolution over time.
 *
 * Docker hub images have host set to docker.io and official images have the "library" repo.
 *
 * TODO; should we resolve latest as well?
 *
 * The tests should clarify a bit.
 */
fn parse_image(image_str: &str) -> Result<Image> {
    lazy_static! {
        static ref RE: regex::Regex = regex::RegexBuilder::new(get_image_ref_regex())
            .size_limit(1024*1024) // 1MB limit (complex regex can explode in size)
            .unicode(false) // unicode is not allowed in image references
            .build().unwrap();
    };

    let captures = RE
        .captures(image_str)
        .ok_or_else(|| anyhow!("Invalid image ref: `{}`", image_str))?;

    let name = captures.name("name").unwrap().as_str();

    let host;
    let mut repo;
    // https://github.com/distribution/distribution/blob/6affafd1f030087d88f88841bf66a8abe2bf4d24/reference/normalize.go#L90
    let i = name.find('/').unwrap_or(0);
    if i == 0
        || (!name[..i].contains(['.', ':'])
            && &name[..i] != "localhost"
            && name[..i].to_lowercase() == name[..i])
    {
        host = DOCKER_HUB_HOSTNAME;
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

    // Handle images like ubuntu:latest, which is actually library/ubuntu:latest
    // Also handle registry-1.docker.io as well as docker.io
    if host.ends_with(DOCKER_HUB_HOSTNAME) && !repo.contains('/') {
        repo = format!("library/{}", repo);
    }

    Ok(Image {
        host: host.to_string(),
        repo,
        tag: tag.to_string(),
    })
}

#[allow(clippy::needless_return)]
fn check_image(
    image_raw: &str,
    local_hosts: &[String],
    image_exists: &dyn Fn(&Image) -> bool,
    deny: &dyn Fn(&Image) -> bool,
    allow: &dyn Fn(&Image) -> bool,
) -> (bool, String) {
    let image = parse_image(image_raw).unwrap();
    if local_hosts.contains(&image.host) {
        //local image
        if image_exists(&image) {
            if deny(&image) {
                return (false, format!("Local image {} on deny list", &image_raw));
            } else {
                let reason = format!("Image {} allowed as local image", &image_raw);
                info!("{}", reason);
                return (true, "".to_owned());
            }
        } else if allow(&image) {
            info!(
                "Local image {} allowed as on allow list (but not in registry)",
                &image_raw
            );
            return (true, "".to_owned());
        } else {
            let reason = format!(
                "Local image {} disallowed as not contained in this registry and not in allow list",
                &image_raw
            );
            info!("{}", reason);
            return (false, reason);
        }
    } else if allow(&image) {
        info!("Remote image {} allowed as on allow list", &image_raw);
        return (true, "".to_owned());
    } else {
        let reason = format!(
            "Remote image {} disallowed as not contained in this registry and not in allow list",
            &image_raw
        );
        return (false, reason);
    }
}

#[tonic::async_trait]
impl AdmissionController for TrowServer {
    async fn validate_admission(
        &self,
        ar: Request<AdmissionRequest>,
    ) -> Result<Response<AdmissionResponse>, Status> {
        let ar = ar.into_inner();
        let mut valid = true;
        let mut reason = "".to_string();

        for image_raw in ar.images {
            //Using a closure here is inefficient but makes it easier to test check_image
            let (v, r) = check_image(
                &image_raw,
                &ar.host_names,
                &|image| self.image_exists(image),
                &|i| self.is_local_denied(i),
                &|i| self.is_allowed(i),
            );
            if !v {
                valid = false;
                reason = r;
                break;
            }
        }

        let ar = AdmissionResponse {
            is_allowed: valid,
            reason,
        };
        Ok(Response::new(ar))
    }
}

#[cfg(test)]
mod test {

    use super::Image;
    use super::{check_image, parse_image};

    #[test]
    fn test_parse() {
        let mut ret = parse_image("debian").unwrap();
        assert_eq!(
            ret,
            Image {
                host: "docker.io".to_string(),
                repo: "library/debian".to_string(),
                tag: "latest".to_string(),
            }
        );
        ret = parse_image("amouat/network-utils").unwrap();
        assert_eq!(
            ret,
            Image {
                host: "docker.io".to_string(),
                repo: "amouat/network-utils".to_string(),
                tag: "latest".to_string(),
            }
        );
        ret = parse_image("amouat/network-utils:beta").unwrap();
        assert_eq!(
            ret,
            Image {
                host: "docker.io".to_string(),
                repo: "amouat/network-utils".to_string(),
                tag: "beta".to_string(),
            }
        );
        ret = parse_image("registry-1.docker.io/mandy").unwrap();
        assert_eq!(
            ret,
            Image {
                host: "registry-1.docker.io".to_string(),
                repo: "library/mandy".to_string(),
                tag: "latest".to_string(),
            }
        );
        ret = parse_image("localhost:8080/myimage:test").unwrap();
        assert_eq!(
            ret,
            Image {
                host: "localhost:8080".to_string(),
                repo: "myimage".to_string(),
                tag: "test".to_string(),
            }
        );
        ret = parse_image("localhost:8080/mydir/myimage:test").unwrap();
        assert_eq!(
            ret,
            Image {
                host: "localhost:8080".to_string(),
                repo: "mydir/myimage".to_string(),
                tag: "test".to_string(),
            }
        );

        ret = parse_image("quay.io/mydir/another/myimage:test").unwrap();
        assert_eq!(
            ret,
            Image {
                host: "quay.io".to_string(),
                repo: "mydir/another/myimage".to_string(),
                tag: "test".to_string(),
            }
        );

        ret = parse_image("quay.io:99/myimage:heh@sha256:1e428d8e87bcc9cd156539c5afeb60075a518b20d2d4657db962df90e6552fa5").unwrap();
        assert_eq!(
            ret,
            Image {
                host: "quay.io:99".to_string(),
                repo: "myimage".to_string(),
                tag: "sha256:1e428d8e87bcc9cd156539c5afeb60075a518b20d2d4657db962df90e6552fa5"
                    .to_string(),
            }
        );
    }

    #[test]
    fn test_check() {
        //Image hosted in this registry, should be ok
        let (v, _) = check_image(
            "localhost:8080/mydir/myimage:test",
            &vec!["localhost:8080".to_owned()],
            &|_| true, //determines if in this registry
            &|_| false,
            &|_| false,
        );
        assert_eq!(true, v);

        //Image refers to this registry but not present in registry (so deny)
        let (v, _) = check_image(
            "localhost:8080/mydir/myimage:test",
            &vec!["localhost:8080".to_owned()],
            &|_| false,
            &|_| false,
            &|_| false,
        );
        assert_eq!(false, v);

        //Image refers to this registry & not present but is in allow list (so allow)
        let (v, _) = check_image(
            "localhost:8080/mydir/myimage:test",
            &vec!["localhost:8080".to_owned()],
            &|_| false, //determines if in this registry
            &|_| false,
            &|_| true,
        );
        assert_eq!(true, v);

        //Image local and present but on deny list
        let (v, _) = check_image(
            "localhost:8080/mydir/myimage:test",
            &vec!["localhost:8080".to_owned()],
            &|_| true, //determines if in this registry
            &|_| true,
            &|_| false,
        );
        assert_eq!(false, v);

        //Image remote and not on allow list (deny)
        let (v, _) = check_image(
            "quay.io/mydir/myimage:test",
            &vec!["localhost:8080".to_owned()],
            &|_| true, //determines if in this registry
            &|_| false,
            &|_| false,
        );
        assert_eq!(false, v);

        //Image remote and on allow list (allow)
        let (v, _) = check_image(
            "quay.io/mydir/myimage:test",
            &vec!["localhost:8080".to_owned()],
            &|_| true, //determines if in this registry
            &|_| false,
            &|_| true,
        );
        assert_eq!(true, v);
    }
}
