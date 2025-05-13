use std::fs::File;
use std::io::prelude::*;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use clap::builder::ArgPredicate;
use trow::{TlsConfig, TrowConfig};

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

    /// Don't actually run Trow, just validate arguments.
    ///
    /// For testing purposes.
    #[arg(long, default_value_t = false)]
    dry_run: bool,

    /// Set the username that can be used to access Trow (e.g. via docker login).
    ///
    /// Must be used with `-P` or `--password`
    #[arg(long, short = 'U', requires_if(ArgPredicate::IsPresent, "password"))]
    user: Option<String>,

    /// Set the password that can be used to access Trow (e.g. via docker login).
    ///
    /// Can also be a path to a file using `file://`.
    #[arg(long, short = 'P', requires_if(ArgPredicate::IsPresent, "user"))]
    password: Option<String>,

    /// Load a YAML file containing the image validation and proxy registry config.
    #[arg(long)]
    config_file: Option<String>,

    /// Enable Cross-Origin Resource Sharing(CORS) requests.
    #[arg(long, value_delimiter(','))]
    cors: Option<Vec<String>>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let addr = SocketAddr::new(args.host, args.port);
    let host_name = args.name.unwrap_or(addr.to_string());

    let mut builder = TrowConfig::new();
    builder.data_dir = PathBuf::from_str(args.data_dir.as_str()).expect("Invalid data path");
    builder.service_name = host_name;
    builder.dry_run = args.dry_run;
    builder.cors.clone_from(&args.cors);

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
        builder.with_user(user, &pass);
    }

    if let Some(config_file) = args.config_file {
        if let Err(e) = builder.with_config(config_file) {
            eprintln!("Failed to load proxy registry config file: {:#}", e);
            std::process::exit(1);
        }
    }
    builder.uses_tls = args.tls.is_some(); // that's pretty bad :(

    let app = builder
        .build_app()
        .await
        .expect("Failed to create trow server");

    let tls = match args.tls {
        Some(tls) => {
            if tls.len() != 2 {
                eprintln!("tls must be a pair of paths, cert then key (got: {tls:?})");
                std::process::exit(1);
            }
            Some(TlsConfig::new(tls[0].clone(), tls[1].clone()))
        }
        None => None,
    };

    serve_app(app, addr, tls).await.unwrap_or_else(|e| {
        eprintln!("Error launching Trow:\n\n{}", e);
        std::process::exit(1);
    });
}

#[derive(thiserror::Error, Debug)]
pub enum ServeAppError {
    #[error("Failed to load TLS certificate and key: {0}")]
    TlsInvalidPemFiles(std::io::Error),
    #[error("Could not serve app: {0}")]
    ServeError(std::io::Error),
}

async fn serve_app(
    app: Router,
    addr: SocketAddr,
    tls: Option<TlsConfig>,
) -> Result<(), ServeAppError> {
    async fn shutdown_signal(handle: axum_server::Handle) {
        use std::time::Duration;

        use tokio::signal;

        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }

        println!("signal received, starting graceful shutdown");
        // Signal the server to shutdown using Handle.
        handle.graceful_shutdown(Some(Duration::from_secs(30)));
    }

    // Listen for termination signal
    let handle = axum_server::Handle::new();
    tokio::spawn(shutdown_signal(handle.clone()));

    tracing::info!("Starting server on {}", addr);
    if let Some(ref tls) = tls {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
        let config = RustlsConfig::from_pem_file(&tls.cert_file, &tls.key_file)
            .await
            .map_err(ServeAppError::TlsInvalidPemFiles)?;
        axum_server::bind_rustls(addr, config)
            .handle(handle)
            .serve(app.into_make_service())
            .await
    } else {
        axum_server::bind(addr)
            .handle(handle)
            .serve(app.into_make_service())
            .await
    }
    .map_err(ServeAppError::ServeError)?;
    Ok(())
}
