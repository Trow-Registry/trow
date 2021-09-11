extern crate clap;
extern crate trow;

use clap::{Arg, ArgMatches};
use std::env;
use std::fs::File;
use std::io::prelude::*;
use trow::{NetAddr, TrowBuilder};

const PROGRAM_NAME: &str = "Trow";
const PROGRAM_DESC: &str = "\nThe Cluster Registry";
const DEFAULT_CERT_PATH: &str = "./certs/domain.crt";
const DEFAULT_KEY_PATH: &str = "./certs/domain.key";

/*
  Responsible for configuring and starting the Trow registry.

  Parses command line arguments and returns ArgMatches object.

  Will cause the program to exit if error or on help/version argument.
*/
fn parse_args<'a>() -> ArgMatches<'a> {
    clap::App::new(PROGRAM_NAME)
        .version("0.1")
        .author("From Container Solutions")
        .about(PROGRAM_DESC)
        .arg(
            Arg::with_name("host")
                .long("host")
                .value_name("host")
                .help("Sets the name of the host or interface to start Trow on. Defaults to 0.0.0.0")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .long("port")
                .value_name("port")
                .help("The port that trow will listen on. Defaults to 8443 with TLS, 8000 without.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("no-tls")
                .long("no-tls")
                .help("Turns off TLS. Normally only used in development and debugging. If used in production, make sure you understand the risks.")
        )
        .arg(
            Arg::with_name("cert")
                .short("c")
                .long("cert")
                .value_name("cert")
                .help(&format!("Path to TLS certificate. Defaults to {}.", DEFAULT_CERT_PATH))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("key")
                .short("k")
                .long("key")
                .value_name("key")
                .help(&format!("Path to TLS private key. Defaults to {}.", DEFAULT_KEY_PATH))
                .takes_value(true),
        )
        .arg(
            Arg::with_name("data-dir")
                .short("d")
                .long("data-dir")
                .value_name("data_dir")
                .help("Directory to store images and metadata in.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("names")
            .short("n")
            .long("names")
            .value_name("names")
            .help("Host names for registry. Used in validation callbacks. Separate with comma or use quotes and spaces")
            .takes_value(true),
        )
        .arg(
            Arg::with_name("dry-run")
            .long("dry-run")
            .value_name("dry_run")
            .help("Don't acutally run Trow, just validate arguments. For testing purposes.")
            .takes_value(false),
        )
        .arg(
            Arg::with_name("allow-docker-official")
            .long("allow-docker-official")
            .value_name("allow_docker_official")
            .help("Docker official images (e.g. the debian base image) will be allowed in validation callbacks.")
            .takes_value(false)
        )
        .arg(
            Arg::with_name("deny-k8s-images")
            .long("deny-k8s-images")
            .value_name("deny_k8s_images")
            .help("By default, validation callbacks will allow various Kubernetes system images by default.
This option will deny those images; be careful as this may disable cluster installation and updates.")
            .takes_value(false)
        )
        .arg(
            Arg::with_name("allow-prefixes")
            .long("allow-prefixes")
            .value_name("allow_prefixes")
            .help("Images that begin with any of the listed prefixes will be allowed in validation callbaks. 
Separate with a comma or use quotes and spaces. 
For example 'quay.io/coreos,myhost.com/' will match quay.io/coreos/etcd and myhost.com/myimage/myrepo:tag. 
Use docker.io as the hostname for the Docker Hub.")
            .takes_value(true)
        )
        .arg(
            Arg::with_name("allow-images")
            .long("allow-images")
            .value_name("allow_images")
            .help("Images that match a full name in the list will be allowed in validation callbacks. 
Separate with a comma or use quotes and spaces. Include the hostname. 
For example 'quay.io/coreos/etcd:latest'. Use docker.io as the hostname for the Docker Hub.")
            .takes_value(true)
        )

        .arg(
            Arg::with_name("disallow-local-prefixes")
            .long("disallow-local-prefixes")
            .value_name("disallow_local_prefixes")
            .help("Disallow local images that match the prefix _not_ including any host name.  
For example 'beta' will match myhost.com/beta/myapp assuming myhost.com is the name of this registry.")
            .takes_value(true)
        )
        .arg(
            Arg::with_name("disallow-local-images")
            .long("disallow-local-images")
            .value_name("disallow_local_images")
            .help("Disallow local images that match the full name _not_ including any host name.  
For example 'beta/myapp:tag' will match myhost.com/beta/myapp:tag assuming myhost.com is the name of this registry.")
            .takes_value(true)
        )
        .arg(
            Arg::with_name("user")
            .long("user")
            .short("u")
            .value_name("user")
            .help("Set the username that can be used to access Trow (e.g. via docker login).
Must be used with --password or --password-file")
            .takes_value(true)
        )
        .arg(
            Arg::with_name("password")
            .long("password")
            .short("p")
            .value_name("password")
            .help("Set the password that can be used to access Trow (e.g. via docker login).
Must be used with --user")
            .takes_value(true)
        )
        .arg(
            Arg::with_name("password-file")
            .long("password-file")
            .value_name("password-file")
            .help("Location of file with password that can be used to access Trow (e.g. via docker login).
Must be used with --user")
            .takes_value(true)
        )
        .arg(
            Arg::with_name("version")
            .long("version")
            .short("v")
            .value_name("version")
            .help("Get the version number of Trow")
            .takes_value(false)
        )
        .arg(
            Arg::with_name("proxy-docker-hub")
            .long("proxy-docker-hub")
            .value_name("proxy-docker-hub")
            .help("Proxies repos at f/docker/<repo_name> to docker.io/<repo_name>. Downloaded images will be cached.")
            .takes_value(false)
        )
        .arg(
            Arg::with_name("hub-user")
            .long("hub-user")
            .value_name("hub-user")
            .help("Set the username for accessing the Docker Hub, used when proxying Docker Hub images.
Must be used with --hub-token or --hub-token-file")
            .takes_value(true)
        )
        .arg(
            Arg::with_name("hub-token")
            .long("hub-token")
            .value_name("hub-token")
            .help("Set the token for accessing the Docker Hub, used when proxying Docker Hub images")
            .takes_value(true)
        )
        .arg(
            Arg::with_name("hub-token-file")
            .long("hub-token-file")
            .value_name("hub-token-file")
            .help("Location of file with token that can be used for accessing the Docker Hub, used when proxying Docker Hub images")
            .takes_value(true)
        )
        .arg(
            Arg::with_name("enable-cors")
                .long("enable-cors")
                .help("Enable Cross-Origin Resource Sharing(CORS) requests. Used to allow access from web apps (e.g. GUIs).")
        )
        .arg(
            Arg::with_name("max-manifest-size")
            .long("max-manifest-size")
            .value_name("max-manifest-size")
            .help("Maximum size in mebibytes of manifest file that can be uploaded. This is JSON metada, so usually relatively small.")
            .takes_value(true)
        )
        .arg(
            Arg::with_name("max-blob-size")
            .long("max-blob-size")
            .value_name("max-blob-size")
            .help("Maximum size in mebibytes of \"blob\" that can be uploaded (a single layer of an image). This can be very large in some images (GBs).")
            .takes_value(true)
        )
        .arg(
            Arg::with_name("log-level")
            .long("log-level")
            .value_name("log-level")
            .help("The log level at which to output to stdout, valid values are OFF, ERROR, WARN, INFO, DEBUG and TRACE")
            .takes_value(true)
        )
        .get_matches()
}

fn parse_list(names: &str) -> Vec<String> {
    //split on , or whitespace
    let ret_str = names.replace(",", " ");
    ret_str.split_whitespace().map(|x| x.to_owned()).collect()
}

fn main() {
    let matches = parse_args();

    if matches.is_present("version") {
        let vcs_ref = env::var("VCS_REF").unwrap_or_default();
        println!("Trow version {} {}", env!("CARGO_PKG_VERSION"), vcs_ref);
        std::process::exit(0);
    }

    let fallback_log_level = env::var("RUST_LOG").unwrap_or("error".to_string());
    let log_level = matches.value_of("log-level").unwrap_or(&fallback_log_level);
    let no_tls = matches.is_present("no-tls");
    let host = matches.value_of("host").unwrap_or("0.0.0.0");
    let default_port = if no_tls { 8000 } else { 8443 };
    let port: u16 = matches.value_of("port").map_or(default_port, |x| {
        x.parse().expect("Failed to parse port number")
    });
    let cert_path = matches.value_of("cert").unwrap_or("./certs/domain.crt");
    let key_path = matches.value_of("key").unwrap_or("./certs/domain.key");
    let data_path = matches.value_of("data-dir").unwrap_or("./data");
    let host_names_str = matches.value_of("names").unwrap_or(host);
    let host_names = parse_list(&host_names_str);
    let dry_run = matches.is_present("dry-run");
    let proxy_hub = matches.is_present("proxy-docker-hub");

    let default_manifest_size: u32 = 4; //mebibytes
    let default_blob_size: u32 = 8192; //mebibytes
    let max_manifest_size = matches
        .value_of("max-manifest-size")
        .map_or(default_manifest_size, |x| {
            x.parse().expect("Failed to parse max manifest size")
        });
    let max_blob_size = matches
        .value_of("max-blob-size")
        .map_or(default_blob_size, |x| {
            x.parse().expect("Failed to parse max blob size")
        });

    let mut allow_prefixes = parse_list(matches.value_of("allow-prefixes").unwrap_or(""));
    if matches.is_present("allow-docker-official") {
        allow_prefixes.push("docker.io/".to_owned());
    }
    if !matches.is_present("deny-k8s-images") {
        allow_prefixes.push("k8s.gcr.io/".to_owned());
        allow_prefixes.push("docker.io/containersol/trow".to_owned());
    }
    let allow_images = parse_list(matches.value_of("allow-images").unwrap_or(""));
    let deny_prefixes = parse_list(matches.value_of("disallow-local-prefixes").unwrap_or(""));
    let deny_images = parse_list(matches.value_of("disallow-local-images").unwrap_or(""));

    let cors = matches.is_present("enable-cors");

    let addr = NetAddr {
        host: host.to_string(),
        port,
    };
    let mut builder = TrowBuilder::new(
        data_path.to_string(),
        addr,
        "127.0.0.1:51000".to_string(),
        host_names,
        proxy_hub,
        allow_prefixes,
        allow_images,
        deny_prefixes,
        deny_images,
        dry_run,
        cors,
        max_manifest_size,
        max_blob_size,
        log_level.to_string(),
    );
    if !no_tls {
        builder.with_tls(cert_path.to_string(), key_path.to_string());
    }
    if matches.is_present("user") {
        let user = matches.value_of("user").expect("Failed to read user name");

        if matches.is_present("password") {
            let pass = matches
                .value_of("password")
                .expect("Failed to read user password");
            builder.with_user(user.to_string(), pass.to_string());
        } else if matches.is_present("password-file") {
            let file_name = matches
                .value_of("password-file")
                .expect("Failed to read user password file");
            let mut file = File::open(file_name)
                .unwrap_or_else(|_| panic!("Failed to read password file {}", file_name));
            let mut pass = String::new();
            file.read_to_string(&mut pass)
                .unwrap_or_else(|_| panic!("Failed to read password file {}", file_name));

            //Remove final newline if present
            if pass.ends_with('\n') {
                pass.pop();
                if pass.ends_with('\r') {
                    pass.pop();
                }
            }

            builder.with_user(user.to_string(), pass);
        } else {
            eprintln!("Either --password or --password-file must be set if --user is set");
            std::process::exit(1);
        }
    }
    if matches.is_present("proxy-docker-hub") && matches.is_present("hub-user") {
        let hub_user = matches
            .value_of("hub-user")
            .expect("Failed to read Docker Hub user name");

        if matches.is_present("hub-token") {
            let hub_token = matches
                .value_of("hub-token")
                .expect("Failed to read Docker Hub token");
            builder.with_hub_auth(hub_user.to_string(), hub_token.to_string());
        } else if matches.is_present("hub-token-file") {
            let file_name = matches
                .value_of("hub-token-file")
                .expect("Failed to read Docker Hub token file");
            let mut file = File::open(file_name)
                .unwrap_or_else(|_| panic!("Failed to read Docker Hub token file {}", file_name));
            let mut token = String::new();
            file.read_to_string(&mut token)
                .unwrap_or_else(|_| panic!("Failed to read Docker Hub token file {}", file_name));

            //Remove final newline if present
            if token.ends_with('\n') {
                token.pop();
                if token.ends_with('\r') {
                    token.pop();
                }
            }

            builder.with_hub_auth(hub_user.to_string(), token);
        } else {
            eprintln!("Either --password or --password-file must be set if --user is set");
            std::process::exit(1);
        }
    }
    builder.start().unwrap_or_else(|e| {
        eprintln!("Error launching Trow:\n\n{}", e);
        std::process::exit(1);
    });
}
