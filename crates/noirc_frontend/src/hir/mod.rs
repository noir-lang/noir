pub mod def_collector;
pub mod def_map;
pub mod resolution;
pub mod scope;
pub mod type_check;

use crate::graph::{CrateGraph, CrateId};
use crate::node_interner::NodeInterner;
use def_map::CrateDefMap;
use fm::FileManager;
use std::collections::HashMap;

/// Helper object which groups together several useful context objects used
/// during name resolution. Once name resolution is finished, only the
/// def_interner is required for type inference and monomorphization.
#[derive(Default)]
pub struct Context {
    pub def_interner: NodeInterner,
    pub crate_graph: CrateGraph,
    pub(crate) def_maps: HashMap<CrateId, CrateDefMap>,
    pub file_manager: FileManager,

    /// Maps a given (contract) module id to the next available storage slot
    /// for that contract.
    pub storage_slots: HashMap<def_map::ModuleId, StorageSlot>,
}

pub type StorageSlot = u32;

impl Context {
    pub fn new(file_manager: FileManager, crate_graph: CrateGraph) -> Context {
        Context {
            def_interner: NodeInterner::default(),
            def_maps: HashMap::new(),
            crate_graph,
            file_manager,
            storage_slots: HashMap::new(),
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

    fn module(&self, module_id: def_map::ModuleId) -> &def_map::ModuleData {
        module_id.module(&self.def_maps)
    }

    /// Returns the next available storage slot in the given module.
    /// Returns None if the given module is not a contract module.
    fn next_storage_slot(&mut self, module_id: def_map::ModuleId) -> Option<StorageSlot> {
        let module = self.module(module_id);

        module.is_contract.then(|| {
            let next_slot = self.storage_slots.entry(module_id).or_insert(0);
            *next_slot += 1;
            *next_slot
        })
    }
}
