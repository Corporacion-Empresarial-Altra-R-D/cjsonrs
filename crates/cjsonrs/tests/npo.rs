use cjsonrs::CJson;
use cjsonrs::CJsonArray;
use cjsonrs::CJsonObject;
use cjsonrs::CJsonRef;

#[test]
fn assert_npo_for_cjson() {
    assert_eq!(
        core::mem::size_of::<Option<CJson>>(),
        core::mem::size_of::<CJson>(),
    )
}

#[test]
fn assert_npo_for_cjsonobject_of_cjson() {
    assert_eq!(
        core::mem::size_of::<Option<CJsonObject<CJson>>>(),
        core::mem::size_of::<CJsonObject<CJson>>(),
    )
}

#[test]
fn assert_npo_for_cjsonarray_of_cjson() {
    assert_eq!(
        core::mem::size_of::<Option<CJsonArray<CJson>>>(),
        core::mem::size_of::<CJsonArray<CJson>>(),
    )
}

#[test]
fn assert_npo_for_cjsonobject_of_cjsonref() {
    assert_eq!(
        core::mem::size_of::<Option<CJsonObject<&CJsonRef>>>(),
        core::mem::size_of::<CJsonObject<&CJsonRef>>(),
    )
}

#[test]
fn assert_npo_for_cjsonarray_of_cjsonref() {
    assert_eq!(
        core::mem::size_of::<Option<CJsonArray<&CJsonRef>>>(),
        core::mem::size_of::<CJsonArray<&CJsonRef>>(),
    )
}
