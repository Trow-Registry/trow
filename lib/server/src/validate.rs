use server::TrowService;

use server::Image;
use trow_protobuf;
use trow_protobuf::server::*;

use futures::Future;

const DOCKER_HUB_HOSTNAME: &str = "docker.io";

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
fn parse_image(image_str: &str) -> Image {
    let host;
    let after_host;
    let repo;
    let tag;

    match image_str.find('/') {
        Some(i) => {
            let left = image_str.get(..i).unwrap();
            if !(left.starts_with("localhost") || left.contains(':') || left.contains('.')) {
                host = DOCKER_HUB_HOSTNAME;
                after_host = image_str;
            } else {
                host = left;
                after_host = image_str.get((i + 1)..).unwrap();
            }
        }
        None => {
            host = DOCKER_HUB_HOSTNAME;
            after_host = image_str;
        }
    }

    match after_host.find(':') {
        None => {
            repo = after_host;
            tag = "latest";
        }
        Some(i) => {
            repo = after_host.get(..i).unwrap();
            tag = after_host.get((i + 1)..).unwrap();
        }
    }

    Image {
        host: host.to_string(),
        repo: repo.to_string(),
        tag: tag.to_string(),
    }
}

#[allow(clippy::needless_return)]
fn check_image(
    image_raw: &str,
    local_hosts: Vec<String>,
    image_exists: &Fn(&Image) -> bool,
    deny: &Fn(&Image) -> bool,
    allow: &Fn(&Image) -> bool,
) -> (bool, String) {

    let image = parse_image(&image_raw);
    if local_hosts.contains(&image.host) { //local image
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

impl trow_protobuf::server_grpc::AdmissionController for TrowService {
    fn validate_admission(
        &self,
        ctx: grpcio::RpcContext,
        ar: AdmissionRequest,
        sink: grpcio::UnarySink<AdmissionResponse>,
    ) {
        let mut resp = AdmissionResponse::new();

        let mut valid = true;
        let mut reason = "".to_string();

        for image_raw in ar.images.into_vec() {
            //Using a closure here is inefficient but makes it easier to test check_image
            let (v, r) = check_image(
                &image_raw,
                ar.host_names.to_vec(),
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

        resp.set_is_allowed(valid);
        resp.set_reason(reason);

        let f = sink
            .success(resp)
            .map_err(|e| warn!("failed to reply! {:?}", e));
        ctx.spawn(f);
    }
}

#[cfg(test)]
mod test {

    use super::Image;
    use super::{check_image, parse_image};

    #[test]
    fn test_parse() {
        let mut ret = parse_image("debian");
        assert_eq!(
            ret,
            Image {
                host: "docker.io".to_string(),
                repo: "debian".to_string(),
                tag: "latest".to_string(),
            }
        );
        ret = parse_image("amouat/network-utils");
        assert_eq!(
            ret,
            Image {
                host: "docker.io".to_string(),
                repo: "amouat/network-utils".to_string(),
                tag: "latest".to_string(),
            }
        );
        ret = parse_image("amouat/network-utils:beta");
        assert_eq!(
            ret,
            Image {
                host: "docker.io".to_string(),
                repo: "amouat/network-utils".to_string(),
                tag: "beta".to_string(),
            }
        );

        ret = parse_image("localhost:8080/myimage:test");
        assert_eq!(
            ret,
            Image {
                host: "localhost:8080".to_string(),
                repo: "myimage".to_string(),
                tag: "test".to_string(),
            }
        );
        ret = parse_image("localhost:8080/mydir/myimage:test");
        assert_eq!(
            ret,
            Image {
                host: "localhost:8080".to_string(),
                repo: "mydir/myimage".to_string(),
                tag: "test".to_string(),
            }
        );

        ret = parse_image("quay.io/mydir/another/myimage:test");
        assert_eq!(
            ret,
            Image {
                host: "quay.io".to_string(),
                repo: "mydir/another/myimage".to_string(),
                tag: "test".to_string(),
            }
        );
    }

    #[test]
    fn test_check() {
        //Image hosted in this registry, should be ok
        let (v, _) = check_image(
            "localhost:8080/mydir/myimage:test",
            vec!["localhost:8080".to_owned()],
            &|_| true, //determines if in this registry
            &|_| false,
            &|_| false,
        );
        assert_eq!(true, v); //Easier to read than assert!(!v)

        //Image refers to this registry but not present in registry (so deny)
        let (v, r) = check_image(
            "localhost:8080/mydir/myimage:test",
            vec!["localhost:8080".to_owned()],
            &|_| false,
            &|_| false,
            &|_| false,
        );
        assert_eq!(false, v);

        //Image refers to this registry & not present but is in allow list (so allow)
        let (v, r) = check_image(
            "localhost:8080/mydir/myimage:test",
            vec!["localhost:8080".to_owned()],
            &|_| false, //determines if in this registry
            &|_| false,
            &|_| true,
        );
        assert_eq!(true, v);

        //Image local and present but on deny list
        let (v, _) = check_image(
            "localhost:8080/mydir/myimage:test",
            vec!["localhost:8080".to_owned()],
            &|_| true, //determines if in this registry
            &|_| true,
            &|_| false,
        );
        assert_eq!(false, v);

        //Image remote and not on allow list (deny)
        let (v, _) = check_image(
            "quay.io/mydir/myimage:test",
            vec!["localhost:8080".to_owned()],
            &|_| true, //determines if in this registry
            &|_| false,
            &|_| false,
        );
        assert_eq!(false, v);

        //Image remote and on allow list (allow)
        let (v, _) = check_image(
            "quay.io/mydir/myimage:test",
            vec!["localhost:8080".to_owned()],
            &|_| true, //determines if in this registry
            &|_| false,
            &|_| true,
        );
        assert_eq!(true, v);
    }

}
