use core::ffi::c_void;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;

use cjsonrs::CJson;
use cjsonrs::Error;

extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

static FAIL: AtomicBool = AtomicBool::new(false);

unsafe extern "C" fn failing_malloc(size: usize) -> *mut c_void {
    if FAIL.load(Ordering::SeqCst) {
        core::ptr::null_mut()
    } else {
        malloc(size)
    }
}

unsafe extern "C" fn passthrough_free(ptr: *mut c_void) {
    free(ptr)
}

#[test]
fn constructors_report_allocation_failures_as_allocation_errors() {
    let mut hooks = cjsonrs_sys::cJSON_Hooks {
        malloc_fn: Some(failing_malloc),
        free_fn: Some(passthrough_free),
    };
    unsafe { cjsonrs_sys::cJSON_InitHooks(&mut hooks) };

    FAIL.store(true, Ordering::SeqCst);
    let results = [
        CJson::null().err(),
        CJson::bool(true).err(),
        CJson::number(1).err(),
        CJson::string(c"s").err(),
        CJson::array().err(),
        CJson::object().err(),
        CJson::string_reference(c"s").err(),
    ];
    FAIL.store(false, Ordering::SeqCst);

    assert_eq!(results, [Some(Error); 7]);
}
