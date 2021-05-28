#[macro_use]
extern crate failure;
extern crate base64;
extern crate futures;
extern crate hostname;
#[macro_use]
extern crate rocket;
extern crate argon2;
extern crate chrono;
extern crate data_encoding;
extern crate derive_more;
extern crate env_logger;
extern crate frank_jwt;
extern crate rand;
extern crate serde;
extern crate trow_server;
extern crate uuid;
use futures::Future;
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
use std::str::FromStr;
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
use fairings::conditional_fairing::AttachConditionalFairing;
use rand::RngCore;
use std::io::Write;

use rocket::http::Method;
use rocket_cors::AllowedHeaders;
use rocket_cors::AllowedOrigins;

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
    proxy_registry_config_dir: String,
    proxy_hub: bool,
    hub_user: Option<String>,
    hub_pass: Option<String>,
    allow_prefixes: Vec<String>,
    allow_images: Vec<String>,
    deny_prefixes: Vec<String>,
    deny_images: Vec<String>,
    dry_run: bool,
    max_manifest_size: u32,
    max_blob_size: u32,
    token_secret: String,
    user: Option<UserConfig>,
    cors: bool,
    log_level: String,
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

fn init_trow_server(
    config: TrowConfig,
) -> Result<impl Future<Output = Result<(), tonic::transport::Error>>, Error> {
    debug!("Starting Trow server");

    //Could pass full config here.
    //Pros: less work, new args added automatically
    //-s: ties frontend to backend, some uneeded/unwanted vars

    let ts = trow_server::build_server(
        &config.data_dir,
        config.grpc.listen.parse::<std::net::SocketAddr>()?,
        config.proxy_registry_config_dir,
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

    Ok(ts.get_server_future())
}

/// Build the logging agent with formatting.
fn init_logger(log_level: String) -> Result<(), SetLoggerError> {
    // If there env variable RUST_LOG is set, then take the configuration from it.
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
        .filter(None, LevelFilter::from_str(&log_level).unwrap());
    builder.init();
    Ok(())
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
        proxy_registry_config_dir: String,
        proxy_hub: bool,
        allow_prefixes: Vec<String>,
        allow_images: Vec<String>,
        deny_prefixes: Vec<String>,
        deny_images: Vec<String>,
        dry_run: bool,
        cors: bool,
        max_manifest_size: u32,
        max_blob_size: u32,
        log_level: String,
    ) -> TrowBuilder {
        let config = TrowConfig {
            data_dir,
            addr,
            tls: None,
            grpc: GrpcConfig { listen },
            host_names,
            proxy_registry_config_dir,
            proxy_hub,
            hub_user: None,
            hub_pass: None,
            allow_prefixes,
            allow_images,
            deny_prefixes,
            deny_images,
            dry_run,
            max_manifest_size,
            max_blob_size,
            token_secret: Uuid::new_v4().to_string(),
            user: None,
            cors,
            log_level,
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

        //TODO: with Rocket 0.5 should be able to pass our config file and let Rocket pick out the parts it wants
        //This will be simpler and allow more flexibility.
        let mut figment = rocket::Config::figment()
            .merge(("address", self.config.addr.host.clone()))
            .merge(("port", self.config.addr.port))
            .merge(("workers", 256))
            .merge(("secret_key", secret_key));

        if let Some(ref tls) = self.config.tls {
            if !(Path::new(&tls.cert_file).is_file() && Path::new(&tls.key_file).is_file()) {
                return  Err(format_err!("Trow requires a TLS certificate and key, but failed to find them. \nExpected to find TLS certificate at {} and key at {}", tls.cert_file, tls.key_file));
            }

            let tls_config =
                rocket::config::TlsConfig::from_paths(tls.cert_file.clone(), tls.key_file.clone());
            figment = figment.merge(("tls", tls_config));
        }
        let cfg = rocket::Config::from(figment);
        Ok(cfg)
    }

    pub fn start(&self) -> Result<(), Error> {
        init_logger(self.config.log_level.clone())?;

        let rocket_config = &self.build_rocket_config()?;
        println!(
            "Starting Trow {} on {}:{}",
            env!("CARGO_PKG_VERSION"),
            self.config.addr.host,
            self.config.addr.port
        );
        println!(
            "\nMaximum blob size: {} Mebibytes",
            self.config.max_blob_size
        );
        println!(
            "Maximum manifest size: {} Mebibytes",
            self.config.max_manifest_size
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
            println!("  Cross-Origin Resource Sharing(CORS) requests are allowed\n");
        }

        if self.config.dry_run {
            println!("Dry run, exiting.");
            std::process::exit(0);
        }
        let s = format!("https://{}", self.config.grpc.listen);
        let ci: ClientInterface = build_handlers(s)?;

        let cors = rocket_cors::CorsOptions {
            allowed_origins: AllowedOrigins::all(),
            allowed_methods: vec![Method::Get, Method::Post, Method::Options]
                .into_iter()
                .map(From::from)
                .collect(),
            allowed_headers: AllowedHeaders::some(&["Authorization", "Content-Type"]),
            allow_credentials: true,
            ..Default::default()
        }
        .to_cors()?;

        let f = rocket::custom(rocket_config.clone())
            .manage(self.config.clone())
            .manage(ci)
            .attach(fairing::AdHoc::on_response(
                "Set API Version Header",
                |_, resp| {
                    Box::pin(async move {
                        //Only serve v2. If we also decide to support older clients, this will to be dropped on some paths
                        resp.set_raw_header("Docker-Distribution-API-Version", "registry/2.0");
                    })
                },
            ))
            .attach(fairing::AdHoc::on_liftoff("Launch Message", |_| {
                Box::pin(async move {
                    println!("Trow is up and running!");
                })
            }))
            .attach_if(self.config.cors, cors)
            .mount("/", routes::routes())
            .register("/", routes::catchers())
            .launch();

        let rt = rocket::tokio::runtime::Builder::new_multi_thread()
            // NOTE: graceful shutdown depends on the "rocket-worker" prefix.
            .thread_name("rocket-worker-thread")
            .enable_all()
            .build()?;

        // Start GRPC Backend thread.
        rt.spawn(init_trow_server(self.config.clone())?);
        //And now rocket
        rt.block_on(f)?;

        Ok(())
    }
}

pub fn build_handlers(listen_addr: String) -> Result<ClientInterface, Error> {
    debug!("Address for backend: {}", listen_addr);

    //TODO this function is useless currently
    ClientInterface::new(listen_addr)
}
