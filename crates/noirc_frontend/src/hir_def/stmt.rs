use std::cell::RefCell;
use std::rc::Rc;

use super::expr::HirInfixExpression;
use crate::node_interner::{ExprId, IdentId, NodeInterner};
use crate::{StructType, Type};
use noirc_errors::Span;

#[derive(Debug, Clone)]
pub struct HirLetStatement {
    pub pattern: HirPattern,
    pub r#type: Type,
    pub expression: ExprId,
}

#[derive(Debug, Clone)]
pub struct HirAssignStatement {
    pub identifier: IdentId,
    pub expression: ExprId,
}

#[derive(Debug, Clone)]
pub struct HirConstrainStatement(pub HirInfixExpression);

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
    Identifier(IdentId),
    Mutable(Box<HirPattern>, Span),
    Tuple(Vec<HirPattern>, Span),
    Struct(Rc<RefCell<StructType>>, Vec<(IdentId, HirPattern)>, Span),
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
    pub fn iter_fields<'a>(
        &'a self,
        interner: &'a NodeInterner,
    ) -> Box<dyn Iterator<Item = (String, &'a HirPattern)> + 'a> {
        match self {
            HirPattern::Struct(_, fields, _) => Box::new(
                fields
                    .iter()
                    .map(move |(name_id, pattern)| (interner.ident_name(name_id), pattern)),
            ),
            HirPattern::Tuple(fields, _) => Box::new(
                fields
                    .iter()
                    .enumerate()
                    .map(|(i, field)| (i.to_string(), field)),
            ),
            other => panic!(
                "Tried to iterate over the fields of '{:?}', which has none",
                other
            ),
        }
    }
}

/// Represents an Ast form that can be assigned to
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum HirLValue {
    Ident(Ident),
    MemberAccess {
        object: Box<LValue>,
        field_name: Ident,
    },
    Index {
        array: Box<LValue>,
        index: Expression,
    },
}
