//! This module holds helpers for setting up the project
//! as well as data-structures for setting and maintaining the
//! system configuration.

use getopts::Occur;
use std;
use std::path::Path;
use std::sync::mpsc;
use std::fs;

use args::Args;
use failure::Error;
use fern;
use ctrlc;
use rocket;
use rocket::fairing;

use backend;
use errors;
use grpc::backend_grpc::PeerClient;
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
pub struct LycaonConfig {
    grpc: backend::config::LycaonBackendConfig,
    web: HttpConfig,
}

impl LycaonConfig {
    pub fn new(file: String) -> Result<Self, Error> {
        use cfg::{Config, Environment, File};
        let mut s = Config::new();

        s.merge(File::with_name(&file))?;
        s.merge(Environment::with_prefix("lycaon"))?;

        s.try_into().map_err(|e| e.into())
    }

    pub fn default() -> Result<Self, Error> {
        LycaonConfig::new("Lycaon.toml".to_owned())
    }

    pub fn from_file(file: Result<String, Error>) -> Result<Self, Error> {
        file.map_err(|e| e.into())
            .and_then(|file: String| LycaonConfig::new(file))
            .or_else(|err| {
                debug!("No config file specified, using default");
                LycaonConfig::default()
            })
    }

    pub fn grpc(&self) -> backend::config::LycaonBackendConfig {
        self.grpc.clone()
    }
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

/// Build the logging agent with formatting.
pub fn main_logger() -> fern::Dispatch {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}] {}",
                record.target(),
                record.level(),
                message
            ))
        })
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

pub struct PeerHandler {
    peers: Vec<PeerClient>,
}

impl PeerHandler {
    fn new(peers: Vec<PeerClient>) -> Self {
        PeerHandler { peers }
    }

    pub fn peers(&self) -> &Vec<PeerClient> {
        &self.peers
    }
}

fn build_handlers(config: &LycaonConfig) -> PeerHandler {
    use grpc;
    use protobuf;
    use std::sync::Arc;

    use grpcio::{ChannelBuilder, EnvBuilder};
    use grpc::backend;

    let env = Arc::new(EnvBuilder::new().build());
    let ch = ChannelBuilder::new(env).connect("127.0.0.1:50055");
    let client = PeerClient::new(ch);
    PeerHandler::new(vec![client])
}

fn build_rocket_config(config: &LycaonConfig) -> rocket::config::Config {
    debug!("Config: {:?}", config.web);
    let bind = config.web.listen();
    rocket::config::Config::build(rocket::config::Environment::Production)
        .address(bind.host())
        .port(bind.port())
        .finalize()
        .expect("Error building Rocket Config")
}

/// Construct the rocket instance and prepare for launch
pub(crate) fn rocket(handler: SocketHandler, args: Args) -> Result<rocket::Rocket, Error> {
    let config = args.value_of("config")
        .map_err(|e| e.into())
        .and_then(|file| LycaonConfig::new(file))
        .or_else(|e| LycaonConfig::default())?;

    let rocket_config = build_rocket_config(&config);
    debug!("Config: {:?}", config);
    Ok(
        rocket::custom(rocket_config, true)
            .manage(handler)
            .manage(build_handlers(&config))
            .manage(config)
            .attach(fairing::AdHoc::on_attach(startup))
            .mount("/", routes::routes())
            .catch(routes::errors()),
    )
}

const PROGRAM_NAME: &'static str = "Lycaon";
const PROGRAM_DESC: &'static str = "The King of Registries";

pub fn parse_args() -> Result<Args, Error> {
    let mut args = Args::new(PROGRAM_NAME, PROGRAM_DESC);

    args.flag("h", "help", "print usage information");
    args.option(
        "c",
        "config",
        "config file",
        "FILE",
        Occur::Optional,
        Some(String::from("Lycaon.toml")),
    );

    debug!("Parsing Arguments from CLI");
    args.parse_from_cli()?;
    if args.value_of("help")? {
        println!("{}", args.full_usage());
        use std;
        return Err(Error::from(std::fmt::Error));
    }

    debug!("Args, all good");
    Ok(args)
}
