
use crate::{FunctionKind,Type, token::Attribute};

use super::def_interner::{IdentId, StmtId};

/// A Hir function is a vector of interned statements
#[derive(Debug, Clone)]
pub struct HirFunction(Vec<StmtId>);

impl HirFunction {
    pub fn empty() -> HirFunction {
        HirFunction(Vec::new())
    }
    pub fn push_stmt(&mut self, id : StmtId) {
        self.0.push(id)
    }

    pub fn statements(&self) -> Vec<StmtId> {
        self.0.clone()
    }
}

/// An interned function parameter from a function definition
#[derive(Debug, Clone)]
pub struct Param(pub IdentId,pub Type);

#[derive(Debug, Clone)]
pub struct FuncMeta {
    pub name : String,

    pub kind : FunctionKind,

    pub attributes : Option<Attribute>,
    pub parameters : Vec<Param>, 
    pub return_type : Type,

    // This flag is needed for the attribute check pass
    pub has_body : bool,
}
