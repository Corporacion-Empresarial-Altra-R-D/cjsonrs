use core::ffi::c_void;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::AtomicIsize;
use core::sync::atomic::Ordering;
use std::panic::catch_unwind;
use std::panic::AssertUnwindSafe;

use cjsonrs::CJson;
use cjsonrs::CJsonObject;

extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

static FAIL: AtomicBool = AtomicBool::new(false);
static LIVE: AtomicIsize = AtomicIsize::new(0);

unsafe extern "C" fn counting_malloc(size: usize) -> *mut c_void {
    if FAIL.load(Ordering::SeqCst) {
        return core::ptr::null_mut();
    }
    let ptr = malloc(size);
    if !ptr.is_null() {
        LIVE.fetch_add(1, Ordering::SeqCst);
    }
    ptr
}

unsafe extern "C" fn counting_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        LIVE.fetch_sub(1, Ordering::SeqCst);
    }
    free(ptr)
}

#[test]
fn object_insert_survives_key_allocation_failure() {
    let mut hooks = cjsonrs_sys::cJSON_Hooks {
        malloc_fn: Some(counting_malloc),
        free_fn: Some(counting_free),
    };
    unsafe { cjsonrs_sys::cJSON_InitHooks(&mut hooks) };

    let mut object = CJsonObject::new().unwrap();
    object.insert(c"key", CJson::number(1).unwrap()).unwrap();
    let value = CJson::number(2).unwrap();

    FAIL.store(true, Ordering::SeqCst);
    let outcome = catch_unwind(AssertUnwindSafe(|| object.insert(c"key", value)));
    FAIL.store(false, Ordering::SeqCst);

    let panicked = outcome.is_err();
    let handed_back = outcome
        .ok()
        .and_then(Result::err)
        .and_then(|value| value.as_number());
    let stored = object.get(c"key").and_then(|value| value.as_number());
    drop(object);
    let leaked = LIVE.load(Ordering::SeqCst);

    assert_eq!(
        (panicked, handed_back, stored, leaked),
        (false, Some(2.0), Some(1.0), 0),
        "(panicked, value handed back, value stored under \"key\", leaked cJSON allocations)"
    );
}
