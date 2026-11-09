use core::convert::Infallible;

/// The error returned when a cJSON operation fails.
///
/// cJSON reports failures by returning a null pointer and never says why, so
/// this error carries no details. It is returned when parsing fails, when an
/// allocation fails, and when a value has the wrong type for a conversion.
#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[error("cJSON operation failed")]
pub struct Error;

impl From<Infallible> for Error {
    fn from(never: Infallible) -> Self {
        match never {}
    }
}
