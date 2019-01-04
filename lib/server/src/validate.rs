use grpcio::{self, RpcStatus, RpcStatusCode, WriteFlags};
use server::TrowService;

use server::Image;
use trow_protobuf;
use trow_protobuf::server::*;

use futures::{stream, Future, Sink};

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

    match image_str.find("/") {
        Some(i) => {
            let left = image_str.get(..i).unwrap();
            if !(left.contains(".") || left.contains(":")) && !left.starts_with("localhost") {
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

    match after_host.find(":") {
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

impl trow_protobuf::server_grpc::AdmissionController for TrowService {
    fn validate_admission(
        &self,
        ctx: grpcio::RpcContext,
        ar: AdmissionRequest,
        sink: grpcio::UnarySink<AdmissionResponse>,
    ) {
        let mut resp = AdmissionResponse::new();

        /*
        Start with check that the image exists in this registry. We are sent the hostnames
        to consider local, which has security implications.
        */

        //TODO: Put enforce local images as cmd switch (maybe allow-repos or something)

        let mut valid = true;
        let mut reason = "".to_string();

        for image_raw in ar.images.into_vec() {
            let image = parse_image(&image_raw);

            if !ar.host_names.contains(&image.host) {
                valid = false;
                reason = format!(
                    "Image {} refers to an untrusted registry: {}",
                    &image_raw, &image.host
                );
            } else if !self.image_exists(&image) {
                valid = false;
                reason = "Image does not exist in this registry".to_string();
            }
            /*
            if not allowed registry {
                //only makes sense if enforce exists isn't enabled
                fail
            }

            if not allowed image {
                fail
            }
            */
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

    use super::parse_image;
    use super::Image;

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

}
