# cjsonrs

High level, `no_std` bindings for [cJSON](https://github.com/DaveGamble/cJSON),
an ultralightweight JSON parser in ANSI C.

## Overview

This crate provides a safe and user-friendly interface around the
[`cjsonrs_sys`] crate. It is intended to be used in codebases that interop with
C and cJSON, such as embedded systems or IoT. Because of the design limitations
of the underlying C library, these bindings rely heavily on [`std::ffi`] types,
such as [`CStr`] and [`CString`].

## Example usage

```rust
use cjsonrs::cjson;
use cjsonrs::CJson;

fn main() -> Result<(), cjsonrs::Error> {
  // Construct a CJson using the cjson! macro.
  let number = 42;
  let cjson: CJson = cjson!({
      // Literals will be automatically treated as constants
      c"key" => c"value",
      c"another_key" => number,
      // Arrays and objects are supported. Nulls are also supported.
      c"list" => [true, false, null, [c"nested", {c"key" => c"value"}]],
      // Expressions are supported (must be wrapped in curly braces)
      c"computed" => { 10 * 10 }
  })?.into();

  // Construct a CJson from parsing a JSON string.
  let parsed_string: CJson = r#"{
      "key": "value",
      "another_key": 42,
      "list": [true, false, null, ["nested", {"key": "value"}]],
      "computed": 100
  }"#.parse()?;

  // Traits such as `PartialEq`, `Debug` and `Display` are implemented for `CJson`.
  assert_eq!(cjson, parsed_string);
  Ok(())
}
```

## Features

Set the following features to enable the corresponding functionality:

- `default` - Enables the `vendored`, `send`, `sync` and `std` features.
- `vendored` - Statically builds the cJSON library and links it to the resulting
  binary. See [`cjsonrs_sys`] docs for more information.
- `send` - Implements `Send` for all types. Please note that this is not safe if
  the cJSON requirements are not met. See [`cjsonrs_sys`] docs for more
  information.
- `sync` - Implements `Sync` for all types. Please note that this is not safe if
  the cJSON requirements are not met. See [`cjsonrs_sys`] docs for more
  information.
- `std` - Enables the use of `std` types. Disabling this feature will make the
  crate `no_std` compatible. Note that `alloc` is still required.
- `serde` - Implements serialize and deserialize traits for all CJson types. It
  also enables the `serde` module. See the serde example for more information.

## FAQs

### Difference with [`cjson-rs`](https://github.com/nemuelw/cjson-rs)?

`cjsonrs` was developed internally before the public cjson-rs project was
established. It remained a private source code project throughout its
development, ensuring that it was rigorously tested and met our high-quality
standards. During this time, the project was exclusively used within our
organization to support our internal needs. We chose not to release it publicly
until it achieved the expected level of quality and reliability.

Additionally, `cjsonrs` provides additional features not present in `cjson-rs`:

- Type guards for objects and arrays: typechecks for object/array operations are
  only performed once.
- Lifetime support: cjson objects can be made to reference strings without
  re-allocations.
- `no_std` support
- Macro for creating cjson objects. It automatically converts literal strings to
  cJSON references and constants.
- Iterators
- `Send` and `Sync` support
- No assumptions: `Send`/`Sync` are feature-gated, allocations are not assumed
  to be made with Rust's allocator, indexing is not assumed to be `usize`, etc.
- More idiomatic Rust API, with many traits implemented
- Serde support

### Anything missing?

Some functions from the original cJSON library are not yet implemented.
Additionally, we plan on adding serde support, so you could serialize and
deserialize Rust structs into cJSON objects. If you need a specific function,
please open an issue or a pull request and we will be happy to help.

[`CStr`]: std::ffi::CStr
[`CString`]: std::ffi::CString
[`std::ffi`]: std::ffi
