cfg_if::cfg_if! {
    if #[cfg(feature = "alloc")] {
        extern crate alloc;
        use alloc::borrow::ToOwned;
    }
}

use core::ffi::CStr;
use core::fmt::Debug;
use core::fmt::Display;
use core::marker::PhantomData;
use core::ptr::NonNull;

use super::CJson;
use super::CJsonArray;
use super::CJsonObject;
use super::CJsonString;
use super::Error;

/// A safe and borrowed wrapper around [`cjsonrs_sys::cJSON`].
///
/// This type represents a reference to a valid CJson value. For more
/// information, refer to the [module-level documentation](super).
#[repr(transparent)]
pub struct CJsonRef<'json>(cjsonrs_sys::cJSON, PhantomData<&'json ()>);

impl<'json> CJsonRef<'json> {
    /// Returns a pointer to the underlying [`cjsonrs_sys::cJSON`] object.
    #[inline(always)]
    pub fn as_ptr(&self) -> *const cjsonrs_sys::cJSON {
        self as *const CJsonRef as _
    }

    /// Returns a mutable pointer to the underlying [`cjsonrs_sys::cJSON`]
    /// object.
    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut cjsonrs_sys::cJSON {
        self as *mut CJsonRef as _
    }

    /// Wraps a raw pointer to a [`cjsonrs_sys::cJSON`] object in a safe
    /// reference.
    ///
    /// # Safety
    ///
    /// - `ptr` must be a valid pointer to a [`cjsonrs_sys::cJSON`] object.
    /// - The memory referenced by the returned CJsonRef must not be mutated for
    ///   the duration of the lifetime `'a`.
    #[inline(always)]
    pub unsafe fn from_ptr(ptr: *const cjsonrs_sys::cJSON) -> &'json Self {
        &*(ptr as *const Self)
    }

    /// Wraps a mutable raw pointer to a [`cjsonrs_sys::cJSON`] object in a
    /// safe mutable reference.
    ///
    /// # Safety
    ///
    /// - `ptr` must be a valid pointer to a [`cjsonrs_sys::cJSON`] object.
    /// - The memory referenced by the returned CJsonRef must not be accessed
    ///   for the duration of the lifetime `'a`.
    #[inline(always)]
    pub unsafe fn from_mut_ptr(ptr: *mut cjsonrs_sys::cJSON) -> &'json mut Self {
        &mut *(ptr as *mut Self)
    }

    /// Returns `true` if the underlying [`cjsonrs_sys::cJSON`] object is a
    /// null.
    #[inline(always)]
    pub fn is_null(&self) -> bool {
        let ptr = self.as_ptr();
        let b = unsafe { cjsonrs_sys::cJSON_IsNull(ptr) };
        b != 0
    }

    /// Returns `true` if the underlying [`cjsonrs_sys::cJSON`] object is a
    /// boolean.
    #[inline(always)]
    pub fn is_bool(&self) -> bool {
        let ptr = self.as_ptr();
        let b = unsafe { cjsonrs_sys::cJSON_IsBool(ptr) };
        b != 0
    }

    /// Returns `true` if the underlying [`cjsonrs_sys::cJSON`] object is a
    /// number.
    #[inline(always)]
    pub fn is_number(&self) -> bool {
        let ptr = self.as_ptr();
        let b = unsafe { cjsonrs_sys::cJSON_IsNumber(ptr) };
        b != 0
    }

    /// Returns `true` if the underlying [`cjsonrs_sys::cJSON`] object is a
    /// string.
    #[inline(always)]
    pub fn is_string(&self) -> bool {
        let ptr = self.as_ptr();
        let b = unsafe { cjsonrs_sys::cJSON_IsString(ptr) };
        b != 0
    }

    /// Returns `true` if the underlying [`cjsonrs_sys::cJSON`] object is an
    /// array.
    #[inline(always)]
    pub fn is_array(&self) -> bool {
        let ptr = self.as_ptr();
        let b = unsafe { cjsonrs_sys::cJSON_IsArray(ptr) };
        b != 0
    }

    /// Returns `true` if the underlying [`cjsonrs_sys::cJSON`] object is an
    /// object.
    #[inline(always)]
    pub fn is_object(&self) -> bool {
        let ptr = self.as_ptr();
        let b = unsafe { cjsonrs_sys::cJSON_IsObject(ptr) };
        b != 0
    }

    /// Returns the underlying [`cjsonrs_sys::cJSON`] object as a boolean, if
    /// possible.
    #[inline(always)]
    pub fn as_bool(&self) -> Option<bool> {
        if !self.is_bool() {
            return None;
        }
        let ptr = self.as_ptr();

        Some(unsafe { cjsonrs_sys::cJSON_IsTrue(ptr) != 0 })
    }

    /// Returns the underlying [`cjsonrs_sys::cJSON`] object as a number, if
    /// possible.
    #[inline(always)]
    pub fn as_number(&self) -> Option<f64> {
        if !self.is_number() {
            return None;
        }
        let ptr = self.as_ptr();

        Some(unsafe { cjsonrs_sys::cJSON_GetNumberValue(ptr) })
    }

    /// Returns the underlying [`cjsonrs_sys::cJSON`] object as a cstring, if
    /// possible.
    #[inline(always)]
    pub fn as_c_string(&self) -> Option<&'_ CStr> {
        if !self.is_string() {
            return None;
        }
        let ptr = self.as_ptr();

        unsafe {
            let ptr = cjsonrs_sys::cJSON_GetStringValue(ptr);
            Some(CStr::from_ptr(ptr))
        }
    }

    /// Returns the underlying [`cjsonrs_sys::cJSON`] object as an array, if
    /// possible.
    #[inline(always)]
    pub fn as_array(&self) -> Option<CJsonArray<&Self>> {
        if self.is_array() {
            unsafe { Some(CJsonArray::from_raw_parts(self)) }
        } else {
            None
        }
    }

    /// Returns the underlying [`cjsonrs_sys::cJSON`] object as a mutable
    /// array, if possible.
    #[inline(always)]
    pub fn as_mut_array(&mut self) -> Option<CJsonArray<&mut Self>> {
        if self.is_array() {
            unsafe { Some(CJsonArray::from_raw_parts(self)) }
        } else {
            None
        }
    }

    /// Returns the underlying [`cjsonrs_sys::cJSON`] object as an object, if
    /// possible.
    #[inline(always)]
    pub fn as_object(&self) -> Option<CJsonObject<&Self>> {
        if self.is_object() {
            unsafe { Some(CJsonObject::from_raw_parts(self)) }
        } else {
            None
        }
    }

    /// Returns the underlying [`cjsonrs_sys::cJSON`] object as a mutable
    /// object, if possible.
    #[inline(always)]
    pub fn as_mut_object(&mut self) -> Option<CJsonObject<&mut Self>> {
        if self.is_object() {
            unsafe { Some(CJsonObject::from_raw_parts(self)) }
        } else {
            None
        }
    }

    /// Serializes the underlying [`cjsonrs_sys::cJSON`] object into a JSON
    /// string.
    #[inline(always)]
    pub fn to_c_string(&self) -> Result<CJsonString, Error> {
        let ptr = self.as_ptr();

        let s = unsafe { cjsonrs_sys::cJSON_PrintUnformatted(ptr) };

        if let Some(ptr) = NonNull::new(s) {
            let s = unsafe { CStr::from_ptr(ptr.as_ptr()) };
            let len = s.to_bytes_with_nul().len();
            unsafe { Ok(CJsonString::from_raw_parts(ptr, len)) }
        } else {
            Err(Error::Allocation)
        }
    }

    /// Serializes the underlying [`cjsonrs_sys::cJSON`] object into a pretty
    /// JSON string.
    #[inline(always)]
    pub fn to_c_string_pretty(&self) -> Result<CJsonString, Error> {
        let ptr = self.as_ptr();

        let s = unsafe { cjsonrs_sys::cJSON_Print(ptr) };

        if let Some(ptr) = NonNull::new(s) {
            let s = unsafe { CStr::from_ptr(ptr.as_ptr()) };
            let len = s.to_bytes_with_nul().len();
            unsafe { Ok(CJsonString::from_raw_parts(ptr, len)) }
        } else {
            Err(Error::Allocation)
        }
    }

    /// Returns either the number of key value pairs in the object or the number
    /// of items in the array. If the object is not an array or an object, this
    /// function returns 0.
    #[inline(always)]
    pub(super) fn len(&self) -> i32 {
        let ptr = self.as_ptr();
        unsafe { cjsonrs_sys::cJSON_GetArraySize(ptr) }
    }

    /// Returns `true` if the length of the array or object is 0.
    ///
    /// See [`CJsonRef::len`] for more information.
    #[inline(always)]
    pub(super) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the key-value pairs of the object or the items
    /// of the array.
    #[inline(always)]
    pub(super) fn iter(&self) -> CJsonIter<'_, 'json> {
        let cjson = self.0.child;

        CJsonIter {
            cjson,
            _phantom: PhantomData,
        }
    }

    /// Returns the key associated to this value, if the [`CJsonRef`] is a value
    /// on an object.
    ///
    /// This is a weird quirk of the underlying C library. The representation
    /// of both arrays and objects is the same and it is based on linked lists.
    /// This forces every key and value pair to be stored within the same
    /// structure.
    #[inline(always)]
    pub(super) fn name(&self) -> Option<&CStr> {
        let ptr = self.0.string;
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CStr::from_ptr(ptr) })
        }
    }

    /// Duplicates the underlying [`cjsonrs_sys::cJSON`] object.
    #[inline(always)]
    pub fn duplicate(&self) -> Result<CJson<'json>, Error> {
        let ptr = self.as_ptr();
        unsafe {
            let ptr = cjsonrs_sys::cJSON_Duplicate(ptr, 1);
            if let Some(ptr) = NonNull::new(ptr) {
                Ok(CJson::from_raw_parts(ptr, PhantomData::<&'json ()>))
            } else {
                Err(Error::Allocation)
            }
        }
    }
}

impl PartialEq for CJsonRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        let ptr1 = self.as_ptr();
        let ptr2 = other.as_ptr();

        let b = unsafe { cjsonrs_sys::cJSON_Compare(ptr1, ptr2, 0) };
        b != 0
    }
}

impl Debug for CJsonRef<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Inspired on [serde_json::Value]
        if self.is_null() {
            f.write_str("Null")
        } else if let Some(b) = self.as_bool() {
            write!(f, "Boolean({b})")
        } else if let Some(s) = self.as_c_string() {
            write!(f, "String({s:?})")
        } else if let Some(n) = self.as_number() {
            write!(f, "Number({n})")
        } else if let Some(a) = self.as_array() {
            write!(f, "{a:?}")
        } else if let Some(o) = self.as_object() {
            write!(f, "{o:?}")
        } else {
            write!(f, "Unknown({:x})", self.0.type_)
        }
    }
}

impl Display for CJsonRef<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = if f.alternate() {
            self.to_c_string_pretty()
        } else {
            self.to_c_string()
        };

        let Ok(s) = s else {
            return Err(core::fmt::Error);
        };

        let Ok(s) = s.to_str() else {
            return Err(core::fmt::Error);
        };

        f.write_str(s)
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl<'json> ToOwned for CJsonRef<'json> {
    type Owned = CJson<'json>;

    fn to_owned(&self) -> Self::Owned {
        self.duplicate().expect("Failed to duplicate CJson")
    }
}

impl<'json> AsRef<CJsonRef<'json>> for CJsonRef<'json> {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<'json> AsMut<CJsonRef<'json>> for CJsonRef<'json> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<'a, 'json> IntoIterator for &'a CJsonRef<'json>
where
    'a: 'json,
{
    type IntoIter = CJsonIter<'a, 'json>;
    type Item = &'a CJsonRef<'json>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// SAFETY: See [crate level docs](::crate) for more information.
#[cfg(feature = "send")]
unsafe impl Send for CJsonRef<'_> {}
// SAFETY: See [crate level docs](::crate) for more information.
#[cfg(feature = "sync")]
unsafe impl Sync for CJsonRef<'_> {}

/// An iterator for [`CJsonRef`] objects and arrays.
pub struct CJsonIter<'r, 'json> {
    cjson: *const cjsonrs_sys::cJSON,
    _phantom: PhantomData<(&'r (), &'json ())>,
}

impl<'r, 'json> Iterator for CJsonIter<'r, 'json>
where
    'json: 'r,
{
    type Item = &'r CJsonRef<'json>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.cjson.is_null() {
            return (0, Some(0));
        }

        let cjson = unsafe { CJsonRef::from_ptr(self.cjson) };
        let len = cjson.len() as _;

        (len, Some(len))
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.cjson.is_null() {
            return None;
        }

        let result = Some(unsafe { CJsonRef::from_ptr(self.cjson) });
        self.cjson = unsafe { *self.cjson }.next;
        result
    }
}

// SAFETY: See [crate level docs](::crate) for more information.
#[cfg(feature = "send")]
unsafe impl Send for CJsonIter<'_, '_> {}
// SAFETY: See [crate level docs](::crate) for more information.
#[cfg(feature = "sync")]
unsafe impl Sync for CJsonIter<'_, '_> {}
