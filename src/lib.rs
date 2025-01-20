mod init_db;
pub mod registry;
pub mod routes;
#[cfg(test)]
pub mod test_utilities;
pub mod types;
#[cfg(feature = "sqlite")]
mod users;

use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::{env, fs};

use anyhow::{Context, Result};
use axum::Router;
use registry::TrowServer;
use sqlx::sqlite::SqlitePool;
use thiserror::Error;
use uuid::Uuid;

use crate::registry::ConfigFile;

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
    pub registry: TrowServer,
    pub config: TrowConfig,
    pub db: SqlitePool,
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

#[derive(Debug, Clone)]
pub struct TrowConfig {
    pub data_dir: PathBuf,
    pub service_name: String,
    pub config_file: Option<ConfigFile>,
    pub dry_run: bool,
    pub token_secret: Vec<u8>,
    user: Option<UserConfig>,
    pub cors: Option<Vec<String>>,
    pub uses_tls: bool,
    pub db_connection: Option<String>,
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
            config_file: None,
            dry_run: false,
            token_secret: Uuid::new_v4().as_bytes().to_vec(),
            user: None,
            cors: None,
            uses_tls: false,
            db_connection: None,
        }
    }

    pub fn with_config(&mut self, config_file: impl AsRef<str>) -> Result<&mut Self> {
        let config_file = config_file.as_ref();
        let config_str = fs::read_to_string(config_file)
            .with_context(|| format!("Could not read file `{}`", config_file))?;
        let config = serde_yaml_ng::from_str::<ConfigFile>(&config_str)
            .with_context(|| format!("Could not parse file `{}`", config_file))?;
        self.config_file = Some(config);
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

    /// Should only be used internally or for integration tests
    #[doc(hidden)]
    pub async fn build_server_state(self) -> Result<Arc<TrowServerState>> {
        println!("Starting Trow {}", env!("CARGO_PKG_VERSION"),);
        println!(
            "Hostname of this registry (for the MutatingWebhook): {:?}",
            self.service_name
        );
        match &self.config_file {
            Some(ConfigFile {
                image_validation: Some(cfg),
                ..
            }) => {
                println!("Image validation webhook configured:");
                println!("  Default action: {}", cfg.default);
                println!("  Allowed prefixes: {:?}", cfg.allow);
                println!("  Denied prefixes: {:?}", cfg.deny);
            }
            _ => println!("Image validation webhook not configured"),
        }
        match &self.config_file {
            Some(ConfigFile {
                registry_proxies: cfg,
                ..
            }) if !cfg.registries.is_empty() => {
                println!("Proxy registries configured:");
                for config in &cfg.registries {
                    println!("  - {}: {}", config.alias, config.host);
                }
            }
            _ => println!("Proxy registries not configured"),
        }

        if self.cors.is_some() {
            println!("Cross-Origin Resource Sharing(CORS) requests are allowed\n");
        }

        if self.dry_run {
            println!("Dry run, exiting.");
            std::process::exit(0);
        }

        let registry = TrowServer::new(self.data_dir.clone(), self.config_file.clone())?;

        let db_in_memory = self.db_connection == Some(":memory:".to_string());
        let db_file = match (&self.db_connection, db_in_memory) {
            (Some(conn), false) => conn.clone(),
            _ => {
                let mut p = self.data_dir.clone();
                p.push("trow.db");
                p.to_string_lossy().to_string()
            }
        };
        let db = init_db::init_db(&db_file, db_in_memory).await?;

        let server_state = TrowServerState {
            config: self,
            registry,
            db,
        };
        Ok(Arc::new(server_state))
    }

    pub async fn build_app(self) -> Result<Router> {
        let state = self.build_server_state().await?;
        Ok(routes::create_app(state))
    }
}
