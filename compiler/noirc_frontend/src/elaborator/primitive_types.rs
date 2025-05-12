use noirc_errors::Located;

use crate::{QuotedType, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    CtString,
    Field,
}

impl PrimitiveType {
    pub fn lookup_by_name(name: &str) -> Option<Self> {
        match name {
            "CtString" => Some(Self::CtString),
            "Field" => Some(Self::Field),
            _ => None,
        }
    }

    pub fn to_type(self, _generics: &Option<Vec<Located<Type>>>) -> Type {
        match self {
            Self::CtString => Type::Quoted(QuotedType::CtString),
            Self::Field => Type::FieldElement,
        }
    }
}
