//! Root level documentation

#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate chrono;
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
mod response;

/// Some documentation for the main.rs file
fn main() {

    let _log = logging::main_logger().apply();
    rocket::ignite()
        .mount("/", routes::routes())
        .catch(routes::errors())
        .launch();
}
