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

// #[macro_use]
// extern crate slog;
// extern crate slog_term;
// extern crate slog_async;

extern crate ctrlc;
#[macro_use(log,info,debug)]
extern crate log;
extern crate fern;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate uuid;
extern crate ring;

mod errors;
mod routes;
pub mod response;
pub mod config;

// use slog::Drain;
use rocket::fairing;


fn main() {
    let _log = config::main_logger().apply();

    rocket::ignite()
        .attach(fairing::AdHoc::on_attach(config::startup))
        .mount("/", routes::routes())
        .catch(routes::errors())
        .launch();
}
