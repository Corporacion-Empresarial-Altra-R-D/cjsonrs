#[cfg(any(feature = "std", feature = "alloc"))]
use alloc::borrow::ToOwned;
use core::ffi::CStr;
use core::fmt::Debug;
use core::fmt::Display;
use core::fmt::Write as _;
use core::marker::PhantomData;
use core::marker::PhantomPinned;
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
pub struct CJsonRef<'json>(PhantomData<(*mut cjsonrs_sys::cJSON, PhantomPinned, &'json ())>);

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

    /// Obtains the raw flags used to represent metadata about the
    /// underlying [`cjsonrs_sys::cJSON`] object.
    ///
    /// The flags are stored in the `type` field of the
    /// [`cjsonrs_sys::cJSON`] object, which is a bitmask that
    /// contains information about the type of the object, whether it is a
    /// reference, and other metadata.
    pub fn flags(&self) -> i32 {
        unsafe { (*self.as_ptr()).type_ }
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
        self.is_bool()
            .then(|| unsafe { cjsonrs_sys::cJSON_IsTrue(self.as_ptr()) != 0 })
    }

    /// Returns the underlying [`cjsonrs_sys::cJSON`] object as a number, if
    /// possible.
    #[inline(always)]
    pub fn as_number(&self) -> Option<f64> {
        self.is_number()
            .then(|| unsafe { cjsonrs_sys::cJSON_GetNumberValue(self.as_ptr()) })
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
            // `cJSON_GetStringValue` hands back `valuestring` verbatim, which a
            // tree built by foreign C code may have left null.
            let ptr = NonNull::new(cjsonrs_sys::cJSON_GetStringValue(ptr))?;
            Some(CStr::from_ptr(ptr.as_ptr()))
        }
    }

    /// Returns the underlying [`cjsonrs_sys::cJSON`] object as an array, if
    /// possible.
    #[inline(always)]
    pub fn as_array(&self) -> Option<CJsonArray<&Self>> {
        self.is_array()
            .then(|| unsafe { CJsonArray::from_raw_parts(self) })
    }

    /// Returns the underlying [`cjsonrs_sys::cJSON`] object as a mutable
    /// array, if possible.
    #[inline(always)]
    pub fn as_mut_array(&mut self) -> Option<CJsonArray<&mut Self>> {
        self.is_array()
            .then(|| unsafe { CJsonArray::from_raw_parts(self) })
    }

    /// Returns the underlying [`cjsonrs_sys::cJSON`] object as an object, if
    /// possible.
    #[inline(always)]
    pub fn as_object(&self) -> Option<CJsonObject<&Self>> {
        self.is_object()
            .then(|| unsafe { CJsonObject::from_raw_parts(self) })
    }

    /// Returns the underlying [`cjsonrs_sys::cJSON`] object as a mutable
    /// object, if possible.
    #[inline(always)]
    pub fn as_mut_object(&mut self) -> Option<CJsonObject<&mut Self>> {
        self.is_object()
            .then(|| unsafe { CJsonObject::from_raw_parts(self) })
    }

    /// Serializes the underlying [`cjsonrs_sys::cJSON`] object into a JSON
    /// string.
    #[inline(always)]
    pub fn to_c_string(&self) -> Result<CJsonString, Error> {
        self.print(cjsonrs_sys::cJSON_PrintUnformatted)
    }

    /// Serializes the underlying [`cjsonrs_sys::cJSON`] object into a pretty
    /// JSON string.
    #[inline(always)]
    pub fn to_c_string_pretty(&self) -> Result<CJsonString, Error> {
        self.print(cjsonrs_sys::cJSON_Print)
    }

    /// Renders this value with one of cJSON's printers.
    #[inline(always)]
    fn print(
        &self,
        printer: unsafe extern "C" fn(*const cjsonrs_sys::cJSON) -> *mut core::ffi::c_char,
    ) -> Result<CJsonString, Error> {
        let ptr = NonNull::new(unsafe { printer(self.as_ptr()) }).ok_or(Error)?;
        let len = unsafe { CStr::from_ptr(ptr.as_ptr()) }
            .to_bytes_with_nul()
            .len();
        Ok(unsafe { CJsonString::from_raw_parts(ptr, len) })
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
        let cjson = unsafe { (*self.as_ptr()).child };
        let remaining = core::iter::successors(NonNull::new(cjson), |item| {
            NonNull::new(unsafe { item.as_ref() }.next)
        })
        .count();

        CJsonIter {
            cjson,
            remaining,
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
        NonNull::new(unsafe { (*self.as_ptr()).string })
            .map(|ptr| unsafe { CStr::from_ptr(ptr.as_ptr()) })
    }

    /// Duplicates the underlying [`cjsonrs_sys::cJSON`] object.
    #[inline(always)]
    pub fn duplicate(&self) -> Result<CJson<'json>, Error> {
        let ptr = unsafe { cjsonrs_sys::cJSON_Duplicate(self.as_ptr(), 1) };
        NonNull::new(ptr)
            .map(|ptr| unsafe { CJson::from_raw_parts(ptr, PhantomData::<&'json ()>) })
            .ok_or(Error)
    }
}

/// Compares two values with `cJSON_Compare`.
///
/// Object keys are compared case-sensitively, and objects with the same
/// entries in a different order are equal.
///
/// Numbers are equal when they differ by at most [`f64::EPSILON`] times the
/// larger magnitude. This makes `==` approximate and not transitive: `1.0`
/// equals `1.0 + EPSILON`, which equals `1.0 + 2.0 * EPSILON`, but `1.0` does
/// not equal `1.0 + 2.0 * EPSILON`. A `NaN` number is not equal to any other
/// value, including another `NaN`.
impl PartialEq for CJsonRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        let ptr1 = self.as_ptr();
        let ptr2 = other.as_ptr();

        let b = unsafe { cjsonrs_sys::cJSON_Compare(ptr1, ptr2, 1) };
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
            write!(f, "Unknown({:x})", self.flags())
        }
    }
}

/// Prints the value as JSON with cJSON's printer: compact by default, indented
/// with the alternate flag (`{:#}`).
///
/// Formatting fails with [`core::fmt::Error`] when cJSON cannot render the
/// value, for example when an allocation fails. In that case `to_string()`
/// panics; use [`CJsonRef::to_c_string`] to handle the failure instead. Bytes that are not valid UTF-8 are printed as U+FFFD.
impl Display for CJsonRef<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = if f.alternate() {
            self.to_c_string_pretty()
        } else {
            self.to_c_string()
        };
        let s = s.map_err(|_| core::fmt::Error)?;

        for chunk in s.to_bytes().utf8_chunks() {
            f.write_str(chunk.valid())?;
            if !chunk.invalid().is_empty() {
                f.write_char(char::REPLACEMENT_CHARACTER)?;
            }
        }
        Ok(())
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
    'json: 'a,
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
    remaining: usize,
    _phantom: PhantomData<(&'r (), &'json ())>,
}

impl<'r, 'json> Iterator for CJsonIter<'r, 'json>
where
    'json: 'r,
{
    type Item = &'r CJsonRef<'json>;

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.cjson.is_null() {
            return None;
        }

        let result = Some(unsafe { CJsonRef::from_ptr(self.cjson) });
        self.cjson = unsafe { *self.cjson }.next;
        self.remaining = self.remaining.saturating_sub(1);
        result
    }
}

// SAFETY: See [crate level docs](::crate) for more information.
#[cfg(feature = "send")]
unsafe impl Send for CJsonIter<'_, '_> {}
// SAFETY: See [crate level docs](::crate) for more information.
#[cfg(feature = "sync")]
unsafe impl Sync for CJsonIter<'_, '_> {}
