//! Serde support for cjson.
//!
//! This module provides an implementation of the Serde serialization framework
//! for [CJson](crate::CJson) values. This allows you to construct CJson values
//! from Rust data structures and vice versa, as well as to serialize them to
//! and from other formats.
//!
//! # Example
//!
//! ```
//! # fn main() -> cjsonrs::serde::Result<()> {
//!
//! use serde::Serialize;
//! use cjsonrs::serde::to_cjson;
//!
//! #[derive(Serialize)]
//! struct Object {
//!    hello: String,
//!    answer: i32,
//! }
//!
//! let expected = cjsonrs::cjson!({
//!    c"hello" => c"world",
//!    c"answer" => 42,
//! })?.into();
//!
//! assert_eq!(
//!    to_cjson(&Object {
//!      hello: "world".to_string(),
//!      answer: 42,
//!    })?,
//!    expected
//! );
//!
//! # Ok(()) }
//! ```
//!

mod de;
mod error;
mod ser;

pub use de::from_cjson;
pub use error::{Error, Result};
pub use ser::{to_cjson, Serializer};
