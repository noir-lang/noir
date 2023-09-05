pub mod def_collector;
pub mod def_map;
pub mod resolution;
pub mod scope;
pub mod type_check;

use crate::graph::{CrateGraph, CrateId, Dependency};
use crate::hir_def::function::FuncMeta;
use crate::node_interner::{FuncId, NodeInterner, StructId};
use def_map::{Contract, CrateDefMap};
use fm::FileManager;
use std::collections::BTreeMap;

use self::def_map::TestFunction;

/// Helper object which groups together several useful context objects used
/// during name resolution. Once name resolution is finished, only the
/// def_interner is required for type inference and monomorphization.
pub struct Context {
    pub def_interner: NodeInterner,
    pub crate_graph: CrateGraph,
    pub(crate) def_maps: BTreeMap<CrateId, CrateDefMap>,
    pub file_manager: FileManager,

    /// Maps a given (contract) module id to the next available storage slot
    /// for that contract.
    pub storage_slots: BTreeMap<def_map::ModuleId, StorageSlot>,
}

#[derive(Debug, Copy, Clone)]
pub enum FunctionNameMatch<'a> {
    Anything,
    Exact(&'a str),
    Contains(&'a str),
}

pub type StorageSlot = u32;

impl Context {
    pub fn new(file_manager: FileManager, crate_graph: CrateGraph) -> Context {
        Context {
            def_interner: NodeInterner::default(),
            def_maps: BTreeMap::new(),
            crate_graph,
            file_manager,
            storage_slots: BTreeMap::new(),
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

    pub fn root_crate_id(&self) -> &CrateId {
        self.crate_graph.root_crate_id()
    }

    pub fn stdlib_crate_id(&self) -> &CrateId {
        self.crate_graph.stdlib_crate_id()
    }

    // TODO: Decide if we actually need `function_name` and `fully_qualified_function_name`
    pub fn function_name(&self, id: &FuncId) -> &str {
        self.def_interner.function_name(id)
    }

    pub fn fully_qualified_function_name(&self, crate_id: &CrateId, id: &FuncId) -> String {
        let def_map = self.def_map(crate_id).expect("The local crate should be analyzed already");

        let name = self.def_interner.function_name(id);

        let meta = self.def_interner.function_meta(id);
        let module = self.module(meta.module_id);

        let parent =
            def_map.get_module_path_with_separator(meta.module_id.local_id.0, module.parent, "::");

        if parent.is_empty() {
            name.into()
        } else {
            format!("{parent}::{name}")
        }
    }

    /// Returns a fully-qualified path to the given [StructId] from the given [CrateId]. This function also
    /// account for the crate names of dependencies.
    ///
    /// For example, if you project contains a `main.nr` and `foo.nr` and you provide the `main_crate_id` and the
    /// `bar_struct_id` where the `Bar` struct is inside `foo.nr`, this function would return `foo::Bar` as a [String].
    pub fn fully_qualified_struct_path(&self, crate_id: &CrateId, id: StructId) -> String {
        let module_id = id.module_id();
        let child_id = module_id.local_id.0;
        let def_map =
            self.def_map(&module_id.krate).expect("The local crate should be analyzed already");

        let module = self.module(module_id);

        let module_path = def_map.get_module_path_with_separator(child_id, module.parent, "::");

        if &module_id.krate == crate_id {
            module_path
        } else {
            let crate_name = &self.crate_graph[crate_id]
                .dependencies
                .iter()
                .find_map(|dep| match dep {
                    Dependency { name, crate_id } if crate_id == &module_id.krate => Some(name),
                    _ => None,
                })
                .expect("The Struct was supposed to be defined in a dependency");
            format!("{crate_name}::{module_path}")
        }
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

        local_crate.main_function()
    }

    /// Returns a list of all functions in the current crate marked with #[test]
    /// whose names contain the given pattern string. An empty pattern string
    /// will return all functions marked with #[test].
    pub fn get_all_test_functions_in_crate_matching(
        &self,
        crate_id: &CrateId,
        pattern: FunctionNameMatch,
    ) -> Vec<(String, TestFunction)> {
        let interner = &self.def_interner;
        let def_map = self.def_map(crate_id).expect("The local crate should be analyzed already");

        def_map
            .get_all_test_functions(interner)
            .filter_map(|test_function| {
                let fully_qualified_name =
                    self.fully_qualified_function_name(crate_id, &test_function.get_id());
                match &pattern {
                    FunctionNameMatch::Anything => Some((fully_qualified_name, test_function)),
                    FunctionNameMatch::Exact(pattern) => (&fully_qualified_name == pattern)
                        .then_some((fully_qualified_name, test_function)),
                    FunctionNameMatch::Contains(pattern) => fully_qualified_name
                        .contains(pattern)
                        .then_some((fully_qualified_name, test_function)),
                }
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
