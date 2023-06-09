use std::{error, fmt};

use axum::body;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

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
    BlobUploadInvalid(String),
    ManifestUnknown(String),
    ManifestInvalid(String),
    Unauthorized,
    BlobUnknown,
    BlobUploadUnknown,
    Unsupported,
    InternalError,
    DigestInvalid,
    NotFound,
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
            Error::BlobUploadInvalid(ref detail) => format_error_json(
                f,
                "BLOB_UPLOAD_INVALID",
                "Invalid request to blob upload",
                Some(json!({ "Reason": detail })),
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
            Error::ManifestInvalid(ref detail) => format_error_json(
                f,
                "MANIFEST_INVALID",
                "Manifest invalid",
                Some(json!({ "detail": detail })),
            ),
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
            Error::NotFound => format_error_json(f, "NOT_FOUND", "Not Found", None),
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
            Error::BlobUploadInvalid(_) => "The blob upload encountered an error and can no longer proceed.",
            Error::InternalError => "An internal error occurred, please consult the logs for more details.",
            Error::DigestInvalid => "When a blob is uploaded, the registry will check that the content matches the digest provided by the client. The error may include a detail structure with the key \"digest\", including the invalid digest string. This error may also be returned when a manifest includes an invalid layer digest.",
            Error::ManifestInvalid(_) => "During upload, manifests undergo several checks ensuring validity. If those checks fail, this error may be returned, unless a more specific error is included. The detail will contain information the failed validation.",
            Error::ManifestUnknown(_) => "This error is returned when the manifest, identified by name and tag is unknown to the repository.",
            Error::NameInvalid(_) => "Invalid repository name encountered either during manifest validation or any API operation.",
            Error::NotFound => "The specified resource could not be found. This error may also occur if the client does not have permission to access the resource.",
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let json = format!("{}", self);

        let status = match self {
            Error::Unsupported => StatusCode::METHOD_NOT_ALLOWED,
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::BlobUploadUnknown | Error::ManifestUnknown(_) => StatusCode::NOT_FOUND,
            Error::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            Error::BlobUploadInvalid(_) => StatusCode::RANGE_NOT_SATISFIABLE,
            Error::DigestInvalid
            | Error::ManifestInvalid(_)
            | Error::BlobUnknown
            | Error::NameInvalid(_) => StatusCode::BAD_REQUEST,
            Error::NotFound => StatusCode::NOT_FOUND,
        };
        Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::CONTENT_LENGTH, json.len())
            .status(status)
            .body(body::Full::from(json))
            .unwrap()
            .into_response()
    }
}
