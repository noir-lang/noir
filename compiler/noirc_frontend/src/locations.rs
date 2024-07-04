use fm::FileId;
use noirc_errors::Location;
use rangemap::RangeMap;
use rustc_hash::FxHashMap;

use crate::{macros_api::NodeInterner, node_interner::ReferenceId};
use petgraph::prelude::NodeIndex as PetGraphIndex;

#[derive(Debug, Default)]
pub(crate) struct LocationIndices {
    map_file_to_range: FxHashMap<FileId, RangeMap<u32, PetGraphIndex>>,
}

impl LocationIndices {
    pub(crate) fn add_location(&mut self, location: Location, node_index: PetGraphIndex) {
        // Some location spans are empty: maybe they are from ficticious nodes?
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

impl NodeInterner {
    pub fn reference_location(&self, reference: ReferenceId) -> Location {
        match reference {
            ReferenceId::Module(id) => self.module_location(&id),
            ReferenceId::Function(id) => self.function_modifiers(&id).name_location,
            ReferenceId::Struct(id) => self.struct_location(&id),
            ReferenceId::Trait(id) => self.trait_location(&id),
            ReferenceId::Global(id) => self.get_global(id).location,
            ReferenceId::Alias(id) => self.get_type_alias(id).borrow().location,
            ReferenceId::Variable(location) => location,
        }
    }

    pub(crate) fn add_reference(&mut self, referenced: ReferenceId, reference: ReferenceId) {
        if !self.track_references {
            return;
        }

        let referenced_index = self.get_or_insert_reference(referenced);
        let reference_location = self.reference_location(reference);
        let reference_index = self.reference_graph.add_node(reference);

        self.reference_graph.add_edge(reference_index, referenced_index, ());
        self.location_indices.add_location(reference_location, reference_index);
    }

    pub(crate) fn add_definition_location(&mut self, referenced: ReferenceId) {
        if !self.track_references {
            return;
        }

        let referenced_index = self.get_or_insert_reference(referenced);
        let referenced_location = self.reference_location(referenced);
        self.location_indices.add_location(referenced_location, referenced_index);
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

    // Is the given location known to this interner?
    pub fn is_location_known(&self, location: Location) -> bool {
        self.location_indices.get_node_from_location(location).is_some()
    }

    // Starting at the given location, find the node referenced by it. Then, gather
    // all locations that reference that node, and return all of them
    // (the references and optionally the reference node if `include_reference` is true).
    // Returns `None` if the location is not known to this interner.
    pub fn find_all_references(
        &self,
        location: Location,
        include_reference: bool,
    ) -> Option<Vec<Location>> {
        let node_index = self.location_indices.get_node_from_location(location)?;

        let reference_node = self.reference_graph[node_index];
        let found_locations: Vec<Location> = match reference_node {
            ReferenceId::Alias(_) | ReferenceId::Global(_) | ReferenceId::Module(_) => todo!(),
            ReferenceId::Function(_) | ReferenceId::Struct(_) | ReferenceId::Trait(_) => {
                self.find_all_references_for_index(node_index, include_reference)
            }

            ReferenceId::Variable(_) => {
                let referenced_node_index = self.referenced_index(node_index)?;
                self.find_all_references_for_index(referenced_node_index, include_reference)
            }
        };
        Some(found_locations)
    }

    // Given a referenced node index, find all references to it and return their locations, optionally together
    // with the reference node's location if `include_reference` is true.
    fn find_all_references_for_index(
        &self,
        referenced_node_index: PetGraphIndex,
        include_reference: bool,
    ) -> Vec<Location> {
        let id = self.reference_graph[referenced_node_index];
        let mut edit_locations = Vec::new();
        if include_reference {
            edit_locations.push(self.reference_location(id));
        }

        self.reference_graph
            .neighbors_directed(referenced_node_index, petgraph::Direction::Incoming)
            .for_each(|reference_node_index| {
                let id = self.reference_graph[reference_node_index];
                edit_locations.push(self.reference_location(id));
            });
        edit_locations
    }

    // Given a reference index, returns the referenced index, if any.
    fn referenced_index(&self, reference_index: PetGraphIndex) -> Option<PetGraphIndex> {
        self.reference_graph
            .neighbors_directed(reference_index, petgraph::Direction::Outgoing)
            .next()
    }
}
