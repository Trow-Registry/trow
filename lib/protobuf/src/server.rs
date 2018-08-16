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
pub struct ReadLocation {
    // message fields
    pub path: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for ReadLocation {}

impl ReadLocation {
    pub fn new() -> ReadLocation {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static ReadLocation {
        static mut instance: ::protobuf::lazy::Lazy<ReadLocation> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ReadLocation,
        };
        unsafe {
            instance.get(ReadLocation::new)
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

impl ::protobuf::Message for ReadLocation {
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

impl ::protobuf::MessageStatic for ReadLocation {
    fn new() -> ReadLocation {
        ReadLocation::new()
    }

    fn descriptor_static(_: ::std::option::Option<ReadLocation>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "path",
                    ReadLocation::get_path_for_reflect,
                    ReadLocation::mut_path_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<ReadLocation>(
                    "ReadLocation",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for ReadLocation {
    fn clear(&mut self) {
        self.clear_path();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ReadLocation {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ReadLocation {
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
    pub location: ::std::string::String,
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

    // string location = 2;

    pub fn clear_location(&mut self) {
        self.location.clear();
    }

    // Param is passed by value, moved
    pub fn set_location(&mut self, v: ::std::string::String) {
        self.location = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_location(&mut self) -> &mut ::std::string::String {
        &mut self.location
    }

    // Take field
    pub fn take_location(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.location, ::std::string::String::new())
    }

    pub fn get_location(&self) -> &str {
        &self.location
    }

    fn get_location_for_reflect(&self) -> &::std::string::String {
        &self.location
    }

    fn mut_location_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.location
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
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.location)?;
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
        if !self.location.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.location);
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
        if !self.location.is_empty() {
            os.write_string(2, &self.location)?;
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
                    "location",
                    VerifiedManifest::get_location_for_reflect,
                    VerifiedManifest::mut_location_for_reflect,
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
        self.clear_location();
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

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x0cserver.proto\x12\x06lycaon\",\n\rUploadRequest\x12\x1b\n\trepo_nam\
    e\x18\x01\x20\x01(\tR\x08repoName\"#\n\rUploadDetails\x12\x12\n\x04uuid\
    \x18\x01\x20\x01(\tR\x04uuid\":\n\x07BlobRef\x12\x1b\n\trepo_name\x18\
    \x01\x20\x01(\tR\x08repoName\x12\x12\n\x04uuid\x18\x02\x20\x01(\tR\x04uu\
    id\"B\n\x0bDownloadRef\x12\x1b\n\trepo_name\x18\x01\x20\x01(\tR\x08repoN\
    ame\x12\x16\n\x06digest\x18\x02\x20\x01(\tR\x06digest\"#\n\rWriteLocatio\
    n\x12\x12\n\x04path\x18\x01\x20\x01(\tR\x04path\"\"\n\x0cReadLocation\
    \x12\x12\n\x04path\x18\x01\x20\x01(\tR\x04path\"c\n\x0fCompleteRequest\
    \x12\x1b\n\trepo_name\x18\x01\x20\x01(\tR\x08repoName\x12\x12\n\x04uuid\
    \x18\x02\x20\x01(\tR\x04uuid\x12\x1f\n\x0buser_digest\x18\x03\x20\x01(\t\
    R\nuserDigest\")\n\x0fCompletedUpload\x12\x16\n\x06digest\x18\x01\x20\
    \x01(\tR\x06digest\"H\n\x0bManifestRef\x12\x1b\n\trepo_name\x18\x01\x20\
    \x01(\tR\x08repoName\x12\x1c\n\treference\x18\x02\x20\x01(\tR\treference\
    \"i\n\x10VerifiedManifest\x12\x16\n\x06digest\x18\x01\x20\x01(\tR\x06dig\
    est\x12\x1a\n\x08location\x18\x02\x20\x01(\tR\x08location\x12!\n\x0ccont\
    ent_type\x18\x03\x20\x01(\tR\x0bcontentType2\xfb\x03\n\x07Backend\x12?\n\
    \rRequestUpload\x12\x15.lycaon.UploadRequest\x1a\x15.lycaon.UploadDetail\
    s\"\0\x12C\n\x17GetWriteLocationForBlob\x12\x0f.lycaon.BlobRef\x1a\x15.l\
    ycaon.WriteLocation\"\0\x12E\n\x16GetReadLocationForBlob\x12\x13.lycaon.\
    DownloadRef\x1a\x14.lycaon.ReadLocation\"\0\x12K\n\x1bGetWriteLocationFo\
    rManifest\x12\x13.lycaon.ManifestRef\x1a\x15.lycaon.WriteLocation\"\0\
    \x12M\n\x1aGetReadLocationForManifest\x12\x13.lycaon.ManifestRef\x1a\x18\
    .lycaon.VerifiedManifest\"\0\x12A\n\x0eVerifyManifest\x12\x13.lycaon.Man\
    ifestRef\x1a\x18.lycaon.VerifiedManifest\"\0\x12D\n\x0eCompleteUpload\
    \x12\x17.lycaon.CompleteRequest\x1a\x17.lycaon.CompletedUpload\"\0b\x06p\
    roto3\
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
