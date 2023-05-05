/// A numeric type in the Intermediate representation
/// Note: we class NativeField as a numeric type
/// though we also apply limitations to it, such as not
/// being able to compare two native fields, whereas this is
/// something that you can do with a signed/unsigned integer.
///
/// Fields do not have a notion of ordering, so this distinction
/// is reasonable.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum NumericType {
    Signed { bit_size: u32 },
    Unsigned { bit_size: u32 },
    NativeField,
}

/// All types representable in the IR.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Type {
    /// Represents numeric types in the IR, including field elements
    Numeric(NumericType),

    /// A reference to some value, such as an array
    Reference,

    /// A function that may be called directly
    Function,

    /// The Unit type with a single value
    Unit,
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

    /// Creates the native field type.
    pub(crate) fn field() -> Type {
        Type::Numeric(NumericType::NativeField)
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Numeric(numeric) => numeric.fmt(f),
            Type::Reference => write!(f, "reference"),
            Type::Function => write!(f, "function"),
            Type::Unit => write!(f, "unit"),
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
