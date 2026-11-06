use core::borrow::Borrow;
use core::fmt::Write as _;
use core::marker::PhantomData;
use core::ptr::NonNull;
use std::collections::hash_map::DefaultHasher;
use std::ffi::CStr;
use std::ffi::CString;
use std::hash::Hash;
use std::hash::Hasher;

use cjsonrs::cjson;
use cjsonrs::CJson;
use cjsonrs::CJsonArray;
use cjsonrs::CJsonObject;
use cjsonrs::CJsonRef;
use cjsonrs::Error;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn raw_value() -> CJson<'static> {
    let ptr = unsafe { cjsonrs_sys::cJSON_CreateRaw(c"1".as_ptr()) };
    let ptr = NonNull::new(ptr).expect("cJSON_CreateRaw allocates");
    unsafe { CJson::from_raw_parts(ptr, PhantomData::<&()>) }
}

fn hash_of(value: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn display(value: &impl core::fmt::Display) -> Result<String, core::fmt::Error> {
    let mut out = String::new();
    write!(out, "{value}")?;
    Ok(out)
}

#[test]
fn array_reads_report_length_and_missing_indices() -> TestResult {
    let empty = CJsonArray::new()?;
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);
    assert_eq!(empty.get(0u32), None);

    let array = cjson!([1, 2])?;
    assert!(!array.is_empty());
    assert_eq!(array.len(), 2);
    assert_eq!(array.get(1u32).and_then(CJsonRef::as_number), Some(2.0));
    assert_eq!(array.get(2u32), None);
    Ok(())
}

#[test]
fn array_writes_insert_and_remove_items() -> TestResult {
    let mut array = cjson!([1, 3])?;

    array.insert(1u32, CJson::number(2)?);
    assert_eq!(array, cjson!([1, 2, 3])?);

    let removed = array.remove(0u32).ok_or("index 0 exists")?;
    assert_eq!(removed.as_number(), Some(1.0));
    assert_eq!(array.remove(5u32), None);
    assert_eq!(array, cjson!([2, 3])?);
    Ok(())
}

#[test]
fn array_items_can_be_mutated_in_place() -> TestResult {
    let mut array = cjson!([[], 1])?;

    let nested = array.get_mut(0u32).ok_or("index 0 exists")?;
    nested
        .as_mut_array()
        .ok_or("item is an array")?
        .push(CJson::bool(true)?);
    assert!(array.get_mut(9u32).is_none());

    assert!(array[1].as_mut_array().is_none());
    assert_eq!(array, cjson!([[true], 1])?);
    Ok(())
}

#[test]
fn arrays_index_through_every_wrapper() -> TestResult {
    let mut owned = cjson!([[1], 2])?;
    assert_eq!(owned[1].as_number(), Some(2.0));

    let mut nested: CJsonArray<&mut CJsonRef> = owned[0].as_mut_array().ok_or("array")?;
    assert!(nested[0].as_mut_array().is_none());
    assert_eq!(nested[0].as_number(), Some(1.0));
    let mutable: &mut CJsonRef = nested.into();
    assert!(mutable.is_array());

    let shared: CJsonArray<&CJsonRef> = owned[0].as_array().ok_or("array")?;
    assert_eq!(shared[0].as_number(), Some(1.0));
    let shared: &CJsonRef = shared.into();
    assert!(shared.is_array());

    let reference: &mut CJsonRef = owned.as_mut();
    assert!(reference.is_array());
    Ok(())
}

#[test]
#[should_panic(expected = "Failed to index CJsonRef")]
fn indexing_an_array_out_of_bounds_panics() {
    let array = cjson!([]).expect("array");
    let _ = &array[0];
}

#[test]
#[should_panic(expected = "Failed to index CJsonRef")]
fn mutably_indexing_an_array_out_of_bounds_panics() {
    let mut array = cjson!([]).expect("array");
    let _ = &mut array[0];
}

#[test]
fn arrays_format_and_convert() -> TestResult {
    let array = cjson!([1, null])?;
    assert_eq!(format!("{array:?}"), "Array [Number(1), Null]");
    assert_eq!(display(&array)?, "[1,null]");

    let value: CJson = array.into();
    assert!(CJsonArray::try_from(value).is_ok());
    assert_eq!(CJsonArray::try_from(CJson::null()?), Err(Error::TypeError));
    Ok(())
}

#[test]
fn objects_report_keys_values_and_length() -> TestResult {
    let empty = CJsonObject::new()?;
    assert!(empty.is_empty());

    let object = cjson!({ c"a" => 1, c"b" => true })?;
    assert!(!object.is_empty());
    assert_eq!(object.len(), 2);
    assert_eq!(object.keys().collect::<Vec<_>>(), [c"a", c"b"]);
    let values: Vec<_> = object.values().collect();
    assert_eq!(values[0].as_number(), Some(1.0));
    assert_eq!(values[1].as_bool(), Some(true));
    Ok(())
}

#[test]
fn objects_replace_existing_keys_on_insert() -> TestResult {
    let mut object = CJsonObject::new()?;
    let key = CString::new("owned")?;

    assert_eq!(object.insert(&key, CJson::number(1)?), None);
    let previous = object
        .insert(&key, CJson::number(2)?)
        .ok_or("key existed")?;
    assert_eq!(previous.as_number(), Some(1.0));

    assert_eq!(object.insert_key_reference(c"ref", CJson::null()?), None);
    let previous = object.insert_key_reference(c"ref", CJson::bool(false)?);
    assert!(previous.ok_or("key existed")?.is_null());

    assert_eq!(object, cjson!({ c"owned" => 2, c"ref" => false })?);
    Ok(())
}

#[test]
fn objects_index_through_every_wrapper() -> TestResult {
    let mut owned = cjson!({ c"inner" => { c"n" => 1 } })?;

    assert!(owned[c"inner"].is_object());
    assert!(owned[c"inner"].as_mut_object().is_some());

    let mut nested: CJsonObject<&mut CJsonRef> = owned[c"inner"].as_mut_object().ok_or("object")?;
    assert_eq!(nested[c"n"].as_number(), Some(1.0));
    assert!(nested[c"n"].as_mut_object().is_none());
    assert!(nested.get_mut(c"missing").is_none());
    let mutable: &mut CJsonRef = nested.into();
    assert!(mutable.is_object());

    let shared: CJsonObject<&CJsonRef> = owned[c"inner"].as_object().ok_or("object")?;
    assert_eq!(shared[c"n"].as_number(), Some(1.0));
    let shared: &CJsonRef = shared.into();
    assert!(shared.is_object());

    let reference: &mut CJsonRef = owned.as_mut();
    assert!(reference.is_object());
    Ok(())
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn objects_index_with_owned_c_strings() -> TestResult {
    let mut owned = cjson!({ c"inner" => { c"n" => 1 } })?;
    let key = CString::new("inner")?;
    let n = CString::new("n")?;

    assert!(owned[key.clone()].is_object());
    assert!(owned[key.clone()].as_mut_object().is_some());

    let mut nested = owned[key.clone()].as_mut_object().ok_or("object")?;
    assert_eq!(nested[n.clone()].as_number(), Some(1.0));
    assert!(nested[n.clone()].as_mut_object().is_none());

    let shared = owned[key].as_object().ok_or("object")?;
    assert_eq!(shared[n].as_number(), Some(1.0));
    Ok(())
}

#[test]
#[should_panic(expected = "Failed to index CJsonRef")]
fn indexing_an_object_with_a_missing_key_panics() {
    let object = cjson!({}).expect("object");
    let _ = &object[c"missing"];
}

#[test]
#[should_panic(expected = "Failed to index CJsonRef")]
fn mutably_indexing_an_object_with_a_missing_key_panics() {
    let mut object = cjson!({}).expect("object");
    let _ = &mut object[c"missing"];
}

#[test]
fn objects_format_and_convert() -> TestResult {
    let object = cjson!({ c"a" => 1 })?;
    assert_eq!(format!("{object:?}"), r#"Object {"a": Number(1)}"#);
    assert_eq!(display(&object)?, r#"{"a":1}"#);

    let value: CJson = object.into();
    assert!(CJsonObject::try_from(value).is_ok());
    assert_eq!(CJsonObject::try_from(CJson::null()?), Err(Error::TypeError));
    Ok(())
}

#[test]
fn values_can_be_parsed_from_c_strings() -> TestResult {
    let value = CJson::from_c_str(c"[1]")?;
    assert!(value.is_array());
    assert_eq!(CJson::from_c_str(c"[").err(), Some(Error::Parse));
    Ok(())
}

#[test]
fn values_convert_into_arrays_and_objects_only_when_they_match() -> TestResult {
    assert!(CJson::null()?.into_array().is_none());
    assert!(CJson::null()?.into_object().is_none());
    assert!(CJson::array()?.into_array().is_some());
    assert!(CJson::object()?.into_object().is_some());

    let mut value = CJson::null()?;
    assert!(value.as_mut_array().is_none());
    assert!(value.as_mut_object().is_none());
    Ok(())
}

#[test]
fn values_convert_from_rust_primitives() -> TestResult {
    assert_eq!(CJson::try_from(1i8)?.as_number(), Some(1.0));
    assert_eq!(CJson::try_from(1i16)?.as_number(), Some(1.0));
    assert_eq!(CJson::try_from(1u8)?.as_number(), Some(1.0));
    assert_eq!(CJson::try_from(1u16)?.as_number(), Some(1.0));
    assert_eq!(CJson::try_from(1u32)?.as_number(), Some(1.0));
    assert_eq!(CJson::try_from(1.5f32)?.as_number(), Some(1.5));
    assert_eq!(CJson::try_from(1.5f64)?.as_number(), Some(1.5));
    assert_eq!(CJson::try_from(Some(true))?.as_bool(), Some(true));
    assert!(CJson::try_from(None::<bool>)?.is_null());
    Ok(())
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn values_convert_from_owned_c_strings_and_into_owned_values() -> TestResult {
    let value = CJson::try_from(CString::new("s")?)?;
    assert_eq!(value.as_c_string(), Some(c"s"));

    let reference: &CJsonRef = &value;
    assert_eq!(reference.to_owned(), value);
    Ok(())
}

#[test]
fn values_clone_duplicate_and_borrow() -> TestResult {
    let value = cjson!({ c"a" => [1, 2] })?;
    let value: CJson = value.into();

    let cloned = value.clone();
    assert_eq!(cloned, value);
    assert_ne!(cloned.as_ptr(), value.as_ptr());

    let reference: &CJsonRef = value.borrow();
    assert_eq!(reference.duplicate()?, value);
    Ok(())
}

#[test]
fn flags_expose_the_raw_type_bitmask() -> TestResult {
    assert_eq!(CJson::null()?.flags(), cjsonrs_sys::cJSON_NULL as i32);
    assert_eq!(CJson::object()?.flags(), cjsonrs_sys::cJSON_Object as i32);

    let reference = CJson::string_reference(c"borrowed")?;
    let expected = (cjsonrs_sys::cJSON_String | cjsonrs_sys::cJSON_IsReference) as i32;
    assert_eq!(reference.flags(), expected);
    Ok(())
}

#[test]
fn values_debug_format_like_serde_json() -> TestResult {
    assert_eq!(format!("{:?}", CJson::null()?), "Null");
    assert_eq!(format!("{:?}", CJson::bool(true)?), "Boolean(true)");
    assert_eq!(format!("{:?}", CJson::string(c"s")?), r#"String("s")"#);
    assert_eq!(format!("{:?}", CJson::number(1.5)?), "Number(1.5)");
    assert_eq!(format!("{:?}", CJson::array()?), "Array []");
    assert_eq!(format!("{:?}", CJson::object()?), "Object {}");
    assert_eq!(format!("{:?}", raw_value()), "Unknown(80)");
    Ok(())
}

#[test]
fn values_display_compact_or_pretty() -> TestResult {
    let value: CJson = cjson!({ c"a" => [1] })?.into();
    assert_eq!(display(&value)?, r#"{"a":[1]}"#);
    assert_eq!(format!("{value:#}"), "{\n\t\"a\":\t[1]\n}");
    assert_eq!(value.to_c_string_pretty()?.to_str()?, "{\n\t\"a\":\t[1]\n}");
    Ok(())
}

#[test]
fn displaying_a_non_utf8_string_fails() -> TestResult {
    let value = CJson::string(c"\xff")?;
    assert_eq!(display(&value), Err(core::fmt::Error));
    Ok(())
}

#[test]
fn references_iterate_over_their_children() -> TestResult {
    let value: CJson = cjson!([1, 2, 3])?.into();
    let reference: &CJsonRef = &value;

    let mut total = 0.0;
    for item in reference {
        total += item.as_number().ok_or("number")?;
    }
    assert_eq!(total, 6.0);
    assert_eq!(reference.into_iter().size_hint(), (3, Some(3)));
    Ok(())
}

#[test]
fn printed_strings_compare_hash_and_format_as_c_strings() -> TestResult {
    let one = CJson::number(1)?.to_c_string()?;
    let also_one = CJson::number(1)?.to_c_string()?;
    let two = CJson::number(2)?.to_c_string()?;

    assert_eq!(one, also_one);
    assert_ne!(one, two);
    assert!(one < two);
    assert_eq!(one.cmp(&two), core::cmp::Ordering::Less);
    assert_eq!(hash_of(&one), hash_of(&also_one));
    assert_eq!(hash_of(&one), hash_of(&c"1"));
    assert_eq!(format!("{one:?}"), r#""1""#);

    let borrowed: &CStr = one.borrow();
    assert_eq!(borrowed, c"1");
    let as_ref: &CStr = one.as_ref();
    assert_eq!(as_ref, c"1");
    Ok(())
}
