use std::rc::Rc;

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
pub(crate) enum NumericType {
    Signed { bit_size: u32 },
    Unsigned { bit_size: u32 },
    NativeField,
}

/// All types representable in the IR.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub(crate) enum Type {
    /// Represents numeric types in the IR, including field elements
    Numeric(NumericType),

    /// A reference to some value, such as an array
    Reference,

    /// An immutable array value with the given element type and length
    Array(Rc<CompositeType>, usize),

    /// An immutable slice value with a given element type
    Slice(Rc<CompositeType>),

    /// A function that may be called directly
    Function,
}

impl Type {
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
}

/// Composite Types are essentially flattened struct or tuple types.
/// Array types may have these as elements where each flattened field is
/// included in the array sequentially.
pub(crate) type CompositeType = Vec<Type>;

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Numeric(numeric) => numeric.fmt(f),
            Type::Reference => write!(f, "reference"),
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
