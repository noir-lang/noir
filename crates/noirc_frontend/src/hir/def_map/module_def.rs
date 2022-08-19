use crate::node_interner::{FuncId, StmtId, StructId};

use super::ModuleId;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ModuleDefId {
    ModuleId(ModuleId),
    FunctionId(FuncId),
    TypeId(StructId),
    ConstId(StmtId),
}

impl ModuleDefId {
    pub fn as_function(&self) -> Option<FuncId> {
        match self {
            ModuleDefId::FunctionId(func_id) => Some(*func_id),
            _ => None,
        }
    }

    pub fn as_type(&self) -> Option<StructId> {
        match self {
            ModuleDefId::TypeId(type_id) => Some(*type_id),
            _ => None,
        }
    }

    // XXX: We are still allocating for error reporting even though strings are stored in binary
    // It is a minor performance issue, which can be addressed by having the error reporting, not allocate
    pub fn as_str(&self) -> &'static str {
        match self {
            ModuleDefId::FunctionId(_) => "function",
            ModuleDefId::TypeId(_) => "type",
            ModuleDefId::ModuleId(_) => "module",
            ModuleDefId::ConstId(_) => "const",
        }
    }
}

impl From<ModuleId> for ModuleDefId {
    fn from(mid: ModuleId) -> Self {
        ModuleDefId::ModuleId(mid)
    }
}

impl From<FuncId> for ModuleDefId {
    fn from(fid: FuncId) -> Self {
        ModuleDefId::FunctionId(fid)
    }
}

pub trait TryFromModuleDefId: Sized {
    fn try_from(id: ModuleDefId) -> Option<Self>;
    fn dummy_id() -> Self;
    fn description() -> String;
}

impl TryFromModuleDefId for FuncId {
    fn try_from(id: ModuleDefId) -> Option<Self> {
        id.as_function()
    }

    fn dummy_id() -> Self {
        FuncId::dummy_id()
    }

    fn description() -> String {
        "function".to_string()
    }
}

impl TryFromModuleDefId for StructId {
    fn try_from(id: ModuleDefId) -> Option<Self> {
        id.as_type()
    }

    fn dummy_id() -> Self {
        StructId::dummy_id()
    }

    fn description() -> String {
        "type".to_string()
    }
}
