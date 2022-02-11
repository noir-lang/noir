/// This module contains two Ident structures, due to the fact that an identifier may or may not return a value
/// statement::Ident does not return a value, while Expression::Ident does.
mod expression;
mod function;
mod statement;
mod structure;

pub use expression::*;
pub use function::*;

use noirc_errors::Span;
pub use statement::*;
pub use structure::*;

use crate::token::{IntType, Keyword};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ArraySize {
    Variable,
    Fixed(u128),
}

impl ArraySize {
    pub fn is_fixed(&self) -> bool {
        matches!(self, ArraySize::Fixed(_))
    }

    pub fn is_variable(&self) -> bool {
        !self.is_fixed()
    }

    // #[allow(clippy::suspicious_operation_groupings)]
    pub fn is_a_super_type_of(&self, argument: &ArraySize) -> bool {
        (self.is_variable() && argument.is_fixed()) || (self == argument)
    }
}

impl std::fmt::Display for ArraySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArraySize::Variable => write!(f, "[]"),
            ArraySize::Fixed(size) => write!(f, "[{}]", size),
        }
    }
}

/// FieldElementType refers to how the Compiler type is interpreted by the proof system
/// Example: FieldElementType::Private means that the Compiler type is seen as a witness/witnesses
#[derive(Debug, Eq, Copy, Clone)]
pub enum FieldElementType {
    Private,
    Public,
    Constant,
}

impl PartialEq for FieldElementType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (FieldElementType::Private, FieldElementType::Private) => true,
            (FieldElementType::Public, FieldElementType::Public) => true,
            (FieldElementType::Constant, FieldElementType::Constant) => true,
            // The reason we manually implement this, is so that Private and Public
            // are seen as equal
            (FieldElementType::Private, FieldElementType::Public) => true,
            (FieldElementType::Public, FieldElementType::Private) => true,
            (FieldElementType::Private, FieldElementType::Constant) => false,
            (FieldElementType::Public, FieldElementType::Constant) => false,
            (FieldElementType::Constant, FieldElementType::Private) => false,
            (FieldElementType::Constant, FieldElementType::Public) => false,
        }
    }
}

impl FieldElementType {
    // In the majority of places, public and private are
    // interchangeable. The place where the difference does matter is
    // when witnesses are being added to the constraint system.
    // For the compiler, the appropriate place would be in the ABI
    pub fn strict_eq(&self, other: &FieldElementType) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }

    /// Return the corresponding keyword for this field type
    pub fn as_keyword(self) -> Keyword {
        match self {
            FieldElementType::Private => Keyword::Priv,
            FieldElementType::Public => Keyword::Pub,
            FieldElementType::Constant => Keyword::Const,
        }
    }
}

impl std::fmt::Display for FieldElementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldElementType::Private => write!(f, "priv"),
            FieldElementType::Constant => write!(f, "const"),
            FieldElementType::Public => write!(f, "pub"),
        }
    }
}

/// The parser parses types as 'UnresolvedType's which
/// require name resolution to resolve any typenames used
/// for structs within, but are otherwise identical to Types.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UnresolvedType {
    FieldElement(FieldElementType),
    Array(FieldElementType, ArraySize, Box<UnresolvedType>), // [4]Witness = Array(4, Witness)
    Integer(FieldElementType, Signedness, u32),              // u32 = Integer(unsigned, 32)
    Bool,
    Unit,
    Struct(FieldElementType, Path),
    Unspecified, // This is for when the user declares a variable without specifying it's type
    Error,
}

impl UnresolvedType {
    // These are here so that the code is more readable.
    pub const WITNESS: UnresolvedType = UnresolvedType::FieldElement(FieldElementType::Private);
    pub const CONSTANT: UnresolvedType = UnresolvedType::FieldElement(FieldElementType::Constant);
    pub const PUBLIC: UnresolvedType = UnresolvedType::FieldElement(FieldElementType::Public);
}

impl Recoverable for UnresolvedType {
    fn error(_: Span) -> Self {
        UnresolvedType::Error
    }
}

impl std::fmt::Display for UnresolvedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use UnresolvedType::*;
        match self {
            FieldElement(fe_type) => write!(f, "{} Field", fe_type),
            Array(fe_type, size, typ) => write!(f, "{} {}{}", fe_type, size, typ),
            Integer(fe_type, sign, num_bits) => match sign {
                Signedness::Signed => write!(f, "{} i{}", fe_type, num_bits),
                Signedness::Unsigned => write!(f, "{} u{}", fe_type, num_bits),
            },
            Struct(fe_type, s) => write!(f, "{} {}", fe_type, s),
            Bool => write!(f, "bool"),
            Unit => write!(f, "()"),
            Error => write!(f, "error"),
            Unspecified => write!(f, "unspecified"),
        }
    }
}

impl UnresolvedType {
    pub fn from_int_tok(field_type: FieldElementType, int_tok: &IntType) -> UnresolvedType {
        use {IntType::*, UnresolvedType::Integer};
        match int_tok {
            Signed(num_bits) => Integer(field_type, Signedness::Signed, *num_bits),
            Unsigned(num_bits) => Integer(field_type, Signedness::Unsigned, *num_bits),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Signedness {
    Unsigned,
    Signed,
}
