use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AdmissionRequest {
    pub uid: String,
    pub object: serde_json::Value,
    pub namespace: String,
    pub operation: String, //CREATE, UPDATE, DELETE, CONNECT
                           //probably want user info as well, but normally it's the service account :(
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AdmissionResponse {
    pub uid: String,
    pub allowed: bool,
    pub status: Option<Status>,
    /* Not yet implemented, Patch, PatchType & AuditAnnotations. */
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Status {
    pub status: String,          //"Success" or "Failure". TODO: use proper type.
    pub message: Option<String>, //Human readable description. Shown in kubectl output.
    /*
    pub reason: String, //Machine readable description of "failure". Not sure where this goes.
    pub details: ?, // Data associated with reason field
    */
    pub code: Option<i32>, // Suggested http return code, 0 if not set
}

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Internal validation error")]
    Internal,
}

#[rocket::async_trait]
pub trait Validation {
    // This function signature is very tied to the implementation.
    // If you develop a new front-end and have problems here, we should change it.
    async fn validate_admission(
        &self,
        admission_req: &AdmissionRequest,
        host_names: &[String],
    ) -> Result<AdmissionResponse, ValidationError>;
}
