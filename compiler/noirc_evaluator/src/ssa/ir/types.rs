use serde::{Deserialize, Serialize};
use std::sync::Arc;

use acvm::{acir::AcirField, FieldElement};
use iter_extended::vecmap;

use crate::ssa::ssa_gen::SSA_WORD_SIZE;

/// A numeric type in the Intermediate representation
/// Note: we class NativeField as a numeric type
/// though we also apply limitations to it, such as not
/// being able to compare two native fields, whereas this is
/// something that you can do with a signed/unsigned integer.
///
/// Fields do not have a notion of ordering, so this distinction
/// is reasonable.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum NumericType {
    Signed { bit_size: u32 },
    Unsigned { bit_size: u32 },
    NativeField,
}

impl NumericType {
    /// Returns the bit size of the provided numeric type.
    pub(crate) fn bit_size(self: &NumericType) -> u32 {
        match self {
            NumericType::NativeField => FieldElement::max_num_bits(),
            NumericType::Unsigned { bit_size } | NumericType::Signed { bit_size } => *bit_size,
        }
    }

    /// Returns None if the given Field value is within the numeric limits
    /// for the current NumericType. Otherwise returns a string describing
    /// the limits, as a range.
    pub(crate) fn value_is_outside_limits(
        self,
        field: FieldElement,
        negative: bool,
    ) -> Option<String> {
        match self {
            NumericType::Unsigned { bit_size } => {
                let max = 2u128.pow(bit_size) - 1;
                if negative {
                    return Some(format!("0..={}", max));
                }
                if field <= max.into() {
                    None
                } else {
                    Some(format!("0..={}", max))
                }
            }
            NumericType::Signed { bit_size } => {
                let min = 2u128.pow(bit_size - 1);
                let max = 2u128.pow(bit_size - 1) - 1;
                let target_max = if negative { min } else { max };
                if field <= target_max.into() {
                    None
                } else {
                    Some(format!("-{}..={}", min, max))
                }
            }
            NumericType::NativeField => None,
        }
    }
}

/// All types representable in the IR.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub(crate) enum Type {
    /// Represents numeric types in the IR, including field elements
    Numeric(NumericType),

    /// A reference to some value, such as an array
    Reference(Arc<Type>),

    /// An immutable array value with the given element type and length
    Array(Arc<CompositeType>, u32),

    /// An immutable slice value with a given element type
    Slice(Arc<CompositeType>),

    /// A function that may be called directly
    Function,
}

impl Type {
    /// Returns whether the `Type` represents an unsigned numeric type.
    pub(crate) fn is_unsigned(&self) -> bool {
        matches!(self, Type::Numeric(NumericType::Unsigned { .. }))
    }

    /// Create a new signed integer type with the given amount of bits.
    pub(crate) fn signed(bit_size: u32) -> Type {
        Type::Numeric(NumericType::Signed { bit_size })
    }

    /// Create a new unsigned integer type with the given amount of bits.
    pub(crate) fn unsigned(bit_size: u32) -> Type {
        Type::Numeric(NumericType::Unsigned { bit_size })
    }

    /// Creates the boolean type, represented as u1.
    pub(crate) fn bool() -> Type {
        Type::unsigned(1)
    }

    /// Creates the char type, represented as u8.
    pub(crate) fn char() -> Type {
        Type::unsigned(8)
    }

    /// Creates the str<N> type, of the given length N
    pub(crate) fn str(length: u32) -> Type {
        Type::Array(Arc::new(vec![Type::char()]), length)
    }

    /// Creates the native field type.
    pub(crate) fn field() -> Type {
        Type::Numeric(NumericType::NativeField)
    }

    /// Creates the type of an array's length.
    pub(crate) fn length_type() -> Type {
        Type::unsigned(SSA_WORD_SIZE)
    }

    /// Returns the bit size of the provided numeric type.
    ///
    /// # Panics
    ///
    /// Panics if `self` is not a [`Type::Numeric`]
    pub(crate) fn bit_size(&self) -> u32 {
        match self {
            Type::Numeric(numeric_type) => numeric_type.bit_size(),
            other => panic!("bit_size: Expected numeric type, found {other}"),
        }
    }

    /// Returns the size of the element type for this array/slice.
    /// The size of a type is defined as representing how many Fields are needed
    /// to represent the type. This is 1 for every primitive type, and is the number of fields
    /// for any flattened tuple type.
    pub(crate) fn element_size(&self) -> usize {
        match self {
            Type::Array(elements, _) | Type::Slice(elements) => elements.len(),
            other => panic!("element_size: Expected array or slice, found {other}"),
        }
    }

    pub(crate) fn contains_slice_element(&self) -> bool {
        match self {
            Type::Array(elements, _) => {
                elements.iter().any(|element| element.contains_slice_element())
            }
            Type::Slice(_) => true,
            Type::Numeric(_) => false,
            Type::Reference(element) => element.contains_slice_element(),
            Type::Function => false,
        }
    }

    /// Returns the flattened size of a Type
    pub(crate) fn flattened_size(&self) -> u32 {
        match self {
            Type::Array(elements, len) => {
                elements.iter().fold(0, |sum, elem| sum + (elem.flattened_size() * len))
            }
            Type::Slice(_) => {
                unimplemented!("ICE: cannot fetch flattened slice size");
            }
            _ => 1,
        }
    }

    pub(crate) fn is_nested_slice(&self) -> bool {
        if let Type::Slice(element_types) | Type::Array(element_types, _) = self {
            element_types.as_ref().iter().any(|typ| typ.contains_slice_element())
        } else {
            false
        }
    }

    /// True if this type is an array (or slice) or internally contains an array (or slice)
    pub(crate) fn contains_an_array(&self) -> bool {
        match self {
            Type::Numeric(_) | Type::Function => false,
            Type::Array(_, _) | Type::Slice(_) => true,
            Type::Reference(element) => element.contains_an_array(),
        }
    }

    /// Retrieves the array or slice type within this type, or panics if there is none.
    pub(crate) fn get_contained_array(&self) -> &Type {
        match self {
            Type::Numeric(_) | Type::Function => panic!("Expected an array type"),
            Type::Array(_, _) | Type::Slice(_) => self,
            Type::Reference(element) => element.get_contained_array(),
        }
    }

    pub(crate) fn element_types(self) -> Arc<Vec<Type>> {
        match self {
            Type::Array(element_types, _) | Type::Slice(element_types) => element_types,
            other => panic!("element_types: Expected array or slice, found {other}"),
        }
    }

    pub(crate) fn first(&self) -> Type {
        match self {
            Type::Numeric(_) | Type::Function => self.clone(),
            Type::Reference(typ) => typ.first(),
            Type::Slice(element_types) | Type::Array(element_types, _) => element_types[0].first(),
        }
    }

    /// True if this is a reference type or if it is a composite type which contains a reference.
    pub(crate) fn contains_reference(&self) -> bool {
        match self {
            Type::Reference(_) => true,
            Type::Numeric(_) | Type::Function => false,
            Type::Array(elements, _) | Type::Slice(elements) => {
                elements.iter().any(|elem| elem.contains_reference())
            }
        }
    }
}

/// Composite Types are essentially flattened struct or tuple types.
/// Array types may have these as elements where each flattened field is
/// included in the array sequentially.
pub(crate) type CompositeType = Vec<Type>;

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Numeric(numeric) => numeric.fmt(f),
            Type::Reference(element) => write!(f, "&mut {element}"),
            Type::Array(element, length) => {
                let elements = vecmap(element.iter(), |element| element.to_string());
                if elements.len() == 1 {
                    write!(f, "[{}; {length}]", elements.join(", "))
                } else {
                    write!(f, "[({}); {length}]", elements.join(", "))
                }
            }
            Type::Slice(element) => {
                let elements = vecmap(element.iter(), |element| element.to_string());
                write!(f, "[{}]", elements.join(", "))
            }
            Type::Function => write!(f, "function"),
        }
    }
}

impl std::fmt::Display for NumericType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumericType::Signed { bit_size } => write!(f, "i{bit_size}"),
            NumericType::Unsigned { bit_size } => write!(f, "u{bit_size}"),
            NumericType::NativeField => write!(f, "Field"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8_value_is_outside_limits() {
        let u8 = NumericType::Unsigned { bit_size: 8 };
        assert!(u8.value_is_outside_limits(FieldElement::from(1_i128), true).is_some());
        assert!(u8.value_is_outside_limits(FieldElement::from(0_i128), false).is_none());
        assert!(u8.value_is_outside_limits(FieldElement::from(255_i128), false).is_none());
        assert!(u8.value_is_outside_limits(FieldElement::from(256_i128), false).is_some());
    }

    #[test]
    fn test_i8_value_is_outside_limits() {
        let i8 = NumericType::Signed { bit_size: 8 };
        assert!(i8.value_is_outside_limits(FieldElement::from(129_i128), true).is_some());
        assert!(i8.value_is_outside_limits(FieldElement::from(128_i128), true).is_none());
        assert!(i8.value_is_outside_limits(FieldElement::from(0_i128), false).is_none());
        assert!(i8.value_is_outside_limits(FieldElement::from(127_i128), false).is_none());
        assert!(i8.value_is_outside_limits(FieldElement::from(128_i128), false).is_some());
    }
}
