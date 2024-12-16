#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::borrow::ToOwned;
#[cfg(not(feature = "std"))]
use alloc::ffi::CString;

#[cfg(feature = "std")]
use std::ffi::CString;

use core::borrow::Borrow;
use core::ffi::CStr;
use core::fmt::Debug;
use core::fmt::Display;
use core::marker::PhantomData;
use core::ops::Deref;
use core::ops::DerefMut;
use core::ptr::NonNull;
use core::str::FromStr;

use super::CJsonArray;
use super::CJsonObject;
use super::CJsonRef;
use super::Error;

/// A safe and owned wrapper around [`cjsonrs_sys::cJSON`].
///
/// This type provides a safe interface to the underlying C library. It is
/// designed to be as ergonomic as possible, while still providing a safe
/// interface to the underlying C library.
///
/// For more information, refer to the [module-level documentation](super).
#[repr(transparent)]
pub struct CJson<'json> {
    cjson: NonNull<cjsonrs_sys::cJSON>,
    _phantom: PhantomData<&'json ()>,
}

impl CJson<'_> {
    // Constructors

    /// Constructs a new [`CJson`] value representing a null value.
    ///
    /// # Errors
    ///
    /// This function returns an error if the allocation fails.
    #[inline(always)]
    pub fn null() -> Result<Self, Error> {
        let cjson = unsafe { cjsonrs_sys::cJSON_CreateNull() };

        if let Some(ptr) = NonNull::new(cjson) {
            Ok(unsafe { CJson::from_raw_parts(ptr, PhantomData::<&'_ ()>) })
        } else {
            Err(Error::Parse)
        }
    }

    /// Constructs a new [`CJson`] value representing a boolean value.
    ///
    /// # Errors
    ///
    /// This function returns an error if the allocation fails.
    #[inline(always)]
    pub fn bool(b: impl Into<bool>) -> Result<Self, Error> {
        let b = b.into();
        let cjson = unsafe { cjsonrs_sys::cJSON_CreateBool(b as _) };

        if let Some(ptr) = NonNull::new(cjson) {
            Ok(unsafe { CJson::from_raw_parts(ptr, PhantomData::<&'_ ()>) })
        } else {
            Err(Error::Parse)
        }
    }

    /// Constructs a new [`CJson`] value representing a number value.
    ///
    /// # Errors
    ///
    /// This function returns an error if the allocation fails.
    #[inline(always)]
    pub fn number(n: impl Into<f64>) -> Result<Self, Error> {
        let cjson = unsafe { cjsonrs_sys::cJSON_CreateNumber(n.into()) };
        if let Some(ptr) = NonNull::new(cjson) {
            Ok(unsafe { CJson::from_raw_parts(ptr, PhantomData::<&'_ ()>) })
        } else {
            Err(Error::Parse)
        }
    }

    /// Constructs a new [`CJson`] value representing a string value.
    ///
    /// # Errors
    ///
    /// This function returns an error if the allocation fails.
    #[inline(always)]
    pub fn string(s: impl AsRef<CStr>) -> Result<Self, Error> {
        let cjson = unsafe { cjsonrs_sys::cJSON_CreateString(s.as_ref().as_ptr()) };
        if let Some(ptr) = NonNull::new(cjson) {
            Ok(unsafe { CJson::from_raw_parts(ptr, PhantomData::<&'_ ()>) })
        } else {
            Err(Error::Parse)
        }
    }

    /// Constructs a new [`CJson`] value representing an array value.
    ///
    /// # Errors
    ///
    /// This function returns an error if the allocation fails.
    #[inline(always)]
    pub fn array() -> Result<Self, Error> {
        let cjson = unsafe { cjsonrs_sys::cJSON_CreateArray() };
        if let Some(ptr) = NonNull::new(cjson) {
            Ok(unsafe { CJson::from_raw_parts(ptr, PhantomData::<&'_ ()>) })
        } else {
            Err(Error::Parse)
        }
    }

    /// Constructs a new [`CJson`] value representing an object value.
    ///
    /// # Errors
    ///
    /// This function returns an error if the allocation fails.
    #[inline(always)]
    pub fn object() -> Result<Self, Error> {
        let cjson = unsafe { cjsonrs_sys::cJSON_CreateObject() };
        if let Some(ptr) = NonNull::new(cjson) {
            Ok(unsafe { CJson::from_raw_parts(ptr, PhantomData::<&'_ ()>) })
        } else {
            Err(Error::Parse)
        }
    }

    /// Parses a C string into a [`CJson`] value.
    ///
    /// # Errors
    ///
    /// This function returns an error if the input is not valid JSON or if
    /// allocation fails.
    #[inline(always)]
    pub fn from_c_str(s: impl AsRef<CStr>) -> Result<Self, Error> {
        let cjson = unsafe { cjsonrs_sys::cJSON_Parse(s.as_ref().as_ptr()) };

        if let Some(ptr) = NonNull::new(cjson) {
            Ok(unsafe { CJson::from_raw_parts(ptr, PhantomData::<&'_ ()>) })
        } else {
            Err(Error::Parse)
        }
    }

    /// Parses a byte slice into a [`CJson`] value.
    ///
    /// # Errors
    ///
    /// This function returns an error if the input is not valid JSON or if
    /// allocation fails.
    #[inline(always)]
    pub fn from_slice(s: &[u8]) -> Result<Self, Error> {
        let cjson = unsafe { cjsonrs_sys::cJSON_ParseWithLength(s.as_ptr() as _, s.len()) };

        if let Some(ptr) = NonNull::new(cjson) {
            Ok(unsafe { CJson::from_raw_parts(ptr, PhantomData::<&'_ ()>) })
        } else {
            Err(Error::Parse)
        }
    }
}

impl<'json> CJson<'json> {
    /// Constructs a new [`CJson`] value representing a string value, using a
    /// reference to a C string.
    ///
    /// # Errors
    ///
    /// This function returns an error if the allocation fails.
    #[inline(always)]
    pub fn string_reference(s: &'json CStr) -> Result<Self, Error> {
        let cjson = unsafe { cjsonrs_sys::cJSON_CreateStringReference(s.as_ptr()) };
        if let Some(ptr) = NonNull::new(cjson) {
            Ok(unsafe { CJson::from_raw_parts(ptr, PhantomData::<&'json ()>) })
        } else {
            Err(Error::Allocation)
        }
    }

    /// Returns the underlying [`cjsonrs_sys::cJSON`] object as an owned
    /// array, if possible.
    #[inline(always)]
    pub fn into_array(self) -> Option<CJsonArray<Self>> {
        if self.is_array() {
            unsafe { Some(CJsonArray::from_raw_parts(self)) }
        } else {
            None
        }
    }

    /// Returns the underlying [`cjsonrs_sys::cJSON`] object as an owned
    /// object, if possible.
    #[inline(always)]
    pub fn into_object(self) -> Option<CJsonObject<Self>> {
        if self.is_object() {
            unsafe { Some(CJsonObject::from_raw_parts(self)) }
        } else {
            None
        }
    }

    /// Constructs a [`CJson`] value from raw parts.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check if the input is a
    /// valid [`cjsonrs_sys::cJSON`] value. The caller must ensure that the
    /// input is valid.
    #[inline(always)]
    pub unsafe fn from_raw_parts<T>(cjson: NonNull<cjsonrs_sys::cJSON>, _: PhantomData<T>) -> Self
    where
        T: 'json,
    {
        Self {
            cjson,
            _phantom: PhantomData,
        }
    }

    /// Consumes the [`CJson`] value and returns the underlying raw parts. Note
    /// that the callee must ensure that the [`cjsonrs_sys::cJSON`] value is
    /// rightfully deallocated.
    ///
    /// # Safety
    ///
    /// This functions is not marked as `unsafe` as it does not violate Rust's
    /// memory safety rules. See [`core::mem::forget`] for more information.
    #[inline(always)]
    pub fn into_raw_parts(self) -> NonNull<cjsonrs_sys::cJSON> {
        let ptr = self.cjson;
        core::mem::forget(self);
        ptr
    }
}

impl Drop for CJson<'_> {
    fn drop(&mut self) {
        unsafe { cjsonrs_sys::cJSON_Delete(self.cjson.as_ptr()) };
    }
}

impl<'json> FromStr for CJson<'json> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_slice(s.as_bytes())
    }
}

impl PartialEq for CJson<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.deref().eq(other)
    }
}

impl<'json> AsRef<CJsonRef<'json>> for CJson<'json> {
    fn as_ref(&self) -> &CJsonRef<'json> {
        self
    }
}

impl<'json> AsMut<CJsonRef<'json>> for CJson<'json> {
    fn as_mut(&mut self) -> &mut CJsonRef<'json> {
        &mut *self
    }
}

impl<'json> Deref for CJson<'json> {
    type Target = CJsonRef<'json>;

    fn deref(&self) -> &Self::Target {
        unsafe { CJsonRef::from_ptr(self.cjson.as_ptr()) }
    }
}

impl<'json> DerefMut for CJson<'json> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { CJsonRef::from_mut_ptr(self.cjson.as_mut()) }
    }
}
impl Debug for CJson<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self.deref(), f)
    }
}

impl Display for CJson<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self.deref(), f)
    }
}

impl Clone for CJson<'_> {
    fn clone(&self) -> Self {
        self.deref().to_owned()
    }
}

impl<'json> Borrow<CJsonRef<'json>> for CJson<'json> {
    fn borrow(&self) -> &CJsonRef<'json> {
        self.deref()
    }
}

macro_rules! generate_try_from_impl {
    ($from:ty, $func:ident) => {
        impl<'json> core::convert::TryFrom<$from> for CJson<'json> {
            type Error = Error;

            fn try_from(value: $from) -> Result<Self, Self::Error> {
                CJson::$func(value)
            }
        }
    };
}

generate_try_from_impl!(bool, bool);
generate_try_from_impl!(&'json CStr, string_reference);
generate_try_from_impl!(CString, string);
generate_try_from_impl!(i8, number);
generate_try_from_impl!(i16, number);
generate_try_from_impl!(i32, number);
generate_try_from_impl!(u8, number);
generate_try_from_impl!(u16, number);
generate_try_from_impl!(u32, number);
generate_try_from_impl!(f32, number);
generate_try_from_impl!(f64, number);

impl<'json, T> TryFrom<Option<T>> for CJson<'json>
where
    T: TryInto<CJson<'json>, Error = Error>,
{
    type Error = Error;

    fn try_from(value: Option<T>) -> Result<Self, Self::Error> {
        if let Some(value) = value {
            value.try_into()
        } else {
            CJson::null()
        }
    }
}

// SAFETY: See [crate level docs](::crate) for more information.
#[cfg(feature = "send")]
unsafe impl Send for CJson<'_> {}
// SAFETY: See [crate level docs](::crate) for more information.
#[cfg(feature = "sync")]
unsafe impl Sync for CJson<'_> {}
