pub mod def_collector;
pub mod def_map;
pub mod resolution;
pub mod scope;
pub mod type_check;

use crate::graph::{CrateGraph, CrateId};
use crate::hir_def::function::FuncMeta;
use crate::node_interner::{FuncId, NodeInterner, StructId};
use crate::parser::ParserError;
use crate::ParsedModule;
use def_map::{Contract, CrateDefMap};
use fm::FileManager;
use noirc_errors::Location;
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};

use self::def_map::TestFunction;

pub type ParsedFiles = HashMap<fm::FileId, (ParsedModule, Vec<ParserError>)>;

/// Helper object which groups together several useful context objects used
/// during name resolution. Once name resolution is finished, only the
/// def_interner is required for type inference and monomorphization.
pub struct Context<'file_manager, 'parsed_files> {
    pub def_interner: NodeInterner,
    pub crate_graph: CrateGraph,
    pub(crate) def_maps: BTreeMap<CrateId, CrateDefMap>,
    // In the WASM context, we take ownership of the file manager,
    // which is why this needs to be a Cow. In all use-cases, the file manager
    // is read-only however, once it has been passed to the Context.
    pub file_manager: Cow<'file_manager, FileManager>,

    /// A map of each file that already has been visited from a prior `mod foo;` declaration.
    /// This is used to issue an error if a second `mod foo;` is declared to the same file.
    pub visited_files: BTreeMap<fm::FileId, Location>,

    // A map of all parsed files.
    // Same as the file manager, we take ownership of the parsed files in the WASM context.
    // Parsed files is also read only.
    pub parsed_files: Cow<'parsed_files, ParsedFiles>,
}

#[derive(Debug, Copy, Clone)]
pub enum FunctionNameMatch<'a> {
    Anything,
    Exact(&'a str),
    Contains(&'a str),
}

impl Context<'_, '_> {
    pub fn new(file_manager: FileManager, parsed_files: ParsedFiles) -> Context<'static, 'static> {
        Context {
            def_interner: NodeInterner::default(),
            def_maps: BTreeMap::new(),
            visited_files: BTreeMap::new(),
            crate_graph: CrateGraph::default(),
            file_manager: Cow::Owned(file_manager),
            parsed_files: Cow::Owned(parsed_files),
        }
    }

    pub fn from_ref_file_manager<'file_manager, 'parsed_files>(
        file_manager: &'file_manager FileManager,
        parsed_files: &'parsed_files ParsedFiles,
    ) -> Context<'file_manager, 'parsed_files> {
        Context {
            def_interner: NodeInterner::default(),
            def_maps: BTreeMap::new(),
            visited_files: BTreeMap::new(),
            crate_graph: CrateGraph::default(),
            file_manager: Cow::Borrowed(file_manager),
            parsed_files: Cow::Borrowed(parsed_files),
        }
    }

    pub fn parsed_file_results(&self, file_id: fm::FileId) -> (ParsedModule, Vec<ParserError>) {
        self.parsed_files.get(&file_id).expect("noir file wasn't parsed").clone()
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

        let module_id = self.def_interner.function_module(*id);
        let module = self.module(module_id);

        let parent =
            def_map.get_module_path_with_separator(module_id.local_id.0, module.parent, "::");

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
            let crates = self
                .find_dependencies(crate_id, &module_id.krate)
                .expect("The Struct was supposed to be defined in a dependency");
            crates.join("::") + "::" + &module_path
        }
    }

    /// Recursively walks down the crate dependency graph from crate_id until we reach requested crate
    /// This is needed in case a library (lib1) re-export a structure defined in another library (lib2)
    /// In that case, we will get [lib1,lib2] when looking for a struct defined in lib2,
    /// re-exported by lib1 and used by the main crate.
    /// Returns the path from crate_id to target_crate_id
    fn find_dependencies(
        &self,
        crate_id: &CrateId,
        target_crate_id: &CrateId,
    ) -> Option<Vec<String>> {
        for dep in &self.crate_graph[crate_id].dependencies {
            if &dep.crate_id == target_crate_id {
                return Some(vec![dep.name.to_string()]);
            }
            if let Some(mut path) = self.find_dependencies(&dep.crate_id, target_crate_id) {
                path.insert(0, dep.name.to_string());
                return Some(path);
            }
        }
        None
    }

    pub fn function_meta(&self, func_id: &FuncId) -> &FuncMeta {
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

    pub fn get_all_exported_functions_in_crate(&self, crate_id: &CrateId) -> Vec<(String, FuncId)> {
        let interner = &self.def_interner;
        let def_map = self.def_map(crate_id).expect("The local crate should be analyzed already");

        def_map
            .get_all_exported_functions(interner)
            .map(|function_id| {
                let function_name = self.function_name(&function_id).to_owned();
                (function_name, function_id)
            })
            .collect()
    }

    /// Return a Vec of all `contract` declarations in the source code and the functions they contain
    pub fn get_all_contracts(&self, crate_id: &CrateId) -> Vec<Contract> {
        self.def_map(crate_id)
            .expect("The local crate should be analyzed already")
            .get_all_contracts(&self.def_interner)
    }

    fn module(&self, module_id: def_map::ModuleId) -> &def_map::ModuleData {
        module_id.module(&self.def_maps)
    }
}
