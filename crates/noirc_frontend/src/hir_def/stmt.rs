use super::expr::HirIdent;
use crate::node_interner::ExprId;
use crate::{Ident, Shared, StructType, Type};
use fm::FileId;
use noirc_errors::Span;

#[derive(Debug, Clone)]
pub struct HirLetStatement {
    pub pattern: HirPattern,
    pub r#type: Type,
    pub expression: ExprId,
}

impl HirLetStatement {
    pub fn ident(&self) -> HirIdent {
        match self.pattern {
            HirPattern::Identifier(ident) => ident,
            _ => panic!("can only fetch hir ident from HirPattern::Identifier"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HirAssignStatement {
    pub lvalue: HirLValue,
    pub expression: ExprId,
}

#[derive(Debug, Clone)]
pub struct HirConstrainStatement(pub ExprId, pub FileId);

#[derive(Debug, Clone)]
pub struct BinaryStatement {
    pub lhs: ExprId,
    pub r#type: Type,
    pub expression: ExprId,
}

#[derive(Debug, Clone)]
pub enum HirStatement {
    Let(HirLetStatement),
    Constrain(HirConstrainStatement),
    Assign(HirAssignStatement),
    Expression(ExprId),
    Semi(ExprId),
    Error,
}

#[derive(Debug, Clone)]
pub enum HirPattern {
    Identifier(HirIdent),
    Mutable(Box<HirPattern>, Span),
    Tuple(Vec<HirPattern>, Span),
    Struct(Shared<StructType>, Vec<(Ident, HirPattern)>, Span),
}

impl HirPattern {
    pub fn field_count(&self) -> usize {
        match self {
            HirPattern::Identifier(_) => 0,
            HirPattern::Mutable(_, _) => 0,
            HirPattern::Tuple(fields, _) => fields.len(),
            HirPattern::Struct(_, fields, _) => fields.len(),
        }
    }

    /// Iterate over the fields of this pattern.
    /// Panics if the type is not a struct or tuple.
    pub fn iter_fields<'a>(&'a self) -> Box<dyn Iterator<Item = (String, &'a HirPattern)> + 'a> {
        match self {
            HirPattern::Struct(_, fields, _) => Box::new(
                fields.iter().map(move |(name, pattern)| (name.0.contents.clone(), pattern)),
            ),
            HirPattern::Tuple(fields, _) => {
                Box::new(fields.iter().enumerate().map(|(i, field)| (i.to_string(), field)))
            }
            other => panic!("Tried to iterate over the fields of '{:?}', which has none", other),
        }
    }
}

/// Represents an Ast form that can be assigned to
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum HirLValue {
    Ident(HirIdent, Type),
    MemberAccess {
        object: Box<HirLValue>,
        field_name: Ident,
        field_index: Option<usize>,
        typ: Type,
    },
    Index {
        array: Box<HirLValue>,
        index: ExprId,
        typ: Type,
    },
}
