use grpcio::{self, RpcStatus, RpcStatusCode, WriteFlags};
use server::TrowService;

use trow_protobuf;
use trow_protobuf::server::*;

use futures::{stream, Future, Sink};

const DOCKER_HUB_HOSTNAME: &str = "docker.io";

#[derive(Clone, Debug, PartialEq)]
struct Image {
    host: String,
    repo: String,
    name: String, //Including any tag?
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
fn parse_image(image_str: &str) -> Image {
    let host;
    let after_host;
    let image_name;
    let repo_dir;

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
    match after_host.rfind("/") {
        None => {
            if host == DOCKER_HUB_HOSTNAME {
                repo_dir = "library";
                image_name = after_host;
            } else {
                //Probably invalid, might want to return err
                repo_dir = "";
                image_name = after_host;
            }
        }
        Some(i) => {
            repo_dir = after_host.get(..i).unwrap();
            image_name = after_host.get((i + 1)..).unwrap();
        }
    }

    Image {
        host: host.to_string(),
        repo: repo_dir.to_string(),
        name: image_name.to_string(),
    }

    /*
    i := strings.IndexRune(name, '/')
    if i == -1 || (!strings.ContainsAny(name[:i], ".:") && name[:i] != "localhost") {
        hostname, remoteName = DefaultHostname, name
    } else {
        hostname, remoteName = name[:i], name[i+1:]
    }
    if hostname == LegacyDefaultHostname {
        hostname = DefaultHostname
    }
    if hostname == DefaultHostname && !strings.ContainsRune(remoteName, '/') {
        remoteName = DefaultRepoPrefix + remoteName
    }
    return
    */
}

impl trow_protobuf::server_grpc::AdmissionController for TrowService {
    fn validate_admission(
        &self,
        ctx: grpcio::RpcContext,
        ar: AdmissionRequest,
        sink: grpcio::UnarySink<AdmissionResponse>,
    ) {
        let mut resp = AdmissionResponse::new();
        resp.set_is_allowed(false);
        resp.set_reason("It smells of elderberries".to_string());

        let f = sink
            .success(resp)
            .map_err(|e| warn!("failed to reply! {:?}", e));
        ctx.spawn(f);

        //set up tests for parse

        /*
        Start with check that the image exists in this registry. We are sent the hostnames
        to consider local, which has security implications.

        for image in ar.images {

        if enforce image exists && image doesn't exist {
            //TODO: add check that enforce exists is on
            fail
        }



        if not allowed registry {
            //only makes sense if enforce exists isn't enabled
            fail
        }

        if not allowed image {
            fail
        }
        */
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
                repo: "library".to_string(),
                name: "debian".to_string(),
            }
        );
        ret = parse_image("amouat/network-utils");
        assert_eq!(
            ret,
            Image {
                host: "docker.io".to_string(),
                repo: "amouat".to_string(),
                name: "network-utils".to_string(),
            }
        );
        ret = parse_image("amouat/network-utils:latest");
        assert_eq!(
            ret,
            Image {
                host: "docker.io".to_string(),
                repo: "amouat".to_string(),
                name: "network-utils:latest".to_string(),
            }
        );

        ret = parse_image("localhost:8080/myimage:test");
        assert_eq!(
            ret,
            Image {
                host: "localhost:8080".to_string(),
                repo: "".to_string(),
                name: "myimage:test".to_string(),
            }
        );
        ret = parse_image("localhost:8080/mydir/myimage:test");
        assert_eq!(
            ret,
            Image {
                host: "localhost:8080".to_string(),
                repo: "mydir".to_string(),
                name: "myimage:test".to_string(),
            }
        );

        ret = parse_image("quay.io/mydir/another/myimage:test");
        assert_eq!(
            ret,
            Image {
                host: "quay.io".to_string(),
                repo: "mydir/another".to_string(),
                name: "myimage:test".to_string(),
            }
        );
    }

}
