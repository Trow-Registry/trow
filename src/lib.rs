#![feature(proc_macro_hygiene, decl_macro)]
#![feature(plugin)]
#![feature(seek_convenience)]

#[macro_use]
extern crate failure;
extern crate base64;
extern crate frank_jwt;
extern crate futures;
extern crate hostname;
extern crate orset;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate rustc_serialize;
extern crate serde;
extern crate serde_json;
extern crate uuid;
#[macro_use]
extern crate display_derive;

extern crate trow_server;

extern crate argon2;
extern crate chrono;
extern crate crypto;
extern crate env_logger;

use log::{LogLevelFilter, LogRecord, SetLoggerError};
#[macro_use]
extern crate failure_derive;
#[macro_use(log, warn, info, debug)]
extern crate log;
#[macro_use]
extern crate serde_derive;
extern crate rand;

#[cfg(test)]
extern crate quickcheck;

use failure::Error;
use rand::Rng;
use std::env;
use std::fs;
use std::path::Path;
use std::thread;
use uuid::Uuid;

use rocket::fairing;

mod client_interface;
pub mod response;
mod routes;
pub mod types;
mod users;
use client_interface::ClientInterface;

//TODO: Make this take a cause or description
#[derive(Fail, Debug)]
#[fail(display = "invalid data directory")]
pub struct ConfigError {}

#[derive(Clone, Debug)]
pub struct NetAddr {
    pub host: String,
    pub port: u16,
}

/*
 * Configuration for Trow. This isn't direct fields on the builder so that we can pass it
 * to Rocket to manage.
 */
#[derive(Clone, Debug)]
pub struct TrowConfig {
    data_dir: String,
    addr: NetAddr,
    tls: Option<TlsConfig>,
    grpc: GrpcConfig,
    host_names: Vec<String>,
    allow_prefixes: Vec<String>,
    allow_images: Vec<String>,
    deny_prefixes: Vec<String>,
    deny_images: Vec<String>,
    dry_run: bool,
    token_secret: String,
    user: Option<UserConfig>,
}

#[derive(Clone, Debug)]
struct GrpcConfig {
    listen: String,
}

#[derive(Clone, Debug)]
struct TlsConfig {
    cert_file: String,
    key_file: String,
}

#[derive(Clone, Debug)]
struct UserConfig {
    user: String,
    hash_encoded: String, //Surprised not bytes
}

fn init_trow_server(config: TrowConfig) -> Result<std::thread::JoinHandle<()>, Error> {
    debug!("Starting Trow server");

    //Could pass full config here.
    //Pros: less work, new args added automatically
    //-s: ties frontend to backend, some uneeded/unwanted vars

    let ts = trow_server::build_server(
        &config.data_dir,
        config.grpc.listen.parse::<std::net::SocketAddr>()?,
        config.allow_prefixes,
        config.allow_images,
        config.deny_prefixes,
        config.deny_images,
    );
    //TODO: probably shouldn't be reusing this cert
    let ts = if let Some(tls) = config.tls {
        ts.add_tls(fs::read(tls.cert_file)?, fs::read(tls.key_file)?)
    } else {
        ts
    };

    Ok(thread::spawn(move || {
        ts.start_trow_sync();
    }))
}

/// Build the logging agent with formatting.
fn init_logger() -> Result<(), SetLoggerError> {
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

pub struct TrowBuilder {
    config: TrowConfig,
}

impl TrowBuilder {
    pub fn new(
        data_dir: String,
        addr: NetAddr,
        listen: String,
        host_names: Vec<String>,
        allow_prefixes: Vec<String>,
        allow_images: Vec<String>,
        deny_prefixes: Vec<String>,
        deny_images: Vec<String>,
        dry_run: bool,
    ) -> TrowBuilder {
        let config = TrowConfig {
            data_dir,
            addr,
            tls: None,
            grpc: GrpcConfig { listen },
            host_names,
            allow_prefixes,
            allow_images,
            deny_prefixes,
            deny_images,
            dry_run,
            token_secret: Uuid::new_v4().to_string(),
            user: None,
        };
        TrowBuilder { config }
    }

    pub fn with_tls(&mut self, cert_file: String, key_file: String) -> &mut TrowBuilder {
        let cfg = TlsConfig {
            cert_file,
            key_file,
        };
        self.config.tls = Some(cfg);
        self
    }

    pub fn with_user(&mut self, user: String, pass: String) -> &mut TrowBuilder {
        let hash_config = argon2::Config::default();
        let hash_encoded =
            argon2::hash_encoded(pass.as_bytes(), Uuid::new_v4().as_bytes(), &hash_config)
                .expect("Error hashing password");
        let usercfg = UserConfig { user, hash_encoded };
        self.config.user = Some(usercfg);
        self
    }

    fn build_rocket_config(&self) -> Result<rocket::config::Config, Error> {
        // When run in production, Rocket wants a secret key for private cookies.
        // As we don't use private cookies, we just generate it here.
        let mut rng = rand::thread_rng();
        let mut key = [0u8; 32];
        rng.fill(&mut key[..]);
        let skey = base64::encode(&key);

        /*
        Keep Alive has to be turned off to mitigate issue whereby some transfers would be cut off.
        Seems to be caused by an old version of hyper in Rocket.
        Keep Alive can be restored when Rocket is upgraded or by moving to different framework.
        See #24
        */
        let mut cfg = rocket::config::Config::build(rocket::config::Environment::Production)
            .address(self.config.addr.host.clone())
            .port(self.config.addr.port)
            .keep_alive(0) // Needed to mitigate #24. See above.
            .secret_key(skey)
            .workers(256);

        if let Some(ref tls) = self.config.tls {
            if !(Path::new(&tls.cert_file).is_file() && Path::new(&tls.key_file).is_file()) {
                return  Err(format_err!("Trow requires a TLS certificate and key, but failed to find them. \nExpected to find TLS certificate at {} and key at {}", tls.cert_file, tls.key_file));
            }
            cfg = cfg.tls(tls.cert_file.clone(), tls.key_file.clone());
        }
        let cfg = cfg.finalize()?;
        Ok(cfg)
    }

    pub fn start(&self) -> Result<(), Error> {
        init_logger()?;

        let rocket_config = &self.build_rocket_config()?;

        // Start GRPC Backend thread.
        let _backend_thread = init_trow_server(self.config.clone())?;

        println!(
            "Starting Trow {} on {}:{}",
            env!("CARGO_PKG_VERSION"),
            self.config.addr.host,
            self.config.addr.port
        );
        println!("\n**Validation callback configuration\n");

        println!("  By default all remote images are denied, and all local images present in the repository are allowed\n");

        println!(
            "  These host names will considered local (refer to this registry): {:?}",
            self.config.host_names
        );
        println!(
            "  Images with these prefixes are explicitly allowed: {:?}",
            self.config.allow_prefixes
        );
        println!(
            "  Images with these names are explicitly allowed: {:?}",
            self.config.allow_images
        );
        println!(
            "  Local images with these prefixes are explicitly denied: {:?}",
            self.config.deny_prefixes
        );
        println!(
            "  Local images with these names are explicitly denied: {:?}\n",
            self.config.deny_images
        );
        if self.config.dry_run {
            println!("Dry run, exiting.");
            std::process::exit(0);
        }
        let s = format!("https://{}", self.config.grpc.listen);
        let ci: ClientInterface = build_handlers(s)?;

        rocket::custom(rocket_config.clone())
            .manage(self.config.clone())
            .manage(ci)
            .attach(fairing::AdHoc::on_attach(
                "SIGTERM handler",
                |r| match attach_sigterm() {
                    Ok(_) => Ok(r),
                    Err(_) => Err(r),
                },
            ))
            .attach(fairing::AdHoc::on_response(
                "Set API Version Header",
                |_, resp| {
                    //Only serve v2. If we also decide to support older clients, this will to be dropped on some paths
                    resp.set_raw_header("Docker-Distribution-API-Version", "registry/2.0");
                },
            ))
            .attach(fairing::AdHoc::on_launch("Launch Message", |_| {
                println!("Trow is up and running!");
            }))
            .mount("/", routes::routes())
            .register(routes::catchers())
            .launch();

        Ok(())
    }
}

fn attach_sigterm() -> Result<(), Error> {
    ctrlc::set_handler(|| {
        info!("SIGTERM caught, shutting down...");
        std::process::exit(0);
    })
    .map_err(|e| e.into())
}

pub fn build_handlers(listen_addr: String) -> Result<ClientInterface, Error> {
    debug!("Address for backend: {}", listen_addr);

    //TODO this function is useless currently
    ClientInterface::new(listen_addr)
}
