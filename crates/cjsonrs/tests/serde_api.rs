#![cfg(feature = "serde")]

use core::marker::PhantomData;
use core::ptr::NonNull;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::ffi::CString;

use cjsonrs::cjson;
use cjsonrs::serde::from_cjson;
use cjsonrs::serde::to_cjson;
use cjsonrs::serde::Error;
use cjsonrs::serde::MapKeySerializer;
use cjsonrs::serde::Serializer as CJsonSerializer;
use cjsonrs::CJson;
use cjsonrs::CJsonRef;
use serde::de::value;
use serde::de::IgnoredAny;
use serde::de::IntoDeserializer;
use serde::de::Visitor;
use serde::forward_to_deserialize_any;
use serde::ser::Error as _;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn raw_value() -> CJson<'static> {
    let ptr = unsafe { cjsonrs_sys::cJSON_CreateRaw(c"1".as_ptr()) };
    let ptr = NonNull::new(ptr).expect("cJSON_CreateRaw allocates");
    unsafe { CJson::from_raw_parts(ptr, PhantomData::<&()>) }
}

fn custom_message(error: Error) -> String {
    match error {
        Error::Custom(message) => message,
        other => panic!("expected a custom error, got {other:?}"),
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
enum Unit {
    A,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum Shapes {
    Unit,
    Newtype(u8),
    Tuple(u8, u8),
    Struct { a: u8 },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Newtype(u32);

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct UnitStruct;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TupleStruct(u8, bool);

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Point {
    x: u8,
    y: u8,
}

#[test]
fn every_primitive_serializes_as_a_number() -> TestResult {
    assert_eq!(to_cjson(-1i8)?.as_number(), Some(-1.0));
    assert_eq!(to_cjson(-1i16)?.as_number(), Some(-1.0));
    assert_eq!(to_cjson(-1i32)?.as_number(), Some(-1.0));
    assert_eq!(to_cjson(-1i64)?.as_number(), Some(-1.0));
    assert_eq!(to_cjson(-1i128)?.as_number(), Some(-1.0));
    assert_eq!(to_cjson(1u8)?.as_number(), Some(1.0));
    assert_eq!(to_cjson(1u16)?.as_number(), Some(1.0));
    assert_eq!(to_cjson(1u32)?.as_number(), Some(1.0));
    assert_eq!(to_cjson(1u64)?.as_number(), Some(1.0));
    assert_eq!(to_cjson(1u128)?.as_number(), Some(1.0));
    assert_eq!(to_cjson(1.5f32)?.as_number(), Some(1.5));
    Ok(())
}

#[test]
fn rust_shapes_serialize_like_serde_json() -> TestResult {
    assert!(to_cjson(())?.is_null());
    assert!(to_cjson(UnitStruct)?.is_null());
    assert!(to_cjson(None::<u8>)?.is_null());
    assert_eq!(to_cjson(Some(1u8))?, CJson::number(1)?);
    assert_eq!(to_cjson(Newtype(7))?, CJson::number(7)?);
    assert_eq!(to_cjson('x')?, CJson::string(c"x")?);
    assert_eq!(to_cjson(TupleStruct(1, true))?, cjson!([1, true])?.into());
    assert_eq!(to_cjson((1u8, 2u8))?, cjson!([1, 2])?.into());
    assert_eq!(
        CJsonSerializer.serialize_bytes(b"ab")?,
        cjson!([97, 98])?.into()
    );
    assert_eq!(
        CJsonSerializer.collect_str(&format_args!("{}-{}", 1, 2))?,
        CJson::string(c"1-2")?,
    );
    Ok(())
}

#[test]
fn custom_serialize_errors_are_reported() {
    struct Fails;

    impl Serialize for Fails {
        fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(S::Error::custom("refused"))
        }
    }

    assert_eq!(custom_message(to_cjson(Fails).unwrap_err()), "refused");
}

#[test]
fn map_keys_are_spelled_as_their_display_form() -> TestResult {
    fn key(value: impl Serialize) -> Result<CString, Error> {
        value.serialize(MapKeySerializer)
    }

    assert_eq!(key(true)?.as_c_str(), c"true");
    assert_eq!(key(false)?.as_c_str(), c"false");
    assert_eq!(key(-1i8)?.as_c_str(), c"-1");
    assert_eq!(key(-1i16)?.as_c_str(), c"-1");
    assert_eq!(key(-1i32)?.as_c_str(), c"-1");
    assert_eq!(key(-1i64)?.as_c_str(), c"-1");
    assert_eq!(key(-1i128)?.as_c_str(), c"-1");
    assert_eq!(key(1u8)?.as_c_str(), c"1");
    assert_eq!(key(1u16)?.as_c_str(), c"1");
    assert_eq!(key(1u32)?.as_c_str(), c"1");
    assert_eq!(key(1u64)?.as_c_str(), c"1");
    assert_eq!(key(1u128)?.as_c_str(), c"1");
    assert_eq!(key(1.1f32)?.as_c_str(), c"1.1");
    assert_eq!(key(1.5f64)?.as_c_str(), c"1.5");
    assert_eq!(key('c')?.as_c_str(), c"c");
    assert_eq!(key("s")?.as_c_str(), c"s");
    assert_eq!(key(Unit::A)?.as_c_str(), c"A");
    assert_eq!(key(Newtype(3))?.as_c_str(), c"3");
    assert_eq!(key(std::net::Ipv4Addr::LOCALHOST)?.as_c_str(), c"127.0.0.1");
    assert_eq!(MapKeySerializer.collect_str(&42)?.as_c_str(), c"42");
    assert_eq!(key(f32::NAN), Err(Error::FloatNotFinite));
    assert_eq!(key(f64::INFINITY), Err(Error::FloatNotFinite));
    Ok(())
}

#[test]
fn map_keys_without_a_string_spelling_are_rejected() {
    fn rejected<T>(result: Result<T, Error>) -> bool {
        matches!(result, Err(Error::KeyMustBeAString))
    }

    assert!(rejected(MapKeySerializer.serialize_bytes(b"x")));
    assert!(rejected(MapKeySerializer.serialize_unit()));
    assert!(rejected(MapKeySerializer.serialize_unit_struct("U")));
    assert!(rejected(
        MapKeySerializer.serialize_newtype_variant("E", 0, "V", &1)
    ));
    assert!(rejected(MapKeySerializer.serialize_none()));
    assert!(rejected(MapKeySerializer.serialize_some(&1)));
    assert!(rejected(MapKeySerializer.serialize_seq(None)));
    assert!(rejected(MapKeySerializer.serialize_tuple(1)));
    assert!(rejected(MapKeySerializer.serialize_tuple_struct("T", 1)));
    assert!(rejected(
        MapKeySerializer.serialize_tuple_variant("E", 0, "V", 1)
    ));
    assert!(rejected(MapKeySerializer.serialize_map(None)));
    assert!(rejected(MapKeySerializer.serialize_struct("S", 1)));
    assert!(rejected(
        MapKeySerializer.serialize_struct_variant("E", 0, "V", 1)
    ));
}

#[test]
fn cjson_values_serialize_into_other_formats() -> TestResult {
    let value = cjson!({
        c"negative" => {-3},
        c"huge" => 1e20,
        c"fraction" => 0.5,
        c"items" => [true, null, c"s"],
    })?;

    let json = serde_json::to_value(&value)?;
    assert_eq!(
        json,
        serde_json::json!({
            "negative": -3,
            "huge": 1e20,
            "fraction": 0.5,
            "items": [true, null, "s"],
        }),
    );

    let items = value
        .get(c"items")
        .and_then(CJsonRef::as_array)
        .ok_or("array")?;
    assert_eq!(
        serde_json::to_value(items)?,
        serde_json::json!([true, null, "s"])
    );
    Ok(())
}

#[test]
fn values_that_are_not_json_fail_to_serialize() -> TestResult {
    assert!(serde_json::to_value(raw_value()).is_err());
    assert!(serde_json::to_value(CJson::string(c"\xff")?).is_err());

    let mut object = cjsonrs::CJsonObject::new()?;
    object.insert(c"\xff", CJson::null()?);
    assert!(serde_json::to_value(&object).is_err());
    Ok(())
}

#[test]
fn type_errors_describe_the_value_that_was_found() -> TestResult {
    fn message<T: serde::de::DeserializeOwned + core::fmt::Debug>(
        value: impl AsRef<CJsonRef<'static>>,
    ) -> String {
        custom_message(from_cjson::<T>(value).unwrap_err())
    }

    assert!(message::<bool>(CJson::number(1.5)?).contains("floating point `1.5`"));
    assert!(message::<bool>(CJson::number(2)?).contains("integer `2`"));
    assert!(message::<bool>(CJson::string(c"s")?).contains("string \"s\""));
    assert!(message::<bool>(CJson::string(c"\xff")?).contains("non UTF-8 string"));
    assert!(message::<bool>(CJson::array()?).contains("sequence"));
    assert!(message::<bool>(CJson::object()?).contains("map"));
    assert!(message::<bool>(raw_value()).contains("unknown cJSON value"));
    assert!(message::<String>(CJson::null()?).contains("unit value"));
    assert!(message::<f64>(CJson::null()?).contains("unit value"));
    assert!(message::<()>(CJson::bool(true)?).contains("boolean `true`"));
    assert!(message::<UnitStruct>(CJson::bool(true)?).contains("boolean"));
    assert!(message::<Vec<u8>>(CJson::null()?).contains("unit value"));
    assert!(message::<BTreeMap<String, u8>>(CJson::null()?).contains("unit value"));
    assert!(message::<Point>(CJson::null()?).contains("unit value"));
    Ok(())
}

#[test]
fn unknown_and_non_utf8_values_cannot_be_deserialized() -> TestResult {
    assert_eq!(from_cjson::<IgnoredAny>(raw_value()).err(), None);
    assert_eq!(
        from_cjson::<serde_json::Value>(raw_value()).err(),
        Some(Error::UnknownValue),
    );
    assert!(matches!(
        from_cjson::<String>(CJson::string(c"\xff")?),
        Err(Error::Utf8String(_)),
    ));
    Ok(())
}

#[test]
fn rust_shapes_deserialize_from_cjson() -> TestResult {
    assert_eq!(from_cjson::<f32>(CJson::number(1.5)?)?, 1.5);
    assert_eq!(from_cjson::<char>(CJson::string(c"x")?)?, 'x');
    assert_eq!(from_cjson::<Option<u8>>(CJson::null()?)?, None);
    assert_eq!(from_cjson::<Option<u8>>(CJson::number(1)?)?, Some(1));
    assert_eq!(from_cjson::<()>(CJson::null()?)?, ());
    assert_eq!(from_cjson::<UnitStruct>(CJson::null()?)?, UnitStruct);
    assert_eq!(from_cjson::<Newtype>(CJson::number(4)?)?, Newtype(4));
    assert_eq!(
        from_cjson::<TupleStruct>(cjson!([1, true])?)?,
        TupleStruct(1, true)
    );
    assert_eq!(from_cjson::<(u8, u8)>(cjson!([1, 2])?)?, (1, 2));
    assert_eq!(from_cjson::<Point>(cjson!([1, 2])?)?, Point { x: 1, y: 2 });
    Ok(())
}

#[test]
fn byte_buffers_reject_values_that_are_not_bytes() -> TestResult {
    #[derive(Deserialize, Debug)]
    struct Bytes(#[serde(with = "bytes")] Vec<u8>);

    mod bytes {
        use serde::Deserializer;

        pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
            struct BytesVisitor;

            impl serde::de::Visitor<'_> for BytesVisitor {
                type Value = Vec<u8>;

                fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    f.write_str("bytes")
                }

                fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> {
                    Ok(v.to_vec())
                }

                fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E> {
                    Ok(v)
                }
            }

            d.deserialize_byte_buf(BytesVisitor)
        }
    }

    assert_eq!(from_cjson::<Bytes>(CJson::string(c"ab")?)?.0, b"ab");
    assert_eq!(from_cjson::<Bytes>(cjson!([1, 2])?)?.0, [1, 2]);
    assert!(from_cjson::<Bytes>(cjson!([300])?).is_err());
    assert!(from_cjson::<Bytes>(CJson::null()?).is_err());
    Ok(())
}

#[test]
fn enum_payloads_must_match_the_variant_shape() -> TestResult {
    assert_eq!(
        from_cjson::<Shapes>(cjson!({ c"Unit" => null })?)?,
        Shapes::Unit
    );
    assert!(from_cjson::<Shapes>(cjson!({ c"Unit" => 1 })?).is_err());
    assert!(from_cjson::<Shapes>(CJson::string(c"Newtype")?).is_err());
    assert!(from_cjson::<Shapes>(CJson::string(c"Tuple")?).is_err());
    assert!(from_cjson::<Shapes>(cjson!({ c"Tuple" => 1 })?).is_err());
    assert!(from_cjson::<Shapes>(CJson::string(c"Struct")?).is_err());
    assert!(from_cjson::<Shapes>(cjson!({ c"Struct" => 1 })?).is_err());
    assert!(from_cjson::<Shapes>(CJson::number(1)?).is_err());
    assert!(from_cjson::<Shapes>(cjson!({})?).is_err());

    let mut non_utf8 = cjsonrs::CJsonObject::new()?;
    non_utf8.insert(c"\xff", CJson::null()?);
    assert!(from_cjson::<Shapes>(&non_utf8).is_err());
    assert!(from_cjson::<BTreeMap<String, ()>>(&non_utf8).is_err());
    Ok(())
}

#[test]
fn map_keys_parse_back_into_their_rust_types() -> TestResult {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Key(f32);

    impl Eq for Key {}

    impl core::hash::Hash for Key {
        fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
            self.0.to_bits().hash(state);
        }
    }

    fn keys<K>(json: &str) -> Result<Vec<K>, Box<dyn std::error::Error>>
    where
        K: serde::de::DeserializeOwned + Ord,
    {
        let cjson: CJson = json.parse()?;
        Ok(from_cjson::<BTreeMap<K, IgnoredAny>>(cjson)?
            .into_keys()
            .collect())
    }

    assert_eq!(keys::<bool>(r#"{"true":0}"#)?, [true]);
    assert_eq!(keys::<i8>(r#"{"-1":0}"#)?, [-1]);
    assert_eq!(keys::<i16>(r#"{"-1":0}"#)?, [-1]);
    assert_eq!(keys::<i32>(r#"{"-1":0}"#)?, [-1]);
    assert_eq!(keys::<i64>(r#"{"-1":0}"#)?, [-1]);
    assert_eq!(keys::<i128>(r#"{"-1":0}"#)?, [-1]);
    assert_eq!(keys::<u8>(r#"{"1":0}"#)?, [1]);
    assert_eq!(keys::<u16>(r#"{"1":0}"#)?, [1]);
    assert_eq!(keys::<u64>(r#"{"1":0}"#)?, [1]);
    assert_eq!(keys::<u128>(r#"{"1":0}"#)?, [1]);
    assert_eq!(keys::<char>(r#"{"c":0}"#)?, ['c']);
    assert_eq!(keys::<Unit>(r#"{"A":0}"#)?, [Unit::A]);
    assert_eq!(keys::<Newtype>(r#"{"5":0}"#)?, [Newtype(5)]);
    assert_eq!(keys::<Option<u8>>(r#"{"5":0}"#)?, [Some(5)]);
    assert!(keys::<u8>(r#"{"x":0}"#).is_err());

    let cjson: CJson = r#"{"1.5":0}"#.parse()?;
    let floats = from_cjson::<HashMap<Key, IgnoredAny>>(&cjson)?;
    assert!(floats.contains_key(&Key(1.5)));
    let floats = from_cjson::<BTreeMap<String, IgnoredAny>>(&cjson)?;
    assert!(floats.contains_key("1.5"));

    struct FloatKeys(Vec<f64>);

    impl<'de> Deserialize<'de> for FloatKeys {
        fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            struct V;

            impl<'de> Visitor<'de> for V {
                type Value = FloatKeys;

                fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                    f.write_str("a map")
                }

                fn visit_map<A: serde::de::MapAccess<'de>>(
                    self,
                    mut map: A,
                ) -> Result<Self::Value, A::Error> {
                    let mut keys = Vec::new();
                    while let Some(key) = map.next_key::<f64>()? {
                        map.next_value::<IgnoredAny>()?;
                        keys.push(key);
                    }
                    Ok(FloatKeys(keys))
                }
            }

            d.deserialize_map(V)
        }
    }

    assert_eq!(from_cjson::<FloatKeys>(&cjson)?.0, [1.5]);
    Ok(())
}

#[test]
fn cjson_handles_deserialize_as_their_contents() -> TestResult {
    let value: CJson = cjson!({ c"a" => [c"x"] })?.into();
    let object = value.as_object().ok_or("object")?;
    let array = object
        .get(c"a")
        .and_then(CJsonRef::as_array)
        .ok_or("array")?;
    let reference: &CJsonRef = &value;

    let from_value: BTreeMap<String, Vec<String>> =
        Deserialize::deserialize((&value).into_deserializer())?;
    let from_reference: BTreeMap<String, Vec<String>> =
        Deserialize::deserialize(reference.into_deserializer())?;
    let from_object: BTreeMap<String, Vec<String>> =
        Deserialize::deserialize((&object).into_deserializer())?;
    let from_array: Vec<String> = Deserialize::deserialize((&array).into_deserializer())?;

    assert_eq!(from_value, from_reference);
    assert_eq!(from_value, from_object);
    assert_eq!(from_array, ["x"]);

    let identifier = CJson::string(c"field")?;
    let identifier: &CJsonRef = &identifier;
    identifier.deserialize_identifier(IgnoredAny)?;
    Ok(())
}

#[test]
fn cjson_deserializes_from_any_self_describing_format() -> TestResult {
    let huge = from_cjson::<serde_json::Value>(CJson::number(1e20)?)?;
    assert_eq!(huge, serde_json::json!(1e20));

    let bytes = value::BytesDeserializer::<value::Error>::new(b"x");
    let error = CJson::deserialize(bytes).unwrap_err().to_string();
    assert!(error.contains("any valid JSON value"));

    let json = r#"{"n":[1,-2,1.5,"s",null,true],"o":{}}"#;
    let value: CJson = serde_json::from_str(json)?;
    assert_eq!(value, json.parse()?);

    let value: CJson = serde_json::from_reader(json.as_bytes())?;
    assert_eq!(value, json.parse()?);

    assert!(serde_json::from_str::<CJson>(r#""a\u0000b""#).is_err());
    assert!(serde_json::from_str::<CJson>(r#"{"a\u0000b":1}"#).is_err());
    Ok(())
}

#[test]
fn cjson_accepts_wide_integers_owned_strings_and_options() -> TestResult {
    type E = value::Error;

    let wide: CJson = Deserialize::deserialize(value::I128Deserializer::<E>::new(-5))?;
    assert_eq!(wide.as_number(), Some(-5.0));
    let wide: CJson = Deserialize::deserialize(value::U128Deserializer::<E>::new(5))?;
    assert_eq!(wide.as_number(), Some(5.0));

    let owned: CJson = Deserialize::deserialize(value::StringDeserializer::<E>::new("s".into()))?;
    assert_eq!(owned.as_c_string(), Some(c"s"));
    let nul = value::StringDeserializer::<E>::new("a\0b".into());
    assert!(CJson::deserialize(nul).is_err());

    struct OptionDeserializer(Option<u8>);

    impl<'de> Deserializer<'de> for OptionDeserializer {
        type Error = E;

        fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, E> {
            match self.0 {
                Some(n) => visitor.visit_some(n.into_deserializer()),
                None => visitor.visit_none(),
            }
        }

        forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf option unit unit_struct newtype_struct seq tuple
            tuple_struct map struct enum identifier ignored_any
        }
    }

    assert!(CJson::deserialize(OptionDeserializer(None))?.is_null());
    assert_eq!(
        CJson::deserialize(OptionDeserializer(Some(3)))?.as_number(),
        Some(3.0)
    );
    Ok(())
}
