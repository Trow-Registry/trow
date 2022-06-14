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
        .version("0.1")
        .author("From Container Solutions")
        .about(PROGRAM_DESC)
        .arg(
            Arg::new("host")
                .long("host")
                .value_name("host")
                .help("Sets the name of the host or interface to start Trow on. Defaults to 0.0.0.0")
                .takes_value(true),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .value_name("port")
                .help("The port that trow will listen on. Defaults to 8443 with TLS, 8000 without.")
                .takes_value(true),
        )
        .arg(
            Arg::new("no-tls")
                .long("no-tls")
                .help("Turns off TLS. Normally only used in development and debugging. If used in production, make sure you understand the risks.")
        )
        .arg(
            Arg::new("cert")
                .short('c')
                .long("cert")
                .value_name("cert")
                .help(format!("Path to TLS certificate. Defaults to {}.", DEFAULT_CERT_PATH).as_str())
                .takes_value(true),
        )
        .arg(
            Arg::new("key")
                .short('k')
                .long("key")
                .value_name("key")
                .help(format!("Path to TLS private key. Defaults to {}.", DEFAULT_KEY_PATH).as_str())
                .takes_value(true),
        )
        .arg(
            Arg::new("data-dir")
                .short('d')
                .long("data-dir")
                .value_name("data_dir")
                .help("Directory to store images and metadata in.")
                .takes_value(true),
        )
        .arg(
            Arg::new("names")
            .short('n')
            .long("names")
            .value_name("names")
            .help("Host names for registry. Used in validation callbacks. Separate with comma or use quotes and spaces")
            .takes_value(true),
        )
        .arg(
            Arg::new("dry-run")
            .long("dry-run")
            .value_name("dry_run")
            .help("Don't acutally run Trow, just validate arguments. For testing purposes.")
            .takes_value(false),
        )
        .arg(
            Arg::new("user")
            .long("user")
            .short('u')
            .value_name("user")
            .help("Set the username that can be used to access Trow (e.g. via docker login).
Must be used with --password or --password-file")
            .takes_value(true)
        )
        .arg(
            Arg::new("password")
            .long("password")
            .short('p')
            .value_name("password")
            .help("Set the password that can be used to access Trow (e.g. via docker login).
Must be used with --user")
            .takes_value(true)
        )
        .arg(
            Arg::new("password-file")
            .long("password-file")
            .value_name("password-file")
            .help("Location of file with password that can be used to access Trow (e.g. via docker login).
Must be used with --user")
            .takes_value(true)
        )
        .arg(
            Arg::new("version")
            .long("version")
            .short('v')
            .value_name("version")
            .help("Get the version number of Trow")
            .takes_value(false)
        )
        .arg(
            Arg::new("image-validation-config-file")
            .long("image-validation-config-file")
            .value_name("FILE")
            .help("Load a YAML file containing the config to validate container images through an admission webhook.")
            .takes_value(true)
        )
        .arg(
            Arg::new("proxy-registry-config-file")
            .long("proxy-registry-config-file")
            .value_name("FILE")
            .help("Load a YAML file containing the config to proxy repos at f/<registry_alias>/<repo_name> to <registry>/<repo_name>.")
            .takes_value(true)
        )
        .arg(
            Arg::new("enable-cors")
                .long("enable-cors")
                .help("Enable Cross-Origin Resource Sharing(CORS) requests. Used to allow access from web apps (e.g. GUIs).")
        )
        .arg(
            Arg::new("max-manifest-size")
            .long("max-manifest-size")
            .value_name("max-manifest-size")
            .help("Maximum size in mebibytes of manifest file that can be uploaded. This is JSON metada, so usually relatively small.")
            .takes_value(true)
        )
        .arg(
            Arg::new("max-blob-size")
            .long("max-blob-size")
            .value_name("max-blob-size")
            .help("Maximum size in mebibytes of \"blob\" that can be uploaded (a single layer of an image). This can be very large in some images (GBs).")
            .takes_value(true)
        )
        .arg(
            Arg::new("log-level")
            .long("log-level")
            .value_name("log-level")
            .help("The log level at which to output to stdout, valid values are OFF, ERROR, WARN, INFO, DEBUG and TRACE")
            .takes_value(true)
        )
        .get_matches()
}

fn parse_list(names: &str) -> Vec<String> {
    //split on , or whitespace
    let ret_str = names.replace(',', " ");
    ret_str.split_whitespace().map(|x| x.to_owned()).collect()
}

fn main() {
    let matches = parse_args();

    if matches.is_present("version") {
        let vcs_ref = env::var("VCS_REF").unwrap_or_default();
        println!("Trow version {} {}", env!("CARGO_PKG_VERSION"), vcs_ref);
        std::process::exit(0);
    }

    let fallback_log_level = env::var("RUST_LOG").unwrap_or_else(|_| "error".to_string());
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
    let host_names = parse_list(host_names_str);
    let dry_run = matches.is_present("dry-run");

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
    if let Some(config_file) = matches.value_of("proxy-registry-config-file") {
        builder.with_proxy_registries(config_file);
    }
    if let Some(config_file) = matches.value_of("image-validation-config-file") {
        builder.with_image_validation(config_file);
    }

    builder.start().unwrap_or_else(|e| {
        eprintln!("Error launching Trow:\n\n{}", e);
        std::process::exit(1);
    });
}
