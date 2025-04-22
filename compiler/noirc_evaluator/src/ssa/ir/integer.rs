use acvm::{AcirField, FieldElement};

use super::{instruction::binary, types::NumericType};

/// A `Signed` or `Unsigned` value of a [Value::NumericConstant], converted to 128 bits.
///
/// This type can be used in loops and other instances where values have to be compared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntegerConstant {
    Signed(i128),
    Unsigned(u128),
}

impl IntegerConstant {
    pub(crate) fn from_numeric_constant(field: FieldElement, typ: NumericType) -> Option<Self> {
        match typ {
            NumericType::Signed { bit_size } => {
                binary::try_convert_field_element_to_signed_integer(field, bit_size)
                    .map(Self::Signed)
            }
            NumericType::Unsigned { .. } => Some(Self::Unsigned(field.to_u128())),
            NumericType::NativeField => None,
        }
    }

    /// Apply functions on two numeric types, or return `None` if they have different signedness.
    pub(crate) fn reduce<T>(
        self,
        other: Self,
        s: impl Fn(i128, i128) -> T,
        u: impl Fn(u128, u128) -> T,
    ) -> Option<T> {
        match (self, other) {
            (Self::Signed(a), Self::Signed(b)) => Some(s(a, b)),
            (Self::Unsigned(a), Self::Unsigned(b)) => Some(u(a, b)),
            _ => None,
        }
    }

    /// Increment the value by 1, saturating at the maximum value.
    pub(crate) fn inc(self) -> Self {
        match self {
            IntegerConstant::Signed(x) => Self::Signed(x.saturating_add(1)),
            IntegerConstant::Unsigned(x) => Self::Unsigned(x.saturating_add(1)),
        }
    }
}
