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

const METHOD_BACKEND_REQUEST_UPLOAD: ::grpcio::Method<super::server::UploadRequest, super::server::UploadDetails> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/lycaon.Backend/RequestUpload",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_BACKEND_GET_WRITE_LOCATION_FOR_BLOB: ::grpcio::Method<super::server::BlobRef, super::server::WriteLocation> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/lycaon.Backend/GetWriteLocationForBlob",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

pub struct BackendClient {
    client: ::grpcio::Client,
}

impl BackendClient {
    pub fn new(channel: ::grpcio::Channel) -> Self {
        BackendClient {
            client: ::grpcio::Client::new(channel),
        }
    }

    pub fn request_upload_opt(&self, req: &super::server::UploadRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::server::UploadDetails> {
        self.client.unary_call(&METHOD_BACKEND_REQUEST_UPLOAD, req, opt)
    }

    pub fn request_upload(&self, req: &super::server::UploadRequest) -> ::grpcio::Result<super::server::UploadDetails> {
        self.request_upload_opt(req, ::grpcio::CallOption::default())
    }

    pub fn request_upload_async_opt(&self, req: &super::server::UploadRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::server::UploadDetails>> {
        self.client.unary_call_async(&METHOD_BACKEND_REQUEST_UPLOAD, req, opt)
    }

    pub fn request_upload_async(&self, req: &super::server::UploadRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::server::UploadDetails>> {
        self.request_upload_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_write_location_for_blob_opt(&self, req: &super::server::BlobRef, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::server::WriteLocation> {
        self.client.unary_call(&METHOD_BACKEND_GET_WRITE_LOCATION_FOR_BLOB, req, opt)
    }

    pub fn get_write_location_for_blob(&self, req: &super::server::BlobRef) -> ::grpcio::Result<super::server::WriteLocation> {
        self.get_write_location_for_blob_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_write_location_for_blob_async_opt(&self, req: &super::server::BlobRef, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::server::WriteLocation>> {
        self.client.unary_call_async(&METHOD_BACKEND_GET_WRITE_LOCATION_FOR_BLOB, req, opt)
    }

    pub fn get_write_location_for_blob_async(&self, req: &super::server::BlobRef) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::server::WriteLocation>> {
        self.get_write_location_for_blob_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Item = (), Error = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait Backend {
    fn request_upload(&self, ctx: ::grpcio::RpcContext, req: super::server::UploadRequest, sink: ::grpcio::UnarySink<super::server::UploadDetails>);
    fn get_write_location_for_blob(&self, ctx: ::grpcio::RpcContext, req: super::server::BlobRef, sink: ::grpcio::UnarySink<super::server::WriteLocation>);
}

pub fn create_backend<S: Backend + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_BACKEND_REQUEST_UPLOAD, move |ctx, req, resp| {
        instance.request_upload(ctx, req, resp)
    });
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_BACKEND_GET_WRITE_LOCATION_FOR_BLOB, move |ctx, req, resp| {
        instance.get_write_location_for_blob(ctx, req, resp)
    });
    builder.build()
}
