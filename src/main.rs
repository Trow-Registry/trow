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
extern crate fern;
#[macro_use(log, warn, info, debug)]
extern crate log;
// use of slog is currently not supported
// https://github.com/SergioBenitez/Rocket/issues/21
extern crate hostname;
extern crate ring;
extern crate rocket;
extern crate rocket_contrib;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate uuid;

extern crate tokio_core;
extern crate tokio_io;
extern crate futures;

/// Loading capn'p
mod http_capnp {
    include!(concat!(env!("OUT_DIR"), "/http_capnp.rs"));
}


mod errors;
mod routes;
pub mod response;
pub mod controller;
pub mod config;
mod test;
mod state;

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
