use core::borrow::Borrow;
use core::ffi::c_char;
use core::ffi::CStr;
use core::fmt::Debug;
use core::fmt::Formatter;
use core::fmt::Result;
use core::hash::Hash;
use core::ops::Deref;
use core::ptr::NonNull;

/// An owned, null-terminated, C-style string.
///
/// This type is used to represent strings that are owned by the caller, but were allocated by the
/// cJSON library. Thus, the drop implementation of the [`CString`](std::ffi::CString) type is
/// unsuitable, as it would attempt to free the using the global allocator. Instead, this type
/// provides a custom drop implementation that calls the `cJSON_free` function to free the string.
///
///
/// For more information, refer to [`CString::from_raw`](std::ffi::CString::from_raw).
pub struct CJsonString {
    ptr: NonNull<c_char>,
    len: usize,
}

impl CJsonString {
    /// Constructs a new `CJsonString` from a raw pointer and length.
    ///
    /// # Safety
    ///
    /// The following invariants must be upheld when calling this function:
    ///
    /// - `ptr` must be a valid pointer to a null-terminated C-style string.
    /// - `ptr` must be allocated by the cJSON library. (i.e. it must be freed using [`cjsonrs_sys::cJSON_free`].)
    /// - `len` must be the length of the string, including the null terminator.
    ///
    pub unsafe fn from_raw_parts(ptr: NonNull<c_char>, len: usize) -> Self {
        Self { ptr, len }
    }
}

impl Drop for CJsonString {
    fn drop(&mut self) {
        unsafe {
            cjsonrs_sys::cJSON_free(self.ptr.as_ptr() as *mut core::ffi::c_void);
        }
    }
}

impl AsRef<CStr> for CJsonString {
    fn as_ref(&self) -> &CStr {
        self
    }
}

impl Deref for CJsonString {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        let slice =
            unsafe { core::slice::from_raw_parts(self.ptr.as_ptr() as *const u8, self.len) };
        unsafe { CStr::from_bytes_with_nul_unchecked(slice) }
    }
}

impl PartialEq for CJsonString {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl Eq for CJsonString {}

impl PartialOrd for CJsonString {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CJsonString {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        (**self).cmp(&**other)
    }
}

impl Debug for CJsonString {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Debug::fmt(&**self, f)
    }
}

impl Borrow<CStr> for CJsonString {
    fn borrow(&self) -> &CStr {
        self
    }
}

impl Hash for CJsonString {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        (**self).hash(state)
    }
}

// SAFETY: See [crate level docs](::crate) for more information.
#[cfg(feature = "send")]
unsafe impl Send for CJsonString {}
// SAFETY: See [crate level docs](::crate) for more information.
#[cfg(feature = "sync")]
unsafe impl Sync for CJsonString {}
