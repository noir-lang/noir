use std::collections::HashMap;

use fm::FileId;

use crate::{
    node_interner::{FuncId, StmtId, StructId, TyAliasId},
    Ident,
};

use super::{ItemScope, LocalModuleId, ModuleDefId, ModuleId, PerNs};

/// Contains the actual contents of a module: its parent (if one exists),
/// children, and scope with all definitions defined within the scope.
#[derive(Debug, PartialEq, Eq)]
pub struct ModuleData {
    pub parent: Option<LocalModuleId>,
    pub children: HashMap<Ident, LocalModuleId>,

    /// Contains all definitions visible to the current module. This includes
    /// all definitions in self.definitions as well as all imported definitions.
    scope: ItemScope,

    /// Contains only the definitions directly defined in the current module
    definitions: ItemScope,

    pub origin: ModuleOrigin,

    /// True if this module is a `contract Foo { ... }` module containing contract functions
    pub is_contract: bool,
}

impl ModuleData {
    pub fn new(
        parent: Option<LocalModuleId>,
        origin: ModuleOrigin,
        is_contract: bool,
    ) -> ModuleData {
        ModuleData {
            parent,
            children: HashMap::new(),
            scope: ItemScope::default(),
            definitions: ItemScope::default(),
            origin,
            is_contract,
        }
    }

    fn declare(&mut self, name: Ident, item_id: ModuleDefId) -> Result<(), (Ident, Ident)> {
        self.scope.add_definition(name.clone(), item_id)?;

        // definitions is a subset of self.scope so it is expected if self.scope.define_func_def
        // returns without error, so will self.definitions.define_func_def.
        self.definitions.add_definition(name, item_id)
    }

    pub fn declare_function(&mut self, name: Ident, id: FuncId) -> Result<(), (Ident, Ident)> {
        self.declare(name, id.into())
    }

    pub fn declare_global(&mut self, name: Ident, id: StmtId) -> Result<(), (Ident, Ident)> {
        self.declare(name, id.into())
    }

    pub fn declare_struct(&mut self, name: Ident, id: StructId) -> Result<(), (Ident, Ident)> {
        self.declare(name, ModuleDefId::TypeId(id))
    }

    pub fn declare_type_alias(&mut self, name: Ident, id: TyAliasId) -> Result<(), (Ident, Ident)> {
        self.declare(name, id.into())
    }

    pub fn declare_child_module(
        &mut self,
        name: Ident,
        child_id: ModuleId,
    ) -> Result<(), (Ident, Ident)> {
        self.declare(name, child_id.into())
    }

    pub fn find_func_with_name(&self, name: &Ident) -> Option<FuncId> {
        self.scope.find_func_with_name(name)
    }

    pub fn import(&mut self, name: Ident, id: ModuleDefId) -> Result<(), (Ident, Ident)> {
        self.scope.add_item_to_namespace(name, id)
    }

    pub fn find_name(&self, name: &Ident) -> PerNs {
        self.scope.find_name(name)
    }

    /// Return an iterator over all definitions defined within this module,
    /// excluding any type definitions.
    pub fn value_definitions(&self) -> impl Iterator<Item = ModuleDefId> + '_ {
        self.definitions.values().values().map(|(id, _)| *id)
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ModuleOrigin {
    CrateRoot(FileId),
    File(FileId),
}

impl ModuleOrigin {
    pub fn file_id(&self) -> FileId {
        match self {
            ModuleOrigin::CrateRoot(file_id) => *file_id,
            ModuleOrigin::File(file_id) => *file_id,
        }
    }
}

impl From<ModuleOrigin> for FileId {
    fn from(origin: ModuleOrigin) -> Self {
        origin.file_id()
    }
}

impl Default for ModuleOrigin {
    fn default() -> Self {
        ModuleOrigin::CrateRoot(FileId::default())
    }
}
