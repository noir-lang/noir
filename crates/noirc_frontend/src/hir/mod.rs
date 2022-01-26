pub mod def_collector;
pub mod def_map;
pub mod resolution;
pub mod scope;
pub mod type_check;

use crate::graph::{CrateGraph, CrateId};
use crate::node_interner::{NodeInterner, TypeId};
use def_map::CrateDefMap;
use fm::FileManager;
use std::collections::HashMap;

/// Global context that is accessible during each stage
#[derive(Debug)]
pub struct Context {
    pub def_interner: NodeInterner,

    pub crate_graph: CrateGraph,

    pub(crate) def_maps: HashMap<CrateId, CrateDefMap>,

    /// Used to generate unique TypeIds for NoirStructs
    pub struct_count: usize,

    pub file_manager: FileManager,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            def_interner: NodeInterner::default(),
            crate_graph: CrateGraph::default(),
            file_manager: FileManager::new(),
            def_maps: HashMap::new(),
            struct_count: 0,
        }
    }
}

impl Context {
    pub fn new(file_manager: FileManager, crate_graph: CrateGraph, struct_count: usize) -> Context {
        Context {
            def_interner: NodeInterner::default(),
            def_maps: HashMap::new(),
            crate_graph,
            file_manager,
            struct_count,
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

    /// Increments the current count and returns the next
    /// unique TypeId
    pub fn next_struct_id(&mut self) -> TypeId {
        let count = self.struct_count;
        self.struct_count += 1;
        TypeId::new(count)
    }
}
