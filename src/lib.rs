#![feature(proc_macro_hygiene, decl_macro)]
#![feature(plugin)]
#![feature(seek_stream_len)]

#[macro_use]
extern crate failure;
extern crate base64;
extern crate futures;
extern crate hostname;
extern crate hyper;
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
extern crate actix_web;
extern crate actix_utils;
extern crate actix_http;
extern crate rustls;
extern crate pin_project;

use env_logger::Env;
use log::{LevelFilter, SetLoggerError};

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

#[cfg(test)]
extern crate quickcheck;

use failure::Error;
use std::env;
use std::fs;
use std::path::Path;
use std::io::BufReader;
use std::fs::File;
use std::thread;
use uuid::Uuid;

mod client_interface;
pub mod response;
#[allow(clippy::too_many_arguments)]
mod routes;
pub mod types;

mod registry_interface;
#[cfg(feature = "sqlite")]
mod users;
mod auth_middleware;

use chrono::Utc;
use client_interface::ClientInterface;
use std::io::Write;

use actix_web::{App, HttpServer, middleware::{self, Logger}, web};
use rustls::internal::pemfile::{certs, pkcs8_private_keys};
use rustls::{NoClientAuth, ServerConfig};

use crate::auth_middleware::AuthGuard;

//TODO: Make this take a cause or description
#[derive(Fail, Debug)]
#[fail(display = "invalid data directory")]
pub struct ConfigError {}

#[derive(Clone, Debug)]
pub struct NetAddr {
    pub host: String,
    pub port: u16,
}

impl NetAddr {
    pub fn as_pair(&self) -> (&str, u16) {
        (self.host.as_str(), self.port)
    }
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

    pub fn start(&self) -> Result<(), Error> {

        let sys = actix_web::rt::System::new();
       
        init_logger().unwrap();


        // Start GRPC Backend thread.
        let _backend_thread = init_trow_server(self.config.clone()).unwrap();

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
            println!("Docker Hub repostories are being proxy-cached under f/docker/\n");
        }
        if self.config.dry_run {
            println!("Dry run, exiting.");
            std::process::exit(0);
        }
        //let s = format!("https://{}", self.config.grpc.listen);

        //TODO: figure out how to attach ci
        //let _ci: ClientInterface = build_handlers(s).unwrap();

        let mut hs = HttpServer::new(|| {
            App::new()
                .wrap(Logger::default())
                .wrap(middleware::DefaultHeaders::new().header("Docker-Distribution-API-Version", "registry/2.0"))
                .service(web::scope("/v2").configure(routes::registry_config))
                .configure(routes::homepage)
        });

        hs = match self.config.tls {
            Some(ref tls) => {
                if !(Path::new(&tls.cert_file).is_file() && Path::new(&tls.key_file).is_file()) {
                    return  Err(format_err!("Trow requires a TLS certificate and key, but failed to find them. \nExpected to find TLS certificate at {} and key at {}", tls.cert_file, tls.key_file));
                }

                let mut tls_config = ServerConfig::new(NoClientAuth::new());
                let cert_file = &mut BufReader::new(File::open(&tls.cert_file)?);
                let key_file = &mut BufReader::new(File::open(&tls.key_file)?);
                let cert_chain  = certs(cert_file).map_err(
                    |_| format_err!("Error reading TLS cert")
                )?;
	            let mut keys: Vec<rustls::PrivateKey> = pkcs8_private_keys(key_file).map_err(
                    |_| format_err!("Error reading TLS key")
                )?;
	            tls_config.set_single_cert(cert_chain, keys.remove(0))?;

                hs.bind_rustls(self.config.addr.as_pair(), tls_config)?
            }
            None => hs.bind(self.config.addr.as_pair())?,
        };
        
        sys.block_on(async move {
            println!("Trow is up and running");
            hs.run().await}
        )?;

        Ok(())
        
    }
}

pub fn build_handlers(listen_addr: String) -> Result<ClientInterface, Error> {
    debug!("Address for backend: {}", listen_addr);

    //TODO this function is useless currently
    ClientInterface::new(listen_addr)
}
