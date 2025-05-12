use noirc_errors::Located;

use crate::{QuotedType, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    Bool,
    CtString,
    Expr,
    Field,
}

impl PrimitiveType {
    pub fn lookup_by_name(name: &str) -> Option<Self> {
        match name {
            "bool" => Some(Self::Bool),
            "CtString" => Some(Self::CtString),
            "Expr" => Some(Self::Expr),
            "Field" => Some(Self::Field),
            _ => None,
        }
    }

    pub fn to_type(self, _generics: &Option<Vec<Located<Type>>>) -> Type {
        match self {
            Self::Bool => Type::Bool,
            Self::CtString => Type::Quoted(QuotedType::CtString),
            Self::Expr => Type::Quoted(QuotedType::Expr),
            Self::Field => Type::FieldElement,
        }
    }

    pub fn to_integer_or_field(self) -> Option<Type> {
        match self {
            Self::Field => Some(Type::FieldElement),
            Self::Bool | Self::CtString | Self::Expr => None,
        }
    }
}
