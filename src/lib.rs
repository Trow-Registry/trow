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
#[macro_use] extern crate rocket;
extern crate rocket_contrib;
extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;
extern crate uuid;
#[macro_use] extern crate display_derive;

extern crate trow_protobuf;
extern crate trow_server;

use trow_protobuf::server_grpc::BackendClient;

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
use std::env;
use std::thread;

use rocket::fairing;

mod client_interface;
pub mod response;
mod routes;
mod types;

use client_interface::ClientInterface;

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
}

struct TlsConfig {
    cert_file: String,
    key_file: String,
}

fn init_trow_server(
    data_path: String,
    listen_host: String,
    listen_port: u16,
) -> Result<std::thread::JoinHandle<()>, Error> {
    debug!("Starting Trow server");

    Ok(thread::spawn(move || {
        trow_server::start_server(&data_path, &listen_host, listen_port);
    }))
}

/// Build the logging agent with formatting.
fn init_logger() -> Result<(), SetLoggerError> {
    let mut builder = env_logger::LogBuilder::new();
    builder
        .format(|record: &LogRecord| {
            format!("{}[{}] {}", record.target(), record.level(), record.args(),)
        }).filter(None, LogLevelFilter::Error);

    if env::var("RUST_LOG").is_ok() {
        builder.parse(&env::var("RUST_LOG").unwrap());
    }

    builder.init()
}



impl TrowBuilder {
    pub fn new(
        data_dir: String,
        addr: NetAddr,
        listen: NetAddr
    ) -> TrowBuilder {
        TrowBuilder {
            data_dir,
            addr,
            tls: None,
            grpc: GrpcConfig { listen },
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
        // GRPC Backend thread.
        let _grpc_thread = init_trow_server(
            self.data_dir.clone(),
            self.grpc.listen.host.clone(),
            self.grpc.listen.port,
        )?;

        //TODO: shouldn't need to clone rocket config
        let rocket_config = &self.build_rocket_config()?;
        rocket::custom(rocket_config.clone())
            .manage(build_handlers(
                &self.grpc.listen.host,
                self.grpc.listen.port,
            ))
            .attach(fairing::AdHoc::on_attach(
                "SIGTERM handler",
                |r| match attach_sigterm() {
                    Ok(_) => Ok(r),
                    Err(_) => Err(r),
                },
            ))
            .attach(
                fairing::AdHoc::on_response(
                    "Set API Version Header", |_, resp| {
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
