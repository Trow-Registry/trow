//! # Lycaon Registry
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

extern crate clap;
extern crate config as cfg;
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
extern crate ring;
extern crate rocket;
extern crate rocket_contrib;
extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;
extern crate tokio_io;
extern crate uuid;

extern crate lycaon_backend as backend;
extern crate lycaon_protobuf as grpc;

extern crate env_logger;
#[macro_use]
extern crate failure_derive;
#[macro_use(log, warn, info, debug)]
extern crate log;
#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate quickcheck;

use std::thread;
use failure::Error;
use clap::ArgMatches;

pub mod controller;
pub mod config;
pub mod response;
mod manifest;
mod routes;
mod state;
mod types;


fn grpc(args: &ArgMatches) -> Result<std::thread::JoinHandle<()>, Error> {
    debug!("Setting up RPC Server");

    let f = args.value_of("config");

    let cnfg = match f {
        Some(v) => config::LycaonConfig::new(&v)?,
        None => config::LycaonConfig::default()?,
    };

    Ok(thread::spawn(move || { backend::server(cnfg.grpc()); }))
}

fn main() {
    config::main_logger().expect("Failed to init logging");

    // Parse command line
    let args = config::parse_args();

    // GRPC Backend thread.
    let _grpc_thread = grpc(&args).expect("Failed to start GRPC");

    //Rocket web stuff
    let rocket = config::rocket(&args).unwrap_or_else(|e| {
        log::error!("Rocket failed to process arguments {}", e);
        std::process::exit(1);
    });
    rocket.launch();
}
