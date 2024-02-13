use fm::FileId;
use noirc_errors::Location;
use rangemap::RangeMap;
use rustc_hash::FxHashMap;

use crate::{macros_api::NodeInterner, node_interner::DependencyId};
use petgraph::prelude::NodeIndex as PetGraphIndex;

#[derive(Debug, Default)]
pub(crate) struct LocationIndices {
    map_file_to_range: FxHashMap<FileId, RangeMap<u32, PetGraphIndex>>,
}

impl LocationIndices {
    pub(crate) fn add_location(&mut self, location: Location, node_index: PetGraphIndex) {
        let range_map = self.map_file_to_range.entry(location.file).or_insert_with(RangeMap::new);
        range_map.insert(location.span.start()..location.span.end(), node_index);
    }

    pub(crate) fn get_node_from_location(&self, location: Location) -> Option<PetGraphIndex> {
        let range_map = self.map_file_to_range.get(&location.file)?;
        Some(*range_map.get(&location.span.start())?)
    }
}

impl NodeInterner {
    pub fn dependency_location(&self, dependency: DependencyId) -> Location {
        match dependency {
            DependencyId::Function(id) => self.function_modifiers(&id).name_location,
            DependencyId::Struct(id) => self.get_struct(id).borrow().location,
            DependencyId::Global(id) => self.get_global(id).location,
            DependencyId::Alias(id) => self.get_type_alias(id).borrow().location,
            DependencyId::Variable(location) => location,
        }
    }

    pub(crate) fn add_reference(&mut self, referenced: DependencyId, reference: DependencyId) {
        let referenced_index = self.get_or_insert_reference(referenced);
        let reference_index = self.reference_graph.add_node(reference);

        let referenced_location = self.dependency_location(referenced);
        let reference_location = self.dependency_location(reference);

        self.reference_graph.add_edge(referenced_index, reference_index, ());
        self.location_indices.add_location(referenced_location, referenced_index);
        self.location_indices.add_location(reference_location, reference_index);
    }

    pub(crate) fn add_reference_for(
        &mut self,
        referenced_id: DependencyId,
        reference: DependencyId,
    ) {
        let Some(referenced_index) = self.reference_graph_indices.get(&referenced_id) else { panic!("Compiler Error: Referenced index not found") };

        let reference_location = self.dependency_location(reference);
        let reference_index = self.reference_graph.add_node(reference);
        self.reference_graph.add_edge(*referenced_index, reference_index, ());
        self.location_indices.add_location(reference_location, reference_index);
    }

    pub(crate) fn add_definition_location(&mut self, referenced: DependencyId) {
        let referenced_index = self.get_or_insert_reference(referenced);
        let referenced_location = self.dependency_location(referenced);
        self.location_indices.add_location(referenced_location, referenced_index);
    }

    #[tracing::instrument(skip(self), ret)]
    pub(crate) fn get_or_insert_reference(&mut self, id: DependencyId) -> PetGraphIndex {
        if let Some(index) = self.reference_graph_indices.get(&id) {
            return *index;
        }

        let index = self.reference_graph.add_node(id);
        self.reference_graph_indices.insert(id, index);
        index
    }

    pub fn check_rename_possible(&self, location: Location) -> bool {
        self.location_indices.get_node_from_location(location).is_some()
    }

    pub fn find_rename_symbols_at(&self, location: Location) -> Option<Vec<Location>> {
        let node_index = self.location_indices.get_node_from_location(location)?;

        let reference_node = self.reference_graph[node_index];
        let found_locations: Vec<Location> = match reference_node {
            DependencyId::Alias(_) | DependencyId::Struct(_) | DependencyId::Global(_) => todo!(),
            DependencyId::Function(_) => self.get_edit_locations(node_index),

            DependencyId::Variable(_) => {
                let referenced_node_index = self
                    .reference_graph
                    .neighbors_directed(node_index, petgraph::Direction::Incoming)
                    .next()?;

                self.get_edit_locations(referenced_node_index)
            }
        };
        Some(found_locations)
    }

    fn get_edit_locations(&self, referenced_node_index: PetGraphIndex) -> Vec<Location> {
        let id = self.reference_graph[referenced_node_index];
        let mut edit_locations = vec![self.dependency_location(id)];

        self.reference_graph
            .neighbors_directed(referenced_node_index, petgraph::Direction::Outgoing)
            .for_each(|reference_node_index| {
                let id = self.reference_graph[reference_node_index];
                edit_locations.push(self.dependency_location(id));
            });
        edit_locations
    }
}
