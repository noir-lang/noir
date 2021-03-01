use noirc_abi::Abi;

use super::{HirBlockExpression, HirExpression};
use crate::node_interner::{ExprId, IdentId, NodeInterner};
use crate::{token::Attribute, FunctionKind, Type};

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
pub struct Parameters(Vec<Param>);

impl Parameters {
    pub fn to_abi(self, interner: &NodeInterner) -> Abi {
        let parameters: Vec<_> = self
            .0
            .into_iter()
            .map(|param| {
                let (param_id, param_type) = (param.0, param.1);
                let param_name = interner.ident_name(&param_id);
                (param_name, param_type.as_abi_type())
            })
            .collect();
        noirc_abi::Abi { parameters }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Param> {
        self.0.iter()
    }
    pub fn into_iter(self) -> impl Iterator<Item = Param> {
        self.0.into_iter()
    }
}

impl From<Vec<Param>> for Parameters {
    fn from(vec: Vec<Param>) -> Parameters {
        Parameters(vec)
    }
}
#[derive(Debug, Clone)]
pub struct FuncMeta {
    pub name: String,

    pub kind: FunctionKind,

    pub attributes: Option<Attribute>,
    pub parameters: Parameters,
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
