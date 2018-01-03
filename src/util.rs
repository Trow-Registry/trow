//! This file should peferably be empty.
//! The use of this file is unclear, but it does at the very least
//! clean out other sections of code.

use failure::Error;

use errors;

pub fn std_err(msg: &str) -> Error {
    Error::from(errors::Server::GenericError(msg.to_owned()))
}
