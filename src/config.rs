//! This module holds helpers for setting up the project
//! as well as data-structures for setting and maintaining the
//! system configuration.

use std;
use std::path::Path;
use std::fs;
use log;
use fern;
use ctrlc;
use rocket;

static DEFAULT_DATA_DIR: &'static str = "data";
static SCRATCH_DIR: &'static str = "scratch";
static LAYERS_DIR: &'static str = "layers";

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
    ctrlc::set_handler(|| {
        info!("SIGTERM caught, shutting down...");
        std::process::exit(127);
    }).expect("Error setting Ctrl-C handler");
}

/*
Creates needed directories under given path if they don't already exist.
*/
fn create_data_dirs(data_path: &Path) {
    let scratch_path = data_path.join(SCRATCH_DIR);

    if !scratch_path.exists() {
        debug!("Creating scratch dir {}", scratch_path.display());
        match fs::create_dir_all(&scratch_path) {
            Err(err) => panic!("Failed to create {}: {}", scratch_path.display(), err),
            Ok(res) => res,
        };
    }
    info!("Scratch directory set to {}", scratch_path.canonicalize().unwrap().display());

    let layers_path = data_path.join(LAYERS_DIR);
    if !layers_path.exists() {
        debug!("Creating layers dir {}", layers_path.display());
        match fs::create_dir_all(&layers_path) {
            Err(err) => panic!("Failed to create {}: {}", layers_path.display(), err),
            Ok(res) => res,
        };
    }
    info!("Layers directory set to {}", layers_path.canonicalize().unwrap().display());
    
}

/// extract configuration values
fn extract_config(rocket: rocket::Rocket) -> rocket::Rocket {
    let state: State;
    {
        let conf = &rocket.config();
        let address = &conf.address;
        let port = conf.port;
        state = State {
            address: address.clone(),
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
pub fn startup(rocket: rocket::Rocket) -> Result<rocket::Rocket, rocket::Rocket> {
    attach_sigterm();

    create_data_dirs(Path::new(DEFAULT_DATA_DIR));

    Ok(extract_config(rocket))
}
