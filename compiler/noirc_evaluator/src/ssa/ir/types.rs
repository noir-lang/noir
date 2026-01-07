use acvm::{FieldElement, acir::AcirField};
use iter_extended::vecmap;
use noirc_frontend::signed_field::SignedField;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
    pub(crate) fn bit_size<F: AcirField>(self: &NumericType) -> u32 {
        match self {
            NumericType::NativeField => F::max_num_bits(),
            NumericType::Unsigned { bit_size } | NumericType::Signed { bit_size } => *bit_size,
        }
    }

    /// Creates a NumericType::Signed type
    pub(crate) fn signed(bit_size: u32) -> NumericType {
        NumericType::Signed { bit_size }
    }

    /// Creates a NumericType::Unsigned type
    pub(crate) fn unsigned(bit_size: u32) -> NumericType {
        NumericType::Unsigned { bit_size }
    }

    /// Creates the u1 type
    pub(crate) fn bool() -> NumericType {
        NumericType::Unsigned { bit_size: 1 }
    }

    /// Creates the char type, represented as u8.
    pub(crate) fn char() -> NumericType {
        NumericType::Unsigned { bit_size: 8 }
    }

    /// Creates the type of an array's length.
    pub(crate) fn length_type() -> NumericType {
        NumericType::Unsigned { bit_size: SSA_WORD_SIZE }
    }

    /// Returns None if the given Field value is within the numeric limits
    /// for the current NumericType. Otherwise returns a string describing
    /// the limits, as a range.
    pub(crate) fn value_is_outside_limits(self, value: SignedField) -> Option<String> {
        match self {
            NumericType::Unsigned { bit_size } => {
                let max = if bit_size == 128 { u128::MAX } else { 2u128.pow(bit_size) - 1 };
                if value.is_negative() || value > SignedField::positive(max) {
                    Some(format!("0..={max}"))
                } else {
                    None
                }
            }
            NumericType::Signed { bit_size } => {
                let min = 2u128.pow(bit_size - 1);
                let max = 2u128.pow(bit_size - 1) - 1;
                if value > SignedField::positive(max) || value < SignedField::negative(min) {
                    Some(format!("-{min}..={max}"))
                } else {
                    None
                }
            }
            NumericType::NativeField => None,
        }
    }

    pub(crate) fn is_field(&self) -> bool {
        matches!(self, NumericType::NativeField)
    }

    pub(crate) fn is_unsigned(&self) -> bool {
        matches!(self, NumericType::Unsigned { .. })
    }

    pub(crate) fn is_signed(&self) -> bool {
        matches!(self, NumericType::Signed { .. })
    }

    pub(crate) fn max_value(&self) -> Result<FieldElement, String> {
        match self {
            NumericType::Unsigned { bit_size } => match bit_size {
                bit_size if *bit_size > 128 => {
                    Err("Cannot get max value for unsigned type: bit size is greater than 128"
                        .to_string())
                }
                128 => Ok(FieldElement::from(u128::MAX)),
                _ => Ok(FieldElement::from(2u128.pow(*bit_size) - 1)),
            },
            other => Err(format!("Cannot get max value for type: {other}")),
        }
    }
}

#[cfg(test)]
mod props {
    use proptest::{
        prelude::{Arbitrary, BoxedStrategy, Just, Strategy as _},
        prop_oneof,
    };

    use super::NumericType;

    impl Arbitrary for NumericType {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            let signed = prop_oneof!(Just(8), Just(16), Just(32), Just(64));
            let unsigned = prop_oneof!(Just(1), Just(8), Just(16), Just(32), Just(64), Just(128));
            prop_oneof![
                signed.prop_map(|bit_size| NumericType::Signed { bit_size }),
                unsigned.prop_map(|bit_size| NumericType::Unsigned { bit_size }),
                Just(NumericType::NativeField),
            ]
            .boxed()
        }
    }
}

/// All types representable in the IR.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Type {
    /// Represents numeric types in the IR, including field elements
    Numeric(NumericType),

    /// A reference to some value, such as an array
    Reference(Arc<Type>),

    /// An immutable array value with the given element type and length
    Array(Arc<CompositeType>, u32),

    /// An immutable vector value with a given element type
    Vector(Arc<CompositeType>),

    /// A function that may be called directly
    Function,
}

impl Type {
    /// Returns whether the `Type` represents an unsigned numeric type.
    pub fn is_unsigned(&self) -> bool {
        matches!(self, Type::Numeric(NumericType::Unsigned { .. }))
    }

    /// Returns whether the `Type` represents an signed numeric type.
    pub fn is_signed(&self) -> bool {
        matches!(self, Type::Numeric(NumericType::Signed { .. }))
    }

    /// Create a new signed integer type with the given amount of bits.
    pub fn signed(bit_size: u32) -> Type {
        Type::Numeric(NumericType::Signed { bit_size })
    }

    /// Create a new unsigned integer type with the given amount of bits.
    pub fn unsigned(bit_size: u32) -> Type {
        Type::Numeric(NumericType::Unsigned { bit_size })
    }

    /// Creates the boolean type, represented as u1.
    pub fn bool() -> Type {
        Type::unsigned(1)
    }

    /// Creates the char type, represented as u8.
    pub fn char() -> Type {
        Type::unsigned(8)
    }

    /// Creates the `str<N>` type, of the given length N
    pub fn str(length: u32) -> Type {
        Type::Array(Arc::new(vec![Type::char()]), length)
    }

    /// Creates the native field type.
    pub fn field() -> Type {
        Type::Numeric(NumericType::NativeField)
    }

    /// Creates the type of an array's length.
    pub fn length_type() -> Type {
        Type::unsigned(SSA_WORD_SIZE)
    }

    /// True if this type is a numeric primitive type.
    pub(crate) fn is_numeric(&self) -> bool {
        matches!(self, Type::Numeric(_))
    }

    /// Returns the inner NumericType if this is one, or panics otherwise
    pub(crate) fn unwrap_numeric(&self) -> NumericType {
        match self {
            Type::Numeric(numeric) => *numeric,
            other => panic!("Expected NumericType, found {other}"),
        }
    }

    /// Returns the bit size of the provided numeric type.
    ///
    /// # Panics
    ///
    /// Panics if `self` is not a [`Type::Numeric`]
    pub(crate) fn bit_size(&self) -> u32 {
        match self {
            Type::Numeric(numeric_type) => numeric_type.bit_size::<FieldElement>(),
            other => panic!("bit_size: Expected numeric type, found {other}"),
        }
    }

    /// Returns the size of the element type for this array/vector.
    /// The size of a type is defined as representing how many Fields are needed
    /// to represent the type. This is 1 for every primitive type, and is the number of fields
    /// for any flattened tuple type.
    ///
    /// Equivalent to `self.element_types().len()`.
    ///
    /// Panics if `self` is not a [`Type::Array`] or [`Type::Vector`].
    pub(crate) fn element_size(&self) -> usize {
        match self {
            Type::Array(elements, _) | Type::Vector(elements) => elements.len(),
            other => panic!("element_size: Expected array or vector, found {other}"),
        }
    }

    /// Return the types of items in this array/vector.
    ///
    /// Panics if `self` is not a [`Type::Array`] or [`Type::Vector`].
    pub(crate) fn element_types(&self) -> Arc<Vec<Type>> {
        match self {
            Type::Array(element_types, _) | Type::Vector(element_types) => element_types.clone(),
            other => panic!("element_types: Expected array or vector, found {other}"),
        }
    }

    pub(crate) fn contains_vector_element(&self) -> bool {
        match self {
            Type::Array(elements, _) => {
                elements.iter().any(|element| element.contains_vector_element())
            }
            Type::Vector(_) => true,
            Type::Numeric(_) => false,
            Type::Reference(element) => element.contains_vector_element(),
            Type::Function => false,
        }
    }

    /// Returns the flattened size of a Type.
    ///
    /// The flattened type is mostly useful in ACIR, where nested arrays are also flattened,
    /// as opposed to SSA, where only tuples get flattened into the array they are in,
    /// but nested arrays appear as a value ID.
    pub(crate) fn flattened_size(&self) -> u32 {
        match self {
            Type::Array(elements, len) => {
                elements.iter().fold(0, |sum, elem| sum + (elem.flattened_size() * len))
            }
            Type::Vector(_) => {
                unimplemented!("ICE: cannot fetch flattened vector size");
            }
            _ => 1,
        }
    }

    /// True if this type is an array (or vector)
    pub(crate) fn is_array(&self) -> bool {
        matches!(self, Type::Array(_, _) | Type::Vector(_))
    }

    pub(crate) fn is_nested_vector(&self) -> bool {
        if let Type::Vector(element_types) | Type::Array(element_types, _) = self {
            element_types.as_ref().iter().any(|typ| typ.contains_vector_element())
        } else {
            false
        }
    }

    /// True if this type is an array (or vector) or internally contains an array (or vector)
    pub(crate) fn contains_an_array(&self) -> bool {
        match self {
            Type::Numeric(_) | Type::Function => false,
            Type::Array(_, _) | Type::Vector(_) => true,
            Type::Reference(element) => element.contains_an_array(),
        }
    }

    pub(crate) fn first(&self) -> Type {
        match self {
            Type::Numeric(_) | Type::Function => self.clone(),
            Type::Reference(typ) => typ.first(),
            Type::Vector(element_types) | Type::Array(element_types, _) => element_types[0].first(),
        }
    }

    /// True if this is a reference type or if it is a composite type which contains a reference.
    pub(crate) fn contains_reference(&self) -> bool {
        match self {
            Type::Reference(_) => true,
            Type::Numeric(_) | Type::Function => false,
            Type::Array(elements, _) | Type::Vector(elements) => {
                elements.iter().any(|elem| elem.contains_reference())
            }
        }
    }

    /// True if this is a function type or if it is a composite type which contains a function.
    pub(crate) fn contains_function(&self) -> bool {
        match self {
            Type::Reference(element_type) => element_type.contains_function(),
            Type::Function => true,
            Type::Numeric(_) => false,
            Type::Array(elements, _) | Type::Vector(elements) => {
                elements.iter().any(|elem| elem.contains_function())
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
            Type::Vector(element) => {
                let elements = vecmap(element.iter(), |element| element.to_string());
                if elements.len() == 1 {
                    write!(f, "[{}]", elements.join(", "))
                } else {
                    write!(f, "[({})]", elements.join(", "))
                }
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
    use proptest::prelude::*;

    #[test]
    fn test_u8_value_is_outside_limits() {
        let u8 = NumericType::Unsigned { bit_size: 8 };
        assert!(u8.value_is_outside_limits(SignedField::negative(1_i128)).is_some());
        assert!(u8.value_is_outside_limits(SignedField::positive(0_i128)).is_none());
        assert!(u8.value_is_outside_limits(SignedField::positive(255_i128)).is_none());
        assert!(u8.value_is_outside_limits(SignedField::positive(256_i128)).is_some());
    }

    #[test]
    fn test_i8_value_is_outside_limits() {
        let i8 = NumericType::Signed { bit_size: 8 };
        assert!(i8.value_is_outside_limits(SignedField::negative(129_i128)).is_some());
        assert!(i8.value_is_outside_limits(SignedField::negative(128_i128)).is_none());
        assert!(i8.value_is_outside_limits(SignedField::positive(0_i128)).is_none());
        assert!(i8.value_is_outside_limits(SignedField::positive(127_i128)).is_none());
        assert!(i8.value_is_outside_limits(SignedField::positive(128_i128)).is_some());
    }

    proptest! {
        #[test]
        fn test_max_value_is_in_limits(input: NumericType) {
            let max_value = input.max_value();
            if let Ok(max_value) = max_value {
                prop_assert!(input.value_is_outside_limits(SignedField::from(max_value)).is_none());
            }
        }
    }
}
