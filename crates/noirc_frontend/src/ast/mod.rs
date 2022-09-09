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

use crate::{token::IntType, util::vecmap, IsConst};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UnresolvedArraySize {
    Variable,
    Fixed(u64),
    FixedVariable(Ident),
}

impl UnresolvedArraySize {
    pub fn is_fixed(&self) -> bool {
        matches!(self, UnresolvedArraySize::Fixed(_))
    }

    pub fn is_fixed_variable(&self) -> bool {
        matches!(self, UnresolvedArraySize::FixedVariable(_))
    }

    pub fn is_variable(&self) -> bool {
        !self.is_fixed()
    }

    pub fn is_subtype_of(&self, argument: &UnresolvedArraySize) -> bool {
        (self.is_fixed() && argument.is_variable()) || (self == argument)
    }
}

impl std::fmt::Display for UnresolvedArraySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnresolvedArraySize::Variable => write!(f, "[]"),
            UnresolvedArraySize::Fixed(size) => write!(f, "[{}]", size),
            UnresolvedArraySize::FixedVariable(ident) => write!(f, "[{}]", ident),
        }
    }
}

/// The parser parses types as 'UnresolvedType's which
/// require name resolution to resolve any typenames used
/// for structs within, but are otherwise identical to Types.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UnresolvedType {
    FieldElement(IsConst),
    Array(UnresolvedArraySize, Box<UnresolvedType>), // [4]Witness = Array(4, Witness)
    Integer(IsConst, Signedness, u32),               // u32 = Integer(unsigned, 32)
    Bool(IsConst),
    Unit,

    /// A Named UnresolvedType can be a struct type or a type variable
    Named(Path, Vec<UnresolvedType>),

    // Note: Tuples have no FieldElementType, instead each of their elements may have one.
    Tuple(Vec<UnresolvedType>),

    Unspecified, // This is for when the user declares a variable without specifying it's type
    Error,
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
            FieldElement(is_const) => write!(f, "{}Field", is_const),
            Array(len, typ) => write!(f, "[{}; {}]", typ, len),
            Integer(is_const, sign, num_bits) => match sign {
                Signedness::Signed => write!(f, "{}i{}", is_const, num_bits),
                Signedness::Unsigned => write!(f, "{}u{}", is_const, num_bits),
            },
            Named(s, args) => {
                let args = vecmap(args, ToString::to_string);
                if args.is_empty() {
                    write!(f, "{}", s)
                } else {
                    write!(f, "{}<{}>", s, args.join(", "))
                }
            }
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
    pub fn from_int_token(token: (IsConst, IntType)) -> UnresolvedType {
        use {IntType::*, UnresolvedType::Integer};
        match token.1 {
            Signed(num_bits) => Integer(token.0, Signedness::Signed, num_bits),
            Unsigned(num_bits) => Integer(token.0, Signedness::Unsigned, num_bits),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Signedness {
    Unsigned,
    Signed,
}
