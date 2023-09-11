use crate::node_interner::{FuncId, StmtId, StructId, TraitId, TypeAliasId};

use super::ModuleId;

/// A generic ID that references either a module, function, type, interface or global
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ModuleDefId {
    ModuleId(ModuleId),
    FunctionId(FuncId),
    TypeId(StructId),
    TypeAliasId(TypeAliasId),
    TraitId(TraitId),
    GlobalId(StmtId),
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

    pub fn as_global(&self) -> Option<StmtId> {
        match self {
            ModuleDefId::GlobalId(stmt_id) => Some(*stmt_id),
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

impl From<StmtId> for ModuleDefId {
    fn from(stmt_id: StmtId) -> Self {
        ModuleDefId::GlobalId(stmt_id)
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

impl TryFromModuleDefId for TypeAliasId {
    fn try_from(id: ModuleDefId) -> Option<Self> {
        id.as_type_alias()
    }

    fn dummy_id() -> Self {
        TypeAliasId::dummy_id()
    }

    fn description() -> String {
        "type alias".to_string()
    }
}

impl TryFromModuleDefId for TraitId {
    fn try_from(id: ModuleDefId) -> Option<Self> {
        id.as_trait()
    }

    fn dummy_id() -> Self {
        TraitId::dummy_id()
    }

    fn description() -> String {
        "trait".to_string()
    }
}

impl TryFromModuleDefId for StmtId {
    fn try_from(id: ModuleDefId) -> Option<Self> {
        id.as_global()
    }

    fn dummy_id() -> Self {
        StmtId::dummy_id()
    }

    fn description() -> String {
        "global".to_string()
    }
}
