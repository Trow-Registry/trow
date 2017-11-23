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
    \n\x16src/grpc/backend.proto\x12\x06lycaon\"5\n\x03Dot\x12\x14\n\x05acto\
    r\x18\x01\x20\x01(\tR\x05actor\x12\x18\n\x07counter\x18\x02\x20\x01(\x04\
    R\x07counter\"\x0f\n\rORSetFullSync\"x\n\nORSetDelta\x12/\n\tdeltatype\
    \x18\x01\x20\x01(\x0e2\x11.lycaon.DeltaTypeR\tdeltatype\x12\x18\n\x07ele\
    ment\x18\x02\x20\x01(\tR\x07element\x12\x1f\n\x04dots\x18\x03\x20\x03(\
    \x0b2\x0b.lycaon.DotR\x04dots\"\\\n\x0fORSetDeltaReply\x12/\n\tdeltatype\
    \x18\x01\x20\x01(\x0e2\x11.lycaon.DeltaTypeR\tdeltatype\x12\x18\n\x07ele\
    ment\x18\x02\x20\x01(\tR\x07element*\x20\n\tDeltaType\x12\x07\n\x03ADD\
    \x10\0\x12\n\n\x06REMOVE\x10\x012B\n\x04Peer\x12:\n\tdeltaSync\x12\x12.l\
    ycaon.ORSetDelta\x1a\x17.lycaon.ORSetDeltaReply\"\0J\xf0\x0e\n\x06\x12\
    \x04\0\08\x01\n\x08\n\x01\x0c\x12\x03\0\0\x12\n\x08\n\x01\x02\x12\x03\
    \x02\x08\x0e\n\x9f\x06\n\x02\x04\0\x12\x04\x18\0\x1b\x01\x1a\x81\x01\x20\
    Generic\x20Types\n\n\x20These\x20are\x20types\x20that\x20are\x20reused\
    \x20throughout\x20the\x20specification\x20and\n\x20are\x20defined\x20at\
    \x20the\x20top\x20for\x20ease\x20of\x20discovery.\n2\x8e\x05\x20Looking\
    \x20at\x20the\x20ORSet\x20library\x20which\x20we\x20are\x20using\x20for\
    \x20our\x20CRDT\n\x20We\x20have\x20two\x20kinds\x20of\x20messages\x20tha\
    t\x20relate\x20to\x20the\x20propogation\x20of\n\x20data.\n\n\x201.\x20Wh\
    en\x20a\x20new\x20Instance\x20comes\x20online\x20and\x20requests\x20a\
    \x20sync.\n\x20\x20\x20-\x20This\x20could\x20be\x20implemented\x20using\
    \x20no.\x202\x20and\x20just\x20applying\x20all\n\x20\x20\x20\x20\x20delt\
    as\x20from\x20an\x20empty\x20ORSet.\n\x202.\x20When\x20an\x20existing\
    \x20instance\x20needs\x20to\x20send\x20a\x20delta\x20to\x20listening\x20\
    instances.\n\n\x20The\x20second\x20set\x20of\x20messages\x20relates\x20t\
    o\x20locating\x20and\x20downloading\n\x20information\x20from\x20other\
    \x20services.\x20This\x20includes\x20(non-exhaustive):\n\n\x20-\x20Query\
    ing\x20a\x20layers\x20existence\x20on\x20a\x20remote\x20instance\n\x20-\
    \x20Querying\x20permissions\x20regarding\x20a\x20layer\n\x20-\x20Propoga\
    ting\x20any\x20state\x20changes\x20(such\x20as\x20deletion\x20requests)\
    \n\n\n\n\x03\x04\0\x01\x12\x03\x18\x08\x0b\n\x0b\n\x04\x04\0\x02\0\x12\
    \x03\x19\x02\x13\n\r\n\x05\x04\0\x02\0\x04\x12\x04\x19\x02\x18\r\n\x0c\n\
    \x05\x04\0\x02\0\x05\x12\x03\x19\x02\x08\n\x0c\n\x05\x04\0\x02\0\x01\x12\
    \x03\x19\t\x0e\n\x0c\n\x05\x04\0\x02\0\x03\x12\x03\x19\x11\x12\n\x0b\n\
    \x04\x04\0\x02\x01\x12\x03\x1a\x02\x15\n\r\n\x05\x04\0\x02\x01\x04\x12\
    \x04\x1a\x02\x19\x13\n\x0c\n\x05\x04\0\x02\x01\x05\x12\x03\x1a\x02\x08\n\
    \x0c\n\x05\x04\0\x02\x01\x01\x12\x03\x1a\t\x10\n\x0c\n\x05\x04\0\x02\x01\
    \x03\x12\x03\x1a\x13\x14\n\n\n\x02\x05\0\x12\x04\x1d\0\x20\x01\n\n\n\x03\
    \x05\0\x01\x12\x03\x1d\x05\x0e\n\x0b\n\x04\x05\0\x02\0\x12\x03\x1e\x02\n\
    \n\x0c\n\x05\x05\0\x02\0\x01\x12\x03\x1e\x02\x05\n\x0c\n\x05\x05\0\x02\0\
    \x02\x12\x03\x1e\x08\t\n\x0b\n\x04\x05\0\x02\x01\x12\x03\x1f\x02\r\n\x0c\
    \n\x05\x05\0\x02\x01\x01\x12\x03\x1f\x02\x08\n\x0c\n\x05\x05\0\x02\x01\
    \x02\x12\x03\x1f\x0b\x0c\n^\n\x02\x04\x01\x12\x03%\0\x18\x1aS\x20ORSet\
    \x20messages\n\n\x20This\x20message\x20is\x20a\x20sync\x20of\x20the\x20e\
    ntire\x20current\x20state\x20of\x20the\x20ORSet.\n\n\n\n\x03\x04\x01\x01\
    \x12\x03%\x08\x15\nB\n\x02\x04\x02\x12\x04(\0,\x01\x1a6\x20This\x20messa\
    ge\x20represents\x20a\x20single\x20Delta\x20of\x20the\x20ORSet.\n\n\n\n\
    \x03\x04\x02\x01\x12\x03(\x08\x12\n\x0b\n\x04\x04\x02\x02\0\x12\x03)\x02\
    \x1a\n\r\n\x05\x04\x02\x02\0\x04\x12\x04)\x02(\x14\n\x0c\n\x05\x04\x02\
    \x02\0\x06\x12\x03)\x02\x0b\n\x0c\n\x05\x04\x02\x02\0\x01\x12\x03)\x0c\
    \x15\n\x0c\n\x05\x04\x02\x02\0\x03\x12\x03)\x18\x19\n\x0b\n\x04\x04\x02\
    \x02\x01\x12\x03*\x02\x15\n\r\n\x05\x04\x02\x02\x01\x04\x12\x04*\x02)\
    \x1a\n\x0c\n\x05\x04\x02\x02\x01\x05\x12\x03*\x02\x08\n\x0c\n\x05\x04\
    \x02\x02\x01\x01\x12\x03*\t\x10\n\x0c\n\x05\x04\x02\x02\x01\x03\x12\x03*\
    \x13\x14\n\x0b\n\x04\x04\x02\x02\x02\x12\x03+\x02\x18\n\x0c\n\x05\x04\
    \x02\x02\x02\x04\x12\x03+\x02\n\n\x0c\n\x05\x04\x02\x02\x02\x06\x12\x03+\
    \x0b\x0e\n\x0c\n\x05\x04\x02\x02\x02\x01\x12\x03+\x0f\x13\n\x0c\n\x05\
    \x04\x02\x02\x02\x03\x12\x03+\x16\x17\n\xb9\x01\n\x02\x04\x03\x12\x041\0\
    4\x01\x1a\xac\x01\x20This\x20message\x20represents\x20a\x20reply\x20to\
    \x20a\x20sent\x20delta.\n\x20Currently\x20this\x20message\x20simply\x20r\
    eturns\x20the\x20DeltaType\x20and\x20the\x20element\n\x20so\x20the\x20cl\
    ient\x20can\x20verify\x20a\x20successful\x20message\x20sent.\n\n\n\n\x03\
    \x04\x03\x01\x12\x031\x08\x17\n\x0b\n\x04\x04\x03\x02\0\x12\x032\x02\x1a\
    \n\r\n\x05\x04\x03\x02\0\x04\x12\x042\x021\x19\n\x0c\n\x05\x04\x03\x02\0\
    \x06\x12\x032\x02\x0b\n\x0c\n\x05\x04\x03\x02\0\x01\x12\x032\x0c\x15\n\
    \x0c\n\x05\x04\x03\x02\0\x03\x12\x032\x18\x19\n\x0b\n\x04\x04\x03\x02\
    \x01\x12\x033\x02\x15\n\r\n\x05\x04\x03\x02\x01\x04\x12\x043\x022\x1a\n\
    \x0c\n\x05\x04\x03\x02\x01\x05\x12\x033\x02\x08\n\x0c\n\x05\x04\x03\x02\
    \x01\x01\x12\x033\t\x10\n\x0c\n\x05\x04\x03\x02\x01\x03\x12\x033\x13\x14\
    \n\n\n\x02\x06\0\x12\x046\08\x01\n\n\n\x03\x06\0\x01\x12\x036\x08\x0c\n\
    \x0b\n\x04\x06\0\x02\0\x12\x037\x029\n\x0c\n\x05\x06\0\x02\0\x01\x12\x03\
    7\x06\x0f\n\x0c\n\x05\x06\0\x02\0\x02\x12\x037\x11\x1b\n\x0c\n\x05\x06\0\
    \x02\0\x03\x12\x037&5b\x06proto3\
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
