use cjsonrs::cjson;

extern "C" {
    fn malloc(size: usize) -> *mut core::ffi::c_void;
    fn free(ptr: *mut core::ffi::c_void);
}

unsafe extern "C" fn malloc_fn(size: usize) -> *mut core::ffi::c_void {
    let ptr = malloc(size);
    println!("Called malloc! size={size} addr={ptr:?}");
    ptr
}

unsafe extern "C" fn free_fn(ptr: *mut core::ffi::c_void) {
    println!("Called free! addr={ptr:?}");
    free(ptr)
}

fn main() {
    let mut hooks = cjsonrs_sys::cJSON_Hooks {
        malloc_fn: Some(malloc_fn),
        free_fn: Some(free_fn),
    };
    println!("Initializing hooks");
    unsafe { cjsonrs_sys::cJSON_InitHooks(&mut hooks as _) };

    println!("Creating object");
    let hello_world = cjson!({
        c"hello" => c"world",
    })
    .expect("failed to construct cJSON");
    println!("{hello_world}");
}
