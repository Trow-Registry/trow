#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum APIRegistryError {
    // This error MAY be returned when a blob is unknown to the registry in a specified repository.
    // This can be returned with a standard get or if a manifest references an unknown layer during upload.
    //#[fail(display = "Unknown blob: {}", message)]
    BlobUnknown { message: String, details: String },

    // The blob upload encountered an error and can no longer proceed.
    //#[fail(display = "Invalid blob upload: {}", message)]
    BlobUploadInvalid { message: String, details: String },

    // If a blob upload has been cancelled or was never started, this error code MAY be returned.
    //#[fail(display = "Unknown blob upload: {}", message)]
    BlobUploadUnknown { message: String, details: String },

    //#[fail(display = "Invalid digest: {}", message)]
    DigestInvalid { message: String, details: String },

    //#[fail(display = "Unknown blob manifest: {}", message)]
    ManifestBlobUnknown { message: String, details: String },

    //#[fail(display = "Invalid manifest: {}", message)]
    ManifestInvalid { message: String, details: String },

    //#[fail(display = "Unknown manifest: {}", message)]
    ManifestUnknown { message: String, details: String },

    //#[fail(display = "Unverified manifest: {}", message)]
    ManifestUnverified { message: String, details: String },

    //#[fail(display = "Invalid name(space): {}", message)]
    NameInvalid { message: String, details: String },

    //#[fail(display = "Unknown name(space): {}", message)]
    NameUnknown { message: String, details: String },

    //#[fail(display = "Invalid size: {}", message)]
    SizeInvalid { message: String, details: String },

    //#[fail(display = "Invalid tag: {}", message)]
    TagInvalid { message: String, details: String },

    //#[fail(display = "Unauthorized: {}", message)]
    Unauthorized { message: String, details: String },

    //#[fail(display = "Access denied: {}", message)]
    Denied { message: String, details: String },

    //#[fail(display = "Unsupported: {}", message)]
    Unsupported { message: String, details: String },

    InternalError { message: String, details: String },
}
