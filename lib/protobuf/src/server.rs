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

use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;

#[derive(PartialEq,Clone,Default)]
pub struct UploadRequest {
    // message fields
    pub repo_name: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for UploadRequest {}

impl UploadRequest {
    pub fn new() -> UploadRequest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static UploadRequest {
        static mut instance: ::protobuf::lazy::Lazy<UploadRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const UploadRequest,
        };
        unsafe {
            instance.get(UploadRequest::new)
        }
    }

    // string repo_name = 1;

    pub fn clear_repo_name(&mut self) {
        self.repo_name.clear();
    }

    // Param is passed by value, moved
    pub fn set_repo_name(&mut self, v: ::std::string::String) {
        self.repo_name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_repo_name(&mut self) -> &mut ::std::string::String {
        &mut self.repo_name
    }

    // Take field
    pub fn take_repo_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.repo_name, ::std::string::String::new())
    }

    pub fn get_repo_name(&self) -> &str {
        &self.repo_name
    }

    fn get_repo_name_for_reflect(&self) -> &::std::string::String {
        &self.repo_name
    }

    fn mut_repo_name_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.repo_name
    }
}

impl ::protobuf::Message for UploadRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.repo_name)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.repo_name.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.repo_name);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.repo_name.is_empty() {
            os.write_string(1, &self.repo_name)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for UploadRequest {
    fn new() -> UploadRequest {
        UploadRequest::new()
    }

    fn descriptor_static(_: ::std::option::Option<UploadRequest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "repo_name",
                    UploadRequest::get_repo_name_for_reflect,
                    UploadRequest::mut_repo_name_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<UploadRequest>(
                    "UploadRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for UploadRequest {
    fn clear(&mut self) {
        self.clear_repo_name();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for UploadRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for UploadRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct UploadDetails {
    // message fields
    pub uuid: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for UploadDetails {}

impl UploadDetails {
    pub fn new() -> UploadDetails {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static UploadDetails {
        static mut instance: ::protobuf::lazy::Lazy<UploadDetails> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const UploadDetails,
        };
        unsafe {
            instance.get(UploadDetails::new)
        }
    }

    // string uuid = 1;

    pub fn clear_uuid(&mut self) {
        self.uuid.clear();
    }

    // Param is passed by value, moved
    pub fn set_uuid(&mut self, v: ::std::string::String) {
        self.uuid = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_uuid(&mut self) -> &mut ::std::string::String {
        &mut self.uuid
    }

    // Take field
    pub fn take_uuid(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.uuid, ::std::string::String::new())
    }

    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }

    fn get_uuid_for_reflect(&self) -> &::std::string::String {
        &self.uuid
    }

    fn mut_uuid_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.uuid
    }
}

impl ::protobuf::Message for UploadDetails {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.uuid)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.uuid.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.uuid);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.uuid.is_empty() {
            os.write_string(1, &self.uuid)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for UploadDetails {
    fn new() -> UploadDetails {
        UploadDetails::new()
    }

    fn descriptor_static(_: ::std::option::Option<UploadDetails>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "uuid",
                    UploadDetails::get_uuid_for_reflect,
                    UploadDetails::mut_uuid_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<UploadDetails>(
                    "UploadDetails",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for UploadDetails {
    fn clear(&mut self) {
        self.clear_uuid();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for UploadDetails {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for UploadDetails {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct BlobRef {
    // message fields
    pub repo_name: ::std::string::String,
    pub uuid: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for BlobRef {}

impl BlobRef {
    pub fn new() -> BlobRef {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static BlobRef {
        static mut instance: ::protobuf::lazy::Lazy<BlobRef> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const BlobRef,
        };
        unsafe {
            instance.get(BlobRef::new)
        }
    }

    // string repo_name = 1;

    pub fn clear_repo_name(&mut self) {
        self.repo_name.clear();
    }

    // Param is passed by value, moved
    pub fn set_repo_name(&mut self, v: ::std::string::String) {
        self.repo_name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_repo_name(&mut self) -> &mut ::std::string::String {
        &mut self.repo_name
    }

    // Take field
    pub fn take_repo_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.repo_name, ::std::string::String::new())
    }

    pub fn get_repo_name(&self) -> &str {
        &self.repo_name
    }

    fn get_repo_name_for_reflect(&self) -> &::std::string::String {
        &self.repo_name
    }

    fn mut_repo_name_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.repo_name
    }

    // string uuid = 2;

    pub fn clear_uuid(&mut self) {
        self.uuid.clear();
    }

    // Param is passed by value, moved
    pub fn set_uuid(&mut self, v: ::std::string::String) {
        self.uuid = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_uuid(&mut self) -> &mut ::std::string::String {
        &mut self.uuid
    }

    // Take field
    pub fn take_uuid(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.uuid, ::std::string::String::new())
    }

    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }

    fn get_uuid_for_reflect(&self) -> &::std::string::String {
        &self.uuid
    }

    fn mut_uuid_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.uuid
    }
}

impl ::protobuf::Message for BlobRef {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.repo_name)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.uuid)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.repo_name.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.repo_name);
        }
        if !self.uuid.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.uuid);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.repo_name.is_empty() {
            os.write_string(1, &self.repo_name)?;
        }
        if !self.uuid.is_empty() {
            os.write_string(2, &self.uuid)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for BlobRef {
    fn new() -> BlobRef {
        BlobRef::new()
    }

    fn descriptor_static(_: ::std::option::Option<BlobRef>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "repo_name",
                    BlobRef::get_repo_name_for_reflect,
                    BlobRef::mut_repo_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "uuid",
                    BlobRef::get_uuid_for_reflect,
                    BlobRef::mut_uuid_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<BlobRef>(
                    "BlobRef",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for BlobRef {
    fn clear(&mut self) {
        self.clear_repo_name();
        self.clear_uuid();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for BlobRef {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for BlobRef {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct DownloadRef {
    // message fields
    pub repo_name: ::std::string::String,
    pub digest: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for DownloadRef {}

impl DownloadRef {
    pub fn new() -> DownloadRef {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static DownloadRef {
        static mut instance: ::protobuf::lazy::Lazy<DownloadRef> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const DownloadRef,
        };
        unsafe {
            instance.get(DownloadRef::new)
        }
    }

    // string repo_name = 1;

    pub fn clear_repo_name(&mut self) {
        self.repo_name.clear();
    }

    // Param is passed by value, moved
    pub fn set_repo_name(&mut self, v: ::std::string::String) {
        self.repo_name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_repo_name(&mut self) -> &mut ::std::string::String {
        &mut self.repo_name
    }

    // Take field
    pub fn take_repo_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.repo_name, ::std::string::String::new())
    }

    pub fn get_repo_name(&self) -> &str {
        &self.repo_name
    }

    fn get_repo_name_for_reflect(&self) -> &::std::string::String {
        &self.repo_name
    }

    fn mut_repo_name_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.repo_name
    }

    // string digest = 2;

    pub fn clear_digest(&mut self) {
        self.digest.clear();
    }

    // Param is passed by value, moved
    pub fn set_digest(&mut self, v: ::std::string::String) {
        self.digest = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_digest(&mut self) -> &mut ::std::string::String {
        &mut self.digest
    }

    // Take field
    pub fn take_digest(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.digest, ::std::string::String::new())
    }

    pub fn get_digest(&self) -> &str {
        &self.digest
    }

    fn get_digest_for_reflect(&self) -> &::std::string::String {
        &self.digest
    }

    fn mut_digest_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.digest
    }
}

impl ::protobuf::Message for DownloadRef {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.repo_name)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.digest)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.repo_name.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.repo_name);
        }
        if !self.digest.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.digest);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.repo_name.is_empty() {
            os.write_string(1, &self.repo_name)?;
        }
        if !self.digest.is_empty() {
            os.write_string(2, &self.digest)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for DownloadRef {
    fn new() -> DownloadRef {
        DownloadRef::new()
    }

    fn descriptor_static(_: ::std::option::Option<DownloadRef>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "repo_name",
                    DownloadRef::get_repo_name_for_reflect,
                    DownloadRef::mut_repo_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "digest",
                    DownloadRef::get_digest_for_reflect,
                    DownloadRef::mut_digest_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<DownloadRef>(
                    "DownloadRef",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for DownloadRef {
    fn clear(&mut self) {
        self.clear_repo_name();
        self.clear_digest();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for DownloadRef {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for DownloadRef {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct WriteLocation {
    // message fields
    pub path: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for WriteLocation {}

impl WriteLocation {
    pub fn new() -> WriteLocation {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static WriteLocation {
        static mut instance: ::protobuf::lazy::Lazy<WriteLocation> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const WriteLocation,
        };
        unsafe {
            instance.get(WriteLocation::new)
        }
    }

    // string path = 1;

    pub fn clear_path(&mut self) {
        self.path.clear();
    }

    // Param is passed by value, moved
    pub fn set_path(&mut self, v: ::std::string::String) {
        self.path = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_path(&mut self) -> &mut ::std::string::String {
        &mut self.path
    }

    // Take field
    pub fn take_path(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.path, ::std::string::String::new())
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }

    fn get_path_for_reflect(&self) -> &::std::string::String {
        &self.path
    }

    fn mut_path_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.path
    }
}

impl ::protobuf::Message for WriteLocation {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.path)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.path.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.path);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.path.is_empty() {
            os.write_string(1, &self.path)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for WriteLocation {
    fn new() -> WriteLocation {
        WriteLocation::new()
    }

    fn descriptor_static(_: ::std::option::Option<WriteLocation>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "path",
                    WriteLocation::get_path_for_reflect,
                    WriteLocation::mut_path_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<WriteLocation>(
                    "WriteLocation",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for WriteLocation {
    fn clear(&mut self) {
        self.clear_path();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for WriteLocation {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for WriteLocation {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct BlobReadLocation {
    // message fields
    pub path: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for BlobReadLocation {}

impl BlobReadLocation {
    pub fn new() -> BlobReadLocation {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static BlobReadLocation {
        static mut instance: ::protobuf::lazy::Lazy<BlobReadLocation> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const BlobReadLocation,
        };
        unsafe {
            instance.get(BlobReadLocation::new)
        }
    }

    // string path = 1;

    pub fn clear_path(&mut self) {
        self.path.clear();
    }

    // Param is passed by value, moved
    pub fn set_path(&mut self, v: ::std::string::String) {
        self.path = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_path(&mut self) -> &mut ::std::string::String {
        &mut self.path
    }

    // Take field
    pub fn take_path(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.path, ::std::string::String::new())
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }

    fn get_path_for_reflect(&self) -> &::std::string::String {
        &self.path
    }

    fn mut_path_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.path
    }
}

impl ::protobuf::Message for BlobReadLocation {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.path)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.path.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.path);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.path.is_empty() {
            os.write_string(1, &self.path)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for BlobReadLocation {
    fn new() -> BlobReadLocation {
        BlobReadLocation::new()
    }

    fn descriptor_static(_: ::std::option::Option<BlobReadLocation>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "path",
                    BlobReadLocation::get_path_for_reflect,
                    BlobReadLocation::mut_path_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<BlobReadLocation>(
                    "BlobReadLocation",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for BlobReadLocation {
    fn clear(&mut self) {
        self.clear_path();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for BlobReadLocation {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for BlobReadLocation {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct CompleteRequest {
    // message fields
    pub repo_name: ::std::string::String,
    pub uuid: ::std::string::String,
    pub user_digest: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for CompleteRequest {}

impl CompleteRequest {
    pub fn new() -> CompleteRequest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static CompleteRequest {
        static mut instance: ::protobuf::lazy::Lazy<CompleteRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const CompleteRequest,
        };
        unsafe {
            instance.get(CompleteRequest::new)
        }
    }

    // string repo_name = 1;

    pub fn clear_repo_name(&mut self) {
        self.repo_name.clear();
    }

    // Param is passed by value, moved
    pub fn set_repo_name(&mut self, v: ::std::string::String) {
        self.repo_name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_repo_name(&mut self) -> &mut ::std::string::String {
        &mut self.repo_name
    }

    // Take field
    pub fn take_repo_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.repo_name, ::std::string::String::new())
    }

    pub fn get_repo_name(&self) -> &str {
        &self.repo_name
    }

    fn get_repo_name_for_reflect(&self) -> &::std::string::String {
        &self.repo_name
    }

    fn mut_repo_name_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.repo_name
    }

    // string uuid = 2;

    pub fn clear_uuid(&mut self) {
        self.uuid.clear();
    }

    // Param is passed by value, moved
    pub fn set_uuid(&mut self, v: ::std::string::String) {
        self.uuid = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_uuid(&mut self) -> &mut ::std::string::String {
        &mut self.uuid
    }

    // Take field
    pub fn take_uuid(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.uuid, ::std::string::String::new())
    }

    pub fn get_uuid(&self) -> &str {
        &self.uuid
    }

    fn get_uuid_for_reflect(&self) -> &::std::string::String {
        &self.uuid
    }

    fn mut_uuid_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.uuid
    }

    // string user_digest = 3;

    pub fn clear_user_digest(&mut self) {
        self.user_digest.clear();
    }

    // Param is passed by value, moved
    pub fn set_user_digest(&mut self, v: ::std::string::String) {
        self.user_digest = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_user_digest(&mut self) -> &mut ::std::string::String {
        &mut self.user_digest
    }

    // Take field
    pub fn take_user_digest(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.user_digest, ::std::string::String::new())
    }

    pub fn get_user_digest(&self) -> &str {
        &self.user_digest
    }

    fn get_user_digest_for_reflect(&self) -> &::std::string::String {
        &self.user_digest
    }

    fn mut_user_digest_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.user_digest
    }
}

impl ::protobuf::Message for CompleteRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.repo_name)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.uuid)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.user_digest)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.repo_name.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.repo_name);
        }
        if !self.uuid.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.uuid);
        }
        if !self.user_digest.is_empty() {
            my_size += ::protobuf::rt::string_size(3, &self.user_digest);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.repo_name.is_empty() {
            os.write_string(1, &self.repo_name)?;
        }
        if !self.uuid.is_empty() {
            os.write_string(2, &self.uuid)?;
        }
        if !self.user_digest.is_empty() {
            os.write_string(3, &self.user_digest)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for CompleteRequest {
    fn new() -> CompleteRequest {
        CompleteRequest::new()
    }

    fn descriptor_static(_: ::std::option::Option<CompleteRequest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "repo_name",
                    CompleteRequest::get_repo_name_for_reflect,
                    CompleteRequest::mut_repo_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "uuid",
                    CompleteRequest::get_uuid_for_reflect,
                    CompleteRequest::mut_uuid_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "user_digest",
                    CompleteRequest::get_user_digest_for_reflect,
                    CompleteRequest::mut_user_digest_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<CompleteRequest>(
                    "CompleteRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for CompleteRequest {
    fn clear(&mut self) {
        self.clear_repo_name();
        self.clear_uuid();
        self.clear_user_digest();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for CompleteRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for CompleteRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct CompletedUpload {
    // message fields
    pub digest: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for CompletedUpload {}

impl CompletedUpload {
    pub fn new() -> CompletedUpload {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static CompletedUpload {
        static mut instance: ::protobuf::lazy::Lazy<CompletedUpload> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const CompletedUpload,
        };
        unsafe {
            instance.get(CompletedUpload::new)
        }
    }

    // string digest = 1;

    pub fn clear_digest(&mut self) {
        self.digest.clear();
    }

    // Param is passed by value, moved
    pub fn set_digest(&mut self, v: ::std::string::String) {
        self.digest = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_digest(&mut self) -> &mut ::std::string::String {
        &mut self.digest
    }

    // Take field
    pub fn take_digest(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.digest, ::std::string::String::new())
    }

    pub fn get_digest(&self) -> &str {
        &self.digest
    }

    fn get_digest_for_reflect(&self) -> &::std::string::String {
        &self.digest
    }

    fn mut_digest_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.digest
    }
}

impl ::protobuf::Message for CompletedUpload {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.digest)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.digest.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.digest);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.digest.is_empty() {
            os.write_string(1, &self.digest)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for CompletedUpload {
    fn new() -> CompletedUpload {
        CompletedUpload::new()
    }

    fn descriptor_static(_: ::std::option::Option<CompletedUpload>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "digest",
                    CompletedUpload::get_digest_for_reflect,
                    CompletedUpload::mut_digest_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<CompletedUpload>(
                    "CompletedUpload",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for CompletedUpload {
    fn clear(&mut self) {
        self.clear_digest();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for CompletedUpload {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for CompletedUpload {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct ManifestRef {
    // message fields
    pub repo_name: ::std::string::String,
    pub reference: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for ManifestRef {}

impl ManifestRef {
    pub fn new() -> ManifestRef {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static ManifestRef {
        static mut instance: ::protobuf::lazy::Lazy<ManifestRef> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ManifestRef,
        };
        unsafe {
            instance.get(ManifestRef::new)
        }
    }

    // string repo_name = 1;

    pub fn clear_repo_name(&mut self) {
        self.repo_name.clear();
    }

    // Param is passed by value, moved
    pub fn set_repo_name(&mut self, v: ::std::string::String) {
        self.repo_name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_repo_name(&mut self) -> &mut ::std::string::String {
        &mut self.repo_name
    }

    // Take field
    pub fn take_repo_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.repo_name, ::std::string::String::new())
    }

    pub fn get_repo_name(&self) -> &str {
        &self.repo_name
    }

    fn get_repo_name_for_reflect(&self) -> &::std::string::String {
        &self.repo_name
    }

    fn mut_repo_name_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.repo_name
    }

    // string reference = 2;

    pub fn clear_reference(&mut self) {
        self.reference.clear();
    }

    // Param is passed by value, moved
    pub fn set_reference(&mut self, v: ::std::string::String) {
        self.reference = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_reference(&mut self) -> &mut ::std::string::String {
        &mut self.reference
    }

    // Take field
    pub fn take_reference(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.reference, ::std::string::String::new())
    }

    pub fn get_reference(&self) -> &str {
        &self.reference
    }

    fn get_reference_for_reflect(&self) -> &::std::string::String {
        &self.reference
    }

    fn mut_reference_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.reference
    }
}

impl ::protobuf::Message for ManifestRef {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.repo_name)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.reference)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.repo_name.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.repo_name);
        }
        if !self.reference.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.reference);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.repo_name.is_empty() {
            os.write_string(1, &self.repo_name)?;
        }
        if !self.reference.is_empty() {
            os.write_string(2, &self.reference)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for ManifestRef {
    fn new() -> ManifestRef {
        ManifestRef::new()
    }

    fn descriptor_static(_: ::std::option::Option<ManifestRef>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "repo_name",
                    ManifestRef::get_repo_name_for_reflect,
                    ManifestRef::mut_repo_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "reference",
                    ManifestRef::get_reference_for_reflect,
                    ManifestRef::mut_reference_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<ManifestRef>(
                    "ManifestRef",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for ManifestRef {
    fn clear(&mut self) {
        self.clear_repo_name();
        self.clear_reference();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ManifestRef {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ManifestRef {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct VerifiedManifest {
    // message fields
    pub digest: ::std::string::String,
    pub content_type: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for VerifiedManifest {}

impl VerifiedManifest {
    pub fn new() -> VerifiedManifest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static VerifiedManifest {
        static mut instance: ::protobuf::lazy::Lazy<VerifiedManifest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const VerifiedManifest,
        };
        unsafe {
            instance.get(VerifiedManifest::new)
        }
    }

    // string digest = 1;

    pub fn clear_digest(&mut self) {
        self.digest.clear();
    }

    // Param is passed by value, moved
    pub fn set_digest(&mut self, v: ::std::string::String) {
        self.digest = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_digest(&mut self) -> &mut ::std::string::String {
        &mut self.digest
    }

    // Take field
    pub fn take_digest(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.digest, ::std::string::String::new())
    }

    pub fn get_digest(&self) -> &str {
        &self.digest
    }

    fn get_digest_for_reflect(&self) -> &::std::string::String {
        &self.digest
    }

    fn mut_digest_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.digest
    }

    // string content_type = 2;

    pub fn clear_content_type(&mut self) {
        self.content_type.clear();
    }

    // Param is passed by value, moved
    pub fn set_content_type(&mut self, v: ::std::string::String) {
        self.content_type = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_content_type(&mut self) -> &mut ::std::string::String {
        &mut self.content_type
    }

    // Take field
    pub fn take_content_type(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.content_type, ::std::string::String::new())
    }

    pub fn get_content_type(&self) -> &str {
        &self.content_type
    }

    fn get_content_type_for_reflect(&self) -> &::std::string::String {
        &self.content_type
    }

    fn mut_content_type_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.content_type
    }
}

impl ::protobuf::Message for VerifiedManifest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.digest)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.content_type)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.digest.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.digest);
        }
        if !self.content_type.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.content_type);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.digest.is_empty() {
            os.write_string(1, &self.digest)?;
        }
        if !self.content_type.is_empty() {
            os.write_string(2, &self.content_type)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for VerifiedManifest {
    fn new() -> VerifiedManifest {
        VerifiedManifest::new()
    }

    fn descriptor_static(_: ::std::option::Option<VerifiedManifest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "digest",
                    VerifiedManifest::get_digest_for_reflect,
                    VerifiedManifest::mut_digest_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "content_type",
                    VerifiedManifest::get_content_type_for_reflect,
                    VerifiedManifest::mut_content_type_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<VerifiedManifest>(
                    "VerifiedManifest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for VerifiedManifest {
    fn clear(&mut self) {
        self.clear_digest();
        self.clear_content_type();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for VerifiedManifest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for VerifiedManifest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct ManifestReadLocation {
    // message fields
    pub digest: ::std::string::String,
    pub path: ::std::string::String,
    pub content_type: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for ManifestReadLocation {}

impl ManifestReadLocation {
    pub fn new() -> ManifestReadLocation {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static ManifestReadLocation {
        static mut instance: ::protobuf::lazy::Lazy<ManifestReadLocation> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ManifestReadLocation,
        };
        unsafe {
            instance.get(ManifestReadLocation::new)
        }
    }

    // string digest = 1;

    pub fn clear_digest(&mut self) {
        self.digest.clear();
    }

    // Param is passed by value, moved
    pub fn set_digest(&mut self, v: ::std::string::String) {
        self.digest = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_digest(&mut self) -> &mut ::std::string::String {
        &mut self.digest
    }

    // Take field
    pub fn take_digest(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.digest, ::std::string::String::new())
    }

    pub fn get_digest(&self) -> &str {
        &self.digest
    }

    fn get_digest_for_reflect(&self) -> &::std::string::String {
        &self.digest
    }

    fn mut_digest_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.digest
    }

    // string path = 2;

    pub fn clear_path(&mut self) {
        self.path.clear();
    }

    // Param is passed by value, moved
    pub fn set_path(&mut self, v: ::std::string::String) {
        self.path = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_path(&mut self) -> &mut ::std::string::String {
        &mut self.path
    }

    // Take field
    pub fn take_path(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.path, ::std::string::String::new())
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }

    fn get_path_for_reflect(&self) -> &::std::string::String {
        &self.path
    }

    fn mut_path_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.path
    }

    // string content_type = 3;

    pub fn clear_content_type(&mut self) {
        self.content_type.clear();
    }

    // Param is passed by value, moved
    pub fn set_content_type(&mut self, v: ::std::string::String) {
        self.content_type = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_content_type(&mut self) -> &mut ::std::string::String {
        &mut self.content_type
    }

    // Take field
    pub fn take_content_type(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.content_type, ::std::string::String::new())
    }

    pub fn get_content_type(&self) -> &str {
        &self.content_type
    }

    fn get_content_type_for_reflect(&self) -> &::std::string::String {
        &self.content_type
    }

    fn mut_content_type_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.content_type
    }
}

impl ::protobuf::Message for ManifestReadLocation {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.digest)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.path)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.content_type)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.digest.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.digest);
        }
        if !self.path.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.path);
        }
        if !self.content_type.is_empty() {
            my_size += ::protobuf::rt::string_size(3, &self.content_type);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.digest.is_empty() {
            os.write_string(1, &self.digest)?;
        }
        if !self.path.is_empty() {
            os.write_string(2, &self.path)?;
        }
        if !self.content_type.is_empty() {
            os.write_string(3, &self.content_type)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for ManifestReadLocation {
    fn new() -> ManifestReadLocation {
        ManifestReadLocation::new()
    }

    fn descriptor_static(_: ::std::option::Option<ManifestReadLocation>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "digest",
                    ManifestReadLocation::get_digest_for_reflect,
                    ManifestReadLocation::mut_digest_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "path",
                    ManifestReadLocation::get_path_for_reflect,
                    ManifestReadLocation::mut_path_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "content_type",
                    ManifestReadLocation::get_content_type_for_reflect,
                    ManifestReadLocation::mut_content_type_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<ManifestReadLocation>(
                    "ManifestReadLocation",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for ManifestReadLocation {
    fn clear(&mut self) {
        self.clear_digest();
        self.clear_path();
        self.clear_content_type();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ManifestReadLocation {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ManifestReadLocation {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct CatalogRequest {
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for CatalogRequest {}

impl CatalogRequest {
    pub fn new() -> CatalogRequest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static CatalogRequest {
        static mut instance: ::protobuf::lazy::Lazy<CatalogRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const CatalogRequest,
        };
        unsafe {
            instance.get(CatalogRequest::new)
        }
    }
}

impl ::protobuf::Message for CatalogRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for CatalogRequest {
    fn new() -> CatalogRequest {
        CatalogRequest::new()
    }

    fn descriptor_static(_: ::std::option::Option<CatalogRequest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let fields = ::std::vec::Vec::new();
                ::protobuf::reflect::MessageDescriptor::new::<CatalogRequest>(
                    "CatalogRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for CatalogRequest {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for CatalogRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for CatalogRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct CatalogEntry {
    // message fields
    pub repo_name: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for CatalogEntry {}

impl CatalogEntry {
    pub fn new() -> CatalogEntry {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static CatalogEntry {
        static mut instance: ::protobuf::lazy::Lazy<CatalogEntry> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const CatalogEntry,
        };
        unsafe {
            instance.get(CatalogEntry::new)
        }
    }

    // string repo_name = 1;

    pub fn clear_repo_name(&mut self) {
        self.repo_name.clear();
    }

    // Param is passed by value, moved
    pub fn set_repo_name(&mut self, v: ::std::string::String) {
        self.repo_name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_repo_name(&mut self) -> &mut ::std::string::String {
        &mut self.repo_name
    }

    // Take field
    pub fn take_repo_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.repo_name, ::std::string::String::new())
    }

    pub fn get_repo_name(&self) -> &str {
        &self.repo_name
    }

    fn get_repo_name_for_reflect(&self) -> &::std::string::String {
        &self.repo_name
    }

    fn mut_repo_name_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.repo_name
    }
}

impl ::protobuf::Message for CatalogEntry {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.repo_name)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.repo_name.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.repo_name);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.repo_name.is_empty() {
            os.write_string(1, &self.repo_name)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for CatalogEntry {
    fn new() -> CatalogEntry {
        CatalogEntry::new()
    }

    fn descriptor_static(_: ::std::option::Option<CatalogEntry>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "repo_name",
                    CatalogEntry::get_repo_name_for_reflect,
                    CatalogEntry::mut_repo_name_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<CatalogEntry>(
                    "CatalogEntry",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for CatalogEntry {
    fn clear(&mut self) {
        self.clear_repo_name();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for CatalogEntry {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for CatalogEntry {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Tag {
    // message fields
    pub tag: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Tag {}

impl Tag {
    pub fn new() -> Tag {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Tag {
        static mut instance: ::protobuf::lazy::Lazy<Tag> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Tag,
        };
        unsafe {
            instance.get(Tag::new)
        }
    }

    // string tag = 1;

    pub fn clear_tag(&mut self) {
        self.tag.clear();
    }

    // Param is passed by value, moved
    pub fn set_tag(&mut self, v: ::std::string::String) {
        self.tag = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_tag(&mut self) -> &mut ::std::string::String {
        &mut self.tag
    }

    // Take field
    pub fn take_tag(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.tag, ::std::string::String::new())
    }

    pub fn get_tag(&self) -> &str {
        &self.tag
    }

    fn get_tag_for_reflect(&self) -> &::std::string::String {
        &self.tag
    }

    fn mut_tag_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.tag
    }
}

impl ::protobuf::Message for Tag {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.tag)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.tag.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.tag);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.tag.is_empty() {
            os.write_string(1, &self.tag)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for Tag {
    fn new() -> Tag {
        Tag::new()
    }

    fn descriptor_static(_: ::std::option::Option<Tag>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "tag",
                    Tag::get_tag_for_reflect,
                    Tag::mut_tag_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Tag>(
                    "Tag",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Tag {
    fn clear(&mut self) {
        self.clear_tag();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Tag {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Tag {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct AdmissionRequest {
    // message fields
    pub images: ::protobuf::RepeatedField<::std::string::String>,
    pub namespace: ::std::string::String,
    pub operation: ::std::string::String,
    pub registry_name: ::std::string::String,
    pub host_names: ::protobuf::RepeatedField<::std::string::String>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for AdmissionRequest {}

impl AdmissionRequest {
    pub fn new() -> AdmissionRequest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static AdmissionRequest {
        static mut instance: ::protobuf::lazy::Lazy<AdmissionRequest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const AdmissionRequest,
        };
        unsafe {
            instance.get(AdmissionRequest::new)
        }
    }

    // repeated string images = 1;

    pub fn clear_images(&mut self) {
        self.images.clear();
    }

    // Param is passed by value, moved
    pub fn set_images(&mut self, v: ::protobuf::RepeatedField<::std::string::String>) {
        self.images = v;
    }

    // Mutable pointer to the field.
    pub fn mut_images(&mut self) -> &mut ::protobuf::RepeatedField<::std::string::String> {
        &mut self.images
    }

    // Take field
    pub fn take_images(&mut self) -> ::protobuf::RepeatedField<::std::string::String> {
        ::std::mem::replace(&mut self.images, ::protobuf::RepeatedField::new())
    }

    pub fn get_images(&self) -> &[::std::string::String] {
        &self.images
    }

    fn get_images_for_reflect(&self) -> &::protobuf::RepeatedField<::std::string::String> {
        &self.images
    }

    fn mut_images_for_reflect(&mut self) -> &mut ::protobuf::RepeatedField<::std::string::String> {
        &mut self.images
    }

    // string namespace = 2;

    pub fn clear_namespace(&mut self) {
        self.namespace.clear();
    }

    // Param is passed by value, moved
    pub fn set_namespace(&mut self, v: ::std::string::String) {
        self.namespace = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_namespace(&mut self) -> &mut ::std::string::String {
        &mut self.namespace
    }

    // Take field
    pub fn take_namespace(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.namespace, ::std::string::String::new())
    }

    pub fn get_namespace(&self) -> &str {
        &self.namespace
    }

    fn get_namespace_for_reflect(&self) -> &::std::string::String {
        &self.namespace
    }

    fn mut_namespace_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.namespace
    }

    // string operation = 3;

    pub fn clear_operation(&mut self) {
        self.operation.clear();
    }

    // Param is passed by value, moved
    pub fn set_operation(&mut self, v: ::std::string::String) {
        self.operation = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_operation(&mut self) -> &mut ::std::string::String {
        &mut self.operation
    }

    // Take field
    pub fn take_operation(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.operation, ::std::string::String::new())
    }

    pub fn get_operation(&self) -> &str {
        &self.operation
    }

    fn get_operation_for_reflect(&self) -> &::std::string::String {
        &self.operation
    }

    fn mut_operation_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.operation
    }

    // string registry_name = 4;

    pub fn clear_registry_name(&mut self) {
        self.registry_name.clear();
    }

    // Param is passed by value, moved
    pub fn set_registry_name(&mut self, v: ::std::string::String) {
        self.registry_name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_registry_name(&mut self) -> &mut ::std::string::String {
        &mut self.registry_name
    }

    // Take field
    pub fn take_registry_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.registry_name, ::std::string::String::new())
    }

    pub fn get_registry_name(&self) -> &str {
        &self.registry_name
    }

    fn get_registry_name_for_reflect(&self) -> &::std::string::String {
        &self.registry_name
    }

    fn mut_registry_name_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.registry_name
    }

    // repeated string host_names = 5;

    pub fn clear_host_names(&mut self) {
        self.host_names.clear();
    }

    // Param is passed by value, moved
    pub fn set_host_names(&mut self, v: ::protobuf::RepeatedField<::std::string::String>) {
        self.host_names = v;
    }

    // Mutable pointer to the field.
    pub fn mut_host_names(&mut self) -> &mut ::protobuf::RepeatedField<::std::string::String> {
        &mut self.host_names
    }

    // Take field
    pub fn take_host_names(&mut self) -> ::protobuf::RepeatedField<::std::string::String> {
        ::std::mem::replace(&mut self.host_names, ::protobuf::RepeatedField::new())
    }

    pub fn get_host_names(&self) -> &[::std::string::String] {
        &self.host_names
    }

    fn get_host_names_for_reflect(&self) -> &::protobuf::RepeatedField<::std::string::String> {
        &self.host_names
    }

    fn mut_host_names_for_reflect(&mut self) -> &mut ::protobuf::RepeatedField<::std::string::String> {
        &mut self.host_names
    }
}

impl ::protobuf::Message for AdmissionRequest {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_repeated_string_into(wire_type, is, &mut self.images)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.namespace)?;
                },
                3 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.operation)?;
                },
                4 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.registry_name)?;
                },
                5 => {
                    ::protobuf::rt::read_repeated_string_into(wire_type, is, &mut self.host_names)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        for value in &self.images {
            my_size += ::protobuf::rt::string_size(1, &value);
        };
        if !self.namespace.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.namespace);
        }
        if !self.operation.is_empty() {
            my_size += ::protobuf::rt::string_size(3, &self.operation);
        }
        if !self.registry_name.is_empty() {
            my_size += ::protobuf::rt::string_size(4, &self.registry_name);
        }
        for value in &self.host_names {
            my_size += ::protobuf::rt::string_size(5, &value);
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        for v in &self.images {
            os.write_string(1, &v)?;
        };
        if !self.namespace.is_empty() {
            os.write_string(2, &self.namespace)?;
        }
        if !self.operation.is_empty() {
            os.write_string(3, &self.operation)?;
        }
        if !self.registry_name.is_empty() {
            os.write_string(4, &self.registry_name)?;
        }
        for v in &self.host_names {
            os.write_string(5, &v)?;
        };
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for AdmissionRequest {
    fn new() -> AdmissionRequest {
        AdmissionRequest::new()
    }

    fn descriptor_static(_: ::std::option::Option<AdmissionRequest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "images",
                    AdmissionRequest::get_images_for_reflect,
                    AdmissionRequest::mut_images_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "namespace",
                    AdmissionRequest::get_namespace_for_reflect,
                    AdmissionRequest::mut_namespace_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "operation",
                    AdmissionRequest::get_operation_for_reflect,
                    AdmissionRequest::mut_operation_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "registry_name",
                    AdmissionRequest::get_registry_name_for_reflect,
                    AdmissionRequest::mut_registry_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "host_names",
                    AdmissionRequest::get_host_names_for_reflect,
                    AdmissionRequest::mut_host_names_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<AdmissionRequest>(
                    "AdmissionRequest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for AdmissionRequest {
    fn clear(&mut self) {
        self.clear_images();
        self.clear_namespace();
        self.clear_operation();
        self.clear_registry_name();
        self.clear_host_names();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for AdmissionRequest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for AdmissionRequest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct AdmissionResponse {
    // message fields
    pub is_allowed: bool,
    pub reason: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for AdmissionResponse {}

impl AdmissionResponse {
    pub fn new() -> AdmissionResponse {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static AdmissionResponse {
        static mut instance: ::protobuf::lazy::Lazy<AdmissionResponse> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const AdmissionResponse,
        };
        unsafe {
            instance.get(AdmissionResponse::new)
        }
    }

    // bool is_allowed = 1;

    pub fn clear_is_allowed(&mut self) {
        self.is_allowed = false;
    }

    // Param is passed by value, moved
    pub fn set_is_allowed(&mut self, v: bool) {
        self.is_allowed = v;
    }

    pub fn get_is_allowed(&self) -> bool {
        self.is_allowed
    }

    fn get_is_allowed_for_reflect(&self) -> &bool {
        &self.is_allowed
    }

    fn mut_is_allowed_for_reflect(&mut self) -> &mut bool {
        &mut self.is_allowed
    }

    // string reason = 2;

    pub fn clear_reason(&mut self) {
        self.reason.clear();
    }

    // Param is passed by value, moved
    pub fn set_reason(&mut self, v: ::std::string::String) {
        self.reason = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_reason(&mut self) -> &mut ::std::string::String {
        &mut self.reason
    }

    // Take field
    pub fn take_reason(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.reason, ::std::string::String::new())
    }

    pub fn get_reason(&self) -> &str {
        &self.reason
    }

    fn get_reason_for_reflect(&self) -> &::std::string::String {
        &self.reason
    }

    fn mut_reason_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.reason
    }
}

impl ::protobuf::Message for AdmissionResponse {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_bool()?;
                    self.is_allowed = tmp;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.reason)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if self.is_allowed != false {
            my_size += 2;
        }
        if !self.reason.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.reason);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.is_allowed != false {
            os.write_bool(1, self.is_allowed)?;
        }
        if !self.reason.is_empty() {
            os.write_string(2, &self.reason)?;
        }
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for AdmissionResponse {
    fn new() -> AdmissionResponse {
        AdmissionResponse::new()
    }

    fn descriptor_static(_: ::std::option::Option<AdmissionResponse>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "is_allowed",
                    AdmissionResponse::get_is_allowed_for_reflect,
                    AdmissionResponse::mut_is_allowed_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "reason",
                    AdmissionResponse::get_reason_for_reflect,
                    AdmissionResponse::mut_reason_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<AdmissionResponse>(
                    "AdmissionResponse",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for AdmissionResponse {
    fn clear(&mut self) {
        self.clear_is_allowed();
        self.clear_reason();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for AdmissionResponse {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for AdmissionResponse {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x0cserver.proto\x12\x06lycaon\",\n\rUploadRequest\x12\x1b\n\trepo_nam\
    e\x18\x01\x20\x01(\tR\x08repoName\"#\n\rUploadDetails\x12\x12\n\x04uuid\
    \x18\x01\x20\x01(\tR\x04uuid\":\n\x07BlobRef\x12\x1b\n\trepo_name\x18\
    \x01\x20\x01(\tR\x08repoName\x12\x12\n\x04uuid\x18\x02\x20\x01(\tR\x04uu\
    id\"B\n\x0bDownloadRef\x12\x1b\n\trepo_name\x18\x01\x20\x01(\tR\x08repoN\
    ame\x12\x16\n\x06digest\x18\x02\x20\x01(\tR\x06digest\"#\n\rWriteLocatio\
    n\x12\x12\n\x04path\x18\x01\x20\x01(\tR\x04path\"&\n\x10BlobReadLocation\
    \x12\x12\n\x04path\x18\x01\x20\x01(\tR\x04path\"c\n\x0fCompleteRequest\
    \x12\x1b\n\trepo_name\x18\x01\x20\x01(\tR\x08repoName\x12\x12\n\x04uuid\
    \x18\x02\x20\x01(\tR\x04uuid\x12\x1f\n\x0buser_digest\x18\x03\x20\x01(\t\
    R\nuserDigest\")\n\x0fCompletedUpload\x12\x16\n\x06digest\x18\x01\x20\
    \x01(\tR\x06digest\"H\n\x0bManifestRef\x12\x1b\n\trepo_name\x18\x01\x20\
    \x01(\tR\x08repoName\x12\x1c\n\treference\x18\x02\x20\x01(\tR\treference\
    \"M\n\x10VerifiedManifest\x12\x16\n\x06digest\x18\x01\x20\x01(\tR\x06dig\
    est\x12!\n\x0ccontent_type\x18\x02\x20\x01(\tR\x0bcontentType\"e\n\x14Ma\
    nifestReadLocation\x12\x16\n\x06digest\x18\x01\x20\x01(\tR\x06digest\x12\
    \x12\n\x04path\x18\x02\x20\x01(\tR\x04path\x12!\n\x0ccontent_type\x18\
    \x03\x20\x01(\tR\x0bcontentType\"\x10\n\x0eCatalogRequest\"+\n\x0cCatalo\
    gEntry\x12\x1b\n\trepo_name\x18\x01\x20\x01(\tR\x08repoName\"\x17\n\x03T\
    ag\x12\x10\n\x03tag\x18\x01\x20\x01(\tR\x03tag\"\xaa\x01\n\x10AdmissionR\
    equest\x12\x16\n\x06images\x18\x01\x20\x03(\tR\x06images\x12\x1c\n\tname\
    space\x18\x02\x20\x01(\tR\tnamespace\x12\x1c\n\toperation\x18\x03\x20\
    \x01(\tR\toperation\x12#\n\rregistry_name\x18\x04\x20\x01(\tR\x0cregistr\
    yName\x12\x1d\n\nhost_names\x18\x05\x20\x03(\tR\thostNames\"J\n\x11Admis\
    sionResponse\x12\x1d\n\nis_allowed\x18\x01\x20\x01(\x08R\tisAllowed\x12\
    \x16\n\x06reason\x18\x02\x20\x01(\tR\x06reason2\xf7\x04\n\x08Registry\
    \x12?\n\rRequestUpload\x12\x15.lycaon.UploadRequest\x1a\x15.lycaon.Uploa\
    dDetails\"\0\x12C\n\x17GetWriteLocationForBlob\x12\x0f.lycaon.BlobRef\
    \x1a\x15.lycaon.WriteLocation\"\0\x12I\n\x16GetReadLocationForBlob\x12\
    \x13.lycaon.DownloadRef\x1a\x18.lycaon.BlobReadLocation\"\0\x12K\n\x1bGe\
    tWriteLocationForManifest\x12\x13.lycaon.ManifestRef\x1a\x15.lycaon.Writ\
    eLocation\"\0\x12Q\n\x1aGetReadLocationForManifest\x12\x13.lycaon.Manife\
    stRef\x1a\x1c.lycaon.ManifestReadLocation\"\0\x12A\n\x0eVerifyManifest\
    \x12\x13.lycaon.ManifestRef\x1a\x18.lycaon.VerifiedManifest\"\0\x12D\n\
    \x0eCompleteUpload\x12\x17.lycaon.CompleteRequest\x1a\x17.lycaon.Complet\
    edUpload\"\0\x12>\n\nGetCatalog\x12\x16.lycaon.CatalogRequest\x1a\x14.ly\
    caon.CatalogEntry\"\00\x01\x121\n\x08ListTags\x12\x14.lycaon.CatalogEntr\
    y\x1a\x0b.lycaon.Tag\"\00\x012a\n\x13AdmissionController\x12J\n\x11Valid\
    ateAdmission\x12\x18.lycaon.AdmissionRequest\x1a\x19.lycaon.AdmissionRes\
    ponse\"\0b\x06proto3\
";

static mut file_descriptor_proto_lazy: ::protobuf::lazy::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::lazy::Lazy {
    lock: ::protobuf::lazy::ONCE_INIT,
    ptr: 0 as *const ::protobuf::descriptor::FileDescriptorProto,
};

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    unsafe {
        file_descriptor_proto_lazy.get(|| {
            parse_descriptor_proto()
        })
    }
}
