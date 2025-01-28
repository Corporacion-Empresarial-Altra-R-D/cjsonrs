cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        use std::ffi::CString;
    } else if #[cfg(feature = "alloc")] {
        extern crate alloc;
        use alloc::ffi::CString;
    }
}

use core::ffi::CStr;
use core::fmt::Debug;
use core::fmt::Display;
use core::marker::PhantomData;
use core::ops::Index;
use core::ops::IndexMut;
use core::ptr::NonNull;

use super::CJson;
use super::CJsonRef;
use super::Error;

/// A guard that wraps a CJson-like value for object operations.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct CJsonObject<R> {
    inner: R,
}

impl CJsonObject<CJson<'_>> {
    /// Creates a new empty [`CJsonObject`].
    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying [`CJson`] value cannot
    /// be created.
    #[inline(always)]
    pub fn new() -> Result<Self, Error> {
        let value = CJson::object()?;
        Ok(unsafe { CJsonObject::from_raw_parts(value) })
    }
}

impl<'json, R> CJsonObject<R>
where
    R: AsRef<CJsonRef<'json>>,
{
    /// Creates a new [`CJsonObject`] that wraps the given CJson-like type.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given CJson-like type is a valid object.
    #[inline(always)]
    pub unsafe fn from_raw_parts(inner: R) -> CJsonObject<R> {
        CJsonObject { inner }
    }

    /// Returns a reference to the value associated with the given key, if any.
    #[inline(always)]
    pub fn get(&self, key: impl AsRef<CStr>) -> Option<&CJsonRef<'json>> {
        let cjsonref = self.inner.as_ref();
        let ptr = cjsonref.as_ptr();
        let cjson =
            unsafe { cjsonrs_sys::cJSON_GetObjectItemCaseSensitive(ptr, key.as_ref().as_ptr()) };

        let result = if cjson.is_null() {
            None
        } else {
            Some(unsafe { CJsonRef::from_ptr(cjson) })
        };

        result
    }

    /// Returns a iterator over the key-value pairs of the object.
    /// The iterator yields the keys and values of the object in the order
    /// they are stored in the object.
    ///
    /// # Example usage
    ///
    /// ```
    /// use cjsonrs::cjson;
    ///
    /// let cjson = cjson!({
    ///    c"key" => c"value",
    ///    c"another_key" => 42,
    /// }).unwrap();
    ///
    /// for (key, value) in cjson.iter() {
    ///    println!("{key:?} => {value:?}");
    /// }
    /// ```
    #[inline(always)]
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a CStr, &'a CJsonRef<'json>)>
    where
        'json: 'a,
    {
        self.inner
            .as_ref()
            .iter()
            .map(|item| ({ item.name().unwrap() }, item))
    }

    /// Returns the number of key value pairs in the object.
    #[inline(always)]
    pub fn len(&self) -> i32 {
        self.inner.as_ref().len()
    }

    /// Returns `true` if the number of entries is 0.
    ///
    /// See [`CJsonObject::len`] for more information.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a iterator over keys of the object. The iterator yields the keys
    /// of the object in the order they are stored in the object.
    pub fn keys<'a>(&'a self) -> impl Iterator<Item = &'a CStr>
    where
        'json: 'a,
    {
        self.inner.as_ref().iter().map(|item| item.name().unwrap())
    }

    /// Returns a iterator over values of the object. The iterator yields the
    /// values of the object in the order they are stored in the object.
    pub fn values<'a>(&'a self) -> impl Iterator<Item = &'a CJsonRef<'json>>
    where
        'json: 'a,
    {
        self.inner.as_ref().iter()
    }
}

impl<'json, R> CJsonObject<R>
where
    R: AsMut<CJsonRef<'json>>,
{
    /// Returns a mutable reference to the value associated with the given key,
    /// if any.
    #[inline(always)]
    pub fn get_mut(&mut self, key: impl AsRef<CStr>) -> Option<&mut CJsonRef<'json>> {
        let cjsonref = self.inner.as_mut();
        let ptr = cjsonref.as_mut_ptr();
        let cjson =
            unsafe { cjsonrs_sys::cJSON_GetObjectItemCaseSensitive(ptr, key.as_ref().as_ptr()) };

        let result = if cjson.is_null() {
            None
        } else {
            Some(unsafe { CJsonRef::from_mut_ptr(cjson) })
        };

        result
    }

    /// Removes the value associated with the given key.
    ///
    /// If the key is present, the associated value is removed and returned.
    ///
    /// If the key is not present, this function returns `None`.
    #[inline(always)]
    pub fn remove(&mut self, key: impl AsRef<CStr>) -> Option<CJson<'json>> {
        let cjsonref = self.inner.as_mut();
        let ptr = cjsonref.as_mut_ptr();
        let key = key.as_ref();
        let detached =
            unsafe { cjsonrs_sys::cJSON_DetachItemFromObjectCaseSensitive(ptr, key.as_ptr()) };

        NonNull::new(detached)
            .map(|ptr| unsafe { CJson::from_raw_parts(ptr, PhantomData::<&'json ()>) })
    }

    /// Inserts a new key-value pair into the object.
    ///
    /// If the key is already present, the associated value is replaced and
    /// returned.
    ///
    /// If the key is not present, `None` is returned.
    #[inline(always)]
    pub fn insert<'a>(
        &mut self,
        key: impl AsRef<CStr>,
        value: impl Into<CJson<'a>>,
    ) -> Option<CJson<'json>>
    where
        'a: 'json,
    {
        let value = value.into();
        let cjsonref = self.inner.as_mut();
        let ptr = cjsonref.as_mut_ptr();
        let key = key.as_ref();
        let result = self.remove(key);

        let return_code = unsafe {
            cjsonrs_sys::cJSON_AddItemToObject(ptr, key.as_ptr(), value.into_raw_parts().as_ptr())
        };

        // We met all preconditions, so this should never fail.
        assert_ne!(return_code, 0, "cJSON_AddItemToObject returned an error");

        result
    }

    /// Inserts a new key-value pair into the object, but the key is stored as a
    /// reference.
    ///
    /// If the key is already present, the associated value is replaced and
    /// returned.
    ///
    /// If the key is not present, `None` is returned.
    #[inline(always)]
    pub fn insert_key_reference<'a>(
        &mut self,
        key: &'a CStr,
        value: impl Into<CJson<'a>>,
    ) -> Option<CJson<'json>>
    where
        'a: 'json,
    {
        let value = value.into();
        let cjsonref = self.inner.as_mut();
        let ptr = cjsonref.as_mut_ptr();
        let result = self.remove(key);

        let return_code = unsafe {
            cjsonrs_sys::cJSON_AddItemToObjectCS(ptr, key.as_ptr(), value.into_raw_parts().as_ptr())
        };

        // We met all preconditions, so this should never fail.
        assert_ne!(return_code, 0, "cJSON_AddItemToObjectCS returned an error");

        result
    }
}

impl<'json, T: AsRef<CJsonRef<'json>>> PartialEq for CJsonObject<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.as_ref() == other.inner.as_ref()
    }
}

impl<'json, T: AsRef<CJsonRef<'json>>> Debug for CJsonObject<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Object ")?;
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<'json, T: AsRef<CJsonRef<'json>>> Display for CJsonObject<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self.inner.as_ref(), f)
    }
}

impl<'json> TryFrom<CJson<'json>> for CJsonObject<CJson<'json>> {
    type Error = Error;

    fn try_from(value: CJson<'json>) -> Result<Self, Self::Error> {
        if value.is_object() {
            Ok(unsafe { CJsonObject::from_raw_parts(value) })
        } else {
            Err(Error::TypeError)
        }
    }
}

impl<'json> From<CJsonObject<CJson<'json>>> for CJson<'json> {
    fn from(value: CJsonObject<CJson<'json>>) -> Self {
        value.inner
    }
}

impl<'json, 'a> From<CJsonObject<&'a CJsonRef<'json>>> for &'a CJsonRef<'json> {
    fn from(value: CJsonObject<&'a CJsonRef<'json>>) -> Self {
        value.inner
    }
}

impl<'json, 'a> From<CJsonObject<&'a mut CJsonRef<'json>>> for &'a mut CJsonRef<'json> {
    fn from(value: CJsonObject<&'a mut CJsonRef<'json>>) -> Self {
        value.inner
    }
}
// macro_rules! index_object {
//     ($index_type:ty, $get_fn:ident, $get_mut_fn:ident) => {
//         impl<'json, T> Index<$index_type> for CJsonObject<T>
//         where
//             T: AsRef<CJsonRef<'json>>,
//         {
//             type Output = CJsonRef<'json>;
//
//             fn index(&self, index: $index_type) -> &Self::Output {
//                 self.$get_fn(index)
//                     .unwrap_or_else(|| panic!("Failed to index CJsonRef"))
//             }
//         }
//
//         impl<T> IndexMut<$index_type> for CJsonObject<T>
//         where
//             T: AsMut<CJsonRef> + AsRef<CJsonRef>,
//         {
//             fn index_mut(&mut self, index: $index_type) -> &mut Self::Output
// {                 self.$get_mut_fn(index)
//                     .unwrap_or_else(|| panic!("Failed to index CJsonRef"))
//             }
//         }
//     };
// }

macro_rules! index_object {
    ($index_type:ty, $reference_type:ty, $get_fn:ident) => {
        impl<'json> Index<$index_type> for CJsonObject<$reference_type> {
            type Output = CJsonRef<'json>;

            fn index(&self, index: $index_type) -> &Self::Output {
                self.$get_fn(index)
                    .unwrap_or_else(|| panic!("Failed to index CJsonRef"))
            }
        }
    };
}

macro_rules! index_mut_object {
    ($index_type:ty, $reference_type:ty, $get_mut_fn:ident) => {
        impl<'json> IndexMut<$index_type> for CJsonObject<$reference_type> {
            fn index_mut(&mut self, index: $index_type) -> &mut Self::Output {
                self.$get_mut_fn(index)
                    .unwrap_or_else(|| panic!("Failed to index CJsonRef"))
            }
        }
    };
}
#[cfg(any(feature = "alloc", feature = "std"))]
index_object!(CString, CJson<'json>, get);
#[cfg(any(feature = "alloc", feature = "std"))]
index_object!(CString, &CJsonRef<'json>, get);
#[cfg(any(feature = "alloc", feature = "std"))]
index_object!(CString, &mut CJsonRef<'json>, get);
#[cfg(any(feature = "alloc", feature = "std"))]
index_mut_object!(CString, CJson<'json>, get_mut);
#[cfg(any(feature = "alloc", feature = "std"))]
index_mut_object!(CString, &mut CJsonRef<'json>, get_mut);

index_object!(&CStr, CJson<'json>, get);
index_object!(&CStr, &CJsonRef<'json>, get);
index_object!(&CStr, &mut CJsonRef<'json>, get);
index_mut_object!(&CStr, CJson<'json>, get_mut);
index_mut_object!(&CStr, &mut CJsonRef<'json>, get_mut);
