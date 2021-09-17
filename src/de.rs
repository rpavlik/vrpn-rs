// Copyright 2018, Collabora, Ltd.
// SPDX-License-Identifier: BSL-1.0
// Author: Ryan A. Pavlik <ryan.pavlik@collabora.com>

use bytes::{Buf, BufMut, BytesMut};
use crate::{BytesRequired, Error, Result};
use serde::de::{
    self, Deserialize, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor,
};
use std::mem::size_of;

pub struct Deserializer<'de, T: Buf> {
    input: &'de mut T,
}

impl<'de, T: Buf> Deserializer<'de, T> {
    pub fn from_buf(input: &'de mut T) -> Self {
        Deserializer { input }
    }
}

pub fn from_buf<'a, T, U>(buf: &'a mut U) -> Result<T>
where
    U: Buf,
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_buf(buf);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.has_remaining() {
        Err(Error::TrailingCharacters)
    } else {
        Ok(t)
    }
}

impl<'de, T: Buf> Deserializer<'de, T> {
    fn peek_bool(&mut self) -> Result<bool> {
        self.peek::<u32>().map(|v| v == 1)
    }
    fn parse_bool(&mut self) -> Result<bool> {
        self.parse::<u32>().map(|v| v == 1)
    }

    fn peek<T: PrimitiveSerde>(&mut self) -> Result<T> {
        let size = T::check_size(&mut self.input)?;
        let mut take = self.input.take(size);
        T::get(&mut take)
    }
    fn parse<T: PrimitiveSerde>(&mut self) -> Result<T> {
        T::check_size(&mut self.input)?;
        T::get(self.input)
    }
}

trait PrimitiveSerde: Sized {
    fn size() -> usize {
        size_of::<Self>()
    }
    fn check_size<T: Buf>(buf: &mut T) -> Result<usize> {
        let size = size_of::<Self>();
        if buf.remaining() < size {
            Err(Error::NeedMoreData(BytesRequired::AtLeast(
                size - buf.remaining(),
            )))
        } else {
            Ok(size)
        }
    }

    fn get(buf: &mut impl Buf) -> Result<Self>;
    fn put(buf: &mut BytesMut, val: Self) -> Result<()>;
}

macro_rules! buffer_primitive {
    ($t:ty, $put:ident, $get:ident) => {
        impl PrimitiveSerde for $t {
            fn put(buf: &mut BytesMut, val: Self) -> Result<()> {
                buf.$put(val);
                Ok(())
            }
            fn get(buf: &mut impl Buf) -> Result<Self> {
                Ok(buf.$get())
            }
        }
    };
}

buffer_primitive!(i8, put_i8, get_i8);
buffer_primitive!(u8, put_u8, get_u8);
buffer_primitive!(i16, put_i16_be, get_i16_be);
buffer_primitive!(u16, put_u16_be, get_u16_be);
buffer_primitive!(i32, put_i32_be, get_i32_be);
buffer_primitive!(u32, put_u32_be, get_u32_be);
buffer_primitive!(i64, put_i64_be, get_i64_be);
buffer_primitive!(u64, put_u64_be, get_u64_be);
buffer_primitive!(f32, put_f32_be, get_f32_be);
buffer_primitive!(f64, put_f64_be, get_f64_be);

// macro_rules! de_primitive {
//     ($t:ty, $method:ident, $peek_method:ident, $get:ident) => {
//         impl<'de, T: Buf> Deserializer<'de, T> {
//             fn $peek_method(&mut self) -> Result<$t> {
//                 let size = size_of::<$t>();
//                 if self.input.remaining() < size {
//                     Err(Error::NeedMoreData(BytesRequired::AtLeast(
//                         size - self.input.remaining(),
//                     )))?;
//                 }
//                 Ok(self.input.take(size).$get())
//             }
//             fn $method(&mut self) -> Result<$t> {
//                 let size = size_of::<$t>();
//                 if self.input.remaining() < size {
//                     Err(Error::NeedMoreData(BytesRequired::Exactly(
//                         size - self.input.remaining(),
//                     )))?;
//                 }
//                 Ok(self.input.$get())
//             }
//         }
//     };
// }

// de_primitive!(i8, parse_i8, peek_i8, get_i8);
// de_primitive!(i16, parse_i16, peek_i16, get_i16_be);
// de_primitive!(u16, parse_u16, peek_u16, get_u16_be);
// de_primitive!(i32, parse_i32, peek_i32, get_i32_be);
// de_primitive!(u32, parse_u32, peek_u32, get_u32_be);
// de_primitive!(i64, parse_i64, peek_i64, get_i64_be);
// de_primitive!(u64, parse_u64, peek_u64, get_u64_be);
// de_primitive!(f32, parse_f32, peek_f32, get_f32_be);
// de_primitive!(f64, parse_f64, peek_f64, get_f64_be);

impl<'de, 'a, T: Buf> de::Deserializer<'de> for &'a mut Deserializer<'de, T> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    // The `parse` function is generic over the integer type `T` so here
    // it is invoked with `T=i8`. The next 8 methods are similar.
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.parse()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.parse()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.parse()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.input.remaining() < 1 {
            Err(Error::NeedMoreData(BytesRequired::Exactly(1)))?;
        }
        let b = self.input.get_u8();
        visitor.visit_char(char::from(b))
    }

    fn deserialize_str<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_string<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // In Serde, unit means an anonymous value containing no data.
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(mut self, v_visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(mut self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple_struct(name, fields.len(), visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
}

impl<'de, T: Buf> SeqAccess<'de> for Deserializer<'de, T> {
    type Error = Error;

    fn next_element_seed<S>(&mut self, seed: T) -> Result<Option<S::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        unimplemented!()
    }
}

impl<'de, T: Buf> MapAccess<'de> for Deserializer<'de, T> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        unimplemented!()
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        unimplemented!()
    }
}

// struct Enum<'a, 'de: 'a> {
//     de: &'a mut Deserializer<'de>,
// }

// impl<'a, 'de> Enum<'a, 'de> {
//     fn new(de: &'a mut Deserializer<'de>) -> Self {
//         Enum { de }
//     }
// }

// `EnumAccess` is provided to the `Visitor` to give it the ability to determine
// which variant of the enum is supposed to be deserialized.
//
// Note that all enum deserialization methods in Serde refer exclusively to the
// "externally tagged" enum representation.
impl<'de, T: Buf> EnumAccess<'de> for Deserializer<'de, T> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        unimplemented!()
    }
}

// `VariantAccess` is provided to the `Visitor` to give it the ability to see
// the content of the single variant that it decided to deserialize.
impl<'de, T: Buf> VariantAccess<'de> for Deserializer<'de, T> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        unimplemented!()
    }

    fn newtype_variant_seed<S>(self, seed: S) -> Result<S::Value>
    where
        T: DeserializeSeed<'de>,
    {
        unimplemented!()
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }
}

////////////////////////////////////////////////////////////////////////////////
