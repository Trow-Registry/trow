//! # Trow Registry
//!
//! The registry is aimed to fix the issues with the current registry
//! options that are currently available
//!
//! There are many features available:
//!

//! - Ability to delete images
//! - replication and masterless
//! - other stuff...

#![feature(decl_macro)]
#![feature(plugin)]
#![feature(use_extern_macros)]
#![plugin(rocket_codegen)]

extern crate config as cfg;
extern crate crypto;
extern crate ctrlc;
extern crate failure;
extern crate fern;
extern crate futures;
extern crate getopts;
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
extern crate tokio_core;
extern crate tokio_io;
extern crate uuid;

extern crate trow_backend as backend;
extern crate trow_protobuf as grpc;

extern crate env_logger;
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

use rocket::fairing;

pub mod config;
pub mod manifest;
pub mod response;
mod routes;
mod state;
mod types;

#[derive(Clone)]
pub struct NetAddr {
    pub host: String,
    pub port: u16,
}

pub struct TrowBuilder {
    addr: NetAddr,
    tls: Option<TlsConfig>,
    grpc: GrpcConfig,
}

#[derive(Clone)]
struct GrpcConfig {
    listen: NetAddr,
    bootstrap: NetAddr,
}

struct TlsConfig {
    cert_file: String,
    key_file: String,
}

fn grpc(
    listen_host: String,
    listen_port: u16,
    bootstrap_host: String,
    bootstrap_port: u16,
) -> Result<std::thread::JoinHandle<()>, Error> {
    debug!("Setting up RPC Server");

    Ok(thread::spawn(move || {
        backend::server(
            &listen_host,
            listen_port,
            &bootstrap_host,
            bootstrap_port,
        );
    }))
}

impl TrowBuilder {
    pub fn new(addr: NetAddr, listen: NetAddr, bootstrap: NetAddr) -> TrowBuilder {
        TrowBuilder {
            addr,
            tls: None,
            grpc: GrpcConfig { listen, bootstrap },
        }
    }

    pub fn with_tls(&mut self, cert_file: String, key_file: String) -> &mut TrowBuilder {
      
        let cfg = TlsConfig {cert_file, key_file};
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
        config::main_logger()?;
        // GRPC Backend thread.
        let _grpc_thread = grpc(self.grpc.listen.host.clone(), self.grpc.listen.port, self.grpc.bootstrap.host.clone(), self.grpc.bootstrap.port)?;

        //TODO: get rid of this clone
        let rocket_config = &self.build_rocket_config()?;
        rocket::custom(rocket_config.clone(), true)
            .manage(config::build_handlers(
                &self.grpc.listen.host,
                self.grpc.listen.port,
            ))
            //.manage(self.clone())
            .attach(fairing::AdHoc::on_attach(config::startup))
            .attach(fairing::AdHoc::on_response(|_, resp| {
                //Only serve v2. If we also decide to support older clients, this will to be dropped on some paths
                resp.set_raw_header("Docker-Distribution-API-Version", "registry/2.0");
            }))
            .mount("/", routes::routes())
            .launch();
            Ok(())
    }
}
