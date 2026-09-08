#![cfg(feature = "serde")]

use cjsonrs::cjson;
use cjsonrs::serde::{from_cjson, from_cjson_borrowed, to_cjson};
use cjsonrs::CJson;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

#[test]
fn float_edge_cases() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Floats {
        nan: f64,
        inf: f64,
        neg_inf: f64,
        neg_zero: f64,
        large: f64,
        small: f64,
    }

    let floats = Floats {
        nan: f64::NAN,
        inf: f64::INFINITY,
        neg_inf: f64::NEG_INFINITY,
        neg_zero: -0.0,
        large: 1e308,
        small: 1e-308,
    };

    // A non finite value survives a round trip through the in-memory tree,
    // because cJSON stores it verbatim in the `valuedouble` field. Note that
    // *printing* the tree still renders it as `null`.
    let ser = to_cjson(&floats).expect("serialize floats");
    let de: Floats = from_cjson(&ser).expect("deserialize floats");

    assert!(de.nan.is_nan());
    assert_eq!(de.inf, f64::INFINITY);
    assert_eq!(de.neg_inf, f64::NEG_INFINITY);
    assert!(de.neg_zero.is_sign_negative());
    assert_eq!(de.large, 1e308);
    assert_eq!(de.small, 1e-308);
}

#[test]
fn integer_widths_round_trip() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Ints {
        i8: i8,
        i16: i16,
        i32: i32,
        i64: i64,
        u8: u8,
        u16: u16,
        u32: u32,
        u64: u64,
    }

    let ints = Ints {
        i8: i8::MIN,
        i16: i16::MIN,
        i32: i32::MIN,
        // Beyond 2^53 a `double` cannot hold every integer, so stay inside the
        // range cJSON can represent exactly.
        i64: -(1i64 << 53),
        u8: u8::MAX,
        u16: u16::MAX,
        u32: u32::MAX,
        u64: 1u64 << 53,
    };

    let ser = to_cjson(&ints).expect("serialize ints");
    let de: Ints = from_cjson(&ser).expect("deserialize ints");

    assert_eq!(ints, de);
}

#[test]
fn integers_outside_the_target_range_are_rejected() {
    // Too large for the target width.
    let cjson = cjson!(300).unwrap();
    assert!(from_cjson::<u8>(&cjson).is_err());

    // Negative where an unsigned type was asked for.
    let cjson = cjson!(-1).unwrap();
    assert!(from_cjson::<u32>(&cjson).is_err());

    // Fractional where an integer was asked for.
    let cjson = cjson!(1.5).unwrap();
    assert!(from_cjson::<i64>(&cjson).is_err());

    // Not finite.
    let cjson = CJson::number(f64::NAN).unwrap();
    assert!(from_cjson::<i64>(&cjson).is_err());

    // Still fine as a float.
    let cjson = cjson!(1.5).unwrap();
    assert_eq!(from_cjson::<f64>(&cjson).unwrap(), 1.5);
}

#[test]
fn integers_are_reported_as_integers_to_self_describing_targets() {
    // `deserialize_any` has to report an exact integral number as an integer,
    // otherwise `#[serde(flatten)]` and `#[serde(untagged)]` reject integer
    // fields. `serde_json::Value` takes the same path.
    let cjson = cjson!({ c"a" => 42, c"b" => 1.5 }).unwrap();
    let value: serde_json::Value = from_cjson(&cjson).unwrap();

    assert_eq!(value["a"], serde_json::json!(42));
    assert!(value["a"].is_u64());
    assert_eq!(value["b"], serde_json::json!(1.5));
}

#[test]
fn i128_round_trips_within_the_representable_range() {
    let ser = to_cjson(1i128 << 52).expect("serialize i128");
    assert_eq!(from_cjson::<i128>(&ser).unwrap(), 1i128 << 52);

    let ser = to_cjson(1u128 << 52).expect("serialize u128");
    assert_eq!(from_cjson::<u128>(&ser).unwrap(), 1u128 << 52);
}

#[test]
fn unicode_and_escaping() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Unicode {
        plain: String,
        emoji: String,
        escaped: String,
        unicode: String,
    }

    let val = Unicode {
        plain: "hello".to_string(),
        emoji: "😀🚀".to_string(),
        escaped: "line\nbreak\tand\\slash\"quote".to_string(),
        unicode: "汉字🌍".to_string(),
    };

    let ser = to_cjson(&val).expect("serialize unicode");
    let de: Unicode = from_cjson(&ser).expect("deserialize unicode");

    assert_eq!(val, de);
}

#[test]
fn interior_nul_bytes_are_rejected_rather_than_truncated() {
    // A C string cannot hold a nul byte, so this has to fail loudly.
    assert!(to_cjson("before\0after").is_err());
}

#[test]
fn strings_can_be_borrowed_from_the_cjson_allocation() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Borrowed<'a> {
        hello: &'a str,
    }

    let cjson = cjson!({ c"hello" => c"world" }).unwrap();
    let de: Borrowed<'_> = from_cjson_borrowed(cjson.as_ref()).expect("borrow from cjson");

    assert_eq!(de, Borrowed { hello: "world" });
}

#[test]
fn chars_round_trip_through_single_character_strings() {
    let ser = to_cjson('ñ').expect("serialize char");
    assert_eq!(from_cjson::<char>(&ser).unwrap(), 'ñ');

    // More than one character is not a `char`.
    let cjson = cjson!(c"no").unwrap();
    assert!(from_cjson::<char>(&cjson).is_err());
}

#[test]
fn byte_strings_round_trip_as_arrays_of_numbers() {
    use serde::de::Visitor;
    use serde::{Deserializer, Serializer};

    // `#[derive]` never reaches `serialize_bytes`/`deserialize_byte_buf`, so
    // drive them by hand.
    #[derive(Debug, PartialEq)]
    struct Bytes(Vec<u8>);

    impl Serialize for Bytes {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_bytes(&self.0)
        }
    }

    impl<'de> Deserialize<'de> for Bytes {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct BytesVisitor;

            impl Visitor<'_> for BytesVisitor {
                type Value = Bytes;

                fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    f.write_str("a byte string")
                }

                fn visit_bytes<E>(self, v: &[u8]) -> Result<Bytes, E> {
                    Ok(Bytes(v.to_vec()))
                }

                fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Bytes, E> {
                    Ok(Bytes(v))
                }
            }

            deserializer.deserialize_byte_buf(BytesVisitor)
        }
    }

    let val = Bytes(vec![0, 1, 254, 255]);

    let ser = to_cjson(&val).expect("serialize bytes");
    let de: Bytes = from_cjson(&ser).expect("deserialize bytes");

    assert_eq!(val, de);

    // A string is accepted as a byte string too, matching `serde_json`.
    let cjson = cjson!(c"hi").unwrap();
    assert_eq!(from_cjson::<Bytes>(&cjson).unwrap(), Bytes(b"hi".to_vec()));

    // An array element that is not a byte is an error, not a silent truncation.
    let cjson = cjson!([300]).unwrap();
    assert!(from_cjson::<Bytes>(&cjson).is_err());
}

#[test]
fn error_handling_malformed_input() {
    // Missing closing brace.
    assert!(CJson::from_str("{\"hello\": 1").is_err());
}

#[test]
fn error_handling_type_mismatch() {
    #[derive(Deserialize, Debug)]
    struct OnlyString {
        #[allow(dead_code)]
        s: String,
    }

    let cjson = cjson!({ c"s" => 123 }).unwrap();
    let result: Result<OnlyString, _> = from_cjson(&cjson);

    let error = result.expect_err("a number is not a string");
    assert!(
        error.to_string().contains("invalid type"),
        "unhelpful error: {error}"
    );
}

#[test]
fn missing_fields_are_reported() {
    #[derive(Deserialize, Debug)]
    struct Required {
        #[allow(dead_code)]
        a: i32,
    }

    let cjson = cjson!({}).unwrap();
    let error = from_cjson::<Required>(&cjson).expect_err("`a` is missing");

    assert!(
        error.to_string().contains("missing field"),
        "unhelpful error: {error}"
    );
}

#[test]
fn serde_default_and_skip() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Defaults {
        #[serde(default)]
        a: i32,
        #[serde(default = "Defaults::default_b")]
        b: String,
        #[serde(skip)]
        skipped: Option<String>,
    }

    impl Defaults {
        fn default_b() -> String {
            "default".to_string()
        }
    }

    let cjson = cjson!({}).unwrap();
    let de: Defaults = from_cjson(&cjson).expect("deserialize with defaults");

    assert_eq!(de.a, 0);
    assert_eq!(de.b, "default");
    assert_eq!(de.skipped, None);

    let ser = to_cjson(&de).expect("serialize with skip");
    let obj = ser.as_object().expect("an object");

    assert!(obj.get(c"skipped").is_none());
}

#[test]
fn serde_flatten() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Inner {
        a: i32,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Outer {
        #[serde(flatten)]
        inner: Inner,
        b: String,
    }

    let val = Outer {
        inner: Inner { a: 42 },
        b: "hello".to_string(),
    };

    let ser = to_cjson(&val).expect("serialize flatten");
    let de: Outer = from_cjson(&ser).expect("deserialize flatten");

    assert_eq!(val, de);
}

#[test]
fn serde_untagged_enums() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[serde(untagged)]
    enum Untagged {
        Int(i32),
        Text(String),
        Pair { x: f64, y: f64 },
    }

    for val in [
        Untagged::Int(-7),
        Untagged::Text("hi".to_string()),
        Untagged::Pair { x: 1.5, y: -2.5 },
    ] {
        let ser = to_cjson(&val).expect("serialize untagged");
        let de: Untagged = from_cjson(&ser).expect("deserialize untagged");

        assert_eq!(val, de);
    }
}

#[test]
fn all_enum_shapes_round_trip() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum Shape {
        Unit,
        Newtype(i32),
        Tuple(i32, String),
        Struct { a: i32 },
    }

    for val in [
        Shape::Unit,
        Shape::Newtype(1),
        Shape::Tuple(2, "two".to_string()),
        Shape::Struct { a: 3 },
    ] {
        let ser = to_cjson(&val).expect("serialize variant");
        let de: Shape = from_cjson(&ser).expect("deserialize variant");

        assert_eq!(val, de);
    }
}

#[test]
fn an_enum_object_must_hold_exactly_one_variant() {
    #[derive(Deserialize, Debug)]
    enum Shape {
        #[allow(dead_code)]
        Newtype(i32),
    }

    let cjson = cjson!({ c"Newtype" => 1, c"Other" => 2 }).unwrap();
    assert!(from_cjson::<Shape>(&cjson).is_err());

    let cjson = cjson!({}).unwrap();
    assert!(from_cjson::<Shape>(&cjson).is_err());
}

#[test]
fn unknown_extra_fields() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Known {
        a: i32,
    }

    let cjson = cjson!({ c"a" => 1, c"b" => 2 }).unwrap();
    let de: Known = from_cjson(&cjson).expect("deserialize with extra");

    assert_eq!(de.a, 1);
}

#[test]
fn non_string_map_keys_round_trip() {
    let mut map = HashMap::new();
    map.insert(7u32, "seven".to_string());

    let ser = to_cjson(&map).expect("serialize integer keys");
    let de: HashMap<u32, String> = from_cjson(&ser).expect("deserialize integer keys");

    assert_eq!(map, de);
}

#[test]
fn custom_serialize_deserialize() {
    use serde::{Deserializer, Serializer};

    #[derive(Debug, PartialEq)]
    struct Uppercase(String);

    impl Serialize for Uppercase {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.0.to_uppercase())
        }
    }

    impl<'de> Deserialize<'de> for Uppercase {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Ok(Uppercase(s.to_lowercase()))
        }
    }

    let val = Uppercase("Hello".to_string());
    let ser = to_cjson(&val).expect("serialize custom");
    let de: Uppercase = from_cjson(&ser).expect("deserialize custom");

    // Serialization uppercases, deserialization lowercases.
    assert_eq!(de, Uppercase("hello".to_string()));
}

#[test]
fn deeply_nested_structures() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Nested {
        level: Box<Option<Nested>>,
    }

    let mut n = Nested {
        level: Box::new(None),
    };

    for _ in 0..10 {
        n = Nested {
            level: Box::new(Some(n)),
        };
    }

    let ser = to_cjson(&n).expect("serialize nested");
    let de: Nested = from_cjson(&ser).expect("deserialize nested");

    assert_eq!(n, de);
}

#[test]
fn large_array_and_object() {
    let arr: Vec<u32> = (0..1000).collect();
    let ser = to_cjson(&arr).expect("serialize large array");
    let de: Vec<u32> = from_cjson(&ser).expect("deserialize large array");

    assert_eq!(arr, de);

    let mut map = HashMap::new();
    for i in 0..1000 {
        map.insert(format!("key{i}"), i);
    }

    let ser = to_cjson(&map).expect("serialize large object");
    let de: HashMap<String, u32> = from_cjson(&ser).expect("deserialize large object");

    assert_eq!(map, de);
}

#[test]
fn cjson_values_round_trip_through_serde_json() {
    let cjson = cjson!({
        c"null" => null,
        c"bool" => true,
        c"int" => 42,
        c"float" => 1.5,
        c"string" => c"text",
        c"array" => [1, c"two", false],
        c"object" => { c"nested" => c"yes" },
    })
    .unwrap();

    let value: serde_json::Value = from_cjson(&cjson).expect("into serde_json");
    let back = to_cjson(&value).expect("back into cjson");

    assert_eq!(back.as_ref(), cjson.as_ref());
}

#[test]
fn wrapper_types_can_be_deserialized_directly() {
    // `CJsonObject` and `CJsonArray` are CJson-like, so `from_cjson` takes them
    // without an intermediate conversion.
    let object = cjson!({ c"a" => 1 }).unwrap();
    assert_eq!(from_cjson::<HashMap<String, i32>>(&object).unwrap()["a"], 1);

    let array = cjson!([1, 2, 3]).unwrap();
    assert_eq!(from_cjson::<Vec<i32>>(&array).unwrap(), vec![1, 2, 3]);
}

#[test]
fn non_finite_numbers_print_as_null() {
    // cJSON keeps the value in memory but has no JSON spelling for it.
    let cjson = to_cjson(f64::NAN).expect("serialize NaN");
    assert_eq!(cjson.to_c_string().unwrap().to_str().unwrap(), "null");

    let cjson = to_cjson(f64::INFINITY).expect("serialize infinity");
    assert_eq!(cjson.to_c_string().unwrap().to_str().unwrap(), "null");
}

#[test]
fn keys_that_have_no_string_spelling_are_rejected() {
    use cjsonrs::serde::{Error, MapKeySerializer};

    let mut map = HashMap::new();
    map.insert((1u8, 2u8), 3u8);

    assert!(matches!(to_cjson(&map), Err(Error::KeyMustBeAString),));

    // A non-finite float has no sensible key spelling either.
    assert!(matches!(
        f64::NAN.serialize(MapKeySerializer),
        Err(Error::FloatNotFinite),
    ));
}

#[test]
fn seq_and_map_size_hints_are_exact() {
    use serde::de::{Deserializer, MapAccess, SeqAccess, Visitor};

    // The hint has to describe the elements still to come. Reporting the
    // *current* node's child count instead looks plausible and silently
    // mis-sizes every collection serde pre-allocates.
    struct SpySeq;
    impl<'de> Visitor<'de> for SpySeq {
        type Value = (Option<usize>, usize);
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a sequence")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let hint = a.size_hint();
            let mut count = 0;
            while a.next_element::<serde_json::Value>()?.is_some() {
                count += 1;
            }
            Ok((hint, count))
        }
    }

    struct SpyMap;
    impl<'de> Visitor<'de> for SpyMap {
        type Value = (Option<usize>, usize);
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a map")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let hint = a.size_hint();
            let mut count = 0;
            while a.next_entry::<String, serde_json::Value>()?.is_some() {
                count += 1;
            }
            Ok((hint, count))
        }
    }

    for cjson in [
        cjson!([1, 2, 3]).unwrap(),
        // A nested array is the case that used to report the inner length.
        cjson!([[1, 2, 3, 4, 5]]).unwrap(),
        cjson!([]).unwrap(),
    ] {
        let (hint, count) = cjson.as_ref().deserialize_seq(SpySeq).unwrap();
        assert_eq!(hint, Some(count), "seq of {count} reported {hint:?}");
    }

    let cjson = cjson!({ c"a" => 1, c"b" => 2, c"c" => 3 }).unwrap();
    let (hint, count) = cjson.as_ref().deserialize_map(SpyMap).unwrap();
    assert_eq!(hint, Some(3));
    assert_eq!(count, 3);
}

#[test]
fn wide_integers_use_their_own_range_not_i64s() {
    // 2^100 is exactly representable as an `f64` and fits an `i128`, so it has
    // to survive. Routing `i128` through the `i64` range would reject it.
    for shift in [52, 63, 64, 100, 126] {
        let value = 1i128 << shift;
        let ser = to_cjson(value).expect("serialize i128");
        assert_eq!(
            from_cjson::<i128>(&ser).ok(),
            Some(value),
            "i128 1<<{shift} did not round trip"
        );

        let value = 1u128 << shift;
        let ser = to_cjson(value).expect("serialize u128");
        assert_eq!(
            from_cjson::<u128>(&ser).ok(),
            Some(value),
            "u128 1<<{shift} did not round trip"
        );
    }

    // Still bounded by what an `f64` can hold exactly.
    let cjson = CJson::number(1.5).unwrap();
    assert!(from_cjson::<i128>(&cjson).is_err());
}

#[test]
fn integers_past_2_pow_53_do_not_survive_a_round_trip() {
    // The serializer rounds these to fit a `double`. Reading one back has two
    // possible outcomes, and neither returns the original value.

    // Rounding pushed the value out of the target range, so it is rejected.
    for value in [i64::MAX, i64::MAX - 1] {
        let ser = to_cjson(value).expect("serialize i64");
        assert!(
            from_cjson::<i64>(&ser).is_err(),
            "{value} should not read back as an i64"
        );
    }

    let ser = to_cjson(u64::MAX).expect("serialize u64");
    assert!(from_cjson::<u64>(&ser).is_err());

    // Rounding landed back inside the range, so the rounded value is returned
    // and the change is silent. This is inherent to `double` storage.
    let ser = to_cjson(i64::MIN + 1).expect("serialize i64");
    assert_eq!(from_cjson::<i64>(&ser).unwrap(), i64::MIN);

    // The documented safe range does round trip exactly.
    for value in [1i64 << 53, -(1i64 << 53), 0, -1] {
        let ser = to_cjson(value).expect("serialize i64");
        assert_eq!(from_cjson::<i64>(&ser).unwrap(), value);
    }
}

#[test]
fn negative_zero_keeps_its_sign_through_a_self_describing_target() {
    // `-0.0` is integral, but reporting it as `0` would drop the sign.
    let cjson = CJson::number(-0.0).unwrap();

    let value: serde_json::Value = from_cjson(&cjson).unwrap();
    assert!(value.is_f64(), "-0.0 became {value}");
    assert!(value.as_f64().unwrap().is_sign_negative());

    assert!(from_cjson::<f64>(&cjson).unwrap().is_sign_negative());

    // Positive zero is still an integer.
    let cjson = CJson::number(0.0).unwrap();
    let value: serde_json::Value = from_cjson(&cjson).unwrap();
    assert!(value.is_u64(), "0.0 became {value}");
}

#[test]
fn serializing_through_another_format_agrees_with_cjsons_own_printer() {
    let cjson = cjson!({
        c"int" => 1,
        c"from_float" => 42.0,
        c"float" => 1.5,
        c"array" => [1, 2],
    })
    .unwrap();

    assert_eq!(
        serde_json::to_string(&cjson.as_ref()).unwrap(),
        cjson.as_ref().to_c_string().unwrap().to_str().unwrap(),
    );
}

#[test]
fn a_string_node_without_string_data_is_not_a_string() {
    // A tree from foreign C code may leave `valuestring` null on a node that
    // still claims to be a string. Reading it must not dereference null.
    let mut raw = unsafe { core::mem::zeroed::<cjsonrs_sys::cJSON>() };
    raw.type_ = cjsonrs_sys::cJSON_String as i32;

    let cjson = unsafe { cjsonrs::CJsonRef::from_ptr(&raw as *const _) };

    assert!(cjson.is_string());
    assert!(cjson.as_c_string().is_none());
    assert!(from_cjson::<String>(cjson).is_err());
}

#[test]
fn converting_an_object_into_an_array_is_rejected() {
    // The type check used to be inverted, which let safe code build a keyless
    // "object" that then panicked in `Serialize` and `Debug`.
    let object = CJson::object().unwrap();
    assert!(cjsonrs::CJsonArray::try_from(object).is_err());

    let array = CJson::array().unwrap();
    assert!(cjsonrs::CJsonArray::try_from(array).is_ok());

    let array = CJson::array().unwrap();
    assert!(cjsonrs::CJsonObject::try_from(array).is_err());
}
