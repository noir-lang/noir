pub mod def_collector;
pub mod def_map;
pub mod resolution;
pub mod scope;
pub mod type_check;

use crate::graph::{CrateGraph, CrateId, CrateType};
use crate::hir_def::function::FuncMeta;
use crate::node_interner::{FuncId, NodeInterner};
use def_map::{Contract, CrateDefMap};
use fm::FileManager;
use std::collections::HashMap;

/// Helper object which groups together several useful context objects used
/// during name resolution. Once name resolution is finished, only the
/// def_interner is required for type inference and monomorphization.
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
    pub fn def_map(&self, crate_id: &CrateId) -> Option<&CrateDefMap> {
        self.def_maps.get(crate_id)
    }

    /// Return the CrateId for each crate that has been compiled
    /// successfully
    pub fn crates(&self) -> impl Iterator<Item = CrateId> + '_ {
        self.crate_graph.iter_keys()
    }

    pub fn function_name(&self, id: &FuncId) -> &str {
        self.def_interner.function_name(id)
    }

    pub fn function_meta(&self, func_id: &FuncId) -> FuncMeta {
        self.def_interner.function_meta(func_id)
    }

    /// Returns the FuncId of the 'main' function in a crate.
    /// - Expects check_crate to be called beforehand
    /// - Panics if no main function is found
    pub fn get_main_function(&self, crate_id: &CrateId) -> Option<FuncId> {
        // Find the local crate, one should always be present
        let local_crate = self.def_map(crate_id).unwrap();

        // Check the crate type
        // We don't panic here to allow users to `evaluate` libraries which will do nothing
        if matches!(self.crate_graph[*crate_id].crate_type, CrateType::Binary) {
            // All Binaries should have a main function
            local_crate.main_function()
        } else {
            None
        }
    }

    /// Returns a list of all functions in the current crate marked with #[test]
    /// whose names contain the given pattern string. An empty pattern string
    /// will return all functions marked with #[test].
    pub fn get_all_test_functions_in_crate_matching(
        &self,
        crate_id: &CrateId,
        pattern: &str,
    ) -> Vec<(String, FuncId)> {
        let interner = &self.def_interner;
        let def_map = self.def_map(crate_id).expect("The local crate should be analyzed already");

        def_map
            .get_all_test_functions(interner)
            .filter_map(|id| {
                let name = interner.function_name(&id);

                let meta = interner.function_meta(&id);
                let module = self.module(meta.module_id);

                let parent = def_map.get_module_path_with_separator(
                    meta.module_id.local_id.0,
                    module.parent,
                    "::",
                );
                let path =
                    if parent.is_empty() { name.into() } else { format!("{parent}::{name}") };

                path.contains(pattern).then_some((path, id))
            })
            .collect()
    }

    /// Return a Vec of all `contract` declarations in the source code and the functions they contain
    pub fn get_all_contracts(&self, crate_id: &CrateId) -> Vec<Contract> {
        self.def_map(crate_id)
            .expect("The local crate should be analyzed already")
            .get_all_contracts()
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
