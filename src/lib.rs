#![feature(decl_macro)]
#![feature(plugin)]
#![feature(use_extern_macros)]
#![plugin(rocket_codegen)]

extern crate crypto;
extern crate ctrlc;
extern crate failure;
extern crate grpcio;
extern crate hostname;
extern crate jwt;
extern crate orset;
extern crate protobuf;
extern crate rocket;
extern crate rocket_contrib;
extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;
extern crate uuid;

//TODO: Remove these short names
extern crate trow_backend as backend;
extern crate trow_protobuf as grpc;

use grpc::backend_grpc::BackendClient;

extern crate env_logger;

use log::{LogLevelFilter, LogRecord, SetLoggerError};
#[macro_use]
extern crate failure_derive;
#[macro_use(log, warn, info, debug)]
extern crate log;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate quickcheck;

use failure::Error;
use std::thread;
use std::path::Path;
use std::env;
use std::fs;

use rocket::fairing;

pub mod manifest;
pub mod response;
mod client_interface;
mod routes;
mod state;
mod types;

use client_interface::ClientInterface;

static SCRATCH_DIR: &'static str = "scratch";
static LAYERS_DIR: &'static str = "layers";


//TODO: Make this take a cause or description
#[derive(Fail, Debug)]
#[fail(display = "invalid data directory")]
pub struct ConfigError {}


pub struct NetAddr {
    pub host: String,
    pub port: u16,
}

pub struct TrowBuilder {
    data_dir: String,
    addr: NetAddr,
    tls: Option<TlsConfig>,
    grpc: GrpcConfig,
}

struct GrpcConfig {
    listen: NetAddr,
    bootstrap: NetAddr,
}

struct TlsConfig {
    cert_file: String,
    key_file: String,
}

fn init_grpc(
    listen_host: String,
    listen_port: u16,
    bootstrap_host: String,
    bootstrap_port: u16,
) -> Result<std::thread::JoinHandle<()>, Error> {
    debug!("Setting up RPC Server");

    Ok(thread::spawn(move || {
        backend::server(&listen_host, listen_port, &bootstrap_host, bootstrap_port);
    }))
}

/// Build the logging agent with formatting.
fn init_logger() -> Result<(), SetLoggerError> {
    let mut builder = env_logger::LogBuilder::new();
    builder
        .format(|record: &LogRecord| {
            format!("{}[{}] {}", record.target(), record.level(), record.args(),)
        })
        .filter(None, LogLevelFilter::Error);

    if env::var("RUST_LOG").is_ok() {
        builder.parse(&env::var("RUST_LOG").unwrap());
    }

    builder.init()
}

fn create_data_dirs(data_path: &Path) -> Result<(), Error> {
    fn setup_path(path: std::path::PathBuf) -> Result<(), Error> {
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        Ok(())
    }

    let scratch_path = data_path.join(SCRATCH_DIR);
    let layers_path = data_path.join(LAYERS_DIR);
    setup_path(scratch_path)
        .and(setup_path(layers_path))
        .map_err(|_| ConfigError {}.into())
}

impl TrowBuilder {
    pub fn new(
        data_dir: String,
        addr: NetAddr,
        listen: NetAddr,
        bootstrap: NetAddr,
    ) -> TrowBuilder {
        TrowBuilder {
            data_dir,
            addr,
            tls: None,
            grpc: GrpcConfig { listen, bootstrap },
        }
    }

    pub fn with_tls(&mut self, cert_file: String, key_file: String) -> &mut TrowBuilder {
        let cfg = TlsConfig {
            cert_file,
            key_file,
        };
        self.tls = Some(cfg);
        self
    }

    fn build_rocket_config(&self) -> Result<rocket::config::Config, Error> {
        let mut cfg = rocket::config::Config::build(rocket::config::Environment::Production)
            .address(self.addr.host.clone())
            .port(self.addr.port);

        if let Some(ref tls) = self.tls {
            cfg = cfg.tls(tls.cert_file.clone(), tls.key_file.clone());
        }
        let cfg = cfg.finalize()?;
        Ok(cfg)
    }

    pub fn start(&self) -> Result<(), Error> {
        init_logger()?;
        create_data_dirs(Path::new(&self.data_dir))?;
        // GRPC Backend thread.
        let _grpc_thread = init_grpc(
            self.grpc.listen.host.clone(),
            self.grpc.listen.port,
            self.grpc.bootstrap.host.clone(),
            self.grpc.bootstrap.port,
        )?;

        //TODO: shouldn't need to clone rocket config
        let rocket_config = &self.build_rocket_config()?;
        rocket::custom(rocket_config.clone())
            .manage(build_handlers(
                &self.grpc.listen.host,
                self.grpc.listen.port,
            ))
            .attach(fairing::AdHoc::on_attach(
                |r| match attach_sigterm() {
                    Ok(_) => Ok(r),
                    Err(_) => Err(r),
                },
            ))
            .attach(fairing::AdHoc::on_response(|_, resp| {
                //Only serve v2. If we also decide to support older clients, this will to be dropped on some paths
                resp.set_raw_header("Docker-Distribution-API-Version", "registry/2.0");
            }))
            .mount("/", routes::routes())
            .launch();
        Ok(())
    }
}

fn attach_sigterm() -> Result<(), Error> {
    ctrlc::set_handler(|| {
        info!("SIGTERM caught, shutting down...");
        std::process::exit(0);
    }).map_err(|e| e.into())
}


pub fn build_handlers(listen_host: &str, listen_port: u16) -> ClientInterface {
    use grpcio::{ChannelBuilder, EnvBuilder};
    use std::sync::Arc;

    debug!("Connecting to backend: {}:{}", listen_host, listen_port);
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(&format!("{}:{}", listen_host, listen_port));
    let client = BackendClient::new(ch);
    ClientInterface::new(client)
}
