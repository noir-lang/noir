
use std::collections::HashMap;

use crate_def_map::CrateDefMap;
use crate_graph::{CrateGraph, CrateId};
use fm::FileManager;
use lower::node_interner::NodeInterner;

pub mod resolution;
pub mod scope;
pub mod crate_def_map;
pub mod crate_graph;
pub mod def_collector_crate;
pub mod def_collector_mod;

pub mod type_check;

pub mod lower;

/// Global context that is accessible during each stage
/// XXX: It's possible to have sub-contexts, however it's better to benchmark first.
#[derive(Debug)]
pub struct Context {

    pub def_interner : NodeInterner,

    pub crate_graph : CrateGraph,

    pub(crate) def_maps : HashMap<CrateId, CrateDefMap>,

    file_manager : FileManager,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            def_interner : NodeInterner::default(),
            crate_graph : CrateGraph::default(),
            file_manager : FileManager::new(),
            def_maps : HashMap::new(),
        }
    }
}

impl Context {
    pub fn new(file_manager : FileManager, crate_graph : CrateGraph) -> Context {
        Context {
            def_interner : NodeInterner::default(),
            def_maps : HashMap::new(),
            crate_graph,
            file_manager,
        }
    }

    pub fn file_manager(&mut self) -> &mut FileManager {
        &mut self.file_manager
    }
    pub fn crate_graph(&self) -> &CrateGraph {
        &self.crate_graph
    }
    pub fn def_map(&self, crate_id : CrateId) -> Option<&CrateDefMap> {
        self.def_maps.get(&crate_id)
    }

    /// Return the CrateId of all of the Crates that have been compiled
    pub fn crates(&self) -> impl Iterator<Item = CrateId> + '_ {
        self.crate_graph().iter_keys()
    }
}