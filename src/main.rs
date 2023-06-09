use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::net::{IpAddr, SocketAddr};
use std::path::Path;

use clap::builder::ArgPredicate;
use clap::Parser;
use trow::TrowBuilder;

#[derive(Parser, Debug)]
#[command(name = "Trow")]
#[command(about = "The Cluster Registry")]
#[command(author, version, long_about = None)]
struct Args {
    /// Name of the host or interface to start Trow on
    #[arg(long, default_value = "0.0.0.0")]
    host: IpAddr,

    /// Port that trow will listen on
    #[arg(
        short,
        long,
        default_value_if("tls", ArgPredicate::IsPresent, "8443"),
        default_value("8000")
    )]
    port: u16,

    /// Path to TLS certificate and key, separated by ','
    #[arg(
        long,
        num_args(0..2),
        default_missing_value = "./certs/domain.crt,./certs/domain.key",
        require_equals(true),
        value_delimiter(',')
    )]
    tls: Option<Vec<String>>,

    /// Path to directory to store images and metadata in
    #[arg(short, long, default_value = "./data")]
    data_dir: String,

    /// Host name for registry.
    ///
    /// Used in AdmissionMutation webhook.
    /// Defaults to `host`.
    #[arg(short, long)]
    name: Option<String>,

    /// Don't acutally run Trow, just validate arguments.
    ///
    /// For testing purposes.
    #[arg(long, default_value_t = false)]
    dry_run: bool,

    /// Set the username that can be used to access Trow (e.g. via docker login).
    ///
    /// Must be used with `--password` or `--password-file`
    #[arg(long, short = 'U', requires_if(ArgPredicate::IsPresent, "password"))]
    user: Option<String>,

    /// Set the password that can be used to access Trow (e.g. via docker login).
    ///
    /// Can also be a path to a file using `file://`.
    #[arg(long, short = 'P', requires_if(ArgPredicate::IsPresent, "user"))]
    password: Option<String>,

    /// Load a YAML file containing the config to validate container images through an admission webhook.
    #[arg(long)]
    image_validation_config_file: Option<String>,

    /// Load a YAML file containing the config to proxy repos at f/<registry_alias>/<repo_name> to <registry>/<repo_name>.
    #[arg(long)]
    proxy_registry_config_file: Option<String>,

    /// Enable Cross-Origin Resource Sharing(CORS) requests.
    #[arg(long, value_delimiter(','))]
    cors: Option<Vec<String>>,

    /// The log level at which to output to stdout, valid values are OFF, ERROR, WARN, INFO, DEBUG and TRACE
    #[arg(long, default_value_t = env::var("RUST_LOG").unwrap_or_else(|_| "error".to_string()))]
    log_level: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let addr = SocketAddr::new(args.host, args.port);
    let host_name = args.name.unwrap_or(addr.to_string());

    let mut builder = TrowBuilder::new(
        args.data_dir.clone(),
        addr,
        "127.0.0.1:51000".to_string(),
        host_name,
        args.dry_run,
        args.cors,
        args.log_level,
    );
    if let Some(tls) = args.tls {
        if tls.len() != 2 {
            eprintln!("tls must be a pair of paths, cert then key (got: {tls:?})");
            std::process::exit(1);
        }
        builder.with_tls(tls[0].clone(), tls[1].clone());
    }
    if let Some(user) = args.user {
        let mut pass = args.password.unwrap();
        if pass.starts_with("file://") {
            let path = Path::new(&pass[7..]).to_owned();
            let mut file = File::open(&path)
                .unwrap_or_else(|_| panic!("Failed to read password file {}", path.display()));
            pass = String::new();
            file.read_to_string(&mut pass)
                .unwrap_or_else(|_| panic!("Failed to read password file {}", path.display()));
            // Remove final newline if present
            if pass.ends_with('\n') {
                pass.pop();
                if pass.ends_with('\r') {
                    pass.pop();
                }
            }
        }
        builder.with_user(user, pass);
    }

    if let Some(config_file) = args.proxy_registry_config_file {
        if let Err(e) = builder.with_proxy_registries(config_file) {
            eprintln!("Failed to load proxy registry config file: {:#}", e);
            std::process::exit(1);
        }
    }
    if let Some(config_file) = args.image_validation_config_file {
        if let Err(e) = builder.with_image_validation(config_file) {
            eprintln!("Failed to load image validation config file: {:#}", e);
            std::process::exit(1);
        }
    }

    builder.start().await.unwrap_or_else(|e| {
        eprintln!("Error launching Trow:\n\n{}", e);
        std::process::exit(1);
    });
}
