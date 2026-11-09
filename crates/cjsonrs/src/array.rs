use core::ffi::c_int;
use core::fmt::Debug;
use core::fmt::Display;
use core::marker::PhantomData;
use core::ops::Index;
use core::ops::IndexMut;
use core::ptr::NonNull;

use super::CJson;
use super::CJsonRef;
use super::Error;

/// A guard that wraps a CJson-like value for array operations.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct CJsonArray<R> {
    inner: R,
}

impl CJsonArray<CJson<'_>> {
    /// Creates a new empty [`CJsonArray`].
    ///
    /// # Errors
    ///
    /// This function returns an error if the underlying [`CJson`] value cannot
    /// be created.
    #[inline(always)]
    pub fn new() -> Result<Self, Error> {
        let value = CJson::array()?;
        Ok(unsafe { CJsonArray::from_raw_parts(value) })
    }
}

impl<'json, R> CJsonArray<R>
where
    R: AsRef<CJsonRef<'json>>,
{
    /// Creates a new [`CJsonArray`] that wraps the given CJson-like type.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given CJson-like type is a valid array.
    #[inline(always)]
    pub unsafe fn from_raw_parts(inner: R) -> CJsonArray<R> {
        CJsonArray { inner }
    }

    /// Returns a reference to the value at the given index, if any.
    #[inline(always)]
    pub fn get(&self, index: impl TryInto<c_int>) -> Option<&CJsonRef<'json>> {
        let index = index.try_into().ok()?;
        let ptr = self.inner.as_ref().as_ptr();
        let cjson = unsafe { cjsonrs_sys::cJSON_GetArrayItem(ptr, index) };
        (!cjson.is_null()).then(|| unsafe { CJsonRef::from_ptr(cjson) })
    }

    /// Returns a iterator over the arrays elements.
    #[inline(always)]
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a CJsonRef<'json>> + 'a
    where
        'json: 'a,
    {
        self.inner.as_ref().iter()
    }

    /// Returns the size of the array.
    #[inline(always)]
    pub fn len(&self) -> i32 {
        self.inner.as_ref().len()
    }

    /// Returns `true` if the number of elements is 0.
    ///
    /// See [`CJsonArray::len`] for more information.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.inner.as_ref().is_empty()
    }
}

impl<'json, R> CJsonArray<R>
where
    R: AsMut<CJsonRef<'json>>,
{
    /// Returns a mutable reference to the value at the given index, if any.
    #[inline(always)]
    pub fn get_mut(&mut self, index: impl TryInto<c_int>) -> Option<&mut CJsonRef<'json>> {
        let index = index.try_into().ok()?;
        let ptr = self.inner.as_mut().as_mut_ptr();
        let cjson = unsafe { cjsonrs_sys::cJSON_GetArrayItem(ptr, index) };
        (!cjson.is_null()).then(|| unsafe { CJsonRef::from_mut_ptr(cjson) })
    }

    /// Removes the value at the given index.
    ///
    /// If the index is present, the associated value is removed and returned.
    ///
    /// If the index is not present, this function returns `None`.
    #[inline(always)]
    pub fn remove(&mut self, index: impl TryInto<c_int>) -> Option<CJson<'json>> {
        let index = index.try_into().ok()?;
        let ptr = self.inner.as_mut().as_mut_ptr();
        let cjson = unsafe { cjsonrs_sys::cJSON_DetachItemFromArray(ptr, index) };

        NonNull::new(cjson)
            .map(|ptr| unsafe { CJson::from_raw_parts(ptr, PhantomData::<&'json ()>) })
    }

    /// Appends a new value to the array.
    #[inline(always)]
    pub fn push<'a>(&mut self, value: impl Into<CJson<'a>>)
    where
        'a: 'json,
    {
        let value = value.into();
        let cjsonref = self.inner.as_mut();
        let ptr = cjsonref.as_mut_ptr();
        let return_code =
            unsafe { cjsonrs_sys::cJSON_AddItemToArray(ptr, value.into_raw_parts().as_ptr()) };

        // We met all preconditions, so this should never fail.
        assert_ne!(return_code, 0, "cJSON_AddItemToArray failed");
    }

    /// Inserts a new value at the given index, shifting all other values to the
    /// right. An index past the end appends the value.
    ///
    /// # Errors
    ///
    /// Returns the value back if the index is negative or does not fit in a
    /// [`c_int`].
    #[inline(always)]
    pub fn insert<'a>(
        &mut self,
        index: impl TryInto<c_int>,
        value: impl Into<CJson<'a>>,
    ) -> Result<(), CJson<'a>>
    where
        'a: 'json,
    {
        let value = value.into();
        let Some(index) = index.try_into().ok().filter(|index| *index >= 0) else {
            return Err(value);
        };
        let ptr = self.inner.as_mut().as_mut_ptr();
        let return_code = unsafe {
            cjsonrs_sys::cJSON_InsertItemInArray(ptr, index, value.into_raw_parts().as_ptr())
        };

        // We met all preconditions, so this should never fail.
        assert_ne!(return_code, 0, "cJSON_InsertItemInArray failed");
        Ok(())
    }
}

impl<'a, T: AsRef<CJsonRef<'a>>> PartialEq for CJsonArray<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner.as_ref() == other.inner.as_ref()
    }
}

impl<'json, T: AsRef<CJsonRef<'json>>> Debug for CJsonArray<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Array ")?;
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<'a, T: AsRef<CJsonRef<'a>>> Display for CJsonArray<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self.inner.as_ref(), f)
    }
}

impl<'json> TryFrom<CJson<'json>> for CJsonArray<CJson<'json>> {
    type Error = Error;

    fn try_from(value: CJson<'json>) -> Result<Self, Self::Error> {
        value.into_array().ok_or(Error)
    }
}

impl<'json, R> AsRef<CJsonRef<'json>> for CJsonArray<R>
where
    R: AsRef<CJsonRef<'json>>,
{
    fn as_ref(&self) -> &CJsonRef<'json> {
        self.inner.as_ref()
    }
}

impl<'json, R> AsMut<CJsonRef<'json>> for CJsonArray<R>
where
    R: AsMut<CJsonRef<'json>>,
{
    fn as_mut(&mut self) -> &mut CJsonRef<'json> {
        self.inner.as_mut()
    }
}

impl<'json> From<CJsonArray<CJson<'json>>> for CJson<'json> {
    fn from(value: CJsonArray<CJson<'json>>) -> Self {
        value.inner
    }
}

impl<'a, 'json> From<CJsonArray<&'a CJsonRef<'json>>> for &'a CJsonRef<'json> {
    fn from(value: CJsonArray<&'a CJsonRef<'json>>) -> Self {
        value.inner
    }
}
impl<'a, 'json> From<CJsonArray<&'a mut CJsonRef<'json>>> for &'a mut CJsonRef<'json> {
    fn from(value: CJsonArray<&'a mut CJsonRef<'json>>) -> Self {
        value.inner
    }
}

macro_rules! index_array {
    ($index_type:ty, $reference_type:ty, $get_fn:ident) => {
        impl<'json> Index<$index_type> for CJsonArray<$reference_type> {
            type Output = CJsonRef<'json>;

            fn index(&self, index: $index_type) -> &Self::Output {
                self.$get_fn(index)
                    .unwrap_or_else(|| panic!("Failed to index CJsonRef"))
            }
        }
    };
}

macro_rules! index_mut_array {
    ($index_type:ty, $reference_type:ty, $get_mut_fn:ident) => {
        impl<'json> IndexMut<$index_type> for CJsonArray<$reference_type> {
            fn index_mut(&mut self, index: $index_type) -> &mut Self::Output {
                self.$get_mut_fn(index)
                    .unwrap_or_else(|| panic!("Failed to index CJsonRef"))
            }
        }
    };
}

index_array!(u32, CJson<'json>, get);
index_array!(u32, &CJsonRef<'json>, get);
index_array!(u32, &mut CJsonRef<'json>, get);
index_mut_array!(u32, CJson<'json>, get_mut);
index_mut_array!(u32, &mut CJsonRef<'json>, get_mut);
