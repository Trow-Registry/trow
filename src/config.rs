//! This module holds helpers for setting up the project
//! as well as data-structures for setting and maintaining the
//! system configuration.

use std;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::fs;

use clap;
use clap::{Arg, ArgMatches};
use env_logger;
use failure::Error;
use ctrlc;
use log::{LogLevelFilter, LogRecord, SetLoggerError};
use rocket;
use rocket::fairing;

use backend;
use grpc::backend_grpc::BackendClient;
use routes;

static DEFAULT_DATA_DIR: &'static str = "data";
static SCRATCH_DIR: &'static str = "scratch";
static LAYERS_DIR: &'static str = "layers";

const PROGRAM_NAME: &'static str = "Trow";
const PROGRAM_DESC: &'static str = "\nThe King of Registries";

/// This encapsulates any stateful data that needs to be preserved and
/// passed around during execution.
#[derive(Debug)]
pub struct Config {
    pub address: String,
    pub port: u16,
    pub console_port: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Service {
    host: String,
    port: u16,
}

impl Service {
    pub fn host(&self) -> String {
        self.host.clone()
    }

    pub fn port(&self) -> u16 {
        self.port.clone()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct HttpConfig {
    listen: Service,
}

impl HttpConfig {
    fn listen(&self) -> Service {
        self.listen.clone()
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct TlsConfig {
    use_tls: bool,
    certs: PathBuf,
    key: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TrowConfig {
    grpc: backend::config::TrowBackendConfig,
    web: HttpConfig,
    tls: TlsConfig, // I have no idea how to make this optional.
}

impl TrowConfig {
    pub fn new(file: &str) -> Result<Self, Error> {
        use cfg::{Config, Environment, File};
        let mut s = Config::new();

        s.merge(File::with_name(&file))?;
        s.merge(Environment::with_prefix("trow"))?;

        s.try_into().map_err(|e| e.into())
    }

    pub fn default() -> Result<Self, Error> {
        TrowConfig::new("Trow.toml")
    }

    pub fn from_file(file: Result<String, Error>) -> Result<Self, Error> {
        file.map_err(|e| e.into())
            .and_then(|file: String| TrowConfig::new(&file))
            .or_else(|_| {
                debug!("No config file specified, using default");
                TrowConfig::default()
            })
    }

    pub fn grpc(&self) -> backend::config::TrowBackendConfig {
        self.grpc.clone()
    }
}

//TODO: Make this take a cause or description
#[derive(Fail, Debug)]
#[fail(display = "invalid data directory")]
pub struct ConfigError {}

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

/// Build the logging agent with formatting.
pub fn main_logger() -> Result<(), SetLoggerError> {
    let mut builder = env_logger::LogBuilder::new();
    builder
        .format(|record: &LogRecord| {
            format!("{}[{}] {}", record.target(), record.level(), record.args(),)
        })
        .filter(None, LogLevelFilter::Error);

    if env::var("RUST_LOG").is_ok() {
        builder.parse(&env::var("RUST_LOG").unwrap());
    }

    builder.init()
}

/// Attaches SIGTERM handler
fn attach_sigterm() -> Result<(), Error> {
    ctrlc::set_handler(|| {
        info!("SIGTERM caught, shutting down...");
        std::process::exit(0);
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
        .map_err(|_| ConfigError {}.into())
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

pub struct BackendHandler {
    backend: BackendClient,
}

impl BackendHandler {
    fn new(backend: BackendClient) -> Self {
        BackendHandler { backend }
    }

    pub fn backend(&self) -> &BackendClient {
        &self.backend
    }
}

fn build_handlers(config: &TrowConfig) -> BackendHandler {
    use std::sync::Arc;
    use grpcio::{ChannelBuilder, EnvBuilder};

    let backend = config.grpc().listen();
    debug!(
        "Connecting to backend: {}:{}",
        backend.host(),
        backend.port()
    );
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(&format!("{}:{}", backend.host(), backend.port()));
    let client = BackendClient::new(ch);
    BackendHandler::new(client)
}

fn build_rocket_config(config: &TrowConfig) -> Result<rocket::config::Config, Error> {
    debug!("Config: {:?}", config.web);
    let bind = config.web.listen();
    let tls = &config.tls;
    let mut cfg = rocket::config::Config::build(rocket::config::Environment::Production)
        .address(bind.host())
        .port(bind.port());
    if tls.use_tls {
        match (tls.certs.to_str(), tls.key.to_str()) {
            (Some(cert), Some(key)) => if !cert.is_empty() && !key.is_empty() {
                cfg = cfg.tls(cert, key);
            },
            (_, _) => (),
        };
    }
    let cfg = cfg.finalize()?;
    Ok(cfg)
}

/// Construct the rocket instance and prepare for launch
pub(crate) fn rocket(args: &ArgMatches) -> Result<rocket::Rocket, Error> {
    let f = args.value_of("config");

    let config = match f {
        Some(v) => TrowConfig::new(&v)?,
        None => TrowConfig::default()?,
    };

    let rocket_config = build_rocket_config(&config)?;
    debug!("Config: {:?}", config);
    Ok(rocket::custom(rocket_config, true)
        .manage(build_handlers(&config))
        .manage(config)
        .attach(fairing::AdHoc::on_attach(startup))
        .attach(fairing::AdHoc::on_response(|_, resp| {
            //Only serve v2. If we also decide to support older clients, this will to be dropped on some paths
            resp.set_raw_header("Docker-Distribution-API-Version", "registry/2.0");
        }))
        .mount("/", routes::routes()))
}

/*
  Parses command line arguments and returns ArgMatches object.
*/
pub fn parse_args<'a>() -> ArgMatches<'a> {
    clap::App::new(PROGRAM_NAME)
        .version("0.1")
        .author("From Container Solutions")
        .about(PROGRAM_DESC)
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true),
        )
        .get_matches()
}
