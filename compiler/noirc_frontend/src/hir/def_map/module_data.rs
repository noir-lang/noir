use std::collections::HashMap;

use noirc_errors::Location;

use super::{ItemScope, LocalModuleId, ModuleDefId, ModuleId, PerNs};
use crate::ast::{Ident, ItemVisibility};
use crate::node_interner::{FuncId, GlobalId, TraitAssociatedTypeId, TraitId, TypeAliasId, TypeId};
use crate::token::SecondaryAttribute;

/// Contains the actual contents of a module: its parent (if one exists),
/// children, and scope with all definitions defined within the scope.
#[derive(Debug, PartialEq, Eq)]
pub struct ModuleData {
    pub parent: Option<LocalModuleId>,
    pub children: HashMap<Ident, LocalModuleId>,

    /// Each child in the order they were declared in the parent module.
    /// E.g. for a module containing `mod foo; mod bar; mod baz` this would
    /// be `vec![foo, bar, baz]`.
    pub child_declaration_order: Vec<LocalModuleId>,

    /// Contains all definitions visible to the current module. This includes
    /// all definitions in `self.definitions` as well as all imported definitions.
    scope: ItemScope,

    /// Contains only the definitions directly defined in the current module
    definitions: ItemScope,

    /// All traits in scope, either from `use` imports or `trait` declarations.
    /// The Ident value is the trait name or the `use` alias, if any.
    /// This is stored separately from `scope` to quickly check if a trait is in scope.
    traits_in_scope: HashMap<TraitId, Ident>,

    pub location: Location,

    /// True if this module is a `contract Foo { ... }` module containing contract functions
    pub is_contract: bool,

    /// True if this module is actually a type
    pub is_type: bool,

    pub attributes: Vec<SecondaryAttribute>,
}

impl ModuleData {
    pub fn new(
        parent: Option<LocalModuleId>,
        location: Location,
        outer_attributes: Vec<SecondaryAttribute>,
        inner_attributes: Vec<SecondaryAttribute>,
        is_contract: bool,
        is_type: bool,
    ) -> ModuleData {
        let mut attributes = outer_attributes;
        attributes.extend(inner_attributes);

        ModuleData {
            parent,
            children: HashMap::new(),
            child_declaration_order: Vec::new(),
            scope: ItemScope::default(),
            definitions: ItemScope::default(),
            traits_in_scope: HashMap::new(),
            location,
            is_contract,
            is_type,
            attributes,
        }
    }

    pub fn scope(&self) -> &ItemScope {
        &self.scope
    }

    pub fn definitions(&self) -> &ItemScope {
        &self.definitions
    }

    fn declare(
        &mut self,
        name: Ident,
        visibility: ItemVisibility,
        item_id: ModuleDefId,
        trait_id: Option<TraitId>,
    ) -> Result<(), (Ident, Ident)> {
        self.scope.add_definition(name.clone(), visibility, item_id, trait_id)?;

        if let ModuleDefId::ModuleId(child) = item_id {
            self.child_declaration_order.push(child.local_id);
        }

        // definitions is a subset of self.scope so it is expected if self.scope.define_func_def
        // returns without error, so will self.definitions.define_func_def.
        self.definitions.add_definition(name, visibility, item_id, trait_id)
    }

    pub fn declare_function(
        &mut self,
        name: Ident,
        visibility: ItemVisibility,
        id: FuncId,
    ) -> Result<(), (Ident, Ident)> {
        self.declare(name, visibility, id.into(), None)
    }

    pub fn declare_trait_function(
        &mut self,
        name: Ident,
        id: FuncId,
        trait_id: TraitId,
    ) -> Result<(), (Ident, Ident)> {
        self.declare(name, ItemVisibility::Public, id.into(), Some(trait_id))
    }

    pub fn remove_function(&mut self, name: &Ident) {
        self.scope.remove_definition(name);
        self.definitions.remove_definition(name);
    }

    pub fn declare_global(
        &mut self,
        name: Ident,
        visibility: ItemVisibility,
        id: GlobalId,
    ) -> Result<(), (Ident, Ident)> {
        self.declare(name, visibility, id.into(), None)
    }

    pub fn declare_type(
        &mut self,
        name: Ident,
        visibility: ItemVisibility,
        id: TypeId,
    ) -> Result<(), (Ident, Ident)> {
        self.declare(name, visibility, ModuleDefId::TypeId(id), None)
    }

    pub fn declare_type_alias(
        &mut self,
        name: Ident,
        visibility: ItemVisibility,
        id: TypeAliasId,
    ) -> Result<(), (Ident, Ident)> {
        self.declare(name, visibility, id.into(), None)
    }

    pub fn declare_trait(
        &mut self,
        name: Ident,
        visibility: ItemVisibility,
        id: TraitId,
    ) -> Result<(), (Ident, Ident)> {
        self.traits_in_scope.insert(id, name.clone());

        self.declare(name, visibility, ModuleDefId::TraitId(id), None)
    }

    pub fn declare_trait_associated_type(
        &mut self,
        name: Ident,
        id: TraitAssociatedTypeId,
    ) -> Result<(), (Ident, Ident)> {
        self.declare(name, ItemVisibility::Public, id.into(), None)
    }

    pub fn declare_child_module(
        &mut self,
        name: Ident,
        visibility: ItemVisibility,
        child_id: ModuleId,
    ) -> Result<(), (Ident, Ident)> {
        self.declare(name, visibility, child_id.into(), None)
    }

    pub fn find_func_with_name(&self, name: &Ident) -> Option<FuncId> {
        self.scope.find_func_with_name(name)
    }

    pub fn import(
        &mut self,
        name: Ident,
        visibility: ItemVisibility,
        id: ModuleDefId,
        is_prelude: bool,
    ) -> Result<(), (Ident, Ident)> {
        if let ModuleDefId::TraitId(trait_id) = id {
            self.traits_in_scope.insert(trait_id, name.clone());
        }

        self.scope.add_item_to_namespace(name, visibility, id, None, is_prelude)
    }

    pub fn find_name(&self, name: &Ident) -> PerNs {
        self.scope.find_name(name)
    }

    /// Finds a trait in scope and returns its name
    /// (either the trait name, or a `use` alias if it was brought to scope like that)
    pub fn find_trait_in_scope(&self, trait_id: TraitId) -> Option<&Ident> {
        self.traits_in_scope.get(&trait_id)
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
