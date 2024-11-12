use fm::FileId;
use noirc_errors::{Location, Span};
use rangemap::RangeMap;
use rustc_hash::FxHashMap as HashMap;

use crate::{
    ast::{FunctionDefinition, ItemVisibility},
    hir::def_map::{ModuleDefId, ModuleId},
    node_interner::{
        DefinitionId, FuncId, GlobalId, NodeInterner, ReferenceId, StructId, TraitId, TypeAliasId,
    },
};
use petgraph::prelude::NodeIndex as PetGraphIndex;

#[derive(Debug, Default)]
pub(crate) struct LocationIndices {
    map_file_to_range: HashMap<FileId, RangeMap<u32, PetGraphIndex>>,
}

impl LocationIndices {
    pub(crate) fn add_location(&mut self, location: Location, node_index: PetGraphIndex) {
        // Some location spans are empty: maybe they are from fictitious nodes?
        if location.span.start() == location.span.end() {
            return;
        }

        let range_map = self.map_file_to_range.entry(location.file).or_default();
        range_map.insert(location.span.start()..location.span.end(), node_index);
    }

    pub(crate) fn get_node_from_location(&self, location: Location) -> Option<PetGraphIndex> {
        let range_map = self.map_file_to_range.get(&location.file)?;
        Some(*range_map.get(&location.span.start())?)
    }
}

pub struct ReferencesTracker<'a> {
    interner: &'a mut NodeInterner,
    file_id: FileId,
}

impl<'a> ReferencesTracker<'a> {
    pub fn new(interner: &'a mut NodeInterner, file_id: FileId) -> Self {
        Self { interner, file_id }
    }

    pub(crate) fn add_reference(
        &mut self,
        module_def_id: ModuleDefId,
        span: Span,
        is_self_type: bool,
    ) {
        let location = Location::new(span, self.file_id);
        self.interner.add_module_def_id_reference(module_def_id, location, is_self_type);
    }
}

impl NodeInterner {
    pub fn reference_location(&self, reference: ReferenceId) -> Location {
        match reference {
            ReferenceId::Module(id) => self.module_attributes(&id).location,
            ReferenceId::Function(id) => self.function_modifiers(&id).name_location,
            ReferenceId::Struct(id) => {
                let struct_type = self.get_struct(id);
                let struct_type = struct_type.borrow();
                Location::new(struct_type.name.span(), struct_type.location.file)
            }
            ReferenceId::StructMember(id, field_index) => {
                let struct_type = self.get_struct(id);
                let struct_type = struct_type.borrow();
                Location::new(
                    struct_type.field_at(field_index).name.span(),
                    struct_type.location.file,
                )
            }
            ReferenceId::Trait(id) => {
                let trait_type = self.get_trait(id);
                Location::new(trait_type.name.span(), trait_type.location.file)
            }
            ReferenceId::Global(id) => self.get_global(id).location,
            ReferenceId::Alias(id) => {
                let alias_type = self.get_type_alias(id);
                let alias_type = alias_type.borrow();
                Location::new(alias_type.name.span(), alias_type.location.file)
            }
            ReferenceId::Local(id) => self.definition(id).location,
            ReferenceId::Reference(location, _) => location,
        }
    }

    pub fn reference_module(&self, reference: ReferenceId) -> Option<&ModuleId> {
        self.reference_modules.get(&reference)
    }

    pub(crate) fn add_module_def_id_reference(
        &mut self,
        def_id: ModuleDefId,
        location: Location,
        is_self_type: bool,
    ) {
        match def_id {
            ModuleDefId::ModuleId(module_id) => {
                self.add_module_reference(module_id, location);
            }
            ModuleDefId::FunctionId(func_id) => {
                self.add_function_reference(func_id, location);
            }
            ModuleDefId::TypeId(struct_id) => {
                self.add_struct_reference(struct_id, location, is_self_type);
            }
            ModuleDefId::TraitId(trait_id) => {
                self.add_trait_reference(trait_id, location, is_self_type);
            }
            ModuleDefId::TypeAliasId(type_alias_id) => {
                self.add_alias_reference(type_alias_id, location);
            }
            ModuleDefId::GlobalId(global_id) => {
                self.add_global_reference(global_id, location);
            }
        };
    }

    pub(crate) fn add_module_reference(&mut self, id: ModuleId, location: Location) {
        self.add_reference(ReferenceId::Module(id), location, false);
    }

    pub(crate) fn add_struct_reference(
        &mut self,
        id: StructId,
        location: Location,
        is_self_type: bool,
    ) {
        self.add_reference(ReferenceId::Struct(id), location, is_self_type);
    }

    pub(crate) fn add_struct_member_reference(
        &mut self,
        id: StructId,
        member_index: usize,
        location: Location,
    ) {
        self.add_reference(ReferenceId::StructMember(id, member_index), location, false);
    }

    pub(crate) fn add_trait_reference(
        &mut self,
        id: TraitId,
        location: Location,
        is_self_type: bool,
    ) {
        self.add_reference(ReferenceId::Trait(id), location, is_self_type);
    }

    pub(crate) fn add_alias_reference(&mut self, id: TypeAliasId, location: Location) {
        self.add_reference(ReferenceId::Alias(id), location, false);
    }

    pub(crate) fn add_function_reference(&mut self, id: FuncId, location: Location) {
        self.add_reference(ReferenceId::Function(id), location, false);
    }

    pub(crate) fn add_global_reference(&mut self, id: GlobalId, location: Location) {
        self.add_reference(ReferenceId::Global(id), location, false);
    }

    pub(crate) fn add_local_reference(&mut self, id: DefinitionId, location: Location) {
        self.add_reference(ReferenceId::Local(id), location, false);
    }

    pub(crate) fn add_reference(
        &mut self,
        referenced: ReferenceId,
        location: Location,
        is_self_type: bool,
    ) {
        if !self.lsp_mode {
            return;
        }

        let reference = ReferenceId::Reference(location, is_self_type);

        let referenced_index = self.get_or_insert_reference(referenced);
        let reference_location = self.reference_location(reference);
        let reference_index = self.reference_graph.add_node(reference);

        self.reference_graph.add_edge(reference_index, referenced_index, ());
        self.location_indices.add_location(reference_location, reference_index);
    }

    pub(crate) fn add_definition_location(
        &mut self,
        referenced: ReferenceId,
        module_id: Option<ModuleId>,
    ) {
        if !self.lsp_mode {
            return;
        }

        let referenced_index = self.get_or_insert_reference(referenced);
        let referenced_location = self.reference_location(referenced);
        self.location_indices.add_location(referenced_location, referenced_index);
        if let Some(module_id) = module_id {
            self.reference_modules.insert(referenced, module_id);
        }
    }

    #[tracing::instrument(skip(self), ret)]
    pub(crate) fn get_or_insert_reference(&mut self, id: ReferenceId) -> PetGraphIndex {
        if let Some(index) = self.reference_graph_indices.get(&id) {
            return *index;
        }

        let index = self.reference_graph.add_node(id);
        self.reference_graph_indices.insert(id, index);
        index
    }

    // Given a reference location, find the location of the referenced node.
    pub fn find_referenced_location(&self, reference_location: Location) -> Option<Location> {
        self.location_indices
            .get_node_from_location(reference_location)
            .and_then(|node_index| self.referenced_index(node_index))
            .map(|node_index| self.reference_location(self.reference_graph[node_index]))
    }

    // Returns the `ReferenceId` that exists at a given location, if any.
    pub fn reference_at_location(&self, location: Location) -> Option<ReferenceId> {
        self.location_indices.get_node_from_location(location)?;

        let node_index = self.location_indices.get_node_from_location(location)?;
        Some(self.reference_graph[node_index])
    }

    // Starting at the given location, find the node referenced by it. Then, gather
    // all locations that reference that node, and return all of them
    // (the references and optionally the referenced node if `include_referenced` is true).
    // If `include_self_type_name` is true, references where "Self" is written are returned,
    // otherwise they are not.
    // Returns `None` if the location is not known to this interner.
    pub fn find_all_references(
        &self,
        location: Location,
        include_referenced: bool,
        include_self_type_name: bool,
    ) -> Option<Vec<Location>> {
        let referenced_node = self.find_referenced(location)?;
        let referenced_node_index = self.reference_graph_indices[&referenced_node];

        let found_locations = self.find_all_references_for_index(
            referenced_node_index,
            include_referenced,
            include_self_type_name,
        );

        Some(found_locations)
    }

    // Returns the `ReferenceId` that is referenced by the given location, if any.
    pub fn find_referenced(&self, location: Location) -> Option<ReferenceId> {
        let node_index = self.location_indices.get_node_from_location(location)?;

        let reference_node = self.reference_graph[node_index];
        if let ReferenceId::Reference(_, _) = reference_node {
            let node_index = self.referenced_index(node_index)?;
            Some(self.reference_graph[node_index])
        } else {
            Some(reference_node)
        }
    }

    // Given a referenced node index, find all references to it and return their locations, optionally together
    // with the reference node's location if `include_referenced` is true.
    // If `include_self_type_name` is true, references where "Self" is written are returned,
    // otherwise they are not.
    fn find_all_references_for_index(
        &self,
        referenced_node_index: PetGraphIndex,
        include_referenced: bool,
        include_self_type_name: bool,
    ) -> Vec<Location> {
        let id = self.reference_graph[referenced_node_index];
        let mut edit_locations = Vec::new();
        if include_referenced && (include_self_type_name || !id.is_self_type_name()) {
            edit_locations.push(self.reference_location(id));
        }

        self.reference_graph
            .neighbors_directed(referenced_node_index, petgraph::Direction::Incoming)
            .for_each(|reference_node_index| {
                let id = self.reference_graph[reference_node_index];
                if include_self_type_name || !id.is_self_type_name() {
                    edit_locations.push(self.reference_location(id));
                }
            });
        edit_locations
    }

    // Given a reference index, returns the referenced index, if any.
    fn referenced_index(&self, reference_index: PetGraphIndex) -> Option<PetGraphIndex> {
        self.reference_graph
            .neighbors_directed(reference_index, petgraph::Direction::Outgoing)
            .next()
    }

    pub(crate) fn register_module(
        &mut self,
        id: ModuleId,
        visibility: ItemVisibility,
        name: String,
    ) {
        self.register_name_for_auto_import(name, ModuleDefId::ModuleId(id), visibility, None);
    }

    pub(crate) fn register_global(
        &mut self,
        id: GlobalId,
        name: String,
        visibility: ItemVisibility,
        parent_module_id: ModuleId,
    ) {
        self.add_definition_location(ReferenceId::Global(id), Some(parent_module_id));
        self.register_name_for_auto_import(name, ModuleDefId::GlobalId(id), visibility, None);
    }

    pub(crate) fn register_struct(
        &mut self,
        id: StructId,
        name: String,
        visibility: ItemVisibility,
        parent_module_id: ModuleId,
    ) {
        self.add_definition_location(ReferenceId::Struct(id), Some(parent_module_id));
        self.register_name_for_auto_import(name, ModuleDefId::TypeId(id), visibility, None);
    }

    pub(crate) fn register_trait(
        &mut self,
        id: TraitId,
        name: String,
        visibility: ItemVisibility,
        parent_module_id: ModuleId,
    ) {
        self.add_definition_location(ReferenceId::Trait(id), Some(parent_module_id));
        self.register_name_for_auto_import(name, ModuleDefId::TraitId(id), visibility, None);
    }

    pub(crate) fn register_type_alias(
        &mut self,
        id: TypeAliasId,
        name: String,
        visibility: ItemVisibility,
        parent_module_id: ModuleId,
    ) {
        self.add_definition_location(ReferenceId::Alias(id), Some(parent_module_id));
        self.register_name_for_auto_import(name, ModuleDefId::TypeAliasId(id), visibility, None);
    }

    pub(crate) fn register_function(&mut self, id: FuncId, func_def: &FunctionDefinition) {
        let name = func_def.name.0.contents.clone();
        let id = ModuleDefId::FunctionId(id);
        self.register_name_for_auto_import(name, id, func_def.visibility, None);
    }

    pub fn register_name_for_auto_import(
        &mut self,
        name: String,
        module_def_id: ModuleDefId,
        visibility: ItemVisibility,
        defining_module: Option<ModuleId>,
    ) {
        if !self.lsp_mode {
            return;
        }

        let entry = self.auto_import_names.entry(name).or_default();
        entry.push((module_def_id, visibility, defining_module));
    }

    #[allow(clippy::type_complexity)]
    pub fn get_auto_import_names(
        &self,
    ) -> &HashMap<String, Vec<(ModuleDefId, ItemVisibility, Option<ModuleId>)>> {
        &self.auto_import_names
    }
}
