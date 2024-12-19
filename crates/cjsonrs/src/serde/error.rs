#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{ffi::NulError, string::String};

#[cfg(feature = "std")]
use std::ffi::NulError;

/// Error type for serde operations
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("Keys in a JSON object must be strings")]
    KeyMustBeAString,
    #[error("Float value is not finite")]
    FloatNotFinite,
    #[error("Failed to construct cJSON object: {0}")]
    CJson(#[from] crate::Error),
    #[error("Failed to construct CString from UTF-8 string: {0}")]
    CString(#[from] NulError),
    #[error("{0}")]
    Custom(String),
}

pub type Result<T> = core::result::Result<T, Error>;
