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
#![plugin(rocket_codegen)]

extern crate args;
extern crate config as cfg;
extern crate ctrlc;
extern crate failure;
extern crate fern;
extern crate futures;
extern crate getopts;
extern crate hostname;
extern crate orset;
extern crate ring;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;
extern crate tokio_io;
extern crate uuid;
extern crate protobuf;
extern crate grpcio;

extern crate lycaon_protobuf as grpc;
extern crate lycaon_backend as backend;

#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate serde_derive;
#[macro_use(log, warn, info, debug)]
extern crate log;
extern crate env_logger;

#[cfg(test)]
extern crate quickcheck;

use std::thread;
use std::sync::mpsc;

// mod backend;
pub mod controller;
pub mod config;
mod errors;
pub mod response;
mod routes;
mod state;
mod test;
mod types;
mod util;

fn grpc() -> std::thread::JoinHandle<()> {
    debug!("Setting up RPC Server");
    thread::spawn(|| {
        let _ = config::parse_args()
            .and_then(|args| {
                args.value_of("config")
                    .map_err(|e| e.into())
                    .and_then(|file| config::LycaonConfig::new(file))
                    .or_else(|_e| config::LycaonConfig::default())
            })
            .map(|config| backend::server(config.grpc()));
    })
}

fn main() {
    // Init Logger
    let _ = env_logger::init()
        .and(config::main_logger().apply())
        .map_err(|e| {
            warn!("Error setting up logging: {:?}", e);
        });

    // Parse Args
    let args = match config::parse_args() {
        Ok(args) => args,
        Err(_) => {
            std::process::exit(0);
        }

    };

    // GRPC Backend thread
    let _grpc_thread = grpc();

    let (tx_a, _rx_a) = mpsc::channel::<config::BackendMessage>();
    let (_tx_b, rx_b) = mpsc::channel::<config::BackendMessage>();

    let backend_handler = config::SocketHandler::new(tx_a, rx_b);
    config::rocket(backend_handler, args).map(|rocket| rocket.launch());
}
