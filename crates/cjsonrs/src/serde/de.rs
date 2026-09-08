//! Deserialization of Rust data structures from [`CJson`] values.
//!
//! The workhorse of this module is the [`Deserializer`] implementation for
//! `&'de CJsonRef<'de>`. Because a [`CJsonRef`] is only ever observed behind a
//! reference, the borrow *is* the deserializer, and string data can be handed
//! to the visitor as `&'de str` without copying.
//!
//! # Number handling
//!
//! cJSON stores every number as a C `double`. That has two consequences:
//!
//! - Integers beyond 2<sup>53</sup> cannot be represented exactly, so such a
//!   value is rounded as it is stored and does not survive a round trip.
//!   Reading it back either fails, because rounding pushed it out of the
//!   target type's range, or returns the rounded value.
//! - There is no way to tell the JSON number `1` from `1.0`. When the target
//!   type is self-describing (i.e. [`Deserializer::deserialize_any`], as used
//!   by `#[serde(flatten)]`, `#[serde(untagged)]` and
//!   [`CJson`] itself), a number that holds an exact integral value is reported
//!   to the visitor as an integer.

use super::number::as_i128;
use super::number::as_i64;
use super::number::as_u128;
use super::number::as_u64;
use super::number::is_exact_integer;
use super::Error;
use crate::CJson;
use crate::CJsonArray;
use crate::CJsonIter;
use crate::CJsonObject;
use crate::CJsonRef;
use alloc::ffi::CString;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt::Formatter;
use serde::de;
use serde::de::value::BorrowedStrDeserializer;
use serde::de::DeserializeOwned;
use serde::de::DeserializeSeed;
use serde::de::EnumAccess;
use serde::de::Error as _;
use serde::de::Expected;
use serde::de::IntoDeserializer;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Unexpected;
use serde::de::VariantAccess;
use serde::de::Visitor;
use serde::forward_to_deserialize_any;
use serde::Deserialize;
use serde::Deserializer;

/// Deserializes a value of type `T` from any CJson-like value.
///
/// # Example
///
/// ```
/// # fn main() -> cjsonrs::serde::Result<()> {
/// use serde::Deserialize;
///
/// #[derive(Deserialize, Debug, PartialEq)]
/// struct Object {
///     hello: String,
///     answer: i32,
/// }
///
/// let cjson = cjsonrs::cjson!({
///     c"hello" => c"world",
///     c"answer" => 42,
/// })?;
///
/// assert_eq!(
///     cjsonrs::serde::from_cjson::<Object>(&cjson)?,
///     Object { hello: "world".to_string(), answer: 42 },
/// );
/// # Ok(()) }
/// ```
#[inline(always)]
pub fn from_cjson<'json, T>(value: impl AsRef<CJsonRef<'json>>) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    let cjson: &CJsonRef<'_> = value.as_ref();
    T::deserialize(cjson)
}

/// Deserializes a value of type `T` that borrows from the given CJson value.
///
/// Unlike [`from_cjson`], `T` may contain borrowed data such as `&str`, which
/// is handed out as a view straight into the cJSON allocation. The borrow keeps
/// the CJson value alive for as long as the result is.
///
/// # Example
///
/// ```
/// # fn main() -> cjsonrs::serde::Result<()> {
/// let cjson = cjsonrs::cjson!({ c"hello" => c"world" })?;
///
/// let borrowed: &str = cjsonrs::serde::from_cjson_borrowed(
///     cjson.get(c"hello").unwrap(),
/// )?;
///
/// assert_eq!(borrowed, "world");
/// # Ok(()) }
/// ```
#[inline(always)]
pub fn from_cjson_borrowed<'de, T>(value: &'de CJsonRef<'de>) -> Result<T, Error>
where
    T: Deserialize<'de>,
{
    T::deserialize(value)
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: core::fmt::Display,
    {
        Error::Custom(msg.to_string())
    }
}

impl<'json> CJsonRef<'json> {
    /// Describes this value for the benefit of serde's error messages.
    fn unexpected(&self) -> Unexpected<'_> {
        if self.is_null() {
            Unexpected::Unit
        } else if let Some(b) = self.as_bool() {
            Unexpected::Bool(b)
        } else if let Some(n) = self.as_number() {
            match as_i64(n) {
                Some(i) => Unexpected::Signed(i),
                None => Unexpected::Float(n),
            }
        } else if let Some(s) = self.as_c_string() {
            match s.to_str() {
                Ok(s) => Unexpected::Str(s),
                Err(_) => Unexpected::Other("non UTF-8 string"),
            }
        } else if self.is_array() {
            Unexpected::Seq
        } else if self.is_object() {
            Unexpected::Map
        } else {
            Unexpected::Other("unknown cJSON value")
        }
    }

    /// Builds an `invalid_type` error describing this value.
    fn invalid_type(&self, expected: &dyn Expected) -> Error {
        Error::invalid_type(self.unexpected(), expected)
    }

    /// Returns the string contents of this value as UTF-8, borrowed for as
    /// long as this reference lives.
    fn as_str(&self) -> Option<Result<&str, Error>> {
        self.as_c_string().map(|s| s.to_str().map_err(Error::from))
    }
}

/// Implements an integer `deserialize_*` method.
///
/// The number is first narrowed to `i64`/`u64` by `$via`, then converted to
/// the requested width with a checked conversion. Both steps must be fallible:
/// cJSON keeps numbers as `double`, so the stored value may be fractional, out
/// of range, or negative where an unsigned type was asked for.
macro_rules! deserialize_number {
    ($($method:ident => $via:ident, $visit:ident($ty:ty);)*) => {
        $(
            #[inline]
            fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: Visitor<'de>,
            {
                let n = self.as_number().ok_or_else(|| self.invalid_type(&visitor))?;

                match $via(n).and_then(|n| <$ty>::try_from(n).ok()) {
                    Some(n) => visitor.$visit(n),
                    // A number that will not fit the requested type is a value
                    // error rather than a type error. `unexpected` names it as
                    // an integer where it can, so the message stays readable.
                    None => Err(Error::invalid_value(self.unexpected(), &visitor)),
                }
            }
        )*
    };
}

impl<'de> Deserializer<'de> for &'de CJsonRef<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if self.is_null() {
            visitor.visit_unit()
        } else if let Some(b) = self.as_bool() {
            visitor.visit_bool(b)
        } else if let Some(n) = self.as_number() {
            // cJSON cannot tell `1` from `1.0`, so report exact integers as
            // integers. `#[serde(flatten)]` and `#[serde(untagged)]` buffer
            // through this method and would otherwise reject integer fields.
            if !is_exact_integer(n) {
                visitor.visit_f64(n)
            } else if let Some(n) = as_u64(n) {
                visitor.visit_u64(n)
            } else if let Some(n) = as_i64(n) {
                visitor.visit_i64(n)
            } else {
                visitor.visit_f64(n)
            }
        } else if let Some(s) = self.as_str() {
            visitor.visit_borrowed_str(s?)
        } else if self.is_array() {
            visitor.visit_seq(ArrayAccess::new(self))
        } else if self.is_object() {
            visitor.visit_map(ObjectAccess::new(self))
        } else {
            Err(Error::UnknownValue)
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.as_bool() {
            Some(b) => visitor.visit_bool(b),
            None => Err(self.invalid_type(&visitor)),
        }
    }

    deserialize_number! {
        deserialize_i8 => as_i64, visit_i8(i8);
        deserialize_i16 => as_i64, visit_i16(i16);
        deserialize_i32 => as_i64, visit_i32(i32);
        deserialize_i64 => as_i64, visit_i64(i64);
        deserialize_i128 => as_i128, visit_i128(i128);
        deserialize_u8 => as_u64, visit_u8(u8);
        deserialize_u16 => as_u64, visit_u16(u16);
        deserialize_u32 => as_u64, visit_u32(u32);
        deserialize_u64 => as_u64, visit_u64(u64);
        deserialize_u128 => as_u128, visit_u128(u128);
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_f64(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.as_number() {
            Some(n) => visitor.visit_f64(n),
            None => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // The `char` visitor turns a one character string into a `char` and
        // rejects anything longer.
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.as_str() {
            Some(s) => visitor.visit_borrowed_str(s?),
            None => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        if let Some(s) = self.as_str() {
            return visitor.visit_borrowed_bytes(s?.as_bytes());
        }

        // The serializer writes byte strings as an array of numbers, so accept
        // that shape back.
        if self.is_array() {
            let mut bytes = Vec::with_capacity(self.len() as usize);

            for item in self.iter() {
                let n = item
                    .as_number()
                    .and_then(as_u64)
                    .and_then(|n| u8::try_from(n).ok())
                    .ok_or_else(|| item.invalid_type(&"a byte between 0 and 255"))?;

                bytes.push(n);
            }

            return visitor.visit_byte_buf(bytes);
        }

        Err(self.invalid_type(&visitor))
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
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
            visitor.visit_unit()
        } else {
            Err(self.invalid_type(&visitor))
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
        if self.is_array() {
            visitor.visit_seq(ArrayAccess::new(self))
        } else {
            Err(self.invalid_type(&visitor))
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
        if self.is_object() {
            visitor.visit_map(ObjectAccess::new(self))
        } else {
            Err(self.invalid_type(&visitor))
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
        // A struct may have been written either as an object keyed by field
        // name or, by a compact encoder, as an array of field values.
        if self.is_array() {
            visitor.visit_seq(ArrayAccess::new(self))
        } else {
            self.deserialize_map(visitor)
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // A unit variant is a bare string; every other variant is an object
        // holding exactly one entry, keyed by the variant name.
        if let Some(variant) = self.as_str() {
            return visitor.visit_enum(EnumAccessor {
                variant: variant?,
                value: None,
            });
        }

        if self.is_object() {
            let mut iter = self.iter();

            let Some(entry) = iter.next() else {
                return Err(Error::invalid_value(
                    Unexpected::Map,
                    &"an object with a single key, the variant name",
                ));
            };

            if iter.next().is_some() {
                return Err(Error::invalid_value(
                    Unexpected::Map,
                    &"an object with a single key, the variant name",
                ));
            }

            let variant = entry
                .name()
                .ok_or_else(|| Error::custom("object entry is missing its key"))?;

            return visitor.visit_enum(EnumAccessor {
                variant: variant.to_str()?,
                value: Some(entry),
            });
        }

        Err(self.invalid_type(&visitor))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // The value is dropped on the floor, so there is no need to walk it.
        visitor.visit_unit()
    }
}

impl<'de> IntoDeserializer<'de, Error> for &'de CJsonRef<'de> {
    type Deserializer = Self;

    #[inline]
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> IntoDeserializer<'de, Error> for &'de CJson<'de> {
    type Deserializer = &'de CJsonRef<'de>;

    #[inline]
    fn into_deserializer(self) -> Self::Deserializer {
        self.as_ref()
    }
}

impl<'de, R> IntoDeserializer<'de, Error> for &'de CJsonArray<R>
where
    R: AsRef<CJsonRef<'de>>,
{
    type Deserializer = &'de CJsonRef<'de>;

    #[inline]
    fn into_deserializer(self) -> Self::Deserializer {
        self.as_ref()
    }
}

impl<'de, R> IntoDeserializer<'de, Error> for &'de CJsonObject<R>
where
    R: AsRef<CJsonRef<'de>>,
{
    type Deserializer = &'de CJsonRef<'de>;

    #[inline]
    fn into_deserializer(self) -> Self::Deserializer {
        self.as_ref()
    }
}

/// Walks the elements of a cJSON array.
struct ArrayAccess<'de> {
    iter: CJsonIter<'de, 'de>,
}

impl<'de> ArrayAccess<'de> {
    #[inline]
    fn new(array: &'de CJsonRef<'de>) -> Self {
        Self { iter: array.iter() }
    }
}

impl<'de> SeqAccess<'de> for ArrayAccess<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(item) => seed.deserialize(item).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match Iterator::size_hint(&self.iter) {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

/// Walks the entries of a cJSON object.
///
/// cJSON keeps keys and values in the same node, so a single pass over the
/// children yields both.
struct ObjectAccess<'de> {
    iter: CJsonIter<'de, 'de>,
    value: Option<&'de CJsonRef<'de>>,
}

impl<'de> ObjectAccess<'de> {
    #[inline]
    fn new(object: &'de CJsonRef<'de>) -> Self {
        Self {
            iter: object.iter(),
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for ObjectAccess<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let Some(entry) = self.iter.next() else {
            return Ok(None);
        };

        let key = entry
            .name()
            .ok_or_else(|| Error::custom("object entry is missing its key"))?;

        self.value = Some(entry);

        seed.deserialize(MapKeyDeserializer { key: key.to_str()? })
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self
            .value
            .take()
            .expect("next_value_seed called before next_key_seed");

        seed.deserialize(value)
    }

    fn size_hint(&self) -> Option<usize> {
        match Iterator::size_hint(&self.iter) {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

/// Deserializes an object key.
///
/// JSON keys are always strings, but the map key type on the Rust side need
/// not be. The serializer writes such keys by formatting them (see
/// [`MapKeySerializer`](super::MapKeySerializer)), so they are parsed back
/// here, which keeps a `HashMap<u32, _>` round trip working.
struct MapKeyDeserializer<'de> {
    key: &'de str,
}

/// Implements a `deserialize_*` method that parses the key string.
macro_rules! deserialize_key_from_str {
    ($($method:ident => $visit:ident($ty:ty);)*) => {
        $(
            #[inline]
            fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: Visitor<'de>,
            {
                match self.key.parse::<$ty>() {
                    Ok(value) => visitor.$visit(value),
                    Err(_) => Err(Error::invalid_value(Unexpected::Str(self.key), &visitor)),
                }
            }
        )*
    };
}

impl<'de> Deserializer<'de> for MapKeyDeserializer<'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.key)
    }

    deserialize_key_from_str! {
        deserialize_bool => visit_bool(bool);
        deserialize_i8 => visit_i8(i8);
        deserialize_i16 => visit_i16(i16);
        deserialize_i32 => visit_i32(i32);
        deserialize_i64 => visit_i64(i64);
        deserialize_i128 => visit_i128(i128);
        deserialize_u8 => visit_u8(u8);
        deserialize_u16 => visit_u16(u16);
        deserialize_u32 => visit_u32(u32);
        deserialize_u64 => visit_u64(u64);
        deserialize_u128 => visit_u128(u128);
        deserialize_f32 => visit_f32(f32);
        deserialize_f64 => visit_f64(f64);
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // A key is always present, so it can never be `None`.
        visitor.visit_some(self)
    }

    #[inline]
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

    #[inline]
    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // Only a unit variant can be spelled as a key.
        visitor.visit_enum(EnumAccessor {
            variant: self.key,
            value: None,
        })
    }

    forward_to_deserialize_any! {
        char str string bytes byte_buf unit unit_struct seq tuple tuple_struct
        map struct identifier ignored_any
    }
}

/// Provides the variant name of an enum, along with its payload if it has one.
struct EnumAccessor<'de> {
    variant: &'de str,
    value: Option<&'de CJsonRef<'de>>,
}

impl<'de> EnumAccess<'de> for EnumAccessor<'de> {
    type Error = Error;
    type Variant = VariantAccessor<'de>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(BorrowedStrDeserializer::<Error>::new(self.variant))?;

        Ok((variant, VariantAccessor { value: self.value }))
    }
}

/// Reads the payload of an enum variant.
struct VariantAccessor<'de> {
    value: Option<&'de CJsonRef<'de>>,
}

impl<'de> VariantAccess<'de> for VariantAccessor<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.value {
            None => Ok(()),
            Some(value) if value.is_null() => Ok(()),
            Some(value) => Err(value.invalid_type(&"a unit variant")),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value),
            None => Err(Error::invalid_type(
                Unexpected::UnitVariant,
                &"a newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(value) if value.is_array() => visitor.visit_seq(ArrayAccess::new(value)),
            Some(value) => Err(value.invalid_type(&visitor)),
            None => Err(Error::invalid_type(
                Unexpected::UnitVariant,
                &"a tuple variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(value) if value.is_object() => visitor.visit_map(ObjectAccess::new(value)),
            Some(value) => Err(value.invalid_type(&visitor)),
            None => Err(Error::invalid_type(
                Unexpected::UnitVariant,
                &"a struct variant",
            )),
        }
    }
}

impl<'de> Deserialize<'de> for CJson<'static> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
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
                self.visit_f64(value as f64)
            }

            #[inline]
            fn visit_i128<E: de::Error>(self, value: i128) -> Result<Self::Value, E> {
                self.visit_f64(value as f64)
            }

            #[inline]
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                self.visit_f64(value as f64)
            }

            #[inline]
            fn visit_u128<E: de::Error>(self, value: u128) -> Result<Self::Value, E> {
                self.visit_f64(value as f64)
            }

            #[inline]
            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
                CJson::number(value).map_err(de::Error::custom)
            }

            #[inline]
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                let cstring = CString::new(value).map_err(de::Error::custom)?;
                CJson::string(cstring).map_err(de::Error::custom)
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
                D: Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                CJson::null().map_err(de::Error::custom)
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut array: CJsonArray<CJson<'static>> =
                    CJsonArray::new().map_err(de::Error::custom)?;

                while let Some(elem) = visitor.next_element::<CJson<'static>>()? {
                    array.push(elem);
                }

                Ok(array.into())
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut object: CJsonObject<CJson<'static>> =
                    CJsonObject::new().map_err(de::Error::custom)?;

                while let Some(key) = visitor.next_key::<String>()? {
                    let key = CString::new(key).map_err(de::Error::custom)?;
                    let value = visitor.next_value::<CJson<'static>>()?;

                    object.insert(key, value);
                }

                Ok(object.into())
            }
        }

        deserializer.deserialize_any(CJsonVisitor)
    }
}
