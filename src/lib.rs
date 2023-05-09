mod client_interface;
mod fairings;

pub mod response;
#[allow(clippy::too_many_arguments)]
mod routes;
pub mod types;

mod registry_interface;
#[cfg(feature = "sqlite")]
mod users;

use anyhow::Context;
use futures::Future;
use log::{LevelFilter, SetLoggerError};
use std::io::Write;

use anyhow::{anyhow, Result};
use base64::{engine::general_purpose as base64_engine, Engine as _};
use chrono::Utc;
use log::debug;
use rand::rngs::OsRng;
use rand::RngCore;
use rocket::fairing;
use rocket::http::Method;
use rocket_cors::AllowedHeaders;
use rocket_cors::AllowedOrigins;
use std::env;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

use client_interface::ClientInterface;
use fairings::conditional_fairing::AttachConditionalFairing;

use trow_server::{ImageValidationConfig, RegistryProxyConfig};

//TODO: Make this take a cause or description
#[derive(Error, Debug)]
#[error("invalid data directory")]
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
    service_name: String,
    proxy_registry_config: Vec<RegistryProxyConfig>,
    image_validation_config: Option<ImageValidationConfig>,
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
) -> Result<impl Future<Output = Result<(), tonic::transport::Error>>> {
    debug!("Starting Trow server");

    //Could pass full config here.
    //Pros: less work, new args added automatically
    //-s: ties frontend to backend, some uneeded/unwanted vars

    let ts = trow_server::build_server(
        &config.data_dir,
        config.grpc.listen.parse::<std::net::SocketAddr>()?,
        config.proxy_registry_config,
        config.image_validation_config,
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
        service_name: String,
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
            service_name,
            proxy_registry_config: Vec::new(),
            image_validation_config: None,
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

    pub fn with_proxy_registries(&mut self, config_file: impl AsRef<str>) -> Result<&mut Self> {
        let config_file = config_file.as_ref();
        let config_str = fs::read_to_string(config_file)
            .with_context(|| format!("Could not read file `{}`", config_file))?;
        let config = serde_yaml::from_str::<Vec<RegistryProxyConfig>>(&config_str)
            .with_context(|| format!("Could not parse file `{}`", config_file))?;
        self.config.proxy_registry_config = config;
        Ok(self)
    }

    pub fn with_image_validation(&mut self, config_file: impl AsRef<str>) -> Result<&mut Self> {
        let config_file = config_file.as_ref();
        let config_str = fs::read_to_string(config_file)
            .with_context(|| format!("Could not read file `{}`", config_file))?;
        let config = serde_yaml::from_str::<ImageValidationConfig>(&config_str)
            .with_context(|| format!("Could not parse file `{}`", config_file))?;
        self.config.image_validation_config = Some(config);
        Ok(self)
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

    fn build_rocket_config(&self) -> Result<rocket::config::Config> {
        // When run in production, Rocket wants a secret key for private cookies.
        // As we don't use private cookies, we just generate it here.
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        let secret_key = base64_engine::STANDARD_NO_PAD.encode(key);

        //TODO: with Rocket 0.5 should be able to pass our config file and let Rocket pick out the parts it wants
        //This will be simpler and allow more flexibility.
        let mut figment = rocket::Config::figment()
            .merge(("address", self.config.addr.host.clone()))
            .merge(("port", self.config.addr.port))
            .merge(("workers", 256))
            .merge(("secret_key", secret_key));

        if let Some(ref tls) = self.config.tls {
            if !(Path::new(&tls.cert_file).is_file() && Path::new(&tls.key_file).is_file()) {
                return  Err(anyhow!("Trow requires a TLS certificate and key, but failed to find them. \nExpected to find TLS certificate at {} and key at {}", tls.cert_file, tls.key_file));
            }

            let tls_config =
                rocket::config::TlsConfig::from_paths(tls.cert_file.clone(), tls.key_file.clone());
            figment = figment.merge(("tls", tls_config));
        }
        let cfg = rocket::Config::from(figment);
        Ok(cfg)
    }

    pub fn start(&self) -> Result<()> {
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

        println!(
            "Hostname of this registry (for the MutatingWebhook): {:?}",
            self.config.service_name
        );
        match self.config.image_validation_config {
            Some(ref config) => {
                println!("Image validation webhook configured:");
                println!("  Default action: {}", config.default);
                println!("  Allowed prefixes: {:?}", config.allow);
                println!("  Denied prefixes: {:?}", config.deny);
            }
            None => println!("Image validation webhook not configured"),
        }
        if !self.config.proxy_registry_config.is_empty() {
            println!("Proxy registries configured:");
            for config in &self.config.proxy_registry_config {
                println!("  - {}: {}", config.alias, config.host);
            }
        } else {
            println!("Proxy registries not configured");
        }

        if self.config.cors {
            println!("Cross-Origin Resource Sharing(CORS) requests are allowed\n");
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
        _ = rt.block_on(f)?;

        Ok(())
    }
}

pub fn build_handlers(listen_addr: String) -> Result<ClientInterface> {
    debug!("Address for backend: {}", listen_addr);

    //TODO this function is useless currently
    ClientInterface::new(listen_addr)
}
