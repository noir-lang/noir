mod contract;
/// This module contains two Ident structures, due to the fact that an identifier may or may not return a value
/// statement::Ident does not return a value, while Expression::Ident does.
mod expression;
mod function;
mod statement;
mod structure;

pub use expression::*;
pub use function::*;

pub use contract::*;
use noirc_errors::Span;
pub use statement::*;
pub use structure::*;

use crate::{
    token::{IntType, Keyword},
    util::vecmap,
    IsConst,
};

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

    pub fn is_subtype_of(&self, argument: &ArraySize) -> bool {
        (self.is_fixed() && argument.is_variable()) || (self == argument)
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
}

impl PartialEq for FieldElementType {
    fn eq(&self, _other: &Self) -> bool {
        // The reason we manually implement this, is so that Private and Public
        // are seen as equal
        true
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
            FieldElementType::Private => panic!("No Keyword for a Private FieldElementType"),
            FieldElementType::Public => Keyword::Pub,
        }
    }
}

impl std::fmt::Display for FieldElementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldElementType::Private => write!(f, "priv"),
            FieldElementType::Public => write!(f, "pub"),
        }
    }
}

/// The parser parses types as 'UnresolvedType's which
/// require name resolution to resolve any typenames used
/// for structs within, but are otherwise identical to Types.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UnresolvedType {
    FieldElement(IsConst, FieldElementType),
    Array(FieldElementType, ArraySize, Box<UnresolvedType>), // [4]Witness = Array(4, Witness)
    Integer(IsConst, FieldElementType, Signedness, u32),     // u32 = Integer(unsigned, 32)
    Bool(IsConst),
    Unit,
    Struct(FieldElementType, Path),

    // Note: Tuples have no FieldElementType, instead each of their elements may have one.
    Tuple(Vec<UnresolvedType>),

    Unspecified, // This is for when the user declares a variable without specifying it's type
    Error,
}

impl UnresolvedType {
    // These are here so that the code is more readable.
    pub const WITNESS: UnresolvedType =
        UnresolvedType::FieldElement(IsConst::No(None), FieldElementType::Private);
    pub const PUBLIC: UnresolvedType =
        UnresolvedType::FieldElement(IsConst::No(None), FieldElementType::Public);
}

impl Recoverable for UnresolvedType {
    fn error(_: Span) -> Self {
        UnresolvedType::Error
    }
}

impl std::fmt::Display for UnresolvedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vis_str = |vis| match vis {
            FieldElementType::Private => "",
            FieldElementType::Public => "pub ",
        };

        use UnresolvedType::*;
        match self {
            FieldElement(is_const, fe_type) => write!(f, "{}{}Field", is_const, vis_str(*fe_type)),
            Array(fe_type, size, typ) => write!(f, "{}{}{}", vis_str(*fe_type), size, typ),
            Integer(is_const, fe_type, sign, num_bits) => match sign {
                Signedness::Signed => write!(f, "{}{}i{}", is_const, vis_str(*fe_type), num_bits),
                Signedness::Unsigned => write!(f, "{}{}u{}", is_const, vis_str(*fe_type), num_bits),
            },
            Struct(fe_type, s) => write!(f, "{}{}", vis_str(*fe_type), s),
            Tuple(elements) => {
                let elements = vecmap(elements, ToString::to_string);
                write!(f, "({})", elements.join(", "))
            }
            Bool(is_const) => write!(f, "{}bool", is_const),
            Unit => write!(f, "()"),
            Error => write!(f, "error"),
            Unspecified => write!(f, "unspecified"),
        }
    }
}

impl UnresolvedType {
    pub fn from_int_tok(
        is_const: IsConst,
        field_type: FieldElementType,
        int_tok: &IntType,
    ) -> UnresolvedType {
        use {IntType::*, UnresolvedType::Integer};
        match int_tok {
            Signed(num_bits) => Integer(is_const, field_type, Signedness::Signed, *num_bits),
            Unsigned(num_bits) => Integer(is_const, field_type, Signedness::Unsigned, *num_bits),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Signedness {
    Unsigned,
    Signed,
}
