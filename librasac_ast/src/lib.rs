/// This module contains two Ident structures, due to the fact that an identifier may or may not return a value
/// statement::Ident does not return a value, while Expression::Ident does.
mod expression;
mod statement;

pub use expression::*;
pub use statement::*;


#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    FieldElement,
    Constant,
    Public,
    Witness,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    Bool,
    Concrete(Ident, Vec<Type>),
    Error, // XXX: Currently have not implemented structs, so this type is a stub
}
