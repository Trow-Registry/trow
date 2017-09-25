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
extern crate uuid;
extern crate ring;

mod errors;
mod logging;
mod routes;
pub mod response;
pub mod config;

// use slog::Drain;
use rocket::fairing;

fn main() {
    ctrlc::set_handler(move || {
        info!("SIGTERM caught, shutting down...");
        std::process::exit(127);
    }).expect("Error setting Ctrl-C handler");

    // let decorator = slog_term::TermDecorator::new().build();
    // let drain = slog_term::FullFormat::new(decorator).build().fuse();
    // let drain = slog_async::Async::new(drain).build().fuse();
    // let _log = slog::Logger::root(drain, o!());

    let _log = logging::main_logger().apply();
    rocket::ignite()
        .attach(fairing::AdHoc::on_attach(|rocket| {
            let state: config::State;
            {
                let conf = &rocket.config();
                let address = &conf.address;
                let port = conf.port;
                state = config::State {
                    address: String::from(address.clone()),
                    port,
                };
            }
            debug!("{:?}", state);
            Ok(rocket.manage(state))
        }))
        .mount("/", routes::routes())
        .catch(routes::errors())
        .launch();
}
