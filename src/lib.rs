#![feature(proc_macro_hygiene, decl_macro)]
#![feature(plugin)]
#![feature(seek_stream_len)]

#[macro_use]
extern crate failure;
extern crate base64;
extern crate futures;
extern crate hostname;
extern crate hyper;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate argon2;
extern crate chrono;
extern crate data_encoding;
extern crate derive_more;
extern crate env_logger;
extern crate frank_jwt;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate trow_server;
extern crate uuid;
use env_logger::Env;
use log::{LevelFilter, SetLoggerError};

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate quickcheck;

use failure::Error;
use rand::rngs::OsRng;
use rocket::fairing;
use std::env;
use std::fs;
use std::path::Path;
use std::thread;
use uuid::Uuid;

mod client_interface;
mod fairings;

pub mod response;
#[allow(clippy::too_many_arguments)]
mod routes;
pub mod types;

mod registry_interface;
#[cfg(feature = "sqlite")]
mod users;

use chrono::Utc;
use client_interface::ClientInterface;
use fairings::{conditional_fairing::AttachConditionalFairing, cors::CORS};
use rand::RngCore;
use std::io::Write;

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
    proxy_hub: bool,
    hub_user: Option<String>,
    hub_pass: Option<String>,
    allow_prefixes: Vec<String>,
    allow_images: Vec<String>,
    deny_prefixes: Vec<String>,
    deny_images: Vec<String>,
    dry_run: bool,
    token_secret: String,
    user: Option<UserConfig>,
    cors: bool,
    allow_cors_origin: String,
    allow_cors_headers: Vec<String>,
    allow_cors_methods: Vec<String>,
    allow_cors_credentials: bool,
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
        config.proxy_hub,
        config.hub_user,
        config.hub_pass,
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
    // If there env variable RUST_LOG is set, then take the configuration from it.
    if env::var("RUST_LOG").is_ok() {
        env_logger::from_env(Env::default().default_filter_or("error")).init();
        Ok(())
    } else {
        // Otherwise create a default logger
        let mut builder = env_logger::Builder::new();
        builder
            .format(|buf, record| {
                writeln!(
                    buf,
                    "{} [{}] {} {}",
                    Utc::now().format("%Y-%m-%dT%H:%M:%S"),
                    record.target(),
                    record.level(),
                    record.args()
                )
            })
            .filter(None, LevelFilter::Error);
        builder.init();
        Ok(())
    }
}

pub struct TrowBuilder {
    config: TrowConfig,
}

impl TrowBuilder {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        data_dir: String,
        addr: NetAddr,
        listen: String,
        host_names: Vec<String>,
        proxy_hub: bool,
        allow_prefixes: Vec<String>,
        allow_images: Vec<String>,
        deny_prefixes: Vec<String>,
        deny_images: Vec<String>,
        dry_run: bool,
        cors: bool,
        allow_cors_origin: String,
        allow_cors_headers: Vec<String>,
        allow_cors_methods: Vec<String>,
        allow_cors_credentials: bool,
    ) -> TrowBuilder {
        let config = TrowConfig {
            data_dir,
            addr,
            tls: None,
            grpc: GrpcConfig { listen },
            host_names,
            proxy_hub,
            hub_user: None,
            hub_pass: None,
            allow_prefixes,
            allow_images,
            deny_prefixes,
            deny_images,
            dry_run,
            token_secret: Uuid::new_v4().to_string(),
            user: None,
            cors,
            allow_cors_origin,
            allow_cors_headers,
            allow_cors_methods,
            allow_cors_credentials,
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

    pub fn with_hub_auth(&mut self, hub_user: String, token: String) -> &mut TrowBuilder {
        self.config.hub_pass = Some(token);
        self.config.hub_user = Some(hub_user);
        self
    }

    fn build_rocket_config(&self) -> Result<rocket::config::Config, Error> {
        // When run in production, Rocket wants a secret key for private cookies.
        // As we don't use private cookies, we just generate it here.
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        let secret_key = base64::encode(&key);

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
            .secret_key(secret_key)
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
            "  These host names will be considered local (refer to this registry): {:?}",
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

        if self.config.proxy_hub {
            println!("  Docker Hub repostories are being proxy-cached under f/docker/\n");
        }

        if self.config.cors {
            println!("  Cross-Origin Resource Sharing(CORS) requests are allowed");
            println!(
                "  Allowed Cross-Origin Resource Sharing(CORS) origin is {:?}",
                self.config.allow_cors_origin
            );
            println!(
                "  Allowed Cross-Origin Resource Sharing(CORS) methods are {:?}",
                self.config.allow_cors_methods
            );
            println!(
                "  Allowed Cross-Origin Resource Sharing(CORS) headers are {:?}",
                self.config.allow_cors_headers
            );
            println!(
                "  Allow Cross-Origin Resource Sharing(CORS) credentials is {:?}\n",
                self.config.allow_cors_credentials
            );
        }

        if self.config.dry_run {
            println!("Dry run, exiting.");
            std::process::exit(0);
        }
        let s = format!("https://{}", self.config.grpc.listen);
        let ci: ClientInterface = build_handlers(s)?;

        let cors = CORS::new()
            .methods(self.config.allow_cors_methods.clone())
            .origin(self.config.allow_cors_origin.clone())
            .headers(self.config.allow_cors_headers.clone())
            .credentials(Some(self.config.allow_cors_credentials.clone()))
            .build();

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
            .attach_if(self.config.cors, cors)
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
