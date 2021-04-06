use crate::node_interner::FuncId;

use super::ModuleId;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ModuleDefId {
    ModuleId(ModuleId),
    FunctionId(FuncId),
}

impl ModuleDefId {
    pub fn as_function(&self) -> Option<FuncId> {
        if let ModuleDefId::FunctionId(func_id) = self {
            return Some(*func_id);
        }
        None
    }
    // XXX: We are still allocating fro error reporting even though strings are stored in binary
    // It is a minor performance issue, which can be addressed by having the error reporting, not allocate
    pub fn as_str(&self) -> &'static str {
        match self {
            ModuleDefId::FunctionId(_) => "function",
            ModuleDefId::ModuleId(_) => "module",
        }
    }
}

impl Into<ModuleDefId> for ModuleId {
    fn into(self) -> ModuleDefId {
        ModuleDefId::ModuleId(self)
    }
}
impl Into<ModuleDefId> for FuncId {
    fn into(self) -> ModuleDefId {
        ModuleDefId::FunctionId(self)
    }
}
