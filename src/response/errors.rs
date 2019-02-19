use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response;
use rocket::response::Responder;
use rocket::Response;
use serde_json::json;
use serde_json::Value;
use std::error;
use std::fmt;
use std::io::Cursor;

#[derive(Debug)]
pub enum Error {
    /*
    BLOB_UNKNOWN,
    BLOB_UPLOAD_INVALID,
    BLOB_UPLOAD_UNKNOWN,
    DIGEST_INVALID,
    MANIFEST_BLOB_UNKNOWN,
    ,
    MANIFEST_UNVERIFIED,
    NAME_INVALID,
    NAME_UNKNOWN,
    SIZE_INVALID,
    TAG_INVALID,
    UNAUTHORIZED,
    DENIED,
    */
    ManifestUnknown(String),
    ManifestInvalid,
    Unauthorized,
    BlobUnknown,
    BlobUploadUnknown,
    Unsupported,
    InternalError,
    DigestInvalid,
}

//Create errormsg struct that serializes to json of appropriate type
#[derive(Serialize, Deserialize)]
struct ErrorMsg {
    code: String,
    message: String,
    detail: Value,
}

//TODO: All errors should return JSON
//This needs refactored.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Unsupported => write!(f, "Unsupported Operation"),
            Error::Unauthorized => write!(f, "Authorization required"),
            Error::BlobUnknown => write!(f, "Blob Unknown"),
            Error::BlobUploadUnknown => write!(f, "Blob Upload Unknown"),
            Error::InternalError => write!(f, "Internal Error"),
            Error::DigestInvalid => write!(f, "Provided digest did not match uploaded content"),
            Error::ManifestInvalid => write!(f, "Manifest Invalid"),

            Error::ManifestUnknown(ref tag) => {
                let emsg = ErrorMsg {
                    code: "MANIFEST_UNKNOWN".to_string(),
                    message: "manifest unknown".to_string(),
                    detail: json!({ "Tag": tag }),
                };
                write!(
                    f,
                    "{{\"errors\":[{}]}}",
                    serde_json::to_string(&emsg).unwrap_or("MANIFEST_UNKNOWN".to_string())
                )
            }
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Unsupported => "The operation was unsupported due to a missing implementation or invalid set of parameters.",
            Error::Unauthorized => "The operation requires authorization.",
            Error::BlobUnknown => "Reference made to an unknown blob (e.g. invalid UUID)",
            Error::BlobUploadUnknown => "If a blob upload has been cancelled or was never started, this error code may be returned.",
            Error::InternalError => "An internal error occured, please consult the logs for more details.",
            Error::DigestInvalid => "When a blob is uploaded, the registry will check that the content matches the digest provided by the client. The error may include a detail structure with the key \"digest\", including the invalid digest string. This error may also be returned when a manifest includes an invalid layer digest.",
            Error::ManifestInvalid => "During upload, manifests undergo several checks ensuring validity. If those checks fail, this error may be returned, unless a more specific error is included. The detail will contain information the failed validation.",
            Error::ManifestUnknown(_) => "This error is returned when the manifest, identified by name and tag is unknown to the repository.",

        }
    }
}

impl<'r> Responder<'r> for Error {
    fn respond_to(self, _req: &Request) -> response::Result<'r> {
        let json = format!("{}", self);

        let status = match self {
            Error::Unsupported => Status::MethodNotAllowed,
            Error::Unauthorized => Status::Unauthorized,
            Error::BlobUploadUnknown | Error::ManifestUnknown(_) => Status::NotFound,
            Error::InternalError => Status::InternalServerError,
            Error::DigestInvalid | Error::ManifestInvalid | Error::BlobUnknown => {
                Status::BadRequest
            }
        };
        Response::build()
            .header(ContentType::JSON)
            .sized_body(Cursor::new(json))
            .status(status)
            .ok()
    }
}
