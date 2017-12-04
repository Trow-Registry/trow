use failure::Error;

type Message = &'static str;
type Detail = &'static str;

/// Internal errors that occur throughout the system
#[derive(Debug, Fail)]
pub enum Server {
    #[fail(display = "Invalid {} input", _0)]
    Invalid(&'static str),
    #[fail(display = "File Not Found: {}", _0)]
    FileNotFound(String),
    #[fail(display = "ConfigError: {}", _0)]
    ConfigError(Error),
    #[fail(display = "GenericError: {}", _0)]
    GenericError(String),
}

/// Client errors that are returned to consumers of the Registry API
#[derive(Serialize, Clone, Debug, Fail)]
#[allow(dead_code, non_camel_case_types)]
pub enum Client {
    #[fail(display = "blob unknown to registry")]
    BLOB_UNKNOWN,
    #[fail(display = "blob upload invalid")]
    BLOB_UPLOAD_INVALID,
    #[fail(display = "IMPLEMENT ME")]
    BLOB_UPLOAD_UNKNOWN,
    #[fail(display = "IMPLEMENT ME")]
    DIGEST_INVALID,
    #[fail(display = "IMPLEMENT ME")]
    MANIFEST_BLOB_UNKNOWN,
    #[fail(display = "IMPLEMENT ME")]
    MANIFEST_INVALID,
    #[fail(display = "IMPLEMENT ME")]
    MANIFEST_UNKNOWN,
    #[fail(display = "IMPLEMENT ME")]
    MANIFEST_UNVERIFIED,
    #[fail(display = "IMPLEMENT ME")]
    NAME_INVALID,
    #[fail(display = "IMPLEMENT ME")]
    NAME_UNKNOWN,
    #[fail(display = "IMPLEMENT ME")]
    SIZE_INVALID,
    #[fail(display = "IMPLEMENT ME")]
    TAG_INVALID,
    #[fail(display = "IMPLEMENT ME")]
    UNAUTHORIZED,
    #[fail(display = "IMPLEMENT ME")]
    DENIED,
    #[fail(display = "IMPLEMENT ME")]
    UNSUPPORTED,
}

impl Client {
    fn message(self) -> Message {
        match self {
            Client::BLOB_UNKNOWN => "blob unknown to registry",
            Client::BLOB_UPLOAD_INVALID => "blob upload invalid",
            Client::BLOB_UPLOAD_UNKNOWN => "blob upload unknown to registry",
            Client::DIGEST_INVALID => "provided digest did not match uploaded content",
            Client::MANIFEST_BLOB_UNKNOWN => "blob unknown to registry",
            Client::MANIFEST_INVALID => "manifest invalid",
            Client::MANIFEST_UNKNOWN => "manifest unknown",
            Client::MANIFEST_UNVERIFIED => "manifest failed signature verification",
            Client::NAME_INVALID => "invalid repository name",
            Client::NAME_UNKNOWN => "repository not known to registry",
            Client::SIZE_INVALID => "provided length did not match content length",
            Client::TAG_INVALID => "manifest tag did not match URI",
            Client::UNAUTHORIZED => "authentication required",
            Client::DENIED => "requested access to the resource is denied",
            Client::UNSUPPORTED => "The operation is unsupported",
        }
    }

    fn detail(self) -> Detail {
        match self {
            Client::BLOB_UNKNOWN => {
                "This error may be returned when a blob is unknown to the registry in a specified repository. This can be returned with a standard get or if a manifest references an unknown layer during upload"
            }
            Client::BLOB_UPLOAD_INVALID => {
                "The blob upload encountered an error and can no longer proceed"
            }
            Client::BLOB_UPLOAD_UNKNOWN => {
                "If a blob upload has been cancelled or was never started, this error code may be returned"
            }
            Client::DIGEST_INVALID => {
                "When a blob is uploaded, the registry will check that the content matches the digest provided by the client. The error may include a detail structure with the key \"digest\" including the invalid digest string. This error may also be returned when a minfest includes an invalid layer digest."
            }
            Client::MANIFEST_BLOB_UNKNOWN => {
                "This error may be returned when a manifest blob is unknown to the registry"
            }
            Client::MANIFEST_INVALID => {
                "During upload, manifests undergo several checks ensuring validity. If those checks fail, this error may be returned, unless a more specific error is included. The detail will contain information the failed validation."
            }
            Client::MANIFEST_UNKNOWN => {
                "This error is returned when the manifest, identified by name and tag is unknown to the repository."
            }
            Client::MANIFEST_UNVERIFIED => {
                "During manifest upload, if the manifest fails signature verification, this error will be returned."
            }
            Client::NAME_INVALID => {
                "Invalid repository name encountered either during manifest validation or any API operation."
            }
            Client::NAME_UNKNOWN => {
                "This is returned if the name used during an operation is unknown to the registry."
            }
            Client::SIZE_INVALID => {
                "When a layer is uploaded, the provided size will be checked against the uploaded content. If they do not match, this error will be returned."
            }
            Client::TAG_INVALID => {
                "During a manifest upload, if the tag in the manifest does not match the uri tag, this error will be returned."
            }
            Client::UNAUTHORIZED => {
                "The access controller was unable to authenticate the client. Often this will be accompanied by a Www-Authenticate HTTP response header indicating how to authenticate."
            }
            Client::DENIED => {
                "The access controller denied access for the operation on a resource."
            }
            Client::UNSUPPORTED => {
                "The operation was unsupported due to a missing implementation or invalid set of parameters."
            }
        }
    }
}
