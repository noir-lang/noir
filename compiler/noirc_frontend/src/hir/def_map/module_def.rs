use crate::node_interner::{FuncId, GlobalId, StructId, TraitId, TypeAliasId};

use super::ModuleId;

/// A generic ID that references either a module, function, type, interface or global
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ModuleDefId {
    ModuleId(ModuleId),
    FunctionId(FuncId),
    TypeId(StructId),
    TypeAliasId(TypeAliasId),
    TraitId(TraitId),
    GlobalId(GlobalId),
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

    pub fn as_type_alias(&self) -> Option<TypeAliasId> {
        match self {
            ModuleDefId::TypeAliasId(type_alias_id) => Some(*type_alias_id),
            _ => None,
        }
    }

    pub fn as_trait(&self) -> Option<TraitId> {
        match self {
            ModuleDefId::TraitId(trait_id) => Some(*trait_id),
            _ => None,
        }
    }

    pub fn as_global(&self) -> Option<GlobalId> {
        match self {
            ModuleDefId::GlobalId(global_id) => Some(*global_id),
            _ => None,
        }
    }

    // XXX: We are still allocating for error reporting even though strings are stored in binary
    // It is a minor performance issue, which can be addressed by having the error reporting, not allocate
    pub fn as_str(&self) -> &'static str {
        match self {
            ModuleDefId::FunctionId(_) => "function",
            ModuleDefId::TypeId(_) => "type",
            ModuleDefId::TypeAliasId(_) => "type alias",
            ModuleDefId::TraitId(_) => "trait",
            ModuleDefId::ModuleId(_) => "module",
            ModuleDefId::GlobalId(_) => "global",
        }
    }

    pub fn as_module(&self) -> Option<ModuleId> {
        match self {
            Self::ModuleId(v) => Some(*v),
            _ => None,
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

impl From<TypeAliasId> for ModuleDefId {
    fn from(fid: TypeAliasId) -> Self {
        ModuleDefId::TypeAliasId(fid)
    }
}

impl From<GlobalId> for ModuleDefId {
    fn from(global_id: GlobalId) -> Self {
        ModuleDefId::GlobalId(global_id)
    }
}

impl From<TraitId> for ModuleDefId {
    fn from(trait_id: TraitId) -> Self {
        ModuleDefId::TraitId(trait_id)
    }
}
