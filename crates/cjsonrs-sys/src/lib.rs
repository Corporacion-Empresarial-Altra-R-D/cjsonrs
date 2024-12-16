#![doc = include_str!("../README.md")]
//!
//! ---
//!
//! Original cJSON README:
//!
//! ---
//!
#![doc = include_str!("../cJSON/README.md")]
#![allow(rustdoc::bare_urls)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![cfg_attr(not(feature = "std"), no_std)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
