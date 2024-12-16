/// Internal implementation of the [cjson!] macro.
#[doc(hidden)]
#[macro_export]
macro_rules! _cjson_internal {
    // Arrays
    ([]) => {
        $crate::CJsonArray::new()?
    };
    ([ $( $tt:tt ),+ $(,)? ]) => {{
        let mut array = $crate::_cjson_internal!([]);
        $(
            array.push($crate::_cjson_internal!($tt));
        )*
        array
    }};

    // Objects
    ({}) => {
        $crate::CJsonObject::new()?
    };

    ({ $( $key:literal => $value:tt ),+ $(,)? }) => {{
        let mut obj = $crate::_cjson_internal!({});

        $(
            obj.insert_key_reference($key, $crate::_cjson_internal!($value));
        )*
        obj
    }};

    ({ $( $key:expr => $value:tt ),+ $(,)? }) => {{
        let mut obj = $crate::_cjson_internal!({});

        $(
            obj.insert($key, $crate::_cjson_internal!($value));
        )*
        obj
    }};

    // Null
    (null) => {
        $crate::CJson::null()?
    };
    // Anything else that can be transformed into CJson
    ($expresion:expr) => {
        $crate::CJson::try_from($expresion)?
    };
}

/// A helper macro for constructing CJson values.
///
/// The `cjson!` macro returns a `Result` with either the constructed object or
/// an [`Error`](super::Error), if any failure occurs.
///
/// # Usage
///
/// ```
/// use cjsonrs::cjson;
///
/// let cjson = cjson!({
///     c"name" => c"John",
///     c"age" => 30,
///     c"city" => c"New York"
/// }).unwrap();
/// # _ = cjson
/// ```
///
/// # Supported macro features
///
/// - **Null Values**: The `null` identifier represents null values within CJson
///   objects.
/// - **Arbitrary Expressions**: Arbitrary Rust expressions can be embedded
///   within any key or value item. This includes other macros, variables,
///   function calls, etc. While most expressions should work directly, wrapping
///   braces may be needed (`{}`). Object keys must implement `AsRef<&CStr>` and
///   values are converted into CJson using the `TryFrom` trait.
/// - **Arrays**: Arrays can be constructed using square brackets (`[]`). An
///   empty array is represented by `[]`, while arrays with elements are
///   specified within square brackets, separated by commas. Trailing commas are
///   supported.
/// - **Objects**: Objects are constructed using curly braces (`{}`). An empty
///   object is represented by `{}`, while objects with key-value pairs are
///   specified within curly braces, with each pair separated by commas and keys
///   and values separated by arrows (`=>`). Trailing commas are supported.
/// - **Error Handling**: The `cjson!` macro returns a `Result` with either the
///   constructed CJson object or an [`Error`](super::Error) if any failure
///   occurs during construction.
/// - **Strong typing**: The `cjson!` macro returns a strongly typed CJson
///   object or array to ease the use of the CJson API.
///
/// For more information, refer to the [module-level documentation](super).
#[macro_export]
macro_rules! cjson {
    ($($cjson:tt)+) => {
        (|| {
            Ok::<_,$crate::Error>($crate::_cjson_internal!($($cjson)+))
        })()
    };
}
