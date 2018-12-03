// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]

const METHOD_REGISTRY_REQUEST_UPLOAD: ::grpcio::Method<super::server::UploadRequest, super::server::UploadDetails> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/lycaon.Registry/RequestUpload",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_REGISTRY_GET_WRITE_LOCATION_FOR_BLOB: ::grpcio::Method<super::server::BlobRef, super::server::WriteLocation> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/lycaon.Registry/GetWriteLocationForBlob",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_REGISTRY_GET_READ_LOCATION_FOR_BLOB: ::grpcio::Method<super::server::DownloadRef, super::server::BlobReadLocation> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/lycaon.Registry/GetReadLocationForBlob",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_REGISTRY_GET_WRITE_LOCATION_FOR_MANIFEST: ::grpcio::Method<super::server::ManifestRef, super::server::WriteLocation> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/lycaon.Registry/GetWriteLocationForManifest",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_REGISTRY_GET_READ_LOCATION_FOR_MANIFEST: ::grpcio::Method<super::server::ManifestRef, super::server::ManifestReadLocation> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/lycaon.Registry/GetReadLocationForManifest",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_REGISTRY_VERIFY_MANIFEST: ::grpcio::Method<super::server::ManifestRef, super::server::VerifiedManifest> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/lycaon.Registry/VerifyManifest",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_REGISTRY_COMPLETE_UPLOAD: ::grpcio::Method<super::server::CompleteRequest, super::server::CompletedUpload> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/lycaon.Registry/CompleteUpload",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_REGISTRY_GET_CATALOG: ::grpcio::Method<super::server::CatalogRequest, super::server::CatalogEntry> = ::grpcio::Method {
    ty: ::grpcio::MethodType::ServerStreaming,
    name: "/lycaon.Registry/GetCatalog",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_REGISTRY_LIST_TAGS: ::grpcio::Method<super::server::CatalogEntry, super::server::Tag> = ::grpcio::Method {
    ty: ::grpcio::MethodType::ServerStreaming,
    name: "/lycaon.Registry/ListTags",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

pub struct RegistryClient {
    client: ::grpcio::Client,
}

impl RegistryClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        RegistryClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn request_upload_opt(&self, req: &super::server::UploadRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::server::UploadDetails> {
        self.client.unary_call(&METHOD_REGISTRY_REQUEST_UPLOAD, req, opt)
    }

    pub fn request_upload(&self, req: &super::server::UploadRequest) -> ::grpcio::Result<super::server::UploadDetails> {
        self.request_upload_opt(req, ::grpcio::CallOption::default())
    }

    pub fn request_upload_async_opt(&self, req: &super::server::UploadRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::server::UploadDetails>> {
        self.client.unary_call_async(&METHOD_REGISTRY_REQUEST_UPLOAD, req, opt)
    }

    pub fn request_upload_async(&self, req: &super::server::UploadRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::server::UploadDetails>> {
        self.request_upload_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_write_location_for_blob_opt(&self, req: &super::server::BlobRef, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::server::WriteLocation> {
        self.client.unary_call(&METHOD_REGISTRY_GET_WRITE_LOCATION_FOR_BLOB, req, opt)
    }

    pub fn get_write_location_for_blob(&self, req: &super::server::BlobRef) -> ::grpcio::Result<super::server::WriteLocation> {
        self.get_write_location_for_blob_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_write_location_for_blob_async_opt(&self, req: &super::server::BlobRef, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::server::WriteLocation>> {
        self.client.unary_call_async(&METHOD_REGISTRY_GET_WRITE_LOCATION_FOR_BLOB, req, opt)
    }

    pub fn get_write_location_for_blob_async(&self, req: &super::server::BlobRef) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::server::WriteLocation>> {
        self.get_write_location_for_blob_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_read_location_for_blob_opt(&self, req: &super::server::DownloadRef, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::server::BlobReadLocation> {
        self.client.unary_call(&METHOD_REGISTRY_GET_READ_LOCATION_FOR_BLOB, req, opt)
    }

    pub fn get_read_location_for_blob(&self, req: &super::server::DownloadRef) -> ::grpcio::Result<super::server::BlobReadLocation> {
        self.get_read_location_for_blob_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_read_location_for_blob_async_opt(&self, req: &super::server::DownloadRef, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::server::BlobReadLocation>> {
        self.client.unary_call_async(&METHOD_REGISTRY_GET_READ_LOCATION_FOR_BLOB, req, opt)
    }

    pub fn get_read_location_for_blob_async(&self, req: &super::server::DownloadRef) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::server::BlobReadLocation>> {
        self.get_read_location_for_blob_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_write_location_for_manifest_opt(&self, req: &super::server::ManifestRef, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::server::WriteLocation> {
        self.client.unary_call(&METHOD_REGISTRY_GET_WRITE_LOCATION_FOR_MANIFEST, req, opt)
    }

    pub fn get_write_location_for_manifest(&self, req: &super::server::ManifestRef) -> ::grpcio::Result<super::server::WriteLocation> {
        self.get_write_location_for_manifest_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_write_location_for_manifest_async_opt(&self, req: &super::server::ManifestRef, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::server::WriteLocation>> {
        self.client.unary_call_async(&METHOD_REGISTRY_GET_WRITE_LOCATION_FOR_MANIFEST, req, opt)
    }

    pub fn get_write_location_for_manifest_async(&self, req: &super::server::ManifestRef) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::server::WriteLocation>> {
        self.get_write_location_for_manifest_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_read_location_for_manifest_opt(&self, req: &super::server::ManifestRef, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::server::ManifestReadLocation> {
        self.client.unary_call(&METHOD_REGISTRY_GET_READ_LOCATION_FOR_MANIFEST, req, opt)
    }

    pub fn get_read_location_for_manifest(&self, req: &super::server::ManifestRef) -> ::grpcio::Result<super::server::ManifestReadLocation> {
        self.get_read_location_for_manifest_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_read_location_for_manifest_async_opt(&self, req: &super::server::ManifestRef, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::server::ManifestReadLocation>> {
        self.client.unary_call_async(&METHOD_REGISTRY_GET_READ_LOCATION_FOR_MANIFEST, req, opt)
    }

    pub fn get_read_location_for_manifest_async(&self, req: &super::server::ManifestRef) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::server::ManifestReadLocation>> {
        self.get_read_location_for_manifest_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn verify_manifest_opt(&self, req: &super::server::ManifestRef, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::server::VerifiedManifest> {
        self.client.unary_call(&METHOD_REGISTRY_VERIFY_MANIFEST, req, opt)
    }

    pub fn verify_manifest(&self, req: &super::server::ManifestRef) -> ::grpcio::Result<super::server::VerifiedManifest> {
        self.verify_manifest_opt(req, ::grpcio::CallOption::default())
    }

    pub fn verify_manifest_async_opt(&self, req: &super::server::ManifestRef, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::server::VerifiedManifest>> {
        self.client.unary_call_async(&METHOD_REGISTRY_VERIFY_MANIFEST, req, opt)
    }

    pub fn verify_manifest_async(&self, req: &super::server::ManifestRef) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::server::VerifiedManifest>> {
        self.verify_manifest_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn complete_upload_opt(&self, req: &super::server::CompleteRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::server::CompletedUpload> {
        self.client.unary_call(&METHOD_REGISTRY_COMPLETE_UPLOAD, req, opt)
    }

    pub fn complete_upload(&self, req: &super::server::CompleteRequest) -> ::grpcio::Result<super::server::CompletedUpload> {
        self.complete_upload_opt(req, ::grpcio::CallOption::default())
    }

    pub fn complete_upload_async_opt(&self, req: &super::server::CompleteRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::server::CompletedUpload>> {
        self.client.unary_call_async(&METHOD_REGISTRY_COMPLETE_UPLOAD, req, opt)
    }

    pub fn complete_upload_async(&self, req: &super::server::CompleteRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::server::CompletedUpload>> {
        self.complete_upload_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_catalog_opt(&self, req: &super::server::CatalogRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientSStreamReceiver<super::server::CatalogEntry>> {
        self.client.server_streaming(&METHOD_REGISTRY_GET_CATALOG, req, opt)
    }

    pub fn get_catalog(&self, req: &super::server::CatalogRequest) -> ::grpcio::Result<::grpcio::ClientSStreamReceiver<super::server::CatalogEntry>> {
        self.get_catalog_opt(req, ::grpcio::CallOption::default())
    }

    pub fn list_tags_opt(&self, req: &super::server::CatalogEntry, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientSStreamReceiver<super::server::Tag>> {
        self.client.server_streaming(&METHOD_REGISTRY_LIST_TAGS, req, opt)
    }

    pub fn list_tags(&self, req: &super::server::CatalogEntry) -> ::grpcio::Result<::grpcio::ClientSStreamReceiver<super::server::Tag>> {
        self.list_tags_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Item = (), Error = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait Registry {
    fn request_upload(&self, ctx: ::grpcio::RpcContext, req: super::server::UploadRequest, sink: ::grpcio::UnarySink<super::server::UploadDetails>);
    fn get_write_location_for_blob(&self, ctx: ::grpcio::RpcContext, req: super::server::BlobRef, sink: ::grpcio::UnarySink<super::server::WriteLocation>);
    fn get_read_location_for_blob(&self, ctx: ::grpcio::RpcContext, req: super::server::DownloadRef, sink: ::grpcio::UnarySink<super::server::BlobReadLocation>);
    fn get_write_location_for_manifest(&self, ctx: ::grpcio::RpcContext, req: super::server::ManifestRef, sink: ::grpcio::UnarySink<super::server::WriteLocation>);
    fn get_read_location_for_manifest(&self, ctx: ::grpcio::RpcContext, req: super::server::ManifestRef, sink: ::grpcio::UnarySink<super::server::ManifestReadLocation>);
    fn verify_manifest(&self, ctx: ::grpcio::RpcContext, req: super::server::ManifestRef, sink: ::grpcio::UnarySink<super::server::VerifiedManifest>);
    fn complete_upload(&self, ctx: ::grpcio::RpcContext, req: super::server::CompleteRequest, sink: ::grpcio::UnarySink<super::server::CompletedUpload>);
    fn get_catalog(&self, ctx: ::grpcio::RpcContext, req: super::server::CatalogRequest, sink: ::grpcio::ServerStreamingSink<super::server::CatalogEntry>);
    fn list_tags(&self, ctx: ::grpcio::RpcContext, req: super::server::CatalogEntry, sink: ::grpcio::ServerStreamingSink<super::server::Tag>);
}

pub fn create_registry<S: Registry + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_REGISTRY_REQUEST_UPLOAD, move |ctx, req, resp| {
        instance.request_upload(ctx, req, resp)
    });
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_REGISTRY_GET_WRITE_LOCATION_FOR_BLOB, move |ctx, req, resp| {
        instance.get_write_location_for_blob(ctx, req, resp)
    });
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_REGISTRY_GET_READ_LOCATION_FOR_BLOB, move |ctx, req, resp| {
        instance.get_read_location_for_blob(ctx, req, resp)
    });
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_REGISTRY_GET_WRITE_LOCATION_FOR_MANIFEST, move |ctx, req, resp| {
        instance.get_write_location_for_manifest(ctx, req, resp)
    });
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_REGISTRY_GET_READ_LOCATION_FOR_MANIFEST, move |ctx, req, resp| {
        instance.get_read_location_for_manifest(ctx, req, resp)
    });
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_REGISTRY_VERIFY_MANIFEST, move |ctx, req, resp| {
        instance.verify_manifest(ctx, req, resp)
    });
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_REGISTRY_COMPLETE_UPLOAD, move |ctx, req, resp| {
        instance.complete_upload(ctx, req, resp)
    });
    let instance = s.clone();
    builder = builder.add_server_streaming_handler(&METHOD_REGISTRY_GET_CATALOG, move |ctx, req, resp| {
        instance.get_catalog(ctx, req, resp)
    });
    let instance = s.clone();
    builder = builder.add_server_streaming_handler(&METHOD_REGISTRY_LIST_TAGS, move |ctx, req, resp| {
        instance.list_tags(ctx, req, resp)
    });
    builder.build()
}

const METHOD_ADMISSION_CONTROLLER_VALIDATE_ADMISSION: ::grpcio::Method<super::server::AdmissionRequest, super::server::AdmissionResponse> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/lycaon.AdmissionController/ValidateAdmission",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

pub struct AdmissionControllerClient {
    client: ::grpcio::Client,
}

impl AdmissionControllerClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        AdmissionControllerClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn validate_admission_opt(&self, req: &super::server::AdmissionRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::server::AdmissionResponse> {
        self.client.unary_call(&METHOD_ADMISSION_CONTROLLER_VALIDATE_ADMISSION, req, opt)
    }

    pub fn validate_admission(&self, req: &super::server::AdmissionRequest) -> ::grpcio::Result<super::server::AdmissionResponse> {
        self.validate_admission_opt(req, ::grpcio::CallOption::default())
    }

    pub fn validate_admission_async_opt(&self, req: &super::server::AdmissionRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::server::AdmissionResponse>> {
        self.client.unary_call_async(&METHOD_ADMISSION_CONTROLLER_VALIDATE_ADMISSION, req, opt)
    }

    pub fn validate_admission_async(&self, req: &super::server::AdmissionRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::server::AdmissionResponse>> {
        self.validate_admission_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Item = (), Error = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait AdmissionController {
    fn validate_admission(&self, ctx: ::grpcio::RpcContext, req: super::server::AdmissionRequest, sink: ::grpcio::UnarySink<super::server::AdmissionResponse>);
}

pub fn create_admission_controller<S: AdmissionController + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_ADMISSION_CONTROLLER_VALIDATE_ADMISSION, move |ctx, req, resp| {
        instance.validate_admission(ctx, req, resp)
    });
    builder.build()
}
