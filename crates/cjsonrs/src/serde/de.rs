cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        use std::ffi::CString;

    } else if #[cfg(feature = "alloc")] {
        extern crate alloc;

        use alloc::ffi::CString;
        use alloc::borrow::ToOwned;
        use alloc::string::ToString;
    }
}

use super::Error;
use crate::CJson;
use crate::CJsonArray;
use crate::CJsonObject;
use crate::CJsonRef;
use core::fmt::Display;
use core::fmt::Formatter;
use serde::de;
use serde::de::Error as _;
use serde::de::*;
use serde::forward_to_deserialize_any;

/// Deserialize a value from a CJson value.
#[inline(always)]
pub fn from_cjson<'json, T>(value: impl AsRef<CJsonRef<'json>>) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    T::deserialize(value.as_ref())
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Custom(msg.to_string())
    }
}

impl CJsonRef<'_> {
    fn unexpected(&self) -> super::Result<Unexpected<'_>> {
        if self.is_null() {
            Ok(Unexpected::Unit)
        } else if let Some(b) = self.as_bool() {
            Ok(Unexpected::Bool(b))
        } else if let Some(n) = self.as_number() {
            Ok(Unexpected::Float(n))
        } else if let Some(s) = self.as_c_string() {
            Ok(Unexpected::Str(s.to_str()?))
        } else if self.is_array() {
            Ok(Unexpected::Seq)
        } else if self.is_object() {
            Ok(Unexpected::Map)
        } else {
            Err(Error::Custom("cJSON value is unknown".into()))
        }
    }
}

impl<'de> Deserialize<'de> for CJson<'static> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct CJsonVisitor;

        impl<'de> Visitor<'de> for CJsonVisitor {
            type Value = CJson<'static>;

            fn expecting(&self, formatter: &mut Formatter) -> core::fmt::Result {
                formatter.write_str("any valid JSON value")
            }

            #[inline]
            fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
                CJson::bool(value).map_err(de::Error::custom)
            }

            #[inline]
            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
                CJson::number(value as f64).map_err(de::Error::custom)
            }

            fn visit_i128<E: de::Error>(self, value: i128) -> Result<Self::Value, E> {
                CJson::number(value as f64).map_err(de::Error::custom)
            }

            #[inline]
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                CJson::number(value as f64).map_err(de::Error::custom)
            }

            fn visit_u128<E: de::Error>(self, value: u128) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                CJson::number(value as f64).map_err(de::Error::custom)
            }

            #[inline]
            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
                CJson::number(value).map_err(de::Error::custom)
            }

            #[inline]
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
                let cstring = CString::new(value).map_err(de::Error::custom)?;
                CJson::string(cstring).map_err(de::Error::custom)
            }

            #[inline]
            fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
                self.visit_unit()
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                de::Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                CJson::null().map_err(de::Error::custom)
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut array = CJsonArray::new().map_err(de::Error::custom)?;

                while let Some(elem) = visitor.next_element()? {
                    let elem: CJson = elem;
                    array.push(elem);
                }

                Ok(array.into())
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut obj = CJsonObject::new().map_err(de::Error::custom)?;

                while let Some((key, value)) = visitor.next_entry()? {
                    let key: String = key;
                    let value: CJson = value;

                    let key = CString::new(key).map_err(de::Error::custom)?;

                    obj.insert(key, value);
                }

                Ok(obj.into())
            }
        }

        deserializer.deserialize_any(CJsonVisitor)
    }
}

impl<'de> de::Deserializer<'de> for &CJsonRef<'de> {
    type Error = super::Error;
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(b) = self.as_bool() {
            visitor.visit_bool(b)
        } else if let Some(n) = self.as_number() {
            visitor.visit_f64(n)
        } else if let Some(s) = self.as_c_string() {
            let s = s.to_str().map_err(Self::Error::custom)?;
            visitor.visit_str(s)
        } else if self.is_null() {
            visitor.visit_unit()
        } else if let Some(ref array_ref) = self.as_array() {
            array_ref.deserialize_any(visitor)
        } else if let Some(ref object_ref) = self.as_object() {
            object_ref.deserialize_any(visitor)
        } else {
            Err(de::Error::custom("unknown JSON value"))
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(b) = self.as_bool() {
            visitor.visit_bool(b)
        } else {
            Err(de::Error::invalid_type(self.unexpected()?, &visitor))
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(n) = self.as_number() {
            visitor.visit_i8(n as i8)
        } else {
            Err(de::Error::invalid_type(self.unexpected()?, &visitor))
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(n) = self.as_number() {
            visitor.visit_i16(n as i16)
        } else {
            Err(de::Error::invalid_type(self.unexpected()?, &visitor))
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(n) = self.as_number() {
            visitor.visit_i32(n as i32)
        } else {
            Err(de::Error::invalid_type(self.unexpected()?, &visitor))
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(n) = self.as_number() {
            visitor.visit_i64(n as i64)
        } else {
            Err(de::Error::invalid_type(self.unexpected()?, &visitor))
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(n) = self.as_number() {
            visitor.visit_u8(n as u8)
        } else {
            Err(de::Error::invalid_type(self.unexpected()?, &visitor))
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(n) = self.as_number() {
            visitor.visit_u16(n as u16)
        } else {
            Err(de::Error::invalid_type(self.unexpected()?, &visitor))
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(n) = self.as_number() {
            visitor.visit_u32(n as u32)
        } else {
            Err(de::Error::invalid_type(self.unexpected()?, &visitor))
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(n) = self.as_number() {
            visitor.visit_u64(n as u64)
        } else {
            Err(de::Error::invalid_type(self.unexpected()?, &visitor))
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(n) = self.as_number() {
            visitor.visit_f32(n as f32)
        } else {
            Err(de::Error::invalid_type(self.unexpected()?, &visitor))
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(n) = self.as_number() {
            visitor.visit_f64(n)
        } else {
            Err(de::Error::invalid_type(self.unexpected()?, &visitor))
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(s) = self.as_c_string() {
            visitor.visit_string(s.to_str().map_err(Self::Error::custom)?.to_owned())
        } else {
            Err(de::Error::invalid_type(self.unexpected()?, &visitor))
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(s) = self.as_c_string() {
            visitor.visit_str(s.to_str().map_err(Self::Error::custom)?)
        } else if let Some(a) = self.as_array() {
            visit_array(a, visitor)
        } else {
            Err(de::Error::invalid_type(self.unexpected()?, &visitor))
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.is_null() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.is_null() {
            visitor.visit_none()
        } else {
            Err(de::Error::invalid_type(self.unexpected()?, &visitor))
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(a) = self.as_array() {
            visit_array(a, visitor)
        } else {
            Err(de::Error::invalid_type(self.unexpected()?, &visitor))
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
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
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(ref object_ref) = self.as_object() {
            object_ref.deserialize_any(visitor)
        } else {
            Err(de::Error::invalid_type(self.unexpected()?, &visitor))
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(ref array_ref) = self.as_array() {
            visit_array(array_ref, visitor)
        } else if let Some(ref object_ref) = self.as_object() {
            object_ref.deserialize_any(visitor)
        } else {
            Err(de::Error::invalid_type(self.unexpected()?, &visitor))
        }
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(ref object_ref) = self.as_object() {
            object_ref.deserialize_enum(name, variants, visitor)
        } else if let Some(variant) = self.as_c_string() {
            visitor.visit_enum(EnumDeserializer {
                variant: variant.to_str().map_err(Self::Error::custom)?.to_string(),
                value: None,
            })
        } else {
            Err(de::Error::invalid_type(
                self.unexpected()?,
                &"string or map",
            ))
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }
}

impl<'de, R> de::Deserializer<'de> for &'de CJsonArray<R>
where
    R: AsRef<CJsonRef<'de>>,
{
    type Error = super::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let elements = self.iter().map(|item| item.as_ref());
        visitor.visit_seq(SeqRefDeserializer::new(elements))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let elements = self.iter().map(|item| item.as_ref());
        visitor.visit_seq(SeqRefDeserializer::new(elements))
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let elements = self.iter().map(|item| item.as_ref());
        visitor.visit_seq(SeqRefDeserializer::new(elements))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let elements = self.iter().map(|item| item.as_ref());
        visitor.visit_seq(SeqRefDeserializer::new(elements))
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf option unit
        unit_struct newtype_struct map struct enum identifier ignored_any
    }
}

impl<'de, R> de::Deserializer<'de> for &'de CJsonObject<R>
where
    R: AsRef<CJsonRef<'de>>,
{
    type Error = super::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let map = self.iter();
        visitor.visit_map(MapRefDeserializer::new(map))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let map = self.iter();
        visitor.visit_map(MapRefDeserializer::new(map))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let map = self.iter();
        visitor.visit_map(MapRefDeserializer::new(map))
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut iter = self.iter();
        let (variant, value) = match iter.next() {
            Some((k, v)) => (k, v),
            None => return Err(de::Error::custom("expected enum variant")),
        };
        if iter.next().is_some() {
            return Err(de::Error::custom("expected only one variant"));
        }
        visitor.visit_enum(EnumDeserializer {
            variant: variant.to_str().map_err(super::Error::custom)?.to_owned(),
            value: Some(value),
        })
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf option unit
        unit_struct newtype_struct seq tuple tuple_struct identifier ignored_any
    }
}

// Sequence deserializer for iterators over &CJsonRef<'de>
struct SeqRefDeserializer<I>(I);

impl<'de, I> SeqRefDeserializer<'de, I> {
    fn new(iter: I) -> Self {
        SeqRefDeserializer(iter)
    }
}

impl<'de, I> SeqAccess<'de> for SeqRefDeserializer<'de, I>
where
    I: Iterator<Item = &'de CJsonRef<'de>>,
{
    type Error = super::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.0.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }
}

// Map deserializer for CJsonObject
struct MapRefDeserializer<'de, I>
where
    I: Iterator<Item = (&'de std::ffi::CStr, &'de CJsonRef<'de>)>,
{
    iter: I,
    value: Option<&'de CJsonRef<'de>>,
}

impl<'de, I> MapRefDeserializer<'de, I>
where
    I: Iterator<Item = (&'de std::ffi::CStr, &'de CJsonRef<'de>)>,
{
    fn new(iter: I) -> Self {
        MapRefDeserializer { iter, value: None }
    }
}

impl<'de, I> serde::de::MapAccess<'de> for MapRefDeserializer<'de, I>
where
    I: Iterator<Item = (&'de std::ffi::CStr, &'de CJsonRef<'de>)>,
{
    type Error = super::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((k, v)) => {
                self.value = Some(v);
                let key = k.to_str().map_err(super::Error::custom)?;
                let de = key.into_deserializer();
                seed.deserialize(de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(v) => seed.deserialize(v),
            None => Err(de::Error::custom("value is missing for key")),
        }
    }
}

fn visit_array<'de, V>(array: CJsonArray<&CJsonRef<'de>>, visitor: V) -> Result<V::Value, Error>
where
    V: Visitor<'de>,
{
    let len = array.len();
    let mut deserializer = SeqDeserializer(array.iter());
    let seq = visitor.visit_seq(&mut deserializer)?;

    if deserializer.0.next().is_none() {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(
            len as _,
            &"fewer elements in array",
        ))
    }
}

struct SeqDeserializer<I>(I);

struct MapDeserializer<'a, I>
where
    I: Iterator,
{
    iter: &'a mut I,
    value: Option<&'a CJsonRef<'a>>,
}

impl<'de, I> MapDeserializer<'de, I>
where
    I: Iterator<Item = (&'de std::ffi::CStr, &'de CJsonRef<'de>)>,
{
    fn new(iter: &'de mut I) -> Self {
        MapDeserializer { iter, value: None }
    }
}

impl<'de, I> serde::de::MapAccess<'de> for MapDeserializer<'de, I>
where
    I: Iterator<Item = (&'de std::ffi::CStr, &'de CJsonRef<'de>)>,
{
    type Error = super::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((k, v)) => {
                self.value = Some(v);
                let key = k.to_str().map_err(super::Error::custom)?;
                let de = key.into_deserializer();
                seed.deserialize(de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(v) => seed.deserialize(v),
            None => Err(de::Error::custom("value is missing for key")),
        }
    }
}

// EnumDeserializer implementation
struct EnumDeserializer<'de> {
    variant: String,
    value: Option<&'de CJsonRef<'de>>,
}

impl<'de> serde::de::EnumAccess<'de> for EnumDeserializer<'de> {
    type Error = super::Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let val = seed.deserialize(self.variant.into_deserializer())?;
        Ok((val, self))
    }
}

impl<'de> serde::de::VariantAccess<'de> for EnumDeserializer<'de> {
    type Error = super::Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.value {
            Some(v) => serde::de::Deserialize::deserialize(v).map(|_: ()| ()),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.value {
            Some(v) => seed.deserialize(v),
            None => Err(de::Error::custom("expected value for newtype variant")),
        }
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Some(v) => v.deserialize_tuple(len, visitor),
            None => Err(de::Error::custom("expected value for tuple variant")),
        }
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Some(v) => v.deserialize_struct("", fields, visitor),
            None => Err(de::Error::custom("expected value for struct variant")),
        }
    }
}

impl<'de, I> SeqAccess<'de> for SeqDeserializer<I>
where
    I: Iterator,
    I::Item: AsRef<CJsonRef<'de>>,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.0.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }
}
