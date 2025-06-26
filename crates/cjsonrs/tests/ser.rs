#![cfg(feature = "serde")]

use cjsonrs::cjson;
use cjsonrs::serde::to_cjson;
use core::error::Error;
use serde::{Deserialize, Serialize};

#[test]
fn assert_that_tuples_can_be_serialized_into_cjson() -> Result<(), Box<dyn Error>> {
    let expected = cjson!([c"hello", 42])?.into();
    assert_eq!(to_cjson(&("hello", 42))?, expected);
    Ok(())
}

#[test]
fn assert_that_arrays_can_be_serialized_into_cjson() -> Result<(), Box<dyn Error>> {
    let expected = cjson!([c"hello"])?.into();
    assert_eq!(to_cjson(&vec!["hello"])?, expected);
    Ok(())
}

#[test]
fn assert_that_structs_can_be_serialized_into_cjson() -> Result<(), Box<dyn Error>> {
    let expected = cjson!({
        c"hello" => c"world",
        c"answer" => 42,
    })?
    .into();

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct Object {
        hello: String,
        answer: i32,
    }

    assert_eq!(
        to_cjson(&Object {
            hello: "world".to_string(),
            answer: 42,
        })?,
        expected
    );
    Ok(())
}

#[test]
fn assert_that_newtype_structs_can_be_serialized_into_cjson() -> Result<(), Box<dyn Error>> {
    let expected = cjson!(c"hello")?;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct Object(String);

    assert_eq!(to_cjson(&Object("hello".to_string()))?, expected);
    Ok(())
}

#[test]
fn assert_that_tuple_structs_can_be_serialized_into_cjson() -> Result<(), Box<dyn Error>> {
    let expected = cjson!([c"hello", 42])?.into();

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct Object(String, i32);

    assert_eq!(to_cjson(&Object("hello".to_string(), 42))?, expected);
    Ok(())
}

#[test]
fn assert_that_struct_variant_can_be_serialized_into_cjson() -> Result<(), Box<dyn Error>> {
    let expected = cjson!({
        c"variant" => {
            c"hello" => c"world",
            c"answer" => 42,
        },
    })?
    .into();

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    enum Object {
        Variant { hello: String, answer: i32 },
    }

    assert_eq!(
        to_cjson(&Object::Variant {
            hello: "world".to_string(),
            answer: 42,
        })?,
        expected
    );
    Ok(())
}

#[test]
fn assert_that_unit_variant_can_be_serialized_into_cjson() -> Result<(), Box<dyn Error>> {
    let expected = cjson!(c"Variant")?.into();

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    enum Object {
        Variant,
    }

    assert_eq!(to_cjson(&Object::Variant)?, expected);
    Ok(())
}
#[test]
fn assert_that_tuple_variant_can_be_serialized_into_cjson() -> Result<(), Box<dyn Error>> {
    let expected = cjson!({c"Variant" => [c"hello", 42]})?.into();

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    enum Object {
        Variant(String, i32),
    }

    assert_eq!(
        to_cjson(&Object::Variant("hello".to_string(), 42))?,
        expected
    );
    Ok(())
}

#[test]
fn assert_that_unit_can_be_serialized_into_cjson() -> Result<(), Box<dyn Error>> {
    let expected = cjson!(null)?;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct Object;

    assert_eq!(to_cjson(&Object)?, expected);
    Ok(())
}

#[test]
fn assert_that_strings_can_be_serialized_into_cjson() -> Result<(), Box<dyn Error>> {
    let expected = cjson!(c"hello")?;
    assert_eq!(to_cjson(&"hello")?, expected);
    Ok(())
}

#[test]
fn assert_that_numbers_can_be_serialized_into_cjson() -> Result<(), Box<dyn Error>> {
    let expected = cjson!(42)?;
    assert_eq!(to_cjson(&42)?, expected);
    Ok(())
}

#[test]
fn assert_that_booleans_can_be_serialized_into_cjson() -> Result<(), Box<dyn Error>> {
    let expected = cjson!(true)?;
    assert_eq!(to_cjson(&true)?, expected);
    Ok(())
}

#[test]
fn assert_that_cjson_null_can_be_serialized_into_serde_json_value() -> Result<(), Box<dyn Error>> {
    let obtained = serde_json::to_value(&cjson!(null)?)?;
    let expected = serde_json::json!(null);

    assert_eq!(obtained, expected);
    Ok(())
}

#[test]
fn assert_that_cjson_string_can_be_serialized_into_serde_json_value() -> Result<(), Box<dyn Error>>
{
    let obtained = serde_json::to_value(&cjson!(c"Hello world!")?)?;
    let expected = serde_json::json!("Hello world!");

    assert_eq!(obtained, expected);
    Ok(())
}

#[test]
fn assert_that_cjson_number_can_be_serialized_into_serde_json_value() -> Result<(), Box<dyn Error>>
{
    let obtained = serde_json::to_value(&cjson!(42.0)?)?;
    let expected = serde_json::json!(42.0);

    assert_eq!(obtained, expected);
    Ok(())
}

#[test]
fn assert_that_cjson_bool_can_be_serialized_into_serde_json_value() -> Result<(), Box<dyn Error>> {
    let obtained = serde_json::to_value(&cjson!(true)?)?;
    let expected = serde_json::json!(true);

    assert_eq!(obtained, expected);
    Ok(())
}

#[test]
fn assert_that_cjson_array_can_be_serialized_into_serde_json_value() -> Result<(), Box<dyn Error>> {
    let obtained = serde_json::to_value(&cjson!([c"hello", null])?)?;
    let expected = serde_json::json!(["hello", null]);

    assert_eq!(obtained, expected);
    Ok(())
}

#[test]
fn assert_that_cjson_object_can_be_serialized_into_serde_json_value() -> Result<(), Box<dyn Error>>
{
    let obtained = serde_json::to_value(&cjson!({c"hello" => c"world"})?)?;
    let expected = serde_json::json!({"hello": "world"});

    assert_eq!(obtained, expected);
    Ok(())
}

#[test]
fn assert_that_complex_cjson_can_be_serialized_into_serde_json_value() -> Result<(), Box<dyn Error>>
{
    let obtained = serde_json::to_value(&cjson!({
        c"hello" => c"world",
        c"answer" => 42.0,
        c"array" => [c"hello", null],
        c"object" => {c"hello" => c"world"},
    })?)?;
    let expected = serde_json::json!({
        "hello": "world",
        "answer": 42.0,
        "array": ["hello", null],
        "object": {"hello": "world"},
    });

    assert_eq!(obtained, expected);
    Ok(())
}
