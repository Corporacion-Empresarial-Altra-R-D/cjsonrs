use std::ffi::CString;

use cjsonrs::cjson;
use cjsonrs::CJson;
use cjsonrs::CJsonRef;

#[test]
fn assert_string_reference_works_with_non_static_lifetimes(
) -> Result<(), Box<dyn std::error::Error>> {
    let cstring = CString::new("hello world")?;
    let cjson = CJson::string_reference(&cstring)?;
    assert_eq!(cjson.as_c_string(), Some(c"hello world"));
    Ok(())
}

#[test]
fn assert_that_cjson_outlives_parsed_content() -> Result<(), Box<dyn std::error::Error>> {
    let cjson = {
        let s = String::from(r#"{"hello": "world"}"#);
        s.parse::<CJson>()?
    };
    assert_eq!(&cjson, &cjson!({c"hello" => c"world"})?.into());
    Ok(())
}

#[test]
fn assert_that_macro_works() -> Result<(), Box<dyn std::error::Error>> {
    let variable = c"mix_object";
    let cjson1 = cjson!({
        c"nested_structures" => {
            c"empty_array" => [],
            c"empty_object" => {},
        },
        c"mix_array" => [
            1,
            true,
            null,
            { c"nested" => [] }
        ],
        variable => {
            c"1" => 1,
            c"2" => null,
            c"3" => true,
            c"4" => variable,
            c"5" => {c"hello world"}
        }
    })?;
    // let cjson1 = cjson!([true, false, {}, { c"hey" => 1, c"world" =>
    // 10 }]);
    let cjson2 = r#"
    {
        "nested_structures": {
            "empty_array": [],
            "empty_object": {}
        },
        "mix_array": [
            1, true, null, {"nested": []}
        ],
        "mix_object": {
            "1": 1,
            "2": null,
            "3": true,
            "4": "mix_object",
            "5": "hello world"
        }
    }
    "#
    .parse::<CJson>()?
    .try_into()?;
    assert_eq!(cjson1, cjson2);
    Ok(())
}

#[test]
fn assert_that_cjson_parse_works() -> Result<(), Box<dyn std::error::Error>> {
    let _cjson: CJson = r#"
    {
        "hello": "world",
        "nested": {
            "array": [1,2, "string"]
        }
    }
    "#
    .parse()?;

    Ok(())
}

// Type testing and upcasting
#[test]
fn assert_that_bool_and_as_bool_works() -> Result<(), Box<dyn std::error::Error>> {
    let b = CJson::bool(true)?;
    assert_eq!(b.as_bool(), Some(true));
    Ok(())
}

#[test]
fn assert_that_string_and_as_c_string_works() -> Result<(), Box<dyn std::error::Error>> {
    let b = CJson::string_reference(c"hello world")?;
    assert_eq!(b.as_c_string(), Some(c"hello world"));
    Ok(())
}

#[test]
fn assert_that_null_and_is_null_works() -> Result<(), Box<dyn std::error::Error>> {
    let b = CJson::null()?;
    assert!(b.is_null());
    Ok(())
}

#[test]
fn assert_that_number_and_as_number_works() -> Result<(), Box<dyn std::error::Error>> {
    let b = CJson::number(10)?;
    assert_eq!(b.as_number(), Some(10f64));
    Ok(())
}

#[test]
fn assert_that_array_and_is_array_works() -> Result<(), Box<dyn std::error::Error>> {
    let b = CJson::array()?;
    assert!(b.is_array());
    Ok(())
}

#[test]
fn assert_that_object_and_is_object_works() -> Result<(), Box<dyn std::error::Error>> {
    let b = CJson::object()?;
    assert!(b.is_object());
    Ok(())
}

// Indexing operations
#[test]
fn assert_that_objects_can_be_indexed() -> Result<(), Box<dyn std::error::Error>> {
    let cjson = cjson!({c"hello" => c"world"})?;
    let value = &cjson[c"hello"];

    assert_eq!(value.as_c_string(), Some(c"world"));
    Ok(())
}

#[test]
fn assert_that_arrays_can_be_indexed() -> Result<(), Box<dyn std::error::Error>> {
    let cjson = cjson!([1, 2, 3, 4, 5, 6, 7, 8, 9, 10])?;
    let value = &cjson[9];

    assert_eq!(value.as_number(), Some(10f64));
    Ok(())
}

// Iteration
#[test]
fn assert_that_arrays_can_be_iterated() -> Result<(), Box<dyn std::error::Error>> {
    let cjson = r#"[1,2,3,4,5,6]"#.parse::<CJson>()?.into_array().unwrap();
    let v: Vec<_> = cjson.iter().flat_map(CJsonRef::as_number).collect();

    assert_eq!(v, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    Ok(())
}

#[test]
fn assert_that_objects_can_be_iterated() -> Result<(), Box<dyn std::error::Error>> {
    let cjson = r#"{"hello": "world", "this": "is"}"#.parse::<CJson>()?.into_object().unwrap();
    let v: Vec<_> = cjson.iter().flat_map(|(_, v)| v.as_c_string()).collect();

    assert_eq!(v, vec![c"world", c"is"]);
    Ok(())
}

// Object mutation
#[test]
fn assert_that_items_can_be_inserted_to_objects() -> Result<(), Box<dyn std::error::Error>> {
    let mut cjson = cjson!({})?;
    let previous = cjson.insert(c"hello", CJson::null()?);

    assert_eq!(previous, None);
    assert!(cjson[c"hello"].is_null());
    Ok(())
}

#[test]
fn assert_that_items_can_be_inserted_to_objects_and_the_previous_value_is_returned(
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cjson = cjson!({c"hello"=>c"world"})?;
    let previous = cjson.insert(c"hello", CJson::null()?);

    assert_eq!(previous, Some(CJson::string_reference(c"world")?));
    assert!(cjson[c"hello"].is_null());
    Ok(())
}

#[test]
fn assert_that_inner_items_can_be_mutated() -> Result<(), Box<dyn std::error::Error>> {
    let mut cjson = cjson!({
        c"key" => {}
    })?;

    cjson[c"key"]
        .as_mut_object()
        .unwrap()
        .insert(c"hello", cjson!(c"value")?);

    assert_eq!(
        cjson,
        cjson!({
            c"key" => {
                c"hello" => {c"value"}
            }
        })?
    );
    Ok(())
}
