use cjsonrs::CJson;
use cjsonrs::CJsonRef;

#[test]
fn cjsonref_is_a_zero_sized_handle() {
    assert_eq!(core::mem::size_of::<CJsonRef>(), 0);
}

#[test]
fn swapping_nodes_across_trees_keeps_both_trees_well_formed() {
    let mut array: CJson = "[1,2,3]".parse().unwrap();
    let mut object: CJson = r#"{"k":"v","z":[4,5]}"#.parse().unwrap();

    core::mem::swap(
        array.as_mut_array().unwrap().get_mut(1).unwrap(),
        object.as_mut_object().unwrap().get_mut(c"k").unwrap(),
    );

    assert_eq!(array.to_string(), "[1,2,3]");
    assert_eq!(object.to_string(), r#"{"k":"v","z":[4,5]}"#);
}

#[test]
fn swapping_roots_keeps_both_trees_well_formed() {
    let mut array: CJson = "[1,2,3]".parse().unwrap();
    let mut object: CJson = r#"{"k":"v"}"#.parse().unwrap();

    core::mem::swap::<CJsonRef>(&mut array, &mut object);

    assert_eq!(array.to_string(), "[1,2,3]");
    assert_eq!(object.to_string(), r#"{"k":"v"}"#);
}

#[test]
fn a_tree_stays_usable_after_the_tree_it_was_swapped_with_is_dropped() {
    let mut array: CJson = "[1,2,3]".parse().unwrap();
    let mut object: CJson = r#"{"k":"v","z":[4,5]}"#.parse().unwrap();

    core::mem::swap(
        array.as_mut_array().unwrap().get_mut(1).unwrap(),
        object.as_mut_object().unwrap().get_mut(c"k").unwrap(),
    );
    drop(array);
    object
        .as_mut_object()
        .unwrap()
        .insert(c"new", CJson::null().unwrap())
        .unwrap();

    assert_eq!(object.to_string(), r#"{"k":"v","z":[4,5],"new":null}"#);
}
