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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
/// All types representable in the IR.
pub(crate) enum Typ {
    /// Represents numeric types in the IR
    /// including field elements
    Numeric(NumericType),
    /// Represents the absence of a Type.
    Unit,
}
