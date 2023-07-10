mod client_interface;

pub mod response;
#[allow(clippy::too_many_arguments)]
mod routes;
pub mod types;

mod registry_interface;
#[cfg(feature = "sqlite")]
mod users;

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::{env, fs};

use anyhow::{anyhow, Context, Result};
use axum::extract::FromRef;
use axum_server::tls_rustls::RustlsConfig;
use client_interface::ClientInterface;
use futures::Future;
use thiserror::Error;
use tracing::{event, Level};
use trow_server::{ImageValidationConfig, RegistryProxiesConfig};
use uuid::Uuid;

//TODO: Make this take a cause or description
#[derive(Error, Debug)]
#[error("invalid data directory")]
pub struct ConfigError {}

#[derive(Clone, Debug)]
pub struct NetAddr {
    pub host: String,
    pub port: u16,
}

#[derive(Debug)]
pub struct TrowServerState {
    pub client: ClientInterface,
    pub config: TrowConfig,
}

impl FromRef<Arc<TrowServerState>> for TrowConfig {
    fn from_ref(state: &Arc<TrowServerState>) -> Self {
        state.config.clone()
    }
}

/*
 * Configuration for Trow. This isn't direct fields on the builder so that we can pass it
 * to Rocket to manage.
 */
#[derive(Clone, Debug)]
pub struct TrowConfig {
    data_dir: String,
    addr: SocketAddr,
    tls: Option<TlsConfig>,
    grpc: GrpcConfig,
    service_name: String,
    proxy_registry_config: Option<RegistryProxiesConfig>,
    image_validation_config: Option<ImageValidationConfig>,
    dry_run: bool,
    token_secret: String,
    user: Option<UserConfig>,
    cors: Option<Vec<String>>,
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
    event!(Level::DEBUG, "Starting Trow server");

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

pub struct TrowBuilder {
    config: TrowConfig,
}

impl TrowBuilder {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        data_dir: String,
        addr: SocketAddr,
        listen: String,
        service_name: String,
        dry_run: bool,
        cors: Option<Vec<String>>,
    ) -> TrowBuilder {
        let config = TrowConfig {
            data_dir,
            addr,
            tls: None,
            grpc: GrpcConfig { listen },
            service_name,
            proxy_registry_config: None,
            image_validation_config: None,
            dry_run,
            token_secret: Uuid::new_v4().to_string(),
            user: None,
            cors,
        };
        TrowBuilder { config }
    }

    pub fn with_proxy_registries(&mut self, config_file: impl AsRef<str>) -> Result<&mut Self> {
        let config_file = config_file.as_ref();
        let config_str = fs::read_to_string(config_file)
            .with_context(|| format!("Could not read file `{}`", config_file))?;
        let config = serde_yaml::from_str::<RegistryProxiesConfig>(&config_str)
            .with_context(|| format!("Could not parse file `{}`", config_file))?;
        self.config.proxy_registry_config = Some(config);
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

    pub async fn start(&self) -> Result<()> {
        println!(
            "Starting Trow {} on {}",
            env!("CARGO_PKG_VERSION"),
            self.config.addr
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
        if let Some(proxy_config) = &self.config.proxy_registry_config {
            println!("Proxy registries configured:");
            for config in &proxy_config.registries {
                println!("  - {}: {}", config.alias, config.host);
            }
        } else {
            println!("Proxy registries not configured");
        }

        if self.config.cors.is_some() {
            println!("Cross-Origin Resource Sharing(CORS) requests are allowed\n");
        }

        if self.config.dry_run {
            println!("Dry run, exiting.");
            std::process::exit(0);
        }

        let s = format!("https://{}", self.config.grpc.listen);
        let server_state = TrowServerState {
            config: self.config.clone(),
            client: build_handlers(s)?,
        };

        let app = routes::create_app(server_state);

        // Start GRPC Backend task
        tokio::task::spawn(init_trow_server(self.config.clone())?);

        if let Some(ref tls) = self.config.tls {
            if !(Path::new(&tls.cert_file).is_file() && Path::new(&tls.key_file).is_file()) {
                return Err(anyhow!(
                    "Could not find TLS certificate and key at {} and {}",
                    tls.cert_file,
                    tls.key_file
                ));
            }
            let config = RustlsConfig::from_pem_file(&tls.cert_file, &tls.key_file).await?;
            axum_server::bind_rustls(self.config.addr, config)
                .serve(app.into_make_service())
                .await?;
        } else {
            axum_server::bind(self.config.addr)
                .serve(app.into_make_service())
                .await?;
        };
        Ok(())
    }
}

pub fn build_handlers(listen_addr: String) -> Result<ClientInterface> {
    event!(Level::DEBUG, "Address for backend: {}", listen_addr);

    //TODO this function is useless currently
    ClientInterface::new(listen_addr)
}
