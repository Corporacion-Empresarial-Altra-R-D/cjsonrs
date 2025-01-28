use core::borrow::Borrow;
use core::ffi::c_char;
use core::ffi::CStr;
use core::fmt::Debug;
use core::fmt::Formatter;
use core::fmt::Result;
use core::hash::Hash;
use core::ops::Deref;
use core::ptr::NonNull;

pub struct CJsonString {
    ptr: NonNull<c_char>,
    len: usize,
}

impl CJsonString {
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
        let slice =
            unsafe { core::slice::from_raw_parts(self.ptr.as_ptr() as *const u8, self.len) };
        unsafe { CStr::from_bytes_with_nul_unchecked(slice) }
    }
}

impl Deref for CJsonString {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl PartialEq for CJsonString {
    fn eq(&self, other: &Self) -> bool {
        let s1: &CStr = self.as_ref();
        let s2: &CStr = other.as_ref();
        s1 == s2
    }
}

impl Eq for CJsonString {}

impl PartialOrd for CJsonString {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        let s1: &CStr = self.as_ref();
        let s2: &CStr = other.as_ref();
        s1.partial_cmp(s2)
    }
}

impl Ord for CJsonString {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let s1: &CStr = self.as_ref();
        let s2: &CStr = other.as_ref();
        s1.cmp(s2)
    }
}

impl Debug for CJsonString {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let s: &CStr = self.as_ref();
        Debug::fmt(s, f)
    }
}

impl Borrow<CStr> for CJsonString {
    fn borrow(&self) -> &CStr {
        self.as_ref()
    }
}

impl Hash for CJsonString {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        let s: &CStr = self.as_ref();
        s.hash(state)
    }
}
