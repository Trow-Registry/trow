use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ComponentError {
    #[error("unknown component error")]
    Unknown,
}

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ApiError {
    #[error("Request failed")]
    RequestFailed,
    #[error("Deserialization failed")]
    DeserializationFailed,
    #[error("Response parsing failed")]
    ResponseParsingFailed,
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("unknown api error")]
    Unknown,
}
