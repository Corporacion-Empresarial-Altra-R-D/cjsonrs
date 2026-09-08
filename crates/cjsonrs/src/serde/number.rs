//! Conversions between cJSON's `double` storage and Rust's integer types.
//!
//! cJSON keeps every number as a C `double`, so both directions of the Serde
//! integration have to decide when such a value stands for an exact integer.
//! Keeping that decision in one place is what keeps serialization and
//! deserialization symmetric.

/// Implements a checked `f64` to integer conversion.
///
/// The bounds come from casting the target type's own limits to `f64`:
///
/// - `MIN as f64` is exact for every type used here (a power of two, or zero),
///   so it is the inclusive lower bound.
/// - `MAX as f64` rounds *up* for these widths, because 2<sup>k</sup>-1 is not
///   representable, which makes it the exclusive upper bound.
///
/// That second property only holds from 64 bits up; do not instantiate this
/// for a narrower type, whose `MAX` would be exact and wrongly excluded.
macro_rules! integral {
    ($(
        $(#[$meta:meta])*
        $name:ident -> $ty:ty;
    )*) => {
        $(
            $(#[$meta])*
            ///
            /// Returns `None` unless `n` holds an exact integral value that
            /// fits the target type. `NaN` fails every comparison and is
            /// rejected, as are both infinities.
            #[inline]
            pub(super) fn $name(n: f64) -> Option<$ty> {
                // The range is checked before the cast because a saturating
                // cast back to `f64` would round MAX up to the exclusive bound
                // and wrongly accept it.
                if ((<$ty>::MIN as f64)..(<$ty>::MAX as f64)).contains(&n)
                    && (n as $ty) as f64 == n
                {
                    Some(n as $ty)
                } else {
                    None
                }
            }
        )*
    };
}

integral! {
    /// Returns `n` as an `i64`.
    as_i64 -> i64;
    /// Returns `n` as a `u64`.
    as_u64 -> u64;
    /// Returns `n` as an `i128`.
    as_i128 -> i128;
    /// Returns `n` as a `u128`.
    as_u128 -> u128;
}

/// Returns `true` if `n` stands for an exact integer.
///
/// Negative zero is deliberately excluded: it is integral, but reporting it as
/// `0` would drop the sign, and the sign is observable through
/// [`f64::is_sign_negative`].
#[inline]
pub(super) fn is_exact_integer(n: f64) -> bool {
    !(n == 0.0 && n.is_sign_negative()) && as_i128(n).is_some()
}
