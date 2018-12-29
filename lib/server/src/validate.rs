use grpcio::{self, RpcStatus, RpcStatusCode, WriteFlags};
use server::TrowService;

use trow_protobuf;
use trow_protobuf::server::*;

use futures::{stream, Future, Sink};

const DOCKER_HUB_HOSTNAME: &str = "docker.io";

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
 * The tests should clarify a bit.
 */
fn parse_image(image_str: &str) -> Image {
    let host;
    let after_host;
    let image_name;
    let repo_dir;

    match image_str.find("/") {
        Some(i) => {
            if !(image_str.contains(".") || image_str.contains(":"))
                && !image_str.starts_with("localhost")
            {
                host = DOCKER_HUB_HOSTNAME;
                after_host = image_str;
            } else {
                host = image_str.get(..i).unwrap();
                after_host = image_str.get(i..).unwrap();
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
            image_name = after_host.get(i..).unwrap();
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
