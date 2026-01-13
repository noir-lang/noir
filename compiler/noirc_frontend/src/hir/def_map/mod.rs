use crate::elaborator::FrontendOptions;
use crate::graph::{CrateGraph, CrateId};
use crate::hir::Context;
use crate::hir::def_collector::dc_crate::{CompilationError, DefCollector};
use crate::node_interner::{FuncId, NodeInterner};
use crate::parse_program;
use crate::parser::{ParsedModule, ParserError};
use crate::token::{FunctionAttributeKind, FuzzingScope, TestScope};
use fm::{FileId, FileManager};
use noirc_arena::{Arena, Index};
use noirc_errors::Location;
use std::collections::{BTreeMap, HashMap, HashSet};
mod module_def;
pub use module_def::*;
mod item_scope;
pub use item_scope::*;
mod module_data;
pub use module_data::*;
mod namespace;
pub use namespace::*;

/// The name that is used for a non-contract program's entry-point function.
pub const MAIN_FUNCTION: &str = "main";

// XXX: Ultimately, we want to constrain an index to be of a certain type just like in RA
/// Lets first check if this is offered by any external crate
/// XXX: RA has made this a crate on crates.io
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash, PartialOrd, Ord)]
pub struct LocalModuleId(Index);

impl LocalModuleId {
    pub fn new(index: Index) -> LocalModuleId {
        LocalModuleId(index)
    }

    /// Gets the index that underlies this local module ID.
    pub fn as_index(self) -> Index {
        self.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModuleId {
    pub krate: CrateId,
    pub local_id: LocalModuleId,
}

impl ModuleId {
    pub fn module(self, def_maps: &DefMaps) -> &ModuleData {
        &def_maps[&self.krate].modules()[self.local_id.0]
    }

    /// Returns this module's parent, if there's any.
    pub fn parent(self, def_maps: &DefMaps) -> Option<ModuleId> {
        let module_data = &def_maps[&self.krate][self.local_id];
        module_data.parent.map(|local_id| ModuleId { krate: self.krate, local_id })
    }
}

pub type DefMaps = BTreeMap<CrateId, CrateDefMap>;

/// Map of all modules and scopes defined within a crate.
///
/// The definitions of the crate are accessible indirectly via the scopes of each module.
#[derive(Debug)]
pub struct CrateDefMap {
    krate: CrateId,

    root: LocalModuleId,

    modules: Arena<ModuleData>,

    /// Maps an external dependency's name to its root module id.
    pub(crate) extern_prelude: BTreeMap<String, ModuleId>,
}

impl std::ops::Index<LocalModuleId> for CrateDefMap {
    type Output = ModuleData;
    fn index(&self, local_module_id: LocalModuleId) -> &ModuleData {
        &self.modules[local_module_id.0]
    }
}

impl std::ops::IndexMut<LocalModuleId> for CrateDefMap {
    fn index_mut(&mut self, local_module_id: LocalModuleId) -> &mut ModuleData {
        &mut self.modules[local_module_id.0]
    }
}

impl CrateDefMap {
    /// Constructs a new `CrateDefMap`, containing only the crate's root module.
    ///
    /// # Arguments
    ///
    /// - `krate`: The [CrateId] of the crate for which this `CrateDefMap` refers to.
    /// - `root_module`: The [ModuleData] for the root module of the crate.
    pub fn new(krate: CrateId, root_module: ModuleData) -> CrateDefMap {
        let mut modules = Arena::default();
        let root = LocalModuleId::new(modules.insert(root_module));
        CrateDefMap { krate, root, modules, extern_prelude: BTreeMap::default() }
    }

    /// Collect all definitions in the crate
    pub fn collect_defs(
        crate_id: CrateId,
        context: &mut Context,
        options: FrontendOptions,
    ) -> Vec<CompilationError> {
        // Check if this Crate has already been compiled
        // XXX: There is probably a better alternative for this.
        // Without this check, the compiler will panic as it does not
        // expect the same crate to be processed twice. It would not
        // make the implementation wrong, if the same crate was processed twice, it just makes it slow.
        let mut errors: Vec<CompilationError> = vec![];
        if context.def_map(&crate_id).is_some() {
            return errors;
        }

        // First parse the root file.
        let root_file_id = context.crate_graph[crate_id].root_file_id;
        let (ast, parsing_errors) = context.parsed_file_results(root_file_id);
        let ast = ast.into_sorted();

        let location = Location::new(Default::default(), root_file_id);

        let root_module = ModuleData::new(
            None,
            location,
            Vec::new(),
            ast.inner_attributes.clone(),
            false, // is contract
            false, // is struct
        );
        let def_map = CrateDefMap::new(crate_id, root_module);

        // Now we want to populate the CrateDefMap using the DefCollector
        errors.extend(DefCollector::collect_crate_and_dependencies(
            def_map,
            context,
            ast,
            root_file_id,
            options,
        ));

        errors.extend(parsing_errors.iter().map(|e| e.clone().into()).collect::<Vec<_>>());

        errors
    }

    pub fn root(&self) -> LocalModuleId {
        self.root
    }

    /// Returns a reference to the [ModuleData] stored at [LocalModuleId] `id` or `None` if none exists.
    pub fn get(&self, id: LocalModuleId) -> Option<&ModuleData> {
        self.modules.get(id.0)
    }

    pub fn modules(&self) -> &Arena<ModuleData> {
        &self.modules
    }

    pub fn modules_mut(&mut self) -> &mut Arena<ModuleData> {
        &mut self.modules
    }

    pub(crate) fn insert_module(&mut self, module: ModuleData) -> LocalModuleId {
        LocalModuleId::new(self.modules.insert(module))
    }

    pub fn krate(&self) -> CrateId {
        self.krate
    }

    /// Find the main function for this crate
    pub fn main_function(&self) -> Option<FuncId> {
        let root_module = &self.modules()[self.root.0];

        // This function accepts an Ident, so we attach a dummy span to
        // "main". Equality is implemented only on the contents.
        root_module.find_func_with_name(&MAIN_FUNCTION.into())
    }

    pub fn file_id(&self, module_id: LocalModuleId) -> FileId {
        self.modules[module_id.0].location.file
    }

    pub fn file_ids(&self) -> HashSet<FileId> {
        self.modules.iter().map(|(_, module_data)| module_data.location.file).collect()
    }

    /// Go through all modules in this crate, and find all functions in
    /// each module with the #[test] attribute
    pub fn get_all_test_functions<'a>(
        &'a self,
        interner: &'a NodeInterner,
    ) -> impl Iterator<Item = TestFunction> + 'a {
        self.modules.iter().flat_map(|(_, module)| {
            module.value_definitions().filter_map(|id| {
                if let Some(func_id) = id.as_function() {
                    let has_arguments = !interner.function_meta(&func_id).parameters.is_empty();
                    let attributes = interner.function_attributes(&func_id);
                    match attributes.function().map(|attr| &attr.kind) {
                        Some(FunctionAttributeKind::Test(scope)) => {
                            let location = interner.function_meta(&func_id).name.location;
                            let scope = scope.clone();
                            Some(TestFunction { id: func_id, scope, location, has_arguments })
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            })
        })
    }

    /// Go through all modules in this crate, and find all functions in
    /// each module with the `#[fuzz]` attribute
    pub fn get_all_fuzzing_harnesses<'a>(
        &'a self,
        interner: &'a NodeInterner,
    ) -> impl Iterator<Item = FuzzingHarness> + 'a {
        self.modules.iter().flat_map(|(_, module)| {
            module.value_definitions().filter_map(|id| {
                if let Some(func_id) = id.as_function() {
                    let attributes = interner.function_attributes(&func_id);
                    match attributes.function().map(|attr| &attr.kind) {
                        Some(FunctionAttributeKind::FuzzingHarness(scope)) => {
                            let location = interner.function_meta(&func_id).name.location;
                            Some(FuzzingHarness { id: func_id, scope: scope.clone(), location })
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            })
        })
    }

    /// Go through all modules in this crate, and find all functions in
    /// each module with the `#[export]` attribute
    pub fn get_all_exported_functions<'a>(
        &'a self,
        interner: &'a NodeInterner,
    ) -> impl Iterator<Item = FuncId> + 'a {
        self.modules.iter().flat_map(|(_, module)| {
            module.value_definitions().filter_map(|id| {
                if let Some(func_id) = id.as_function() {
                    let attributes = interner.function_attributes(&func_id);
                    attributes.has_export().then_some(func_id)
                } else {
                    None
                }
            })
        })
    }

    /// Returns an iterator over all contract modules within the crate.
    pub fn get_all_contracts(&self) -> impl Iterator<Item = (LocalModuleId, String)> {
        self.modules.iter().filter_map(|(id, module)| {
            if module.is_contract {
                let name = self.get_module_path(LocalModuleId::new(id), module.parent);
                Some((LocalModuleId(id), name))
            } else {
                None
            }
        })
    }

    /// Find a child module's name by inspecting its parent.
    /// Currently required as modules do not store their own names.
    pub fn get_module_path(
        &self,
        child_id: LocalModuleId,
        parent: Option<LocalModuleId>,
    ) -> String {
        self.get_module_path_with_separator(child_id, parent, ".")
    }

    pub fn get_module_path_with_separator(
        &self,
        child_id: LocalModuleId,
        parent: Option<LocalModuleId>,
        separator: &str,
    ) -> String {
        self.get_module_path_with_separator_inner(child_id.0, parent, separator)
    }

    fn get_module_path_with_separator_inner(
        &self,
        child_id: Index,
        parent: Option<LocalModuleId>,
        separator: &str,
    ) -> String {
        if let Some(id) = parent {
            let parent = &self.modules[id.0];
            let name = parent
                .children
                .iter()
                .find(|(_, id)| id.0 == child_id)
                .map(|(name, _)| name.as_str())
                .expect("Child module was not a child of the given parent module");

            let parent_name =
                self.get_module_path_with_separator_inner(id.0, parent.parent, separator);
            if parent_name.is_empty() {
                name.to_string()
            } else {
                format!("{parent_name}{separator}{name}")
            }
        } else {
            String::new()
        }
    }

    /// Return a topological ordering of each module such that any child modules
    /// are before their parent modules. Sibling modules will respect the ordering
    /// declared from their parent module (the `mod foo; mod bar;` declarations).
    pub fn get_module_topological_order(&self) -> HashMap<LocalModuleId, usize> {
        let mut ordering = HashMap::default();
        self.topologically_sort_modules(self.root, &mut 0, &mut ordering);
        ordering
    }

    fn topologically_sort_modules(
        &self,
        current: LocalModuleId,
        index: &mut usize,
        ordering: &mut HashMap<LocalModuleId, usize>,
    ) {
        for child in &self.modules[current.0].child_declaration_order {
            self.topologically_sort_modules(*child, index, ordering);
        }

        ordering.insert(current, *index);
        *index += 1;
    }
}

pub fn fully_qualified_module_path(
    def_maps: &DefMaps,
    crate_graph: &CrateGraph,
    crate_id: &CrateId,
    module_id: ModuleId,
) -> String {
    let child_id = module_id.local_id;

    let def_map =
        def_maps.get(&module_id.krate).expect("The local crate should be analyzed already");

    let module = &def_map.modules()[module_id.local_id.0];

    let module_path = def_map.get_module_path_with_separator(child_id, module.parent, "::");

    if &module_id.krate == crate_id {
        module_path
    } else {
        let crates = crate_graph
            .find_dependencies(crate_id, &module_id.krate)
            .expect("The module was supposed to be defined in a dependency");
        crates.join("::") + "::" + &module_path
    }
}

/// Given a FileId, fetch the File, from the FileManager and parse it's content
pub fn parse_file(fm: &FileManager, file_id: FileId) -> (ParsedModule, Vec<ParserError>) {
    let file_source = fm.fetch_file(file_id).expect("File does not exist");
    parse_program(file_source, file_id)
}

pub struct TestFunction {
    pub id: FuncId,
    pub scope: TestScope,
    pub location: Location,
    pub has_arguments: bool,
}

impl TestFunction {
    /// Returns true if the test function has been specified to fail
    /// This is done by annotating the function with `#[test(should_fail)]`
    /// or `#[test(should_fail_with = "reason")]`
    pub fn should_fail(&self) -> bool {
        match self.scope {
            TestScope::ShouldFailWith { .. } => true,
            TestScope::OnlyFailWith { .. } | TestScope::None => false,
        }
    }

    /// Returns the reason for the test function to fail if specified
    /// by the user.
    pub fn failure_reason(&self) -> Option<&str> {
        match &self.scope {
            TestScope::None => None,
            TestScope::ShouldFailWith { reason } => reason.as_deref(),
            TestScope::OnlyFailWith { reason } => Some(reason.as_str()),
        }
    }
}

pub struct FuzzingHarness {
    pub id: FuncId,
    pub scope: FuzzingScope,
    pub location: Location,
}

impl FuzzingHarness {
    /// Returns true if the fuzzing harness has been specified to fail only under specific reason
    /// This is done by annotating the function with
    /// `#[fuzz(only_fail_with = "reason")]`
    pub fn only_fail_enabled(&self) -> bool {
        match self.scope {
            FuzzingScope::OnlyFailWith { .. } => true,
            FuzzingScope::None => false,
            FuzzingScope::ShouldFailWith { .. } => false,
        }
    }
    /// Returns true if the fuzzing harness has been specified to fail
    /// This is done by annotating the function with `#[fuzz(should_fail)]`
    /// or `#[fuzz(should_fail_with = "reason")]`
    pub fn should_fail_enabled(&self) -> bool {
        match self.scope {
            FuzzingScope::OnlyFailWith { .. } => false,
            FuzzingScope::None => false,
            FuzzingScope::ShouldFailWith { .. } => true,
        }
    }

    /// Returns the reason for the fuzzing harness to fail if specified
    /// by the user.
    pub fn failure_reason(&self) -> Option<String> {
        match &self.scope {
            FuzzingScope::None => None,
            FuzzingScope::OnlyFailWith { reason } => Some(reason.clone()),
            FuzzingScope::ShouldFailWith { reason } => reason.clone(),
        }
    }
}
