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
pub struct Layer {
    // message fields
    pub name: ::std::string::String,
    pub repo: ::std::string::String,
    pub digest: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Layer {}

impl Layer {
    pub fn new() -> Layer {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Layer {
        static mut instance: ::protobuf::lazy::Lazy<Layer> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Layer,
        };
        unsafe {
            instance.get(Layer::new)
        }
    }

    // string name = 1;

    pub fn clear_name(&mut self) {
        self.name.clear();
    }

    // Param is passed by value, moved
    pub fn set_name(&mut self, v: ::std::string::String) {
        self.name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_name(&mut self) -> &mut ::std::string::String {
        &mut self.name
    }

    // Take field
    pub fn take_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.name, ::std::string::String::new())
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    fn get_name_for_reflect(&self) -> &::std::string::String {
        &self.name
    }

    fn mut_name_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.name
    }

    // string repo = 2;

    pub fn clear_repo(&mut self) {
        self.repo.clear();
    }

    // Param is passed by value, moved
    pub fn set_repo(&mut self, v: ::std::string::String) {
        self.repo = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_repo(&mut self) -> &mut ::std::string::String {
        &mut self.repo
    }

    // Take field
    pub fn take_repo(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.repo, ::std::string::String::new())
    }

    pub fn get_repo(&self) -> &str {
        &self.repo
    }

    fn get_repo_for_reflect(&self) -> &::std::string::String {
        &self.repo
    }

    fn mut_repo_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.repo
    }

    // string digest = 3;

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

impl ::protobuf::Message for Layer {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.name)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.repo)?;
                },
                3 => {
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
        if !self.name.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.name);
        }
        if !self.repo.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.repo);
        }
        if !self.digest.is_empty() {
            my_size += ::protobuf::rt::string_size(3, &self.digest);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.name.is_empty() {
            os.write_string(1, &self.name)?;
        }
        if !self.repo.is_empty() {
            os.write_string(2, &self.repo)?;
        }
        if !self.digest.is_empty() {
            os.write_string(3, &self.digest)?;
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

impl ::protobuf::MessageStatic for Layer {
    fn new() -> Layer {
        Layer::new()
    }

    fn descriptor_static(_: ::std::option::Option<Layer>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "name",
                    Layer::get_name_for_reflect,
                    Layer::mut_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "repo",
                    Layer::get_repo_for_reflect,
                    Layer::mut_repo_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "digest",
                    Layer::get_digest_for_reflect,
                    Layer::mut_digest_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Layer>(
                    "Layer",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Layer {
    fn clear(&mut self) {
        self.clear_name();
        self.clear_repo();
        self.clear_digest();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Layer {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Layer {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Manifest {
    // message fields
    pub schemaVersion: u32,
    pub mediaType: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Manifest {}

impl Manifest {
    pub fn new() -> Manifest {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Manifest {
        static mut instance: ::protobuf::lazy::Lazy<Manifest> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Manifest,
        };
        unsafe {
            instance.get(Manifest::new)
        }
    }

    // uint32 schemaVersion = 1;

    pub fn clear_schemaVersion(&mut self) {
        self.schemaVersion = 0;
    }

    // Param is passed by value, moved
    pub fn set_schemaVersion(&mut self, v: u32) {
        self.schemaVersion = v;
    }

    pub fn get_schemaVersion(&self) -> u32 {
        self.schemaVersion
    }

    fn get_schemaVersion_for_reflect(&self) -> &u32 {
        &self.schemaVersion
    }

    fn mut_schemaVersion_for_reflect(&mut self) -> &mut u32 {
        &mut self.schemaVersion
    }

    // string mediaType = 2;

    pub fn clear_mediaType(&mut self) {
        self.mediaType.clear();
    }

    // Param is passed by value, moved
    pub fn set_mediaType(&mut self, v: ::std::string::String) {
        self.mediaType = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_mediaType(&mut self) -> &mut ::std::string::String {
        &mut self.mediaType
    }

    // Take field
    pub fn take_mediaType(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.mediaType, ::std::string::String::new())
    }

    pub fn get_mediaType(&self) -> &str {
        &self.mediaType
    }

    fn get_mediaType_for_reflect(&self) -> &::std::string::String {
        &self.mediaType
    }

    fn mut_mediaType_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.mediaType
    }
}

impl ::protobuf::Message for Manifest {
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
                    let tmp = is.read_uint32()?;
                    self.schemaVersion = tmp;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.mediaType)?;
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
        if self.schemaVersion != 0 {
            my_size += ::protobuf::rt::value_size(1, self.schemaVersion, ::protobuf::wire_format::WireTypeVarint);
        }
        if !self.mediaType.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.mediaType);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.schemaVersion != 0 {
            os.write_uint32(1, self.schemaVersion)?;
        }
        if !self.mediaType.is_empty() {
            os.write_string(2, &self.mediaType)?;
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

impl ::protobuf::MessageStatic for Manifest {
    fn new() -> Manifest {
        Manifest::new()
    }

    fn descriptor_static(_: ::std::option::Option<Manifest>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint32>(
                    "schemaVersion",
                    Manifest::get_schemaVersion_for_reflect,
                    Manifest::mut_schemaVersion_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "mediaType",
                    Manifest::get_mediaType_for_reflect,
                    Manifest::mut_mediaType_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Manifest>(
                    "Manifest",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Manifest {
    fn clear(&mut self) {
        self.clear_schemaVersion();
        self.clear_mediaType();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Manifest {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Manifest {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Empty {
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Empty {}

impl Empty {
    pub fn new() -> Empty {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Empty {
        static mut instance: ::protobuf::lazy::Lazy<Empty> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Empty,
        };
        unsafe {
            instance.get(Empty::new)
        }
    }
}

impl ::protobuf::Message for Empty {
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

impl ::protobuf::MessageStatic for Empty {
    fn new() -> Empty {
        Empty::new()
    }

    fn descriptor_static(_: ::std::option::Option<Empty>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let fields = ::std::vec::Vec::new();
                ::protobuf::reflect::MessageDescriptor::new::<Empty>(
                    "Empty",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Empty {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Empty {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Empty {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Result {
    // message fields
    pub success: bool,
    pub text: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Result {}

impl Result {
    pub fn new() -> Result {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Result {
        static mut instance: ::protobuf::lazy::Lazy<Result> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Result,
        };
        unsafe {
            instance.get(Result::new)
        }
    }

    // bool success = 1;

    pub fn clear_success(&mut self) {
        self.success = false;
    }

    // Param is passed by value, moved
    pub fn set_success(&mut self, v: bool) {
        self.success = v;
    }

    pub fn get_success(&self) -> bool {
        self.success
    }

    fn get_success_for_reflect(&self) -> &bool {
        &self.success
    }

    fn mut_success_for_reflect(&mut self) -> &mut bool {
        &mut self.success
    }

    // string text = 2;

    pub fn clear_text(&mut self) {
        self.text.clear();
    }

    // Param is passed by value, moved
    pub fn set_text(&mut self, v: ::std::string::String) {
        self.text = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_text(&mut self) -> &mut ::std::string::String {
        &mut self.text
    }

    // Take field
    pub fn take_text(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.text, ::std::string::String::new())
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    fn get_text_for_reflect(&self) -> &::std::string::String {
        &self.text
    }

    fn mut_text_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.text
    }
}

impl ::protobuf::Message for Result {
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
                    self.success = tmp;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.text)?;
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
        if self.success != false {
            my_size += 2;
        }
        if !self.text.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.text);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.success != false {
            os.write_bool(1, self.success)?;
        }
        if !self.text.is_empty() {
            os.write_string(2, &self.text)?;
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

impl ::protobuf::MessageStatic for Result {
    fn new() -> Result {
        Result::new()
    }

    fn descriptor_static(_: ::std::option::Option<Result>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "success",
                    Result::get_success_for_reflect,
                    Result::mut_success_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "text",
                    Result::get_text_for_reflect,
                    Result::mut_text_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Result>(
                    "Result",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Result {
    fn clear(&mut self) {
        self.clear_success();
        self.clear_text();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Result {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Result {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct LayerExistsResult {
    // message fields
    pub success: bool,
    pub length: u64,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for LayerExistsResult {}

impl LayerExistsResult {
    pub fn new() -> LayerExistsResult {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static LayerExistsResult {
        static mut instance: ::protobuf::lazy::Lazy<LayerExistsResult> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const LayerExistsResult,
        };
        unsafe {
            instance.get(LayerExistsResult::new)
        }
    }

    // bool success = 1;

    pub fn clear_success(&mut self) {
        self.success = false;
    }

    // Param is passed by value, moved
    pub fn set_success(&mut self, v: bool) {
        self.success = v;
    }

    pub fn get_success(&self) -> bool {
        self.success
    }

    fn get_success_for_reflect(&self) -> &bool {
        &self.success
    }

    fn mut_success_for_reflect(&mut self) -> &mut bool {
        &mut self.success
    }

    // uint64 length = 2;

    pub fn clear_length(&mut self) {
        self.length = 0;
    }

    // Param is passed by value, moved
    pub fn set_length(&mut self, v: u64) {
        self.length = v;
    }

    pub fn get_length(&self) -> u64 {
        self.length
    }

    fn get_length_for_reflect(&self) -> &u64 {
        &self.length
    }

    fn mut_length_for_reflect(&mut self) -> &mut u64 {
        &mut self.length
    }
}

impl ::protobuf::Message for LayerExistsResult {
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
                    self.success = tmp;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.length = tmp;
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
        if self.success != false {
            my_size += 2;
        }
        if self.length != 0 {
            my_size += ::protobuf::rt::value_size(2, self.length, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.success != false {
            os.write_bool(1, self.success)?;
        }
        if self.length != 0 {
            os.write_uint64(2, self.length)?;
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

impl ::protobuf::MessageStatic for LayerExistsResult {
    fn new() -> LayerExistsResult {
        LayerExistsResult::new()
    }

    fn descriptor_static(_: ::std::option::Option<LayerExistsResult>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeBool>(
                    "success",
                    LayerExistsResult::get_success_for_reflect,
                    LayerExistsResult::mut_success_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                    "length",
                    LayerExistsResult::get_length_for_reflect,
                    LayerExistsResult::mut_length_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<LayerExistsResult>(
                    "LayerExistsResult",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for LayerExistsResult {
    fn clear(&mut self) {
        self.clear_success();
        self.clear_length();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for LayerExistsResult {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for LayerExistsResult {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct GenUuidResult {
    // message fields
    pub uuid: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for GenUuidResult {}

impl GenUuidResult {
    pub fn new() -> GenUuidResult {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static GenUuidResult {
        static mut instance: ::protobuf::lazy::Lazy<GenUuidResult> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const GenUuidResult,
        };
        unsafe {
            instance.get(GenUuidResult::new)
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

impl ::protobuf::Message for GenUuidResult {
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

impl ::protobuf::MessageStatic for GenUuidResult {
    fn new() -> GenUuidResult {
        GenUuidResult::new()
    }

    fn descriptor_static(_: ::std::option::Option<GenUuidResult>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "uuid",
                    GenUuidResult::get_uuid_for_reflect,
                    GenUuidResult::mut_uuid_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<GenUuidResult>(
                    "GenUuidResult",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for GenUuidResult {
    fn clear(&mut self) {
        self.clear_uuid();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for GenUuidResult {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for GenUuidResult {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct UuidList {
    // message fields
    pub uuids: ::protobuf::RepeatedField<GenUuidResult>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for UuidList {}

impl UuidList {
    pub fn new() -> UuidList {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static UuidList {
        static mut instance: ::protobuf::lazy::Lazy<UuidList> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const UuidList,
        };
        unsafe {
            instance.get(UuidList::new)
        }
    }

    // repeated .lycaon.GenUuidResult uuids = 1;

    pub fn clear_uuids(&mut self) {
        self.uuids.clear();
    }

    // Param is passed by value, moved
    pub fn set_uuids(&mut self, v: ::protobuf::RepeatedField<GenUuidResult>) {
        self.uuids = v;
    }

    // Mutable pointer to the field.
    pub fn mut_uuids(&mut self) -> &mut ::protobuf::RepeatedField<GenUuidResult> {
        &mut self.uuids
    }

    // Take field
    pub fn take_uuids(&mut self) -> ::protobuf::RepeatedField<GenUuidResult> {
        ::std::mem::replace(&mut self.uuids, ::protobuf::RepeatedField::new())
    }

    pub fn get_uuids(&self) -> &[GenUuidResult] {
        &self.uuids
    }

    fn get_uuids_for_reflect(&self) -> &::protobuf::RepeatedField<GenUuidResult> {
        &self.uuids
    }

    fn mut_uuids_for_reflect(&mut self) -> &mut ::protobuf::RepeatedField<GenUuidResult> {
        &mut self.uuids
    }
}

impl ::protobuf::Message for UuidList {
    fn is_initialized(&self) -> bool {
        for v in &self.uuids {
            if !v.is_initialized() {
                return false;
            }
        };
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_repeated_message_into(wire_type, is, &mut self.uuids)?;
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
        for value in &self.uuids {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        for v in &self.uuids {
            os.write_tag(1, ::protobuf::wire_format::WireTypeLengthDelimited)?;
            os.write_raw_varint32(v.get_cached_size())?;
            v.write_to_with_cached_sizes(os)?;
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

impl ::protobuf::MessageStatic for UuidList {
    fn new() -> UuidList {
        UuidList::new()
    }

    fn descriptor_static(_: ::std::option::Option<UuidList>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<GenUuidResult>>(
                    "uuids",
                    UuidList::get_uuids_for_reflect,
                    UuidList::mut_uuids_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<UuidList>(
                    "UuidList",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for UuidList {
    fn clear(&mut self) {
        self.clear_uuids();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for UuidList {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for UuidList {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x11src/backend.proto\x12\x06lycaon\"G\n\x05Layer\x12\x12\n\x04name\
    \x18\x01\x20\x01(\tR\x04name\x12\x12\n\x04repo\x18\x02\x20\x01(\tR\x04re\
    po\x12\x16\n\x06digest\x18\x03\x20\x01(\tR\x06digest\"N\n\x08Manifest\
    \x12$\n\rschemaVersion\x18\x01\x20\x01(\rR\rschemaVersion\x12\x1c\n\tmed\
    iaType\x18\x02\x20\x01(\tR\tmediaType\"\x07\n\x05Empty\"6\n\x06Result\
    \x12\x18\n\x07success\x18\x01\x20\x01(\x08R\x07success\x12\x12\n\x04text\
    \x18\x02\x20\x01(\tR\x04text\"E\n\x11LayerExistsResult\x12\x18\n\x07succ\
    ess\x18\x01\x20\x01(\x08R\x07success\x12\x16\n\x06length\x18\x02\x20\x01\
    (\x04R\x06length\"#\n\rGenUuidResult\x12\x12\n\x04uuid\x18\x01\x20\x01(\
    \tR\x04uuid\"7\n\x08UuidList\x12+\n\x05uuids\x18\x01\x20\x03(\x0b2\x15.l\
    ycaon.GenUuidResultR\x05uuids2\xeb\x02\n\x07Backend\x129\n\x0blayerExist\
    s\x12\r.lycaon.Layer\x1a\x19.lycaon.LayerExistsResult\"\0\x121\n\x07GenU\
    uid\x12\r.lycaon.Layer\x1a\x15.lycaon.GenUuidResult\"\0\x12-\n\nUuidExis\
    ts\x12\r.lycaon.Layer\x1a\x0e.lycaon.Result\"\0\x12/\n\x0ccancelUpload\
    \x12\r.lycaon.Layer\x1a\x0e.lycaon.Result\"\0\x12-\n\ndeleteUuid\x12\r.l\
    ycaon.Layer\x1a\x0e.lycaon.Result\"\0\x124\n\x0euploadManifest\x12\x10.l\
    ycaon.Manifest\x1a\x0e.lycaon.Result\"\0\x12-\n\x08getUuids\x12\r.lycaon\
    .Empty\x1a\x10.lycaon.UuidList\"\0J\xcb\x12\n\x06\x12\x04\0\0N\x01\n\x08\
    \n\x01\x0c\x12\x03\0\0\x12\n\x08\n\x01\x02\x12\x03\x02\x08\x0e\n\xdc\x01\
    \n\x02\x04\0\x12\x04\x0b\0\x0f\x012\xcf\x01\x20The\x20backend\x20Protobu\
    f\x20protocol\x20is\x20used\x20between\x20the\x20Client-facing\n\x20fron\
    tend\x20and\x20the\x20Business-logic\x20backend.\n\n\x20A\x20single\x20s\
    ervice\x20defines\x20the\x20legal\x20rpc\x20calls\x20that\x20can\x20be\
    \x20made\x20to\n\x20the\x20backend\x20from\x20the\x20Frontend.\n\n\n\n\n\
    \x03\x04\0\x01\x12\x03\x0b\x08\r\n\x0b\n\x04\x04\0\x02\0\x12\x03\x0c\x02\
    \x12\n\r\n\x05\x04\0\x02\0\x04\x12\x04\x0c\x02\x0b\x0f\n\x0c\n\x05\x04\0\
    \x02\0\x05\x12\x03\x0c\x02\x08\n\x0c\n\x05\x04\0\x02\0\x01\x12\x03\x0c\t\
    \r\n\x0c\n\x05\x04\0\x02\0\x03\x12\x03\x0c\x10\x11\n\x0b\n\x04\x04\0\x02\
    \x01\x12\x03\r\x02\x12\n\r\n\x05\x04\0\x02\x01\x04\x12\x04\r\x02\x0c\x12\
    \n\x0c\n\x05\x04\0\x02\x01\x05\x12\x03\r\x02\x08\n\x0c\n\x05\x04\0\x02\
    \x01\x01\x12\x03\r\t\r\n\x0c\n\x05\x04\0\x02\x01\x03\x12\x03\r\x10\x11\n\
    \x0b\n\x04\x04\0\x02\x02\x12\x03\x0e\x02\x14\n\r\n\x05\x04\0\x02\x02\x04\
    \x12\x04\x0e\x02\r\x12\n\x0c\n\x05\x04\0\x02\x02\x05\x12\x03\x0e\x02\x08\
    \n\x0c\n\x05\x04\0\x02\x02\x01\x12\x03\x0e\t\x0f\n\x0c\n\x05\x04\0\x02\
    \x02\x03\x12\x03\x0e\x12\x13\n\n\n\x02\x04\x01\x12\x04\x11\0\x16\x01\n\n\
    \n\x03\x04\x01\x01\x12\x03\x11\x08\x10\n\x0b\n\x04\x04\x01\x02\0\x12\x03\
    \x12\x02\x1b\n\r\n\x05\x04\x01\x02\0\x04\x12\x04\x12\x02\x11\x12\n\x0c\n\
    \x05\x04\x01\x02\0\x05\x12\x03\x12\x02\x08\n\x0c\n\x05\x04\x01\x02\0\x01\
    \x12\x03\x12\t\x16\n\x0c\n\x05\x04\x01\x02\0\x03\x12\x03\x12\x19\x1a\n!\
    \n\x04\x04\x01\x02\x01\x12\x03\x13\x02\x17\"\x14\x20config..\n\x20layers\
    ..\n\n\r\n\x05\x04\x01\x02\x01\x04\x12\x04\x13\x02\x12\x1b\n\x0c\n\x05\
    \x04\x01\x02\x01\x05\x12\x03\x13\x02\x08\n\x0c\n\x05\x04\x01\x02\x01\x01\
    \x12\x03\x13\t\x12\n\x0c\n\x05\x04\x01\x02\x01\x03\x12\x03\x13\x15\x16\n\
    =\n\x02\x04\x02\x12\x03\x19\0\x10\x1a2\x20An\x20Empty\x20message\x20used\
    \x20where\x20no\x20inputs\x20are\x20needed\n\n\n\n\x03\x04\x02\x01\x12\
    \x03\x19\x08\r\n5\n\x02\x04\x03\x12\x04\x1c\0\x1f\x01\x1a)\x20A\x20gener\
    ic\x20success/fail\x20response\x20message\n\n\n\n\x03\x04\x03\x01\x12\
    \x03\x1c\x08\x0e\n\x0b\n\x04\x04\x03\x02\0\x12\x03\x1d\x02\x13\n\r\n\x05\
    \x04\x03\x02\0\x04\x12\x04\x1d\x02\x1c\x10\n\x0c\n\x05\x04\x03\x02\0\x05\
    \x12\x03\x1d\x02\x06\n\x0c\n\x05\x04\x03\x02\0\x01\x12\x03\x1d\x07\x0e\n\
    \x0c\n\x05\x04\x03\x02\0\x03\x12\x03\x1d\x11\x12\n\x0b\n\x04\x04\x03\x02\
    \x01\x12\x03\x1e\x02\x12\n\r\n\x05\x04\x03\x02\x01\x04\x12\x04\x1e\x02\
    \x1d\x13\n\x0c\n\x05\x04\x03\x02\x01\x05\x12\x03\x1e\x02\x08\n\x0c\n\x05\
    \x04\x03\x02\x01\x01\x12\x03\x1e\t\r\n\x0c\n\x05\x04\x03\x02\x01\x03\x12\
    \x03\x1e\x10\x11\n\xa8\x01\n\x02\x04\x04\x12\x04%\0(\x01\x1a\x9b\x01\x20\
    The\x20result\x20message\x20for\x20a\x20request\x20of\x20image\x20existe\
    nce.\n\n\x20success\x20:=\x20whether\x20or\x20not\x20the\x20image\x20exi\
    sts\n\x20length\x20\x20:=\x20the\x20length\x20of\x20the\x20queried\x20im\
    age\x20(if\x20exists)\n\n\n\n\x03\x04\x04\x01\x12\x03%\x08\x19\n\x0b\n\
    \x04\x04\x04\x02\0\x12\x03&\x02\x13\n\r\n\x05\x04\x04\x02\0\x04\x12\x04&\
    \x02%\x1b\n\x0c\n\x05\x04\x04\x02\0\x05\x12\x03&\x02\x06\n\x0c\n\x05\x04\
    \x04\x02\0\x01\x12\x03&\x07\x0e\n\x0c\n\x05\x04\x04\x02\0\x03\x12\x03&\
    \x11\x12\n\x0b\n\x04\x04\x04\x02\x01\x12\x03'\x02\x14\n\r\n\x05\x04\x04\
    \x02\x01\x04\x12\x04'\x02&\x13\n\x0c\n\x05\x04\x04\x02\x01\x05\x12\x03'\
    \x02\x08\n\x0c\n\x05\x04\x04\x02\x01\x01\x12\x03'\t\x0f\n\x0c\n\x05\x04\
    \x04\x02\x01\x03\x12\x03'\x12\x13\nR\n\x02\x04\x05\x12\x04,\0.\x01\x1aF\
    \x20The\x20result\x20message\x20of\x20a\x20uuid\x20Generation.\n\x20uuid\
    \x20:=\x20the\x20generated\x20uuid\n\n\n\n\x03\x04\x05\x01\x12\x03,\x08\
    \x15\n\x0b\n\x04\x04\x05\x02\0\x12\x03-\x02\x12\n\r\n\x05\x04\x05\x02\0\
    \x04\x12\x04-\x02,\x17\n\x0c\n\x05\x04\x05\x02\0\x05\x12\x03-\x02\x08\n\
    \x0c\n\x05\x04\x05\x02\0\x01\x12\x03-\t\r\n\x0c\n\x05\x04\x05\x02\0\x03\
    \x12\x03-\x10\x11\n&\n\x02\x04\x06\x12\x042\04\x01\x1a\x1a\x20A\x20list\
    \x20of\x20Uuids\n\x20:Admin:\n\n\n\n\x03\x04\x06\x01\x12\x032\x08\x10\n\
    \x0b\n\x04\x04\x06\x02\0\x12\x033\x02#\n\x0c\n\x05\x04\x06\x02\0\x04\x12\
    \x033\x02\n\n\x0c\n\x05\x04\x06\x02\0\x06\x12\x033\x0b\x18\n\x0c\n\x05\
    \x04\x06\x02\0\x01\x12\x033\x19\x1e\n\x0c\n\x05\x04\x06\x02\0\x03\x12\
    \x033!\"\n\n\n\x02\x06\0\x12\x046\0N\x01\n\n\n\x03\x06\0\x01\x12\x036\
    \x08\x0f\nZ\n\x04\x06\0\x02\0\x12\x039\x028\x1aM\x20-----\x20Image\x20Up\
    load\x20Flow\x20----------\n\x20Check\x20if\x20a\x20layer\x20exists\x20i\
    n\x20the\x20Registry\n\n\x0c\n\x05\x06\0\x02\0\x01\x12\x039\x06\x11\n\
    \x0c\n\x05\x06\0\x02\0\x02\x12\x039\x13\x18\n\x0c\n\x05\x06\0\x02\0\x03\
    \x12\x039#4\n=\n\x04\x06\0\x02\x01\x12\x03<\x020\x1a0\x20Generate\x20a\
    \x20uuid\x20for\x20a\x20new\x20layer\x20being\x20uploaded\n\n\x0c\n\x05\
    \x06\0\x02\x01\x01\x12\x03<\x06\r\n\x0c\n\x05\x06\0\x02\x01\x02\x12\x03<\
    \x0f\x14\n\x0c\n\x05\x06\0\x02\x01\x03\x12\x03<\x1f,\nC\n\x04\x06\0\x02\
    \x02\x12\x03?\x02,\x1a6\x20Given\x20a\x20Uuid,\x20check\x20whether\x20it\
    \x20exists\x20in\x20the\x20cluster\n\n\x0c\n\x05\x06\0\x02\x02\x01\x12\
    \x03?\x06\x10\n\x0c\n\x05\x06\0\x02\x02\x02\x12\x03?\x12\x17\n\x0c\n\x05\
    \x06\0\x02\x02\x03\x12\x03?\"(\n^\n\x04\x06\0\x02\x03\x12\x03C\x02.\x1aQ\
    \x20Cancel\x20a\x20pending\x20upload\n\x20The\x20digest\x20field\x20is\
    \x20used\x20for\x20the\x20uuid\x20in\x20this\x20rpc\x20call\n\n\x0c\n\
    \x05\x06\0\x02\x03\x01\x12\x03C\x06\x12\n\x0c\n\x05\x06\0\x02\x03\x02\
    \x12\x03C\x14\x19\n\x0c\n\x05\x06\0\x02\x03\x03\x12\x03C$*\n8\n\x04\x06\
    \0\x02\x04\x12\x03F\x02,\x1a+\x20Delete\x20an\x20uuid\x20after\x20a\x20s\
    uccessful\x20upload.\n\n\x0c\n\x05\x06\0\x02\x04\x01\x12\x03F\x06\x10\n\
    \x0c\n\x05\x06\0\x02\x04\x02\x12\x03F\x12\x17\n\x0c\n\x05\x06\0\x02\x04\
    \x03\x12\x03F\"(\n\x0b\n\x04\x06\0\x02\x05\x12\x03H\x023\n\x0c\n\x05\x06\
    \0\x02\x05\x01\x12\x03H\x06\x14\n\x0c\n\x05\x06\0\x02\x05\x02\x12\x03H\
    \x16\x1e\n\x0c\n\x05\x06\0\x02\x05\x03\x12\x03H)/\nu\n\x04\x06\0\x02\x06\
    \x12\x03M\x02,\x1a7\x20returns\x20a\x20list\x20of\x20all\x20Uuids\x20cur\
    rently\x20in\x20the\x20\x20backend\n2/\x20------------\x20Admin\x20calls\
    \x20--------------------\n\n\x0c\n\x05\x06\0\x02\x06\x01\x12\x03M\x06\
    \x0e\n\x0c\n\x05\x06\0\x02\x06\x02\x12\x03M\x10\x15\n\x0c\n\x05\x06\0\
    \x02\x06\x03\x12\x03M\x20(b\x06proto3\
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
