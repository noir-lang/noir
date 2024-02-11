use fm::FileId;
use noirc_errors::Location;
use rangemap::RangeMap;
use rustc_hash::FxHashMap;

use crate::{macros_api::NodeInterner, node_interner::DependencyId};
use petgraph::prelude::NodeIndex as PetGraphIndex;

#[derive(Debug, Default)]
pub(crate) struct LocationStore {
    map_file_to_range: FxHashMap<FileId, RangeMap<u32, PetGraphIndex>>,
}

impl LocationStore {
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
    pub(crate) fn add_reference(
        &mut self,
        referenced: (DependencyId, Location),
        reference: (DependencyId, Location),
    ) {
        let referenced_index = self.get_or_insert_reference(referenced);
        let reference_index = self.graph_references.add_node((reference.0, reference.1));
        self.graph_references.add_edge(referenced_index, reference_index, ());
        self.location_store.add_location(referenced.1, referenced_index);
        self.location_store.add_location(reference.1, reference_index);
    }

    pub(crate) fn add_reference_for(
        &mut self,
        referenced_id: DependencyId,
        reference: (DependencyId, Location),
    ) {
        let Some(referenced_index) = self.graph_references_indices.get(&referenced_id) else { panic!("Compiler Error: Referenced index not found") };

        let reference_index = self.graph_references.add_node((reference.0, reference.1));
        self.graph_references.add_edge(*referenced_index, reference_index, ());
        self.location_store.add_location(reference.1, reference_index);
    }

    pub(crate) fn add_definiton(&mut self, referenced: (DependencyId, Location)) {
        let referenced_index = self.get_or_insert_reference(referenced);
        self.location_store.add_location(referenced.1, referenced_index);
    }

    #[tracing::instrument(skip(self), ret)]
    pub(crate) fn get_or_insert_reference(
        &mut self,
        (id, location): (DependencyId, Location),
    ) -> PetGraphIndex {
        if let Some(index) = self.graph_references_indices.get(&id) {
            return *index;
        }

        let index = self.graph_references.add_node((id, location));
        self.graph_references_indices.insert(id, index);
        index
    }

    pub fn check_rename_possible(&self, location: Location) -> bool {
        self.location_store.get_node_from_location(location).is_some()
    }

    pub fn find_rename_symbols_at(&self, location: Location) -> Option<Vec<Location>> {
        let node_index = self.location_store.get_node_from_location(location)?;

        // let mut edit_locations: Vec<Location> = Vec::new();

        let reference_node = self.graph_references[node_index];
        let found_locations: Vec<Location> = match reference_node.0 {
            DependencyId::Alias(_) | DependencyId::Struct(_) | DependencyId::Global(_) => todo!(),
            DependencyId::Function(_) | DependencyId::GlobalDefinition(_) => {
                self.get_edit_locations(node_index)
            }

            DependencyId::GlobalReference | DependencyId::FunctionCall => {
                let mut edit_locations: Vec<Location> = Vec::new();
                if let Some(referenced_node_index) = self
                    .graph_references
                    .neighbors_directed(node_index, petgraph::Direction::Incoming)
                    .next()
                {
                    edit_locations.extend(self.get_edit_locations(referenced_node_index));
                }
                edit_locations
            }
        };
        Some(found_locations)
    }

    fn get_edit_locations(&self, referenced_node_index: PetGraphIndex) -> Vec<Location> {
        let mut edit_locations: Vec<Location> = Vec::new();
        let (_referenced_id, referencing_location) = self.graph_references[referenced_node_index];
        edit_locations.push(referencing_location);

        self.graph_references
            .neighbors_directed(referenced_node_index, petgraph::Direction::Outgoing)
            .for_each(|reference_node_index| {
                let (_reference_dependency_id, reference_location) =
                    self.graph_references[reference_node_index];
                edit_locations.push(reference_location);
            });
        edit_locations
    }
}
