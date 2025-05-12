use noirc_errors::Located;

use crate::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    Field,
}

impl PrimitiveType {
    pub fn lookup_by_name(name: &str) -> Option<Self> {
        match name {
            "Field" => Some(Self::Field),
            _ => None,
        }
    }

    pub fn to_type(self, _generics: &Option<Vec<Located<Type>>>) -> Type {
        match self {
            Self::Field => Type::FieldElement,
        }
    }
}
