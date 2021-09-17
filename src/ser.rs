// Copyright 2018, Collabora, Ltd.
// SPDX-License-Identifier: BSL-1.0
// Author: Ryan A. Pavlik <ryan.pavlik@collabora.com>

use bytes::{BufMut, Bytes, BytesMut};
use crate::{Error, Result};
use serde::ser::{self, Serialize};
use std::mem::{size_of, size_of_val};

pub struct Serializer {
    output: BytesMut,
}

pub fn to_bytes<T>(value: &T) -> Result<Bytes>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        output: BytesMut::new(),
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output.freeze())
}

impl<'a> ser::Serializer for &'a mut Serializer {
    // The output type produced by this `Serializer` during successful
    // serialization. Most serializers that produce text or binary output should
    // set `Ok = ()` and serialize into an `io::Write` or buffer contained
    // within the `Serializer` instance, as happens here. Serializers that build
    // in-memory data structures may be simplified by using `Ok` to propagate
    // the data structure around.
    type Ok = ();

    // The error type when some error occurs during serialization.
    type Error = Error;

    // Associated types for keeping track of additional state while serializing
    // compound data structures like sequences and maps. In this case no
    // additional state is required beyond what is already stored in the
    // Serializer struct.
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.serialize_i16(if v { 1_i16 } else { 0_i16 })
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.output.reserve(size_of::<i8>());
        self.output.put_i8(v);
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.output.reserve(size_of::<i16>());
        self.output.put_i16_be(v);
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.output.reserve(size_of::<i32>());
        self.output.put_i32_be(v);
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.output.reserve(size_of::<i64>());
        self.output.put_i64_be(v);
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.output.reserve(size_of::<u8>());
        self.output.put_u8(v);
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.output.reserve(size_of::<u16>());
        self.output.put_u16_be(v);
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.output.reserve(size_of::<u32>());
        self.output.put_u32_be(v);
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.output.reserve(size_of::<u64>());
        self.output.put_u64_be(v);
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.output.reserve(size_of::<f32>());
        self.output.put_f32_be(v);
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.output.reserve(size_of::<f64>());
        self.output.put_f64_be(v);
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        // if !v.is_ascii() {
        //     Err(Error::OtherMessage(String::from(
        //         "Got a non-ascii char to serialize",
        //     )))?;
        // }
        // let mut b = [0; 1];

        // let result = v.encode_utf8(&mut b);

        // self.output.put(&b);
        // Ok(())
        unimplemented!();
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        unimplemented!();
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.output.reserve(v.len());
        self.output.put(v);
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        unimplemented!();
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!();
    }

    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!();
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        // len is number of elements, not number of bytes.
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        unimplemented!();
        // Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        unimplemented!();
        // Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        unimplemented!();
        // Ok(self)
    }
}

// The following 7 impls deal with the serialization of compound types like
// sequences and maps. Serialization of such types is begun by a Serializer
// method and followed by zero or more calls to serialize individual elements of
// the compound type and one call to end the compound type.
//
// This impl is SerializeSeq so these methods are called after `serialize_seq`
// is called on the Serializer.
impl<'a> ser::SerializeSeq for &'a mut Serializer {
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    // Close the sequence.
    fn end(self) -> Result<()> {
        Ok(())
    }
}

// Same thing but for tuples.
impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

// Same thing but for tuple structs.
impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self);
        unimplemented!();
    }

    fn end(self) -> Result<()> {
        unimplemented!();
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!();
        key.serialize(&mut **self);
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        unimplemented!();
        value.serialize(&mut **self);
    }

    fn end(self) -> Result<()> {
        unimplemented!();
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}
