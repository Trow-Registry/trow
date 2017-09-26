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

/// given a directory, check all subdirectories exist
///
/// If needed subdirectories don't exist, create them. (possible extension)...
fn check_paths_exists(path: &Path) {
    let subdirectories = [
        "scratch",
        "layers",
    ];

    for dir in subdirectories.iter() {
        let path = path.join(dir);
        debug!("Checking {} subdirectory", path.display());
        match path.exists() {
            true => info!("subdirectory {} exists", path.display()),
            false => {
                panic!("{} does not exist try create it with: `mkdir {}`",
                       path.display(),
                       path.display());
            },
        };
    }

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

    check_paths_exists(Path::new("data"));

    Ok(extract_config(rocket))
}
