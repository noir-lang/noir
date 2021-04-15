pub mod def_collector;
pub mod def_map;
pub mod resolution;
pub mod scope;
pub mod type_check;

use crate::graph::{CrateGraph, CrateId};
use crate::node_interner::NodeInterner;
use def_map::CrateDefMap;
use fm::FileManager;
use noir_field::FieldElement;
use std::collections::HashMap;

/// Global context that is accessible during each stage
#[derive(Debug)]
pub struct Context<F: FieldElement> {
    pub def_interner: NodeInterner<F>,

    pub crate_graph: CrateGraph,

    pub(crate) def_maps: HashMap<CrateId, CrateDefMap>,

    pub file_manager: FileManager,
}

impl<F: FieldElement> Default for Context<F> {
    fn default() -> Self {
        Context {
            def_interner: NodeInterner::default(),
            crate_graph: CrateGraph::default(),
            file_manager: FileManager::new(),
            def_maps: HashMap::new(),
        }
    }
}

impl<F: FieldElement> Context<F> {
    pub fn new(file_manager: FileManager, crate_graph: CrateGraph) -> Context<F> {
        Context {
            def_interner: NodeInterner::default(),
            def_maps: HashMap::new(),
            crate_graph,
            file_manager,
        }
    }
    /// Returns the CrateDefMap for a given CrateId.
    /// It is perfectly valid for the compiler to look
    /// up a CrateDefMap and it is not available.
    /// This is how the compiler knows to compile a Crate.
    pub fn def_map(&self, crate_id: CrateId) -> Option<&CrateDefMap> {
        self.def_maps.get(&crate_id)
    }

    /// Return the CrateId for each crate that has been compiled
    /// successfully
    pub fn crates(&self) -> impl Iterator<Item = CrateId> + '_ {
        self.crate_graph.iter_keys()
    }
}
