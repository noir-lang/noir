use std::cmp::Ordering;

use acvm::AcirField;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use num_traits::Zero;

use super::{instruction::binary, types::NumericType};

/// A `Signed` or `Unsigned` value of a `Value::NumericConstant`, converted to 128 bits.
///
/// This type can be used in loops and other instances where values have to be compared,
/// with correct handling of negative values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntegerConstant {
    Signed { value: i128, bit_size: u32 },
    Unsigned { value: u128, bit_size: u32 },
}

impl IntegerConstant {
    pub(crate) fn from_numeric_constant(constant: BigInt, typ: NumericType) -> Option<Self> {
        match typ {
            NumericType::Signed { bit_size } => {
                binary::try_convert_bigint_to_signed_integer(constant, bit_size)
                    .map(|value| Self::Signed { value, bit_size })
            }
            NumericType::Unsigned { bit_size } => {
                constant.to_u128().map(|value| Self::Unsigned { value, bit_size })
            }
            NumericType::NativeField => None,
        }
    }

    /// Convert back into a field.
    pub(crate) fn into_numeric_constant(self) -> (BigInt, NumericType) {
        match self {
            Self::Signed { value, bit_size } => {
                (BigInt::from(value), NumericType::signed(bit_size))
            }
            Self::Unsigned { value, bit_size } => {
                (BigInt::from(value), NumericType::unsigned(bit_size))
            }
        }
    }

    /// Reduce two constants into a result by applying functions on them if their signedness matches.
    pub(crate) fn reduce<T>(
        self,
        other: Self,
        s: impl Fn(i128, i128) -> T,
        u: impl Fn(u128, u128) -> T,
    ) -> Option<T> {
        match (self, other) {
            (Self::Signed { value: a, .. }, Self::Signed { value: b, .. }) => Some(s(a, b)),
            (Self::Unsigned { value: a, .. }, Self::Unsigned { value: b, .. }) => Some(u(a, b)),
            _ => None,
        }
    }

    /// Apply functions on signed/unsigned values.
    pub(crate) fn apply<T>(&self, s: impl Fn(i128) -> T, u: impl Fn(u128) -> T) -> T {
        match self {
            Self::Signed { value, .. } => s(*value),
            Self::Unsigned { value, .. } => u(*value),
        }
    }

    /// Increment the value by 1, saturating at the maximum value.
    pub(crate) fn inc(self) -> Self {
        match self {
            Self::Signed { value, bit_size } => {
                Self::Signed { value: value.saturating_add(1), bit_size }
            }
            Self::Unsigned { value, bit_size } => {
                Self::Unsigned { value: value.saturating_add(1), bit_size }
            }
        }
    }

    /// Decrement the value by 1, saturating at the minimum value.
    pub(crate) fn dec(self) -> Self {
        match self {
            Self::Signed { value, bit_size } => {
                Self::Signed { value: value.saturating_sub(1), bit_size }
            }
            Self::Unsigned { value, bit_size } => {
                Self::Unsigned { value: value.saturating_sub(1), bit_size }
            }
        }
    }

    pub(crate) fn is_zero(&self) -> bool {
        match self {
            Self::Signed { value, .. } => value.is_zero(),
            Self::Unsigned { value, .. } => value.is_zero(),
        }
    }

    pub(crate) fn is_negative(&self) -> bool {
        match self {
            Self::Signed { value, .. } => value.is_negative(),
            Self::Unsigned { .. } => false,
        }
    }
}

impl PartialOrd for IntegerConstant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Signed { value: a, .. }, Self::Signed { value: b, .. }) => a.partial_cmp(b),
            (Self::Signed { value: a, .. }, Self::Unsigned { value: b, .. }) => {
                if a.is_negative() {
                    Some(Ordering::Less)
                } else {
                    (*a).try_into().ok().and_then(|a: u128| a.partial_cmp(b))
                }
            }
            (Self::Unsigned { value: a, .. }, Self::Signed { value: b, .. }) => {
                if b.is_negative() {
                    Some(Ordering::Greater)
                } else {
                    (*b).try_into().ok().and_then(|b: u128| a.partial_cmp(&b))
                }
            }
            (Self::Unsigned { value: a, .. }, Self::Unsigned { value: b, .. }) => a.partial_cmp(b),
        }
    }
}
