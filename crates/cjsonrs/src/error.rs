use core::convert::Infallible;

/// An error type for [`CJson`]
///
/// [`CJson`]: super::CJson
#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Error variant for failing string parsing due to malformed JSON
    #[error("Failed to parse JSON string")]
    Parse,
    /// Error variant for allocation errors
    #[error("Failed to allocate enough memory for CJson")]
    Allocation,
    /// Error variant for type errors.
    #[error("Type error occurred when converting CJson to a different type")]
    TypeError,
}

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!("Infallible type should never be constructed")
    }
}
