/// Create a new `Self` value from the given `T` value
///
/// The standard library implements `From` only for conversions that can't lose information:
/// widening an integer to a larger type of the same signedness, an unsigned integer to [`Field`],
/// and `bool` to an integer or `Field`. For conversions that can truncate, use `as` or
/// [`AsPrimitive`]. Implementing `From` also provides [`Into`] in the other direction.
///
/// Example:
///
/// ```noir
/// struct Meters {
///     value: u32,
/// }
///
/// impl From<u32> for Meters {
///     fn from(value: u32) -> Meters {
///         Meters { value }
///     }
/// }
///
/// fn main() {
///     let m = Meters::from(5);
///     assert_eq(m.value, 5);
///
///     let wide = u64::from(7_u8);
///     assert_eq(wide, 7);
/// }
/// ```
// docs:start:from-trait
pub trait From<T> {
    /// Converts `input` to `Self`.
    fn from(input: T) -> Self;
}
// docs:end:from-trait

impl<T> From<T> for T {
    fn from(input: T) -> T {
        input
    }
}

/// Convert a `Self` value into a `T` value
///
/// `Into` is implemented automatically for every [`From`] conversion, so implement [`From`] rather
/// than `Into`. The target type usually has to be given by a type annotation.
///
/// Example:
///
/// ```noir
/// let x: u64 = 7_u8.into();
/// assert_eq(x, 7);
/// ```
// docs:start:into-trait
pub trait Into<T> {
    /// Converts `self` to `T`.
    fn into(self) -> T;
}

impl<T, U> Into<T> for U
where
    T: From<U>,
{
    fn into(self) -> T {
        T::from(self)
    }
}
// docs:end:into-trait

// docs:start:from-impls
// Unsigned integers

impl From<u8> for u16 {
    fn from(value: u8) -> u16 {
        value as u16
    }
}

impl From<u8> for u32 {
    fn from(value: u8) -> u32 {
        value as u32
    }
}

impl From<u16> for u32 {
    fn from(value: u16) -> u32 {
        value as u32
    }
}

impl From<u8> for u64 {
    fn from(value: u8) -> u64 {
        value as u64
    }
}

impl From<u16> for u64 {
    fn from(value: u16) -> u64 {
        value as u64
    }
}

impl From<u32> for u64 {
    fn from(value: u32) -> u64 {
        value as u64
    }
}

impl From<u8> for u128 {
    fn from(value: u8) -> u128 {
        value as u128
    }
}

impl From<u16> for u128 {
    fn from(value: u16) -> u128 {
        value as u128
    }
}

impl From<u32> for u128 {
    fn from(value: u32) -> u128 {
        value as u128
    }
}
impl From<u64> for u128 {
    fn from(value: u64) -> u128 {
        value as u128
    }
}

impl From<u8> for Field {
    fn from(value: u8) -> Field {
        value as Field
    }
}

impl From<u16> for Field {
    fn from(value: u16) -> Field {
        value as Field
    }
}

impl From<u32> for Field {
    fn from(value: u32) -> Field {
        value as Field
    }
}
impl From<u64> for Field {
    fn from(value: u64) -> Field {
        value as Field
    }
}

impl From<u128> for Field {
    fn from(value: u128) -> Field {
        value as Field
    }
}

// Signed integers

impl From<i8> for i16 {
    fn from(value: i8) -> i16 {
        value as i16
    }
}

impl From<i8> for i32 {
    fn from(value: i8) -> i32 {
        value as i32
    }
}

impl From<i16> for i32 {
    fn from(value: i16) -> i32 {
        value as i32
    }
}

impl From<i8> for i64 {
    fn from(value: i8) -> i64 {
        value as i64
    }
}

impl From<i16> for i64 {
    fn from(value: i16) -> i64 {
        value as i64
    }
}

impl From<i32> for i64 {
    fn from(value: i32) -> i64 {
        value as i64
    }
}

// Booleans
impl From<bool> for u8 {
    fn from(value: bool) -> u8 {
        value as u8
    }
}
impl From<bool> for u16 {
    fn from(value: bool) -> u16 {
        value as u16
    }
}
impl From<bool> for u32 {
    fn from(value: bool) -> u32 {
        value as u32
    }
}
impl From<bool> for u64 {
    fn from(value: bool) -> u64 {
        value as u64
    }
}
impl From<bool> for u128 {
    fn from(value: bool) -> u128 {
        value as u128
    }
}
impl From<bool> for i8 {
    fn from(value: bool) -> i8 {
        value as i8
    }
}
impl From<bool> for i16 {
    fn from(value: bool) -> i16 {
        value as i16
    }
}
impl From<bool> for i32 {
    fn from(value: bool) -> i32 {
        value as i32
    }
}
impl From<bool> for i64 {
    fn from(value: bool) -> i64 {
        value as i64
    }
}
impl From<bool> for Field {
    fn from(value: bool) -> Field {
        value as Field
    }
}
// docs:end:from-impls

/// A generic interface for casting between primitive types, so generic code can cast with
/// `T: AsPrimitive<U>`.
///
/// It is implemented between every pair of `bool` and integer types, and between [`Field`] and
/// `bool` or the unsigned integer types. There is no implementation between `Field` and the
/// signed integer types.
///
/// Casting to a number behaves like `as`, and never fails:
///
/// - Casting to a smaller integer type keeps the low bits, so `300 as u8` is `44`.
/// - Casting between signed and unsigned types of the same size reinterprets the bits, so
///   `-1_i8 as u8` is `255`.
/// - Casting a `Field` to an integer type keeps the low bits of its integer value in `0..p`.
///
/// Casting to `bool` is `self != 0`; `as` doesn't allow casting a number to `bool`. Use [`From`]
/// for conversions that can't lose information.
///
/// Example:
///
/// ```noir
/// use std::convert::AsPrimitive;
///
/// let x: Field = 1234567890;
/// let y: u8 = x as u8;
/// let z: u8 = x.as_();
/// assert_eq(y, z);
///
/// let nonzero: bool = 5_u8.as_();
/// assert(nonzero);
/// ```
pub trait AsPrimitive<T> {
    /// Casts `self` to `T`.
    fn as_(self) -> T;
}

#[generate_as_primitive_impls]
comptime fn generate_as_primitive_impls(_: FunctionDefinition) -> Quoted {
    let types = [
        quote { bool },
        quote { u8 },
        quote { u16 },
        quote { u32 },
        quote { u64 },
        quote { u128 },
        quote { i8 },
        quote { i16 },
        quote { i32 },
        quote { i64 },
    ];

    let mut impls = [].as_vector();
    for type1 in types {
        for type2 in types {
            let body = if type1 == type2 {
                quote { self }
            } else if type1 == quote { bool } {
                quote { self != 0 }
            } else {
                quote { self as $type1 }
            };

            impls = impls.push_back(
                quote {
                    impl AsPrimitive<$type1> for $type2 {
                        fn as_(self) -> $type1 {
                            $body
                        }
                    }
                },
            );
        }
    }

    let u_types =
        [quote { bool }, quote { u8 }, quote { u16 }, quote { u32 }, quote { u64 }, quote { u128 }];

    for type2 in u_types {
        let body = quote { self as Field };

        impls = impls.push_back(
            quote {
                impl AsPrimitive<Field> for $type2 {
                    fn as_(self) -> Field {
                        $body
                    }
                }
            },
        );
    }

    for type1 in u_types {
        let body = if type1 == quote { bool } {
            quote { self != 0 }
        } else {
            quote { self as $type1 }
        };

        impls = impls.push_back(
            quote {
                impl AsPrimitive<$type1> for Field {
                    fn as_(self) -> $type1 {
                        $body
                    }
                }
            },
        );
    }

    impls.join(quote {})
}

mod tests {
    use crate::meta::unquote;
    use super::{From, Into};

    #[test]
    fn from_identity() {
        let x: u32 = 42;
        assert_eq(u32::from(x), 42);
    }

    #[test]
    fn from_unsigned_widening() {
        comptime {
            let pairs = @[
                (quote { u8 }, quote { u16 }),
                (quote { u8 }, quote { u32 }),
                (quote { u8 }, quote { u64 }),
                (quote { u8 }, quote { u128 }),
                (quote { u16 }, quote { u32 }),
                (quote { u16 }, quote { u64 }),
                (quote { u16 }, quote { u128 }),
                (quote { u32 }, quote { u64 }),
                (quote { u32 }, quote { u128 }),
                (quote { u64 }, quote { u128 }),
            ];
            for pair in pairs {
                let source = pair.0;
                let target = pair.1;
                let src_zero = f"0_{source}".quoted_contents();
                let src_value = f"123_{source}".quoted_contents();
                let dst_zero = f"0_{target}".quoted_contents();
                let dst_value = f"123_{target}".quoted_contents();
                unquote!(
                    quote {
                        assert_eq($target::from($src_zero), $dst_zero);
                        assert_eq($target::from($src_value), $dst_value);
                    },
                );
            }
        }
    }

    #[test]
    fn from_unsigned_to_field() {
        comptime {
            for source in @[
                quote { u8 },
                quote { u16 },
                quote { u32 },
                quote { u64 },
                quote { u128 },
            ] {
                let src_zero = f"0_{source}".quoted_contents();
                let src_value = f"123_{source}".quoted_contents();
                unquote!(
                    quote {
                        assert_eq(Field::from($src_zero), 0);
                        assert_eq(Field::from($src_value), 123);
                    },
                );
            }
        }
    }

    #[test]
    fn from_signed_widening() {
        comptime {
            let pairs = @[
                (quote { i8 }, quote { i16 }),
                (quote { i8 }, quote { i32 }),
                (quote { i8 }, quote { i64 }),
                (quote { i16 }, quote { i32 }),
                (quote { i16 }, quote { i64 }),
                (quote { i32 }, quote { i64 }),
            ];
            for pair in pairs {
                let source = pair.0;
                let target = pair.1;
                let src_pos = f"42_{source}".quoted_contents();
                let dst_pos = f"42_{target}".quoted_contents();
                let src_neg = f"-42_{source}".quoted_contents();
                let dst_neg = f"-42_{target}".quoted_contents();
                unquote!(
                    quote {
                        assert_eq($target::from($src_pos), $dst_pos);
                        assert_eq($target::from($src_neg), $dst_neg);
                    },
                );
            }
        }
    }

    #[test]
    fn from_bool() {
        comptime {
            for target in @[
                quote { u8 },
                quote { u16 },
                quote { u32 },
                quote { u64 },
                quote { u128 },
                quote { i8 },
                quote { i16 },
                quote { i32 },
                quote { i64 },
                quote { Field },
            ] {
                let one = f"1_{target}".quoted_contents();
                let zero = f"0_{target}".quoted_contents();
                unquote!(
                    quote {
                        assert_eq($target::from(true), $one);
                        assert_eq($target::from(false), $zero);
                    },
                );
            }
        }
    }

    #[test]
    fn into_widens_unsigned() {
        let value: u8 = 7;
        let widened: u32 = value.into();
        assert_eq(widened, 7);
    }
}
