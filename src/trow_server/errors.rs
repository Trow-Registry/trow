use thiserror::Error;

#[derive(Error, Debug)]
#[error("Error getting proxied repo {msg:?}")]
pub struct ProxyError {
    pub msg: String,
}

#[derive(Error, Debug)]
#[error("Expected digest {user_digest:?} but got {actual_digest:?}")]
pub struct DigestValidationError {
    pub user_digest: String,
    pub actual_digest: String,
}
