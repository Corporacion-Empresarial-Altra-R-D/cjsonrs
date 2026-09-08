//! Serde support for cJSON.
//!
//! This module implements the [Serde](https://serde.rs) data model for
//! [`CJson`](crate::CJson) values, which lets you build a CJson tree from any
//! Rust type that implements [`Serialize`](serde::Serialize) and read one back
//! into any type that implements [`Deserialize`](serde::Deserialize).
//!
//! Use [`to_cjson`] to go from Rust to cJSON and [`from_cjson`] to come back.
//! [`from_cjson_borrowed`] is the same as [`from_cjson`], except the result may
//! borrow its strings straight out of the cJSON allocation instead of copying
//! them.
//!
//! Because [`CJson`](crate::CJson) itself implements both traits, this module
//! also bridges cJSON to any other Serde format: deserialize a
//! [`CJson`](crate::CJson) from a `serde_json` reader, or serialize one into a
//! `postcard` buffer.
//!
//! This module requires the `serde` feature, which in turn enables `alloc`: an
//! allocator is needed to build the C strings that cJSON stores.
//!
//! # Example
//!
//! ```
//! # fn main() -> cjsonrs::serde::Result<()> {
//! use cjsonrs::serde::{from_cjson, to_cjson};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize, Deserialize, Debug, PartialEq)]
//! struct Object {
//!     hello: String,
//!     answer: i32,
//! }
//!
//! let object = Object {
//!     hello: "world".to_string(),
//!     answer: 42,
//! };
//!
//! let expected = cjsonrs::cjson!({
//!     c"hello" => c"world",
//!     c"answer" => 42,
//! })?
//! .into();
//!
//! let cjson = to_cjson(&object)?;
//! assert_eq!(cjson, expected);
//! assert_eq!(from_cjson::<Object>(&cjson)?, object);
//!
//! # Ok(()) }
//! ```
//!
//! # How Rust types are represented
//!
//! The mapping follows `serde_json`, so a value written here and printed with
//! [`CJsonRef::to_c_string`](crate::CJsonRef::to_c_string) looks the way a JSON
//! reader would expect:
//!
//! | Rust                             | cJSON                              |
//! | -------------------------------- | ---------------------------------- |
//! | `()`, `None`, a unit struct      | `null`                             |
//! | `bool`                           | boolean                            |
//! | every integer and float type     | number                             |
//! | `char`, `String`, `&str`         | string                             |
//! | `&[u8]`                          | array of numbers                   |
//! | sequences, tuples, tuple structs | array                              |
//! | maps, structs                    | object                             |
//! | `E::Unit`                        | `"Unit"`                           |
//! | `E::Newtype(v)`                  | `{"Newtype": v}`                   |
//! | `E::Tuple(a, b)`                 | `{"Tuple": [a, b]}`                |
//! | `E::Struct { a }`                | `{"Struct": {"a": a}}`             |
//!
//! Map keys have to be strings in JSON, so a non-string key is formatted on the
//! way in and parsed back on the way out. A `HashMap<u32, _>` therefore round
//! trips. Accepted key types are strings, every integer and float type, `bool`,
//! `char`, unit variants, newtype structs wrapping any of those, and anything
//! that serializes through `collect_str` (`IpAddr`, `Uuid`, a
//! `#[serde(serialize_with)]` helper). Any other key type is rejected with
//! [`Error::KeyMustBeAString`].
//!
//! # Numbers
//!
//! cJSON stores every number as a C `double`, which has two consequences worth
//! knowing about:
//!
//! - Integers with a magnitude above 2<sup>53</sup> cannot be held exactly, so
//!   such a value is rounded as it is stored and will not come back as it went
//!   in. Deserializing into an integer type rejects a number that is
//!   fractional or out of that type's range, so reading a rounded value back
//!   either errors — `to_cjson(i64::MAX)` stores 2<sup>63</sup>, which is no
//!   longer an `i64` — or returns the rounded number, which for
//!   `i64::MIN + 1` is `i64::MIN`. Stay within ±2<sup>53</sup> if you need
//!   integers to survive a round trip.
//! - There is no way to tell the JSON number `1` from `1.0`. When the target
//!   type is self-describing — `#[serde(flatten)]`, `#[serde(untagged)]`,
//!   [`CJson`](crate::CJson) or `serde_json::Value` — a number holding an exact
//!   integral value is reported as an integer.
//!
//! A non-finite float (`NaN` or an infinity) is stored verbatim and survives a
//! round trip through the tree in memory. Printing it still renders `null`,
//! matching `serde_json`. As a map key it is rejected with
//! [`Error::FloatNotFinite`], since there is no sensible JSON key for it.

mod de;
mod error;
mod number;
mod ser;

pub use de::{from_cjson, from_cjson_borrowed};
pub use error::{Error, Result};
pub use ser::{
    to_cjson, MapKeySerializer, SerializeCJsonArray, SerializeCJsonObject, SerializeStructVariant,
    SerializeTupleVariant, Serializer,
};
