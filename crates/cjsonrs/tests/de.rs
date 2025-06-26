#![cfg(feature = "serde")]

use cjsonrs::cjson;
use cjsonrs::serde::from_cjson;
use serde::Deserialize;

#[test]
fn assert_that_structs_can_be_deserialized_from_cjson() -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Deserialize, Debug, PartialEq, Eq)]
    struct Object {
        hello: String,
        answer: i32,
    }

    let cjson = cjson!({
        c"hello" => c"world",
        c"answer" => 42,
    })?;

    let obj: Object = from_cjson(&cjson)?;
    assert_eq!(
        obj,
        Object {
            hello: "world".to_string(),
            answer: 42,
        }
    );
    Ok(())
}

#[test]
fn assert_that_arrays_can_be_deserialized_from_cjson() -> Result<(), Box<dyn std::error::Error>> {
    let cjson = cjson!([1, 2, 3])?;
    let v: Vec<i32> = from_cjson(&cjson)?;
    assert_eq!(v, vec![1, 2, 3]);
    Ok(())
}

#[test]
fn assert_that_tuples_can_be_deserialized_from_cjson() -> Result<(), Box<dyn std::error::Error>> {
    let cjson = cjson!([c"hello", 42])?;
    let tup: (String, i32) = from_cjson(&cjson)?;
    assert_eq!(tup, ("hello".to_string(), 42));
    Ok(())
}

#[test]
fn assert_that_newtype_structs_can_be_deserialized_from_cjson(
) -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Deserialize, Debug, PartialEq, Eq)]
    struct Object(String);

    let cjson = cjson!(c"hello")?;
    let obj: Object = from_cjson(&cjson)?;
    assert_eq!(obj, Object("hello".to_string()));
    Ok(())
}

#[test]
fn assert_that_tuple_structs_can_be_deserialized_from_cjson(
) -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Deserialize, Debug, PartialEq, Eq)]
    struct Object(String, i32);

    let cjson = cjson!([c"hello", 42])?;
    let obj: Object = from_cjson(&cjson)?;
    assert_eq!(obj, Object("hello".to_string(), 42));
    Ok(())
}

#[test]
fn assert_that_struct_variant_can_be_deserialized_from_cjson(
) -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Deserialize, Debug, PartialEq, Eq)]
    enum Object {
        Variant { hello: String, answer: i32 },
    }

    let cjson = cjson!({
        c"Variant" => {
            c"hello" => c"world",
            c"answer" => 42,
        }
    })?;

    let obj: Object = from_cjson(&cjson)?;
    assert_eq!(
        obj,
        Object::Variant {
            hello: "world".to_string(),
            answer: 42,
        }
    );
    Ok(())
}

#[test]
fn assert_that_unit_variant_can_be_deserialized_from_cjson(
) -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Deserialize, Debug, PartialEq, Eq)]
    enum Object {
        Variant,
    }

    let cjson = cjson!(c"Variant")?;
    let obj: Object = from_cjson(&cjson)?;
    assert_eq!(obj, Object::Variant);
    Ok(())
}

#[test]
fn assert_that_tuple_variant_can_be_deserialized_from_cjson(
) -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Deserialize, Debug, PartialEq, Eq)]
    enum Object {
        Variant(String, i32),
    }

    let cjson = cjson!({c"Variant" => [c"hello", 42]})?;
    let obj: Object = from_cjson(&cjson)?;
    assert_eq!(obj, Object::Variant("hello".to_string(), 42));
    Ok(())
}

#[test]
fn assert_that_unit_can_be_deserialized_from_cjson() -> Result<(), Box<dyn std::error::Error>> {
    #[derive(Deserialize, Debug, PartialEq, Eq)]
    struct Object;

    let cjson = cjson!(null)?;
    let obj: Object = from_cjson(&cjson)?;
    assert_eq!(obj, Object);
    Ok(())
}

#[test]
fn assert_that_strings_can_be_deserialized_from_cjson() -> Result<(), Box<dyn std::error::Error>> {
    let cjson = cjson!(c"hello")?;
    let s: String = from_cjson(&cjson)?;
    assert_eq!(s, "hello".to_string());
    Ok(())
}

#[test]
fn assert_that_numbers_can_be_deserialized_from_cjson() -> Result<(), Box<dyn std::error::Error>> {
    let cjson = cjson!(42)?;
    let n: i32 = from_cjson(&cjson)?;
    assert_eq!(n, 42);
    Ok(())
}

#[test]
fn assert_that_booleans_can_be_deserialized_from_cjson() -> Result<(), Box<dyn std::error::Error>> {
    let cjson = cjson!(true)?;
    let b: bool = from_cjson(&cjson)?;
    assert_eq!(b, true);
    Ok(())
}

#[test]
fn assert_that_option_some_can_be_deserialized_from_cjson() -> Result<(), Box<dyn std::error::Error>>
{
    let cjson = cjson!(42)?;
    let opt: Option<i32> = from_cjson(&cjson)?;
    assert_eq!(opt, Some(42));
    Ok(())
}

#[test]
fn assert_that_option_none_can_be_deserialized_from_cjson() -> Result<(), Box<dyn std::error::Error>>
{
    let cjson = cjson!(null)?;
    let opt: Option<i32> = from_cjson(&cjson)?;
    assert_eq!(opt, None);
    Ok(())
}
