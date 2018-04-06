extern crate clap;
extern crate trow;

use clap::{Arg, ArgMatches};
use trow::{NetAddr, TrowBuilder};

const PROGRAM_NAME: &str = "Trow";
const PROGRAM_DESC: &str = "\nThe Cluster Registry";

/*
  Parses command line arguments and returns ArgMatches object.

  Will cause the program to exit if error or on help/version argument.
*/
fn parse_args<'a>() -> ArgMatches<'a> {
    clap::App::new(PROGRAM_NAME)
        .version("0.1")
        .author("From Container Solutions")
        .about(PROGRAM_DESC)
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .get_matches()
}

fn main() {
    let cert_path = "./certs/ca.crt";
    let key_path = "./certs/domain.key";

    // Parse command line
    let _args = parse_args();
    let addr = NetAddr {
        host: "0.0.0.0".to_owned(),
        port: 8443,
    };
    let grpc_listen = NetAddr {
        host: "127.0.0.1".to_owned(),
        port: 51000,
    };
    let grpc_boot = NetAddr {
        host: "127.0.0.1".to_owned(),
        port: 3117,
    };
    let mut builder = TrowBuilder::new(addr, grpc_listen, grpc_boot);
    builder.with_tls(cert_path.to_string(), key_path.to_string());
    builder.start().unwrap_or_else(|e| {
        eprintln!("Error launching Trow {}", e);
        std::process::exit(1);
    });
}
