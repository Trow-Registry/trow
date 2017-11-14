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

extern crate capnp;
extern crate capnp_rpc;
extern crate ctrlc;
extern crate failure;
extern crate fern;
extern crate futures;
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


#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate serde_derive;
#[macro_use(log, warn, info, debug)]
extern crate log;

#[cfg(test)]
extern crate quickcheck;

/// Loading capn'p
#[allow(dead_code)]
mod http_capnp {
    include!(concat!(env!("OUT_DIR"), "/http_capnp.rs"));
}


pub mod controller;
pub mod config;
mod errors;
pub mod response;
mod routes;
mod state;
mod test;
mod types;
mod util;

use std::thread;

fn main() {
    let _log = config::main_logger().apply();

    // TODO: this name needs a change
    let handle = thread::spawn(|| {
        debug!("Starting state thread...");
        state::main().expect("Backend Service has exited unexpectedly");
    });
    config::rocket().launch();
    handle.join().unwrap();
}
