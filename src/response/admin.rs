//! Administrative functions
use failure::Error;
use rocket::State;

use backend;
use config;

#[derive(Debug, Serialize)]
pub enum Admin {
    Uuids(Vec<String>),
}


impl Admin {
    pub fn get_uuids(handler: State<config::BackendHandler>) -> Result<bool, Error> {
        let backend = handler.backend();
        Ok(true)
    }
}
