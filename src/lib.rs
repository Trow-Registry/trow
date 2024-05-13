pub mod response;
#[allow(clippy::too_many_arguments)]
mod routes;
pub mod types;

pub mod registry;
#[cfg(feature = "sqlite")]
mod users;

use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::{env, fs};

use anyhow::{Context, Result};
use axum::extract::FromRef;
use axum::Router;
use thiserror::Error;
use registry::{ImageValidationConfig, RegistryProxiesConfig, TrowServer};
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
    pub client: TrowServer,
    pub config: TrowConfig,
}

impl FromRef<Arc<TrowServerState>> for TrowConfig {
    fn from_ref(state: &Arc<TrowServerState>) -> Self {
        state.config.clone()
    }
}

#[derive(Clone, Debug)]
pub struct TlsConfig {
    pub cert_file: String,
    pub key_file: String,
}

impl TlsConfig {
    pub fn new(cert_file: String, key_file: String) -> Self {
        Self {
            cert_file,
            key_file,
        }
    }
}

#[derive(Clone, Debug)]
struct UserConfig {
    user: String,
    hash_encoded: String, //Surprised not bytes
}

#[derive(Clone, Debug)]
pub struct TrowConfig {
    pub data_dir: PathBuf,
    pub service_name: String,
    pub proxy_registry_config: Option<RegistryProxiesConfig>,
    pub image_validation_config: Option<ImageValidationConfig>,
    pub dry_run: bool,
    pub token_secret: String,
    user: Option<UserConfig>,
    pub cors: Option<Vec<String>>,
    pub uses_tls: bool,
}

impl Default for TrowConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl TrowConfig {
    #[allow(clippy::too_many_arguments)]
    pub fn new() -> Self {
        Self {
            data_dir: PathBuf::from_str("./data").unwrap(),
            service_name: "http://trow".to_string(),
            proxy_registry_config: None,
            image_validation_config: None,
            dry_run: false,
            token_secret: Uuid::new_v4().to_string(),
            user: None,
            cors: None,
            uses_tls: false,
        }
    }

    pub fn with_proxy_registries(&mut self, config_file: impl AsRef<str>) -> Result<&mut Self> {
        let config_file = config_file.as_ref();
        let config_str = fs::read_to_string(config_file)
            .with_context(|| format!("Could not read file `{}`", config_file))?;
        let config = serde_yaml::from_str::<RegistryProxiesConfig>(&config_str)
            .with_context(|| format!("Could not parse file `{}`", config_file))?;
        self.proxy_registry_config = Some(config);
        Ok(self)
    }

    pub fn with_image_validation(&mut self, config_file: impl AsRef<str>) -> Result<&mut Self> {
        let config_file = config_file.as_ref();
        let config_str = fs::read_to_string(config_file)
            .with_context(|| format!("Could not read file `{}`", config_file))?;
        let config = serde_yaml::from_str::<ImageValidationConfig>(&config_str)
            .with_context(|| format!("Could not parse file `{}`", config_file))?;
        self.image_validation_config = Some(config);
        Ok(self)
    }

    pub fn with_user(&mut self, user: String, pass: &str) -> &mut Self {
        let mut hash_config = argon2::Config::rfc9106();
        hash_config.mem_cost = 4066;
        hash_config.time_cost = 3;
        let hash_encoded =
            argon2::hash_encoded(pass.as_bytes(), Uuid::new_v4().as_bytes(), &hash_config)
                .expect("Error hashing password");
        let usercfg = UserConfig { user, hash_encoded };
        self.user = Some(usercfg);
        self
    }

    pub async fn build_app(&self) -> Result<Router> {
        println!("Starting Trow {}", env!("CARGO_PKG_VERSION"),);
        println!(
            "Hostname of this registry (for the MutatingWebhook): {:?}",
            self.service_name
        );
        match self.image_validation_config {
            Some(ref config) => {
                println!("Image validation webhook configured:");
                println!("  Default action: {}", config.default);
                println!("  Allowed prefixes: {:?}", config.allow);
                println!("  Denied prefixes: {:?}", config.deny);
            }
            None => println!("Image validation webhook not configured"),
        }
        if let Some(proxy_config) = &self.proxy_registry_config {
            println!("Proxy registries configured:");
            for config in &proxy_config.registries {
                println!("  - {}: {}", config.alias, config.host);
            }
        } else {
            println!("Proxy registries not configured");
        }

        if self.cors.is_some() {
            println!("Cross-Origin Resource Sharing(CORS) requests are allowed\n");
        }

        if self.dry_run {
            println!("Dry run, exiting.");
            std::process::exit(0);
        }

        let ts_builder = registry::build_server(
            self.data_dir.clone(),
            self.proxy_registry_config.clone(),
            self.image_validation_config.clone(),
        );
        let registry = ts_builder.get_server()?;

        let server_state = TrowServerState {
            config: self.clone(),
            client: registry,
        };
        Ok(routes::create_app(server_state))
    }
}
