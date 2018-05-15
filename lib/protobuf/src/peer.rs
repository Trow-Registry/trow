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
pub struct Dot {
    // message fields
    pub actor: ::std::string::String,
    pub counter: u64,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Dot {}

impl Dot {
    pub fn new() -> Dot {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Dot {
        static mut instance: ::protobuf::lazy::Lazy<Dot> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Dot,
        };
        unsafe {
            instance.get(Dot::new)
        }
    }

    // string actor = 1;

    pub fn clear_actor(&mut self) {
        self.actor.clear();
    }

    // Param is passed by value, moved
    pub fn set_actor(&mut self, v: ::std::string::String) {
        self.actor = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_actor(&mut self) -> &mut ::std::string::String {
        &mut self.actor
    }

    // Take field
    pub fn take_actor(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.actor, ::std::string::String::new())
    }

    pub fn get_actor(&self) -> &str {
        &self.actor
    }

    fn get_actor_for_reflect(&self) -> &::std::string::String {
        &self.actor
    }

    fn mut_actor_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.actor
    }

    // uint64 counter = 2;

    pub fn clear_counter(&mut self) {
        self.counter = 0;
    }

    // Param is passed by value, moved
    pub fn set_counter(&mut self, v: u64) {
        self.counter = v;
    }

    pub fn get_counter(&self) -> u64 {
        self.counter
    }

    fn get_counter_for_reflect(&self) -> &u64 {
        &self.counter
    }

    fn mut_counter_for_reflect(&mut self) -> &mut u64 {
        &mut self.counter
    }
}

impl ::protobuf::Message for Dot {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.actor)?;
                },
                2 => {
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_uint64()?;
                    self.counter = tmp;
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
        if !self.actor.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.actor);
        }
        if self.counter != 0 {
            my_size += ::protobuf::rt::value_size(2, self.counter, ::protobuf::wire_format::WireTypeVarint);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.actor.is_empty() {
            os.write_string(1, &self.actor)?;
        }
        if self.counter != 0 {
            os.write_uint64(2, self.counter)?;
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

impl ::protobuf::MessageStatic for Dot {
    fn new() -> Dot {
        Dot::new()
    }

    fn descriptor_static(_: ::std::option::Option<Dot>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "actor",
                    Dot::get_actor_for_reflect,
                    Dot::mut_actor_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeUint64>(
                    "counter",
                    Dot::get_counter_for_reflect,
                    Dot::mut_counter_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<Dot>(
                    "Dot",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Dot {
    fn clear(&mut self) {
        self.clear_actor();
        self.clear_counter();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Dot {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Dot {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct ORSetFullSync {
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for ORSetFullSync {}

impl ORSetFullSync {
    pub fn new() -> ORSetFullSync {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static ORSetFullSync {
        static mut instance: ::protobuf::lazy::Lazy<ORSetFullSync> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ORSetFullSync,
        };
        unsafe {
            instance.get(ORSetFullSync::new)
        }
    }
}

impl ::protobuf::Message for ORSetFullSync {
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

impl ::protobuf::MessageStatic for ORSetFullSync {
    fn new() -> ORSetFullSync {
        ORSetFullSync::new()
    }

    fn descriptor_static(_: ::std::option::Option<ORSetFullSync>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let fields = ::std::vec::Vec::new();
                ::protobuf::reflect::MessageDescriptor::new::<ORSetFullSync>(
                    "ORSetFullSync",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for ORSetFullSync {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ORSetFullSync {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ORSetFullSync {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct ORSetDelta {
    // message fields
    pub deltatype: DeltaType,
    pub element: ::std::string::String,
    pub dots: ::protobuf::RepeatedField<Dot>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for ORSetDelta {}

impl ORSetDelta {
    pub fn new() -> ORSetDelta {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static ORSetDelta {
        static mut instance: ::protobuf::lazy::Lazy<ORSetDelta> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ORSetDelta,
        };
        unsafe {
            instance.get(ORSetDelta::new)
        }
    }

    // .lycaon.DeltaType deltatype = 1;

    pub fn clear_deltatype(&mut self) {
        self.deltatype = DeltaType::ADD;
    }

    // Param is passed by value, moved
    pub fn set_deltatype(&mut self, v: DeltaType) {
        self.deltatype = v;
    }

    pub fn get_deltatype(&self) -> DeltaType {
        self.deltatype
    }

    fn get_deltatype_for_reflect(&self) -> &DeltaType {
        &self.deltatype
    }

    fn mut_deltatype_for_reflect(&mut self) -> &mut DeltaType {
        &mut self.deltatype
    }

    // string element = 2;

    pub fn clear_element(&mut self) {
        self.element.clear();
    }

    // Param is passed by value, moved
    pub fn set_element(&mut self, v: ::std::string::String) {
        self.element = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_element(&mut self) -> &mut ::std::string::String {
        &mut self.element
    }

    // Take field
    pub fn take_element(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.element, ::std::string::String::new())
    }

    pub fn get_element(&self) -> &str {
        &self.element
    }

    fn get_element_for_reflect(&self) -> &::std::string::String {
        &self.element
    }

    fn mut_element_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.element
    }

    // repeated .lycaon.Dot dots = 3;

    pub fn clear_dots(&mut self) {
        self.dots.clear();
    }

    // Param is passed by value, moved
    pub fn set_dots(&mut self, v: ::protobuf::RepeatedField<Dot>) {
        self.dots = v;
    }

    // Mutable pointer to the field.
    pub fn mut_dots(&mut self) -> &mut ::protobuf::RepeatedField<Dot> {
        &mut self.dots
    }

    // Take field
    pub fn take_dots(&mut self) -> ::protobuf::RepeatedField<Dot> {
        ::std::mem::replace(&mut self.dots, ::protobuf::RepeatedField::new())
    }

    pub fn get_dots(&self) -> &[Dot] {
        &self.dots
    }

    fn get_dots_for_reflect(&self) -> &::protobuf::RepeatedField<Dot> {
        &self.dots
    }

    fn mut_dots_for_reflect(&mut self) -> &mut ::protobuf::RepeatedField<Dot> {
        &mut self.dots
    }
}

impl ::protobuf::Message for ORSetDelta {
    fn is_initialized(&self) -> bool {
        for v in &self.dots {
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
                    if wire_type != ::protobuf::wire_format::WireTypeVarint {
                        return ::std::result::Result::Err(::protobuf::rt::unexpected_wire_type(wire_type));
                    }
                    let tmp = is.read_enum()?;
                    self.deltatype = tmp;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.element)?;
                },
                3 => {
                    ::protobuf::rt::read_repeated_message_into(wire_type, is, &mut self.dots)?;
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
        if self.deltatype != DeltaType::ADD {
            my_size += ::protobuf::rt::enum_size(1, self.deltatype);
        }
        if !self.element.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.element);
        }
        for value in &self.dots {
            let len = value.compute_size();
            my_size += 1 + ::protobuf::rt::compute_raw_varint32_size(len) + len;
        };
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.deltatype != DeltaType::ADD {
            os.write_enum(1, self.deltatype.value())?;
        }
        if !self.element.is_empty() {
            os.write_string(2, &self.element)?;
        }
        for v in &self.dots {
            os.write_tag(3, ::protobuf::wire_format::WireTypeLengthDelimited)?;
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

impl ::protobuf::MessageStatic for ORSetDelta {
    fn new() -> ORSetDelta {
        ORSetDelta::new()
    }

    fn descriptor_static(_: ::std::option::Option<ORSetDelta>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeEnum<DeltaType>>(
                    "deltatype",
                    ORSetDelta::get_deltatype_for_reflect,
                    ORSetDelta::mut_deltatype_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "element",
                    ORSetDelta::get_element_for_reflect,
                    ORSetDelta::mut_element_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeMessage<Dot>>(
                    "dots",
                    ORSetDelta::get_dots_for_reflect,
                    ORSetDelta::mut_dots_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<ORSetDelta>(
                    "ORSetDelta",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for ORSetDelta {
    fn clear(&mut self) {
        self.clear_deltatype();
        self.clear_element();
        self.clear_dots();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ORSetDelta {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ORSetDelta {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct ORSetDeltaReply {
    // message fields
    pub deltatype: DeltaType,
    pub element: ::std::string::String,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for ORSetDeltaReply {}

impl ORSetDeltaReply {
    pub fn new() -> ORSetDeltaReply {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static ORSetDeltaReply {
        static mut instance: ::protobuf::lazy::Lazy<ORSetDeltaReply> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ORSetDeltaReply,
        };
        unsafe {
            instance.get(ORSetDeltaReply::new)
        }
    }

    // .lycaon.DeltaType deltatype = 1;

    pub fn clear_deltatype(&mut self) {
        self.deltatype = DeltaType::ADD;
    }

    // Param is passed by value, moved
    pub fn set_deltatype(&mut self, v: DeltaType) {
        self.deltatype = v;
    }

    pub fn get_deltatype(&self) -> DeltaType {
        self.deltatype
    }

    fn get_deltatype_for_reflect(&self) -> &DeltaType {
        &self.deltatype
    }

    fn mut_deltatype_for_reflect(&mut self) -> &mut DeltaType {
        &mut self.deltatype
    }

    // string element = 2;

    pub fn clear_element(&mut self) {
        self.element.clear();
    }

    // Param is passed by value, moved
    pub fn set_element(&mut self, v: ::std::string::String) {
        self.element = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_element(&mut self) -> &mut ::std::string::String {
        &mut self.element
    }

    // Take field
    pub fn take_element(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.element, ::std::string::String::new())
    }

    pub fn get_element(&self) -> &str {
        &self.element
    }

    fn get_element_for_reflect(&self) -> &::std::string::String {
        &self.element
    }

    fn mut_element_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.element
    }
}

impl ::protobuf::Message for ORSetDeltaReply {
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
                    let tmp = is.read_enum()?;
                    self.deltatype = tmp;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.element)?;
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
        if self.deltatype != DeltaType::ADD {
            my_size += ::protobuf::rt::enum_size(1, self.deltatype);
        }
        if !self.element.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.element);
        }
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if self.deltatype != DeltaType::ADD {
            os.write_enum(1, self.deltatype.value())?;
        }
        if !self.element.is_empty() {
            os.write_string(2, &self.element)?;
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

impl ::protobuf::MessageStatic for ORSetDeltaReply {
    fn new() -> ORSetDeltaReply {
        ORSetDeltaReply::new()
    }

    fn descriptor_static(_: ::std::option::Option<ORSetDeltaReply>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeEnum<DeltaType>>(
                    "deltatype",
                    ORSetDeltaReply::get_deltatype_for_reflect,
                    ORSetDeltaReply::mut_deltatype_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "element",
                    ORSetDeltaReply::get_element_for_reflect,
                    ORSetDeltaReply::mut_element_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<ORSetDeltaReply>(
                    "ORSetDeltaReply",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for ORSetDeltaReply {
    fn clear(&mut self) {
        self.clear_deltatype();
        self.clear_element();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for ORSetDeltaReply {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for ORSetDeltaReply {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(PartialEq,Clone,Default)]
pub struct Heartbeat {
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for Heartbeat {}

impl Heartbeat {
    pub fn new() -> Heartbeat {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static Heartbeat {
        static mut instance: ::protobuf::lazy::Lazy<Heartbeat> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const Heartbeat,
        };
        unsafe {
            instance.get(Heartbeat::new)
        }
    }
}

impl ::protobuf::Message for Heartbeat {
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

impl ::protobuf::MessageStatic for Heartbeat {
    fn new() -> Heartbeat {
        Heartbeat::new()
    }

    fn descriptor_static(_: ::std::option::Option<Heartbeat>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let fields = ::std::vec::Vec::new();
                ::protobuf::reflect::MessageDescriptor::new::<Heartbeat>(
                    "Heartbeat",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for Heartbeat {
    fn clear(&mut self) {
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for Heartbeat {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for Heartbeat {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

#[derive(Clone,PartialEq,Eq,Debug,Hash)]
pub enum DeltaType {
    ADD = 0,
    REMOVE = 1,
}

impl ::protobuf::ProtobufEnum for DeltaType {
    fn value(&self) -> i32 {
        *self as i32
    }

    fn from_i32(value: i32) -> ::std::option::Option<DeltaType> {
        match value {
            0 => ::std::option::Option::Some(DeltaType::ADD),
            1 => ::std::option::Option::Some(DeltaType::REMOVE),
            _ => ::std::option::Option::None
        }
    }

    fn values() -> &'static [Self] {
        static values: &'static [DeltaType] = &[
            DeltaType::ADD,
            DeltaType::REMOVE,
        ];
        values
    }

    fn enum_descriptor_static(_: ::std::option::Option<DeltaType>) -> &'static ::protobuf::reflect::EnumDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::EnumDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::EnumDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                ::protobuf::reflect::EnumDescriptor::new("DeltaType", file_descriptor_proto())
            })
        }
    }
}

impl ::std::marker::Copy for DeltaType {
}

impl ::std::default::Default for DeltaType {
    fn default() -> Self {
        DeltaType::ADD
    }
}

impl ::protobuf::reflect::ProtobufValue for DeltaType {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Enum(self.descriptor())
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\npeer.proto\x12\x06lycaon\"5\n\x03Dot\x12\x14\n\x05actor\x18\x01\x20\
    \x01(\tR\x05actor\x12\x18\n\x07counter\x18\x02\x20\x01(\x04R\x07counter\
    \"\x0f\n\rORSetFullSync\"x\n\nORSetDelta\x12/\n\tdeltatype\x18\x01\x20\
    \x01(\x0e2\x11.lycaon.DeltaTypeR\tdeltatype\x12\x18\n\x07element\x18\x02\
    \x20\x01(\tR\x07element\x12\x1f\n\x04dots\x18\x03\x20\x03(\x0b2\x0b.lyca\
    on.DotR\x04dots\"\\\n\x0fORSetDeltaReply\x12/\n\tdeltatype\x18\x01\x20\
    \x01(\x0e2\x11.lycaon.DeltaTypeR\tdeltatype\x12\x18\n\x07element\x18\x02\
    \x20\x01(\tR\x07element\"\x0b\n\tHeartbeat*\x20\n\tDeltaType\x12\x07\n\
    \x03ADD\x10\0\x12\n\n\x06REMOVE\x10\x012w\n\x04Peer\x123\n\theartbeat\
    \x12\x11.lycaon.Heartbeat\x1a\x11.lycaon.Heartbeat\"\0\x12:\n\tdeltaSync\
    \x12\x12.lycaon.ORSetDelta\x1a\x17.lycaon.ORSetDeltaReply\"\0b\x06proto3\
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
