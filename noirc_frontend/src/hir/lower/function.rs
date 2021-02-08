use crate::{token::Attribute, FunctionKind, Type};

use super::{
    node_interner::{ExprId, IdentId, NodeInterner, StmtId},
    HirBlockExpression, HirExpression,
};

/// A Hir function is a block expression
/// with a list of statements
#[derive(Debug, Clone)]
pub struct HirFunction(ExprId);

impl HirFunction {
    pub fn empty() -> HirFunction {
        HirFunction(ExprId::empty_block_id())
    }

    // This function is marked as unsafe because
    // the expression kind is not being checked
    pub const fn unsafe_from_expr(expr_id: ExprId) -> HirFunction {
        HirFunction(expr_id)
    }

    // This function is marked as unsafe because
    // the expression kind is not being checked
    pub const fn as_expr(&self) -> &ExprId {
        &self.0
    }

    pub fn block(&self, interner: &NodeInterner) -> HirBlockExpression {
        match interner.expression(&self.0) {
            HirExpression::Block(block_expr) => block_expr,
            _ => unreachable!("ice: functions can only be block expressions"),
        }
    }
}

/// An interned function parameter from a function definition
#[derive(Debug, Clone)]
pub struct Param(pub IdentId, pub Type);

#[derive(Debug, Clone)]
pub struct FuncMeta {
    pub name: String,

    pub kind: FunctionKind,

    pub attributes: Option<Attribute>,
    pub parameters: Vec<Param>,
    pub return_type: Type,

    // This flag is needed for the attribute check pass
    pub has_body: bool,
}

impl FuncMeta {
    /// Builtin and LowLevel functions usually have the return type
    /// declared, however their function bodies will be empty
    /// So this method tells the type checker to ignore the return
    /// of the empty function, which is unit
    pub fn can_ignore_return_type(&self) -> bool {
        match self.kind {
            FunctionKind::LowLevel | FunctionKind::Builtin => true,
            FunctionKind::Normal => false,
        }
    }
}
