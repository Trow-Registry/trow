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

const METHOD_BACKEND_CREATE_UUID: ::grpcio::Method<super::backend::CreateUuidRequest, super::backend::CreateUuidResult> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/lycaon.Backend/CreateUuid",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_BACKEND_GEN_UUID: ::grpcio::Method<super::backend::Layer, super::backend::GenUuidResult> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/lycaon.Backend/GenUuid",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_BACKEND_UUID_EXISTS: ::grpcio::Method<super::backend::Layer, super::backend::Result> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/lycaon.Backend/UuidExists",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_BACKEND_CANCEL_UPLOAD: ::grpcio::Method<super::backend::Layer, super::backend::Result> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/lycaon.Backend/cancelUpload",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_BACKEND_DELETE_UUID: ::grpcio::Method<super::backend::Layer, super::backend::Result> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/lycaon.Backend/deleteUuid",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_BACKEND_UPLOAD_MANIFEST: ::grpcio::Method<super::backend::Manifest, super::backend::Result> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/lycaon.Backend/uploadManifest",
    req_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
    resp_mar: ::grpcio::Marshaller { ser: ::grpcio::pb_ser, de: ::grpcio::pb_de },
};

const METHOD_BACKEND_GET_UUIDS: ::grpcio::Method<super::backend::Empty, super::backend::UuidList> = ::grpcio::Method {
    ty: ::grpcio::MethodType::Unary,
    name: "/lycaon.Backend/getUuids",
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

    pub fn create_uuid_opt(&self, req: &super::backend::CreateUuidRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::backend::CreateUuidResult> {
        self.client.unary_call(&METHOD_BACKEND_CREATE_UUID, req, opt)
    }

    pub fn create_uuid(&self, req: &super::backend::CreateUuidRequest) -> ::grpcio::Result<super::backend::CreateUuidResult> {
        self.create_uuid_opt(req, ::grpcio::CallOption::default())
    }

    pub fn create_uuid_async_opt(&self, req: &super::backend::CreateUuidRequest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::backend::CreateUuidResult>> {
        self.client.unary_call_async(&METHOD_BACKEND_CREATE_UUID, req, opt)
    }

    pub fn create_uuid_async(&self, req: &super::backend::CreateUuidRequest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::backend::CreateUuidResult>> {
        self.create_uuid_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn gen_uuid_opt(&self, req: &super::backend::Layer, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::backend::GenUuidResult> {
        self.client.unary_call(&METHOD_BACKEND_GEN_UUID, req, opt)
    }

    pub fn gen_uuid(&self, req: &super::backend::Layer) -> ::grpcio::Result<super::backend::GenUuidResult> {
        self.gen_uuid_opt(req, ::grpcio::CallOption::default())
    }

    pub fn gen_uuid_async_opt(&self, req: &super::backend::Layer, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::backend::GenUuidResult>> {
        self.client.unary_call_async(&METHOD_BACKEND_GEN_UUID, req, opt)
    }

    pub fn gen_uuid_async(&self, req: &super::backend::Layer) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::backend::GenUuidResult>> {
        self.gen_uuid_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn uuid_exists_opt(&self, req: &super::backend::Layer, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::backend::Result> {
        self.client.unary_call(&METHOD_BACKEND_UUID_EXISTS, req, opt)
    }

    pub fn uuid_exists(&self, req: &super::backend::Layer) -> ::grpcio::Result<super::backend::Result> {
        self.uuid_exists_opt(req, ::grpcio::CallOption::default())
    }

    pub fn uuid_exists_async_opt(&self, req: &super::backend::Layer, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::backend::Result>> {
        self.client.unary_call_async(&METHOD_BACKEND_UUID_EXISTS, req, opt)
    }

    pub fn uuid_exists_async(&self, req: &super::backend::Layer) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::backend::Result>> {
        self.uuid_exists_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn cancel_upload_opt(&self, req: &super::backend::Layer, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::backend::Result> {
        self.client.unary_call(&METHOD_BACKEND_CANCEL_UPLOAD, req, opt)
    }

    pub fn cancel_upload(&self, req: &super::backend::Layer) -> ::grpcio::Result<super::backend::Result> {
        self.cancel_upload_opt(req, ::grpcio::CallOption::default())
    }

    pub fn cancel_upload_async_opt(&self, req: &super::backend::Layer, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::backend::Result>> {
        self.client.unary_call_async(&METHOD_BACKEND_CANCEL_UPLOAD, req, opt)
    }

    pub fn cancel_upload_async(&self, req: &super::backend::Layer) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::backend::Result>> {
        self.cancel_upload_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn delete_uuid_opt(&self, req: &super::backend::Layer, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::backend::Result> {
        self.client.unary_call(&METHOD_BACKEND_DELETE_UUID, req, opt)
    }

    pub fn delete_uuid(&self, req: &super::backend::Layer) -> ::grpcio::Result<super::backend::Result> {
        self.delete_uuid_opt(req, ::grpcio::CallOption::default())
    }

    pub fn delete_uuid_async_opt(&self, req: &super::backend::Layer, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::backend::Result>> {
        self.client.unary_call_async(&METHOD_BACKEND_DELETE_UUID, req, opt)
    }

    pub fn delete_uuid_async(&self, req: &super::backend::Layer) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::backend::Result>> {
        self.delete_uuid_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn upload_manifest_opt(&self, req: &super::backend::Manifest, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::backend::Result> {
        self.client.unary_call(&METHOD_BACKEND_UPLOAD_MANIFEST, req, opt)
    }

    pub fn upload_manifest(&self, req: &super::backend::Manifest) -> ::grpcio::Result<super::backend::Result> {
        self.upload_manifest_opt(req, ::grpcio::CallOption::default())
    }

    pub fn upload_manifest_async_opt(&self, req: &super::backend::Manifest, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::backend::Result>> {
        self.client.unary_call_async(&METHOD_BACKEND_UPLOAD_MANIFEST, req, opt)
    }

    pub fn upload_manifest_async(&self, req: &super::backend::Manifest) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::backend::Result>> {
        self.upload_manifest_async_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_uuids_opt(&self, req: &super::backend::Empty, opt: ::grpcio::CallOption) -> ::grpcio::Result<super::backend::UuidList> {
        self.client.unary_call(&METHOD_BACKEND_GET_UUIDS, req, opt)
    }

    pub fn get_uuids(&self, req: &super::backend::Empty) -> ::grpcio::Result<super::backend::UuidList> {
        self.get_uuids_opt(req, ::grpcio::CallOption::default())
    }

    pub fn get_uuids_async_opt(&self, req: &super::backend::Empty, opt: ::grpcio::CallOption) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::backend::UuidList>> {
        self.client.unary_call_async(&METHOD_BACKEND_GET_UUIDS, req, opt)
    }

    pub fn get_uuids_async(&self, req: &super::backend::Empty) -> ::grpcio::Result<::grpcio::ClientUnaryReceiver<super::backend::UuidList>> {
        self.get_uuids_async_opt(req, ::grpcio::CallOption::default())
    }
    pub fn spawn<F>(&self, f: F) where F: ::futures::Future<Item = (), Error = ()> + Send + 'static {
        self.client.spawn(f)
    }
}

pub trait Backend {
    fn create_uuid(&self, ctx: ::grpcio::RpcContext, req: super::backend::CreateUuidRequest, sink: ::grpcio::UnarySink<super::backend::CreateUuidResult>);
    fn gen_uuid(&self, ctx: ::grpcio::RpcContext, req: super::backend::Layer, sink: ::grpcio::UnarySink<super::backend::GenUuidResult>);
    fn uuid_exists(&self, ctx: ::grpcio::RpcContext, req: super::backend::Layer, sink: ::grpcio::UnarySink<super::backend::Result>);
    fn cancel_upload(&self, ctx: ::grpcio::RpcContext, req: super::backend::Layer, sink: ::grpcio::UnarySink<super::backend::Result>);
    fn delete_uuid(&self, ctx: ::grpcio::RpcContext, req: super::backend::Layer, sink: ::grpcio::UnarySink<super::backend::Result>);
    fn upload_manifest(&self, ctx: ::grpcio::RpcContext, req: super::backend::Manifest, sink: ::grpcio::UnarySink<super::backend::Result>);
    fn get_uuids(&self, ctx: ::grpcio::RpcContext, req: super::backend::Empty, sink: ::grpcio::UnarySink<super::backend::UuidList>);
}

pub fn create_backend<S: Backend + Send + Clone + 'static>(s: S) -> ::grpcio::Service {
    let mut builder = ::grpcio::ServiceBuilder::new();
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_BACKEND_CREATE_UUID, move |ctx, req, resp| {
        instance.create_uuid(ctx, req, resp)
    });
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_BACKEND_GEN_UUID, move |ctx, req, resp| {
        instance.gen_uuid(ctx, req, resp)
    });
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_BACKEND_UUID_EXISTS, move |ctx, req, resp| {
        instance.uuid_exists(ctx, req, resp)
    });
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_BACKEND_CANCEL_UPLOAD, move |ctx, req, resp| {
        instance.cancel_upload(ctx, req, resp)
    });
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_BACKEND_DELETE_UUID, move |ctx, req, resp| {
        instance.delete_uuid(ctx, req, resp)
    });
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_BACKEND_UPLOAD_MANIFEST, move |ctx, req, resp| {
        instance.upload_manifest(ctx, req, resp)
    });
    let instance = s.clone();
    builder = builder.add_unary_handler(&METHOD_BACKEND_GET_UUIDS, move |ctx, req, resp| {
        instance.get_uuids(ctx, req, resp)
    });
    builder.build()
}
