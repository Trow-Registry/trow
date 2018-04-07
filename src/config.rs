//! This module should die and code moved to correct files

use std;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use ctrlc;
use env_logger;
use failure::Error;
use log::{LogLevelFilter, LogRecord, SetLoggerError};

use backend;
use grpc::backend_grpc::BackendClient;

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
        self.port
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct HttpConfig {
    listen: Service,
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

        s.merge(File::with_name(file))?;
        s.merge(Environment::with_prefix("trow"))?;

        s.try_into().map_err(|e| e.into())
    }

    pub fn default() -> Result<Self, Error> {
        TrowConfig::new("Trow.toml")
    }

    pub fn from_file(file: Result<String, Error>) -> Result<Self, Error> {
        file.map_err(|e| e) // this looks broken or redundant...
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
pub fn attach_sigterm() -> Result<(), Error> {
    ctrlc::set_handler(|| {
        info!("SIGTERM caught, shutting down...");
        std::process::exit(0);
    }).map_err(|e| e.into())
}

/// Creates needed directories under given path if they don't already exist.
///
pub fn create_data_dirs(data_path: &Path) -> Result<(), Error> {
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

pub fn build_handlers(listen_host: &str, listen_port: u16) -> BackendHandler {
    use grpcio::{ChannelBuilder, EnvBuilder};
    use std::sync::Arc;

    debug!(
        "Connecting to backend: {}:{}",
        listen_host,
        listen_port
    );
    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect(&format!("{}:{}", listen_host, listen_port));
    let client = BackendClient::new(ch);
    BackendHandler::new(client)
}






