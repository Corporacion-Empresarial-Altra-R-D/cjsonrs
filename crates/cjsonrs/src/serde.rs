use std::ffi::CString;
use std::ops::Deref;

use serde::de::Visitor;
use serde::ser::SerializeMap;
use serde::ser::SerializeSeq;
use serde::Deserialize;
use serde::Serialize;

use super::CJson;
use super::CJsonArray;
use super::CJsonObject;
use super::CJsonRef;

// Serialization
impl Serialize for CJson {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.deref().serialize(serializer)
    }
}

impl<R: AsRef<CJsonRef>> Serialize for CJsonObject<R> {
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

impl<R: AsRef<CJsonRef>> Serialize for CJsonArray<R> {
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

impl Serialize for CJsonRef {
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
            serializer.serialize_unit()
        }
    }
}

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

pub struct Serializer;

impl serde::Serializer for Serializer {
    type Error = Error;
    type Ok = CJson;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        CJson::bool(v)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v.into())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        CJson::number(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v.into())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v.into())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v.into())
    }
    
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v.into())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v.into())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v.into())
    }
}
