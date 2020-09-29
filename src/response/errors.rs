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

    BLOB_UPLOAD_UNKNOWN,
    DIGEST_INVALID,
    MANIFEST_BLOB_UNKNOWN,
    ,
    MANIFEST_UNVERIFIED,
    NAME_UNKNOWN,
    SIZE_INVALID,
    TAG_INVALID,
    UNAUTHORIZED,
    DENIED,
    */
    NameInvalid(String),
    BlobUploadInvalid,
    ManifestUnknown(String),
    ManifestInvalid,
    Unauthorized,
    BlobUnknown,
    BlobUploadUnknown,
    Unsupported,
    InternalError,
    DigestInvalid,
}

// Create ErrorMsg struct that serializes to json of appropriate type
#[derive(Serialize, Deserialize)]
struct ErrorMsg {
    code: String,
    message: String,
    detail: Option<Value>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Unsupported => format_error_json(f, "UNSUPPORTED", "Unsupported", None),
            Error::Unauthorized => {
                format_error_json(f, "UNAUTHORIZED", "Authorization required", None)
            }
            Error::BlobUnknown => format_error_json(f, "BLOB_UNKNOWN", "Blob Unknown", None),
            Error::BlobUploadUnknown => write!(f, "Blob Upload Unknown"),
            Error::BlobUploadInvalid => format_error_json(
                f,
                "BLOB_UPLOAD_INVALID",
                "Invalid request to blob upload",
                None,
            ),
            // TODO: INTERNAL_ERROR code is not in the distribution spec
            Error::InternalError => {
                format_error_json(f, "INTERNAL_ERROR", "Internal Server Error", None)
            }
            Error::DigestInvalid => format_error_json(
                f,
                "DIGEST_INVALID",
                "Provided digest did not match uploaded content",
                None,
            ),
            Error::ManifestInvalid => {
                format_error_json(f, "MANIFEST_INVALID", "Manifest invalid", None)
            }
            Error::ManifestUnknown(ref tag) => format_error_json(
                f,
                "MANIFEST_UNKNOWN",
                "Manifest unknown",
                Some(json!({ "Tag": tag })),
            ),
            Error::NameInvalid(ref name) => format_error_json(
                f,
                "NAME_INVALID",
                "Invalid repository name",
                Some(json!({ "Repository": name })),
            ),
        }
    }
}

fn format_error_json(
    f: &mut fmt::Formatter,
    code: &str,
    message: &str,
    detail: Option<Value>,
) -> fmt::Result {
    let emsg = ErrorMsg {
        code: code.to_string(),
        message: message.to_string(),
        detail,
    };

    write!(
        f,
        "{{\"errors\":[{}]}}",
        serde_json::to_string(&emsg).unwrap()
    )
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Unsupported => "The operation was unsupported due to a missing implementation or invalid set of parameters.",
            Error::Unauthorized => "The operation requires authorization.",
            Error::BlobUnknown => "Reference made to an unknown blob (e.g. invalid UUID)",
            Error::BlobUploadUnknown => "If a blob upload has been cancelled or was never started, this error code may be returned.",
            Error::BlobUploadInvalid => "The blob upload encountered an error and can no longer proceed.",
            Error::InternalError => "An internal error occured, please consult the logs for more details.",
            Error::DigestInvalid => "When a blob is uploaded, the registry will check that the content matches the digest provided by the client. The error may include a detail structure with the key \"digest\", including the invalid digest string. This error may also be returned when a manifest includes an invalid layer digest.",
            Error::ManifestInvalid => "During upload, manifests undergo several checks ensuring validity. If those checks fail, this error may be returned, unless a more specific error is included. The detail will contain information the failed validation.",
            Error::ManifestUnknown(_) => "This error is returned when the manifest, identified by name and tag is unknown to the repository.",
            Error::NameInvalid(_) => "Invalid repository name encountered either during manifest validation or any API operation."

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
            Error::BlobUploadInvalid => Status::RangeNotSatisfiable,
            Error::DigestInvalid
            | Error::ManifestInvalid
            | Error::BlobUnknown
            | Error::NameInvalid(_) => Status::BadRequest,
        };
        Response::build()
            .header(ContentType::JSON)
            .sized_body(Cursor::new(json))
            .status(status)
            .ok()
    }
}
