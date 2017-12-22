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

#![feature(plugin)]
#![feature(use_extern_macros)]
#![plugin(rocket_codegen)]

extern crate args;
extern crate config as cfg;
extern crate ctrlc;
extern crate failure;
extern crate fern;
extern crate futures;
extern crate getopts;
extern crate grpcio;
extern crate hostname;
extern crate orset;
extern crate protobuf;
extern crate ring;
extern crate rocket;
extern crate rocket_contrib;
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

pub mod controller;
pub mod config;
mod errors;
pub mod response;
mod routes;
mod state;
mod test;
mod types;
mod util;

fn grpc(args: &args::Args) -> Result<std::thread::JoinHandle<()>, Error> {
    debug!("Setting up RPC Server");

    let f: Result<String, args::ArgsError> = args.value_of("config");

    let cnfg = match f {
        Ok(f) => config::LycaonConfig::new(&f)?,
        Err(_) => config::LycaonConfig::default()?,
    };

    Ok(thread::spawn(move || {
        backend::server(cnfg.grpc());
    }))
}

fn main() {
    config::main_logger().expect("Failed to init logging");

    // Parse Args
    let args = config::parse_args().unwrap_or_else(|e| {
        log::error!("Failed to process configuration {}", e);
        std::process::exit(1);
    });

    // GRPC Backend thread.
    let _grpc_thread = grpc(&args).expect("Failed to start GRPC");

    //Rocket web stuff
    let rocket = config::rocket(&args).unwrap_or_else(|e| {
        log::error!("Rocket failed to process arguments {}", e);
        std::process::exit(1);
    });
    rocket.launch();
}
