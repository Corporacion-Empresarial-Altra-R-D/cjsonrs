#![doc = "../README.md"]
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(clippy::std_instead_of_core)]

mod array;
mod cjson;
mod cjsonref;
mod error;
mod object;
mod string;
#[macro_use]
mod macros;
// #[cfg(feature = "serde")]
// pub mod serde;

// Re-export module contents
pub use array::*;
pub use cjson::*;
pub use cjsonref::*;
pub use error::*;
pub use object::*;
pub use string::*;
