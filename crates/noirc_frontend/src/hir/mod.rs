pub mod def_collector;
pub mod def_map;
pub mod resolution;
pub mod scope;
pub mod type_check;

use crate::graph::{CrateGraph, CrateId};
use crate::node_interner::NodeInterner;
use acvm::Language;
use def_map::CrateDefMap;
use fm::FileManager;
use std::collections::HashMap;

/// Helper object which groups together several useful context objects used
/// during name resolution. Once name resolution is finished, only the
/// def_interner is required for type inference and monomorphisation.
#[derive(Default)]
pub struct Context {
    pub def_interner: NodeInterner,
    pub crate_graph: CrateGraph,
    pub(crate) def_maps: HashMap<CrateId, CrateDefMap>,
    pub file_manager: FileManager,
}

impl Context {
    pub fn new(file_manager: FileManager, crate_graph: CrateGraph, language: Language) -> Context {
        let mut ctx = Context {
            def_interner: NodeInterner::default(),
            def_maps: HashMap::new(),
            crate_graph,
            file_manager,
        };
        ctx.def_interner.set_language(&language);
        ctx
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
