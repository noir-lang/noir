/// This module contains two Ident structures, due to the fact that an identifier may or may not return a value
/// statement::Ident does not return a value, while Expression::Ident does.
mod expression;
mod statement;

pub use expression::*;
pub use statement::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Type {
    FieldElement,
    Constant,
    Public,
    Witness,
    Integer(Signedness, u32), // u32 = Integer(unsigned, 32)
    Bool,
    Error, // XXX: Currently have not implemented structs, so this type is a stub
}

use libnoirc_lexer::token::IntType;

impl From<&IntType> for Type {
    fn from(it: &IntType) -> Type {
        match it {
            IntType::Signed(num_bits) => Type::Integer(Signedness::Signed, *num_bits),
            IntType::Unsigned(num_bits) => Type::Integer(Signedness::Unsigned, *num_bits),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Signedness {
    Unsigned,
    Signed,
}
