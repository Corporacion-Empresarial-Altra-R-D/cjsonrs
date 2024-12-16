# `cjsonrs-sys`

Raw bindings to the [cJSON](https://github.com/DaveGamble/cJSON) library. For a
more user-friendly interface, see the `cjsonrs` crate.

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
cjsonrs-sys = "*"
```

or run the following command:

```sh
cargo add cjsonrs-sys
```

## Available features

Set the following features to enable the corresponding functionality:

- `vendored` - Statically builds the cJSON library and links it to the resulting
  binary.
- `std` - Enables the use of `std` types. Disabling this feature will make the
  crate `no_std` compatible. Note that `alloc` is still required.

## Supported environment variables

Set the following environment variables to customize how the library is built:

- `CJSON_INCLUDE_PATH` - path to cJSON header files (if unset it defaults to
  vendored headers)
- `CJSON_LIB_PATH` - path to cJSON library files (if unset it defaults to
  building from source)
- `CJSON_SRC_PATH` - path to cJSON source files (if unset it defaults to
  vendored sources)
