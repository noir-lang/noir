use std::rc::Rc;

use acvm::FieldElement;
use iter_extended::vecmap;

/// A numeric type in the Intermediate representation
/// Note: we class NativeField as a numeric type
/// though we also apply limitations to it, such as not
/// being able to compare two native fields, whereas this is
/// something that you can do with a signed/unsigned integer.
///
/// Fields do not have a notion of ordering, so this distinction
/// is reasonable.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
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

    /// Returns true if the given Field value is within the numeric limits
    /// for the current NumericType.
    pub(crate) fn value_is_within_limits(self, field: FieldElement) -> bool {
        match self {
            NumericType::Signed { bit_size } | NumericType::Unsigned { bit_size } => {
                let max = 2u128.pow(bit_size) - 1;
                field <= max.into()
            }
            NumericType::NativeField => true,
        }
    }
}

/// All types representable in the IR.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub(crate) enum Type {
    /// Represents numeric types in the IR, including field elements
    Numeric(NumericType),

    /// A reference to some value, such as an array
    Reference(Rc<Type>),

    /// An immutable array value with the given element type and length
    Array(Rc<CompositeType>, usize),

    /// An immutable slice value with a given element type
    Slice(Rc<CompositeType>),

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

    /// Creates the native field type.
    pub(crate) fn field() -> Type {
        Type::Numeric(NumericType::NativeField)
    }

    /// Creates the type of an array's length.
    pub(crate) fn length_type() -> Type {
        Type::unsigned(64)
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
    pub(crate) fn flattened_size(&self) -> usize {
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

    pub(crate) fn element_types(self) -> Rc<Vec<Type>> {
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
                write!(f, "[{}; {length}]", elements.join(", "))
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
