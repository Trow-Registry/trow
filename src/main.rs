extern crate clap;
extern crate trow;

use clap::{Arg, ArgMatches};
use trow::{NetAddr, TrowBuilder};

const PROGRAM_NAME: &str = "Trow";
const PROGRAM_DESC: &str = "\nThe Cluster Registry";
const DEFAULT_CERT_PATH: &str = "./certs/ca.crt";
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
        .get_matches()
}

fn main() {
    let matches = parse_args();

    let no_tls = matches.is_present("no-tls");
    let host = matches.value_of("host").unwrap_or("0.0.0.0");
    let default_port = if no_tls { 8000 } else { 8443 };
    let port: u16 = matches
        .value_of("port")
        .map_or(default_port, |x| x.parse().unwrap());
    let cert_path = matches.value_of("cert").unwrap_or("./certs/ca.crt");
    let key_path = matches.value_of("key").unwrap_or("./certs/domain.key");
    let data_path = matches.value_of("data-dir").unwrap_or("./data");

    let addr = NetAddr {
        host: host.to_string(),
        port: port,
    };
    let grpc_listen = NetAddr {
        host: "127.0.0.1".to_owned(),
        port: 51000,
    };
    let mut builder = TrowBuilder::new(data_path.to_string(), addr, grpc_listen);
    if !no_tls {
        builder.with_tls(cert_path.to_string(), key_path.to_string());
    }
    builder.start().unwrap_or_else(|e| {
        eprintln!("Error launching Trow {}", e);
        std::process::exit(1);
    });
}



