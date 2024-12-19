#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{borrow::ToOwned, ffi::CString, string::ToString};

#[cfg(feature = "std")]
use std::ffi::CString;

use super::Error;
use crate::CJson;
use crate::CJsonArray;
use crate::CJsonObject;
use crate::CJsonRef;
use core::fmt::Display;
use serde::de::*;

/// Deserialize a value from a CJson value.
#[inline(always)]
pub fn from_cjson<'de, T: DeserializeOwned>(cjson: &'_ CJsonRef<'_>) -> Result<T, Error> {
    todo!("from_cjson")
    // T::deserialize(Deserializer(cjson))
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Custom(msg.to_string())
    }
}

pub struct Deserializer<'de>(pub &'de CJsonRef<'de>);

/*
// Deserialization





impl<'de> Deserialize<'de> for CJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CJsonVisitor;

        impl<'vi> Visitor<'vi> for CJsonVisitor {
            type Value = CJson;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("any valid cJSON value")
            }

            fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_f64(v.into())
            }

            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_f64(v.into())
            }

            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_f64(v.into())
            }

            fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_f64(v.into())
            }

            fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_f64(v.into())
            }

            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_f64(v.into())
            }

            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_f64(v.into())
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                CJson::number(v).map_err(serde::de::Error::custom)
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                CJson::bool(v).map_err(serde::de::Error::custom)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let v = CString::new(v).map_err(serde::de::Error::custom)?;
                CJson::string(v).map_err(serde::de::Error::custom)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_unit()
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'vi>,
            {
                Deserialize::deserialize(deserializer)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                CJson::null().map_err(serde::de::Error::custom)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'vi>,
            {
                let mut array = CJsonArray::new().map_err(serde::de::Error::custom)?;

                while let Some(v) = seq.next_element::<CJson>()? {
                    array.push(v);
                }

                Ok(array.into())
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'vi>,
            {
                let mut obj = CJsonObject::new().map_err(serde::de::Error::custom)?;

                while let Some((k, v)) = map.next_entry()? {
                    let k: String = k;
                    let v: CJson = v;

                    let k = CString::new(k).map_err(serde::de::Error::custom)?;

                    obj.insert(k, v);
                }
                Ok(obj.into())
            }
        }

        deserializer.deserialize_any(CJsonVisitor)
    }
}


 */
