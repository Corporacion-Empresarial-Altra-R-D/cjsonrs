//! Serde support for cjson.
//!
//! This module provides a [`Serializer`] and [`Deserializer`] implementation
//! for [CJson](crate::CJson) values. This allows you to construct CJson values
//! from Rust data structures and vice versa.
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

pub use de::{from_cjson, Deserializer};
pub use error::{Error, Result};
pub use ser::{to_cjson, Serializer};
