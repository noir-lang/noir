use std::collections::HashMap;

use noirc_errors::Location;

use crate::{
    node_interner::{FuncId, GlobalId, StructId, TraitId, TypeAliasId},
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

    pub location: Location,

    /// True if this module is a `contract Foo { ... }` module containing contract functions
    pub is_contract: bool,
}

impl ModuleData {
    pub fn new(parent: Option<LocalModuleId>, location: Location, is_contract: bool) -> ModuleData {
        ModuleData {
            parent,
            children: HashMap::new(),
            scope: ItemScope::default(),
            definitions: ItemScope::default(),
            location,
            is_contract,
        }
    }

    pub(crate) fn scope(&self) -> &ItemScope {
        &self.scope
    }

    fn declare(
        &mut self,
        name: Ident,
        item_id: ModuleDefId,
        trait_id: Option<TraitId>,
    ) -> Result<(), (Ident, Ident)> {
        self.scope.add_definition(name.clone(), item_id, trait_id)?;

        // definitions is a subset of self.scope so it is expected if self.scope.define_func_def
        // returns without error, so will self.definitions.define_func_def.
        self.definitions.add_definition(name, item_id, trait_id)
    }

    pub fn declare_function(&mut self, name: Ident, id: FuncId) -> Result<(), (Ident, Ident)> {
        self.declare(name, id.into(), None)
    }

    pub fn declare_trait_function(
        &mut self,
        name: Ident,
        id: FuncId,
        trait_id: TraitId,
    ) -> Result<(), (Ident, Ident)> {
        self.declare(name, id.into(), Some(trait_id))
    }

    pub fn remove_function(&mut self, name: &Ident) {
        self.scope.remove_definition(name);
        self.definitions.remove_definition(name);
    }

    pub fn declare_global(&mut self, name: Ident, id: GlobalId) -> Result<(), (Ident, Ident)> {
        self.declare(name, id.into(), None)
    }

    pub fn declare_struct(&mut self, name: Ident, id: StructId) -> Result<(), (Ident, Ident)> {
        self.declare(name, ModuleDefId::TypeId(id), None)
    }

    pub fn declare_type_alias(
        &mut self,
        name: Ident,
        id: TypeAliasId,
    ) -> Result<(), (Ident, Ident)> {
        self.declare(name, id.into(), None)
    }

    pub fn declare_trait(&mut self, name: Ident, id: TraitId) -> Result<(), (Ident, Ident)> {
        self.declare(name, ModuleDefId::TraitId(id), None)
    }

    pub fn declare_child_module(
        &mut self,
        name: Ident,
        child_id: ModuleId,
    ) -> Result<(), (Ident, Ident)> {
        self.declare(name, child_id.into(), None)
    }

    pub fn find_func_with_name(&self, name: &Ident) -> Option<FuncId> {
        self.scope.find_func_with_name(name)
    }

    pub fn import(
        &mut self,
        name: Ident,
        id: ModuleDefId,
        is_prelude: bool,
    ) -> Result<(), (Ident, Ident)> {
        self.scope.add_item_to_namespace(name, id, None, is_prelude)
    }

    pub fn find_name(&self, name: &Ident) -> PerNs {
        self.scope.find_name(name)
    }

    pub fn type_definitions(&self) -> impl Iterator<Item = ModuleDefId> + '_ {
        self.definitions.types().values().flat_map(|a| a.values().map(|(id, _, _)| *id))
    }

    /// Return an iterator over all definitions defined within this module,
    /// excluding any type definitions.
    pub fn value_definitions(&self) -> impl Iterator<Item = ModuleDefId> + '_ {
        self.definitions.values().values().flat_map(|a| a.values().map(|(id, _, _)| *id))
    }
}
