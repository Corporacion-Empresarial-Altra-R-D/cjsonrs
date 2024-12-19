#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::ffi::CString;

#[cfg(feature = "std")]
use std::ffi::CString;

use super::Error;
use crate::CJson;
use crate::CJsonArray;
use crate::CJsonObject;
use crate::CJsonRef;
use core::fmt::Display;
use serde::ser::*;

// Based on https://github.com/serde-rs/json/blob/master/src/value/ser.rs

/// Serializes a value to a CJson instance.
#[inline(always)]
pub fn to_cjson<T: Serialize>(value: T) -> super::Result<CJson<'static>> {
    value.serialize(Serializer)
}

impl serde::ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}

impl Serialize for CJson<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_ref().serialize(serializer)
    }
}

impl<'json, R: AsRef<CJsonRef<'json>>> Serialize for CJsonObject<R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.len() as usize))?;
        for (k, v) in self.iter() {
            let k = k.to_str().map_err(serde::ser::Error::custom)?;
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

impl<'json, R: AsRef<CJsonRef<'json>>> Serialize for CJsonArray<R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len() as usize))?;
        for v in self.iter() {
            seq.serialize_element(v)?;
        }
        seq.end()
    }
}

impl Serialize for CJsonRef<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(b) = self.as_bool() {
            serializer.serialize_bool(b)
        } else if let Some(n) = self.as_number() {
            serializer.serialize_f64(n)
        } else if let Some(s) = self.as_c_string() {
            let s = s.to_str().map_err(serde::ser::Error::custom)?;
            serializer.serialize_str(s)
        } else if let Some(a) = self.as_array() {
            a.serialize(serializer)
        } else if let Some(o) = self.as_object() {
            o.serialize(serializer)
        } else {
            unreachable!("Malformed cJSON")
        }
    }
}

/// A serializer for CJson values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Serializer;

impl serde::Serializer for Serializer {
    type Ok = CJson<'static>;

    type Error = Error;

    type SerializeSeq = SerializeCJsonArray;
    type SerializeTuple = SerializeCJsonArray;
    type SerializeTupleStruct = SerializeCJsonArray;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeCJsonObject;
    type SerializeStruct = SerializeCJsonObject;
    type SerializeStructVariant = SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        let cjson = CJson::bool(v)?;
        Ok(cjson)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as _)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as _)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as _)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as _)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as _)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as _)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as _)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as _)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as _)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        let cjson = CJson::number(v)?;
        Ok(cjson)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(v.to_string().as_str())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let v = CString::new(v)?;
        let cjson = CJson::string(v)?;
        Ok(cjson)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let mut seq = self.serialize_seq(v.len().into())?;
        for byte in v {
            SerializeSeq::serialize_element(&mut seq, byte)?;
        }
        SerializeSeq::end(seq)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        let cjson = CJson::null()?;
        Ok(cjson)
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_none()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
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
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let variant = CString::new(variant)?;
        let value = value.serialize(self)?;
        let cjson = cjson!({variant => value})?.into();
        Ok(cjson)
    }

    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let array = CJsonArray::new()?;
        Ok(SerializeCJsonArray { array })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(SerializeTupleVariant {
            name: CString::new(variant)?,
            array: CJsonArray::new()?,
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeCJsonObject {
            object: CJsonObject::new()?,
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(len.into())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(SerializeStructVariant {
            name: CString::new(variant)?,
            object: CJsonObject::new()?,
        })
    }
    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Display,
    {
        let value = value.to_string();
        let cstring = CString::new(value)?;
        let cjson = CJson::string(cstring)?;
        Ok(cjson)
    }
}

pub struct SerializeCJsonArray {
    array: CJsonArray<CJson<'static>>,
}

impl SerializeSeq for SerializeCJsonArray {
    type Ok = CJson<'static>;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let value = to_cjson(value)?;
        self.array.push(value);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.array.into())
    }
}

impl SerializeTuple for SerializeCJsonArray {
    type Ok = CJson<'static>;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl SerializeTupleStruct for SerializeCJsonArray {
    type Ok = CJson<'static>;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

pub struct SerializeCJsonObject {
    object: CJsonObject<CJson<'static>>,
    next_key: Option<CString>,
}

impl SerializeMap for SerializeCJsonObject {
    type Ok = CJson<'static>;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.next_key = Some(key.serialize(MapKeySerializer)?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let key = self
            .next_key
            .take()
            .expect("serialize_value called before serialize_key");
        let value = to_cjson(value)?;
        self.object.insert(key, value);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.object.into())
    }
}

impl SerializeStruct for SerializeCJsonObject {
    type Ok = CJson<'static>;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeMap::end(self)
    }
}

pub struct SerializeTupleVariant {
    name: CString,
    array: CJsonArray<CJson<'static>>,
}

impl serde::ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = CJson<'static>;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.array.push(to_cjson(value)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let SerializeTupleVariant { name, array } = self;
        Ok(cjson!({name => array})?.into())
    }
}

pub struct SerializeStructVariant {
    name: CString,
    object: CJsonObject<CJson<'static>>,
}

impl serde::ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = CJson<'static>;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.object.insert(CString::new(key)?, to_cjson(value)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let SerializeStructVariant { name, object } = self;
        Ok(cjson!({name => object})?.into())
    }
}

pub struct MapKeySerializer;

impl serde::Serializer for MapKeySerializer {
    type Ok = CString;
    type Error = Error;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(CString::new(variant)?)
    }

    #[inline]
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        Ok(if value { c"true" } else { c"false" }.to_owned())
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        let value = format!("{value}");
        Ok(CString::new(value)?)
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        let value = format!("{value}");
        Ok(CString::new(value)?)
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        let value = format!("{value}");
        Ok(CString::new(value)?)
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        let value = format!("{value}");
        Ok(CString::new(value)?)
    }

    fn serialize_i128(self, value: i128) -> Result<Self::Ok, Self::Error> {
        let value = format!("{value}");
        Ok(CString::new(value)?)
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        let value = format!("{value}");
        Ok(CString::new(value)?)
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
        let value = format!("{value}");
        Ok(CString::new(value)?)
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
        let value = format!("{value}");
        Ok(CString::new(value)?)
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        let value = format!("{value}");
        Ok(CString::new(value)?)
    }

    fn serialize_u128(self, value: u128) -> Result<Self::Ok, Self::Error> {
        let value = format!("{value}");
        Ok(CString::new(value)?)
    }

    fn serialize_f32(self, value: f32) -> Result<Self::Ok, Self::Error> {
        if value.is_finite() {
            let value = format!("{value}");
            Ok(CString::new(value)?)
        } else {
            Err(Error::FloatNotFinite)
        }
    }

    fn serialize_f64(self, value: f64) -> Result<Self::Ok, Self::Error> {
        if value.is_finite() {
            let value = format!("{value}");
            Ok(CString::new(value)?)
        } else {
            Err(Error::FloatNotFinite)
        }
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        Ok(CString::new(value.to_string())?)
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        Ok(CString::new(value)?)
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Error::KeyMustBeAString)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::KeyMustBeAString)
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Display,
    {
        Ok(CString::new(value.to_string())?)
    }
}
