//! This module holds helpers for setting up the project
//! as well as data-structures for setting and maintaining the
//! system configuration.

use std;
use std::path::Path;
use std::sync::mpsc;
use std::fs;
use log;
use failure::Error;
use fern;
use ctrlc;
use rocket;
use rocket::fairing;

use errors;
use routes;

static DEFAULT_DATA_DIR: &'static str = "data";
static SCRATCH_DIR: &'static str = "scratch";
static LAYERS_DIR: &'static str = "layers";

/// This encapsulates any stateful data that needs to be preserved and
/// passed around during execution.
#[derive(Debug)]
pub struct Config {
    pub address: String,
    pub port: u16,
    pub console_port: i64,
}

#[derive(Debug)]
pub enum Backend {
    Test,
}
#[derive(Debug)]
pub enum Frontend {
    TestResponse,
}

#[derive(Debug)]
pub enum BackendMessage {
    Backend(Backend),
    Frontend(Frontend),
}

pub type SendSock = mpsc::Sender<BackendMessage>;
pub type RecvSock = mpsc::Receiver<BackendMessage>;

#[derive(Debug)]
pub struct SocketHandler {
    tx: SendSock,
    rx: RecvSock,
}
unsafe impl Sync for SocketHandler {}

impl SocketHandler {
    pub fn new(tx: SendSock, rx: RecvSock) -> SocketHandler {
        SocketHandler { tx, rx }
    }

    pub fn tx(&self) -> SendSock {
        self.tx.clone()
    }

    pub fn rx(&self) -> &RecvSock {
        &self.rx
    }
}

/// Build the logging agent with formatting and the correct log-level.
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
fn attach_sigterm() -> Result<(), Error> {
    ctrlc::set_handler(|| {
        info!("SIGTERM caught, shutting down...");
        std::process::exit(127);
    }).map_err(|e| e.into())
}

/// Creates needed directories under given path if they don't already exist.
///
fn create_data_dirs(data_path: &Path) -> Result<(), Error> {
    fn setup_path(path: std::path::PathBuf) -> Result<(), Error> {
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }
        Ok(())
    }

    let scratch_path = data_path.join(SCRATCH_DIR);
    let layers_path = data_path.join(LAYERS_DIR);
    setup_path(scratch_path)
        .and(setup_path(layers_path))
        .map_err(|e| errors::Server::ConfigError(e).into())
}


/// extract configuration values
///
pub(crate) fn extract_config(conf: &rocket::Config) -> Result<Config, Error> {
    let address = &conf.address;
    let port = conf.port;
    let console_port = match conf.get_int("console_port") {
        Ok(x) => x,
        Err(_) => 29999,
    };
    Ok(Config {
        address: address.clone(),
        port,
        console_port,
    })
}

/// Handle all code relating to bootstrapping the project
///
/// - attach SIGTERM handler
/// - Check necessary paths exist
/// - Extract configuration values needed for runtime
fn startup(rocket: rocket::Rocket) -> Result<rocket::Rocket, rocket::Rocket> {
    attach_sigterm()
        .and(create_data_dirs(Path::new(DEFAULT_DATA_DIR)))
        .and(extract_config(rocket.config()))
        .and_then(|config| Ok(rocket.manage(config)))
        .map_err(|e| panic!("{}", e))
}

/// Construct the rocket instance and prepare for launch
pub(crate) fn rocket(handler: SocketHandler) -> rocket::Rocket {
    rocket::ignite()
        .manage(handler)
        .attach(fairing::AdHoc::on_attach(startup))
        .mount("/", routes::routes())
        .catch(routes::errors())
}
