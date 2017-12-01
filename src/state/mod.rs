use failure;
use futures::{Future, Stream};
use orset::ORSet;
use rocket;
use tokio_core::reactor;
use tokio_io::AsyncRead;

use config;

/// Export Module layers
mod layers;
pub(crate) mod uuid;

// TODO: merge this into the Config struct in config.rs
pub struct ConsoleConfig {
    pub console_port: i64,
}
impl ConsoleConfig {
    fn default() -> ConsoleConfig {
        ConsoleConfig { console_port: 29999 }
    }
}

fn get_config() -> ConsoleConfig {
    let rkt = rocket::Rocket::ignite();
    let cfg = rkt.config();

    ConsoleConfig {
        // TODO: This is currently duplicated in the config.rs file (where it should be).
        console_port: match cfg.get_int("console_port") {

            Ok(x) => x,
            Err(_) => ConsoleConfig::default().console_port,
        },
    }
}
