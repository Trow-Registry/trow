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
fn parse_args() -> ArgMatches {
    clap::Command::new(PROGRAM_NAME)
        .version(env!("CARGO_PKG_VERSION"))
        .author("From Container Solutions")
        .about(PROGRAM_DESC)
        .arg(
            Arg::new("host")
                .long("host")
                .value_name("host")
                .help("Sets the name of the host or interface to start Trow on. Defaults to 0.0.0.0")
                .num_args(1),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .value_name("port")
                .help("The port that trow will listen on. Defaults to 8443 with TLS, 8000 without.")
                .num_args(1)
                .value_parser(clap::value_parser!(u16)),
        )
        .arg(
            Arg::new("no-tls")
                .long("no-tls")
                .help("Turns off TLS. Normally only used in development and debugging. If used in production, make sure you understand the risks.")
                .num_args(0)
        )
        .arg(
            Arg::new("cert")
                .short('c')
                .long("cert")
                .value_name("cert")
                .help(format!("Path to TLS certificate. Defaults to {}.", DEFAULT_CERT_PATH))
                .num_args(1),
        )
        .arg(
            Arg::new("key")
                .short('k')
                .long("key")
                .value_name("key")
                .help(format!("Path to TLS private key. Defaults to {}.", DEFAULT_KEY_PATH))
                .num_args(1),
        )
        .arg(
            Arg::new("data-dir")
                .short('d')
                .long("data-dir")
                .value_name("data_dir")
                .help("Directory to store images and metadata in.")
                .num_args(1),
        )
        .arg(
            Arg::new("name")
            .short('n')
            .long("name")
            .value_name("name")
            .help("Host name for registry. Used in AdmissionMutation webhook.")
            .num_args(1),
        )
        .arg(
            Arg::new("dry-run")
            .long("dry-run")
            .value_name("dry_run")
            .help("Don't acutally run Trow, just validate arguments. For testing purposes.")
            .num_args(0),
        )
        .arg(
            Arg::new("user")
            .long("user")
            .short('u')
            .value_name("user")
            .help("Set the username that can be used to access Trow (e.g. via docker login).
Must be used with --password or --password-file")
            .num_args(1)
        )
        .arg(
            Arg::new("password")
            .long("password")
            .short('p')
            .value_name("password")
            .help("Set the password that can be used to access Trow (e.g. via docker login).
Must be used with --user")
            .num_args(1)
        )
        .arg(
            Arg::new("password-file")
            .long("password-file")
            .value_name("password-file")
            .help("Location of file with password that can be used to access Trow (e.g. via docker login).
Must be used with --user")
            .num_args(1)
        )
        .arg(
            Arg::new("image-validation-config-file")
            .long("image-validation-config-file")
            .value_name("FILE")
            .help("Load a YAML file containing the config to validate container images through an admission webhook.")
            .num_args(1)
        )
        .arg(
            Arg::new("proxy-registry-config-file")
            .long("proxy-registry-config-file")
            .value_name("FILE")
            .help("Load a YAML file containing the config to proxy repos at f/<registry_alias>/<repo_name> to <registry>/<repo_name>.")
            .num_args(1)
        )
        .arg(
            Arg::new("enable-cors")
                .long("enable-cors")
                .help("Enable Cross-Origin Resource Sharing(CORS) requests. Used to allow access from web apps (e.g. GUIs).")
                .num_args(0)
        )
        .arg(
            Arg::new("max-manifest-size")
            .long("max-manifest-size")
            .value_name("max-manifest-size")
            .help("Maximum size in mebibytes of manifest file that can be uploaded. This is JSON metada, so usually relatively small.")
            .num_args(1)
            .value_parser(clap::value_parser!(u32))
        )
        .arg(
            Arg::new("max-blob-size")
            .long("max-blob-size")
            .value_name("max-blob-size")
            .help("Maximum size in mebibytes of \"blob\" that can be uploaded (a single layer of an image). This can be very large in some images (GBs).")
            .num_args(1)
            .value_parser(clap::value_parser!(u32))
        )
        .arg(
            Arg::new("log-level")
            .long("log-level")
            .value_name("log-level")
            .help("The log level at which to output to stdout, valid values are OFF, ERROR, WARN, INFO, DEBUG and TRACE")
            .num_args(1)
        )
        .get_matches()
}

fn main() {
    let matches = parse_args();

    let fallback_log_level = env::var("RUST_LOG").unwrap_or_else(|_| "error".to_string());
    let log_level = matches
        .get_one::<String>("log-level")
        .unwrap_or(&fallback_log_level);
    let no_tls = matches.get_flag("no-tls");
    let host = matches
        .get_one::<String>("host")
        .map(String::as_str)
        .unwrap_or("0.0.0.0");
    let default_port = if no_tls { 8000 } else { 8443 };
    let port: u16 = *matches.get_one::<u16>("port").unwrap_or(&default_port);
    let cert_path = matches
        .get_one::<String>("cert")
        .map(String::as_str)
        .unwrap_or("./certs/domain.crt");
    let key_path = matches
        .get_one::<String>("key")
        .map(String::as_str)
        .unwrap_or("./certs/domain.key");
    let data_path = matches
        .get_one::<String>("data-dir")
        .map(String::as_str)
        .unwrap_or("./data");

    let host_name = matches
        .get_one::<String>("name")
        .map(String::as_str)
        .unwrap_or(host);
    let dry_run = matches.get_flag("dry-run");

    let default_manifest_size: u32 = 4; //mebibytes
    let default_blob_size: u32 = 8192; //mebibytes
    let max_manifest_size = *matches
        .get_one::<u32>("max-manifest-size")
        .unwrap_or(&default_manifest_size);
    let max_blob_size = *matches
        .get_one::<u32>("max-blob-size")
        .unwrap_or(&default_blob_size);

    let cors = matches.get_flag("enable-cors");

    let addr = NetAddr {
        host: host.to_string(),
        port,
    };
    let mut builder = TrowBuilder::new(
        data_path.to_string(),
        addr,
        "127.0.0.1:51000".to_string(),
        host_name.to_owned(),
        dry_run,
        cors,
        max_manifest_size,
        max_blob_size,
        log_level.to_string(),
    );
    if !no_tls {
        builder.with_tls(cert_path.to_string(), key_path.to_string());
    }
    if let Some(user) = matches.get_one::<String>("user") {
        if let Some(pass) = matches.get_one::<String>("password") {
            builder.with_user(user.to_string(), pass.to_string());
        } else if let Some(pass_file) = matches.get_one::<String>("password-file") {
            let mut file = File::open(pass_file)
                .unwrap_or_else(|_| panic!("Failed to read password file {}", pass_file));
            let mut pass = String::new();
            file.read_to_string(&mut pass)
                .unwrap_or_else(|_| panic!("Failed to read password file {}", pass_file));

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
    if let Some(config_file) = matches.get_one::<String>("proxy-registry-config-file") {
        if let Err(e) = builder.with_proxy_registries(config_file) {
            eprintln!("Failed to load proxy registry config file: {:#}", e);
            std::process::exit(1);
        }
    }
    if let Some(config_file) = matches.get_one::<String>("image-validation-config-file") {
        if let Err(e) = builder.with_image_validation(config_file) {
            eprintln!("Failed to load image validation config file: {:#}", e);
            std::process::exit(1);
        }
    }

    builder.start().unwrap_or_else(|e| {
        eprintln!("Error launching Trow:\n\n{}", e);
        std::process::exit(1);
    });
}
