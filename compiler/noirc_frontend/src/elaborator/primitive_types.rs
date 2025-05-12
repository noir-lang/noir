use noirc_errors::Located;

use crate::{QuotedType, Type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    Bool,
    CtString,
    Expr,
    Field,
    FunctionDefinition,
    Module,
    Quoted,
    TraitConstraint,
    TraitDefinition,
    TraitImpl,
    TypeDefinition,
    TypedExpr,
}

impl PrimitiveType {
    pub fn lookup_by_name(name: &str) -> Option<Self> {
        match name {
            "bool" => Some(Self::Bool),
            "CtString" => Some(Self::CtString),
            "Expr" => Some(Self::Expr),
            "Field" => Some(Self::Field),
            "FunctionDefinition" => Some(Self::FunctionDefinition),
            "Module" => Some(Self::Module),
            "Quoted" => Some(Self::Quoted),
            "TraitConstraint" => Some(Self::TraitConstraint),
            "TraitDefinition" => Some(Self::TraitDefinition),
            "TraitImpl" => Some(Self::TraitImpl),
            "TypeDefinition" => Some(Self::TypeDefinition),
            "TypedExpr" => Some(Self::TypedExpr),
            _ => None,
        }
    }

    pub fn to_type(self, _generics: &Option<Vec<Located<Type>>>) -> Type {
        match self {
            Self::Bool => Type::Bool,
            Self::CtString => Type::Quoted(QuotedType::CtString),
            Self::Expr => Type::Quoted(QuotedType::Expr),
            Self::Field => Type::FieldElement,
            Self::FunctionDefinition => Type::Quoted(QuotedType::FunctionDefinition),
            Self::Module => Type::Quoted(QuotedType::Module),
            Self::Quoted => Type::Quoted(QuotedType::Quoted),
            Self::TraitConstraint => Type::Quoted(QuotedType::TraitConstraint),
            Self::TraitDefinition => Type::Quoted(QuotedType::TraitDefinition),
            Self::TraitImpl => Type::Quoted(QuotedType::TraitImpl),
            Self::TypeDefinition => Type::Quoted(QuotedType::TypeDefinition),
            Self::TypedExpr => Type::Quoted(QuotedType::TypedExpr),
        }
    }

    pub fn to_integer_or_field(self) -> Option<Type> {
        match self {
            Self::Field => Some(Type::FieldElement),
            Self::Bool
            | Self::CtString
            | Self::Expr
            | Self::FunctionDefinition
            | Self::Module
            | Self::Quoted
            | Self::TraitConstraint
            | Self::TraitDefinition
            | Self::TraitImpl
            | Self::TypeDefinition
            | Self::TypedExpr => None,
        }
    }
}
