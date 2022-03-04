use std::rc::Rc;

use super::expr::HirInfixExpression;
use crate::node_interner::{ExprId, IdentId};
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
    Struct(Rc<StructType>, Vec<(IdentId, HirPattern)>, Span),
}
