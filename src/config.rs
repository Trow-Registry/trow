//! This module holds helpers for setting up the project
//! as well as data-structures for setting and maintaining the
//! system configuration.

use std;
use std::path::Path;
use log;
use fern;
use ctrlc;
use rocket;

/// This encapsulates any stateful data that needs to be preserved and
/// passed around during execution.
#[derive(Debug)]
pub struct State {
    pub address: String,
    pub port: u16,
}

/// Bulid the logging agent with formatting and the correct log-level.
///
/// The log-level is set using the `DEBUG` environment variable.
pub fn main_logger() -> fern::Dispatch {
    let level = match std::env::var("DEBUG") {
        Ok(_) => log::LogLevelFilter::Debug,
        Err(_) => log::LogLevelFilter::Info,
    };
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}] {}",
                record.target(),
                record.level(),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
}

/// Attaches SIGTERM handler
fn attach_sigterm() {
    ctrlc::set_handler(move || {
        info!("SIGTERM caught, shutting down...");
        std::process::exit(127);
    }).expect("Error setting Ctrl-C handler");
}

fn check_path_exists(path: &Path) {
    match path.exists() {
        true => info!("Using data directory: {}", path.display()),
        false => {
            panic!("Path {} does not exist", path.display());
        },
    };
}

/// extract configuration values
fn extract_config(rocket: rocket::Rocket) -> rocket::Rocket {
    let state: State;
    {
        let conf = &rocket.config();
        let address = &conf.address;
        let port = conf.port;
        state = State {
            address: String::from(address.clone()),
            port,
        };
    }
    debug!("{:?}", state);
    rocket.manage(state)
}

/// Handle all code relating to bootstrapping the project
///
/// - attach SIGTERM handler
/// - Check necessary paths exist
/// - Extract configuration values needed for runtime
pub fn startup(rocket: rocket::Rocket) -> Result<rocket::Rocket, rocket::Rocket>{
    attach_sigterm();

    check_path_exists(Path::new("data/scratch"));

    Ok(extract_config(rocket))
}
