use thiserror::Error;
use crate::types::{AdmissionRequest, AdmissionResponse};
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Internal validation error")]
    Internal,
}
pub trait Validation {
    // This function signature is very tied to the implementation.
    // If you develop a new front-end and have problems here, we should change it.
    fn validate_admission(
        &self,
        admission_req: &AdmissionRequest,
        host_names: &Vec<String>,
    ) -> Result<AdmissionResponse, ValidationError>;
}