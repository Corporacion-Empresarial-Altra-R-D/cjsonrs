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
