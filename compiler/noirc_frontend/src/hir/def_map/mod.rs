use crate::graph::CrateId;
use crate::hir::def_collector::dc_crate::{CompilationError, DefCollector};
use crate::hir::Context;
use crate::node_interner::{FuncId, GlobalId, NodeInterner, StructId};
use crate::parse_program;
use crate::parser::{ParsedModule, ParserError};
use crate::token::{FunctionAttribute, SecondaryAttribute, TestScope};
use fm::{FileId, FileManager};
use noirc_arena::{Arena, Index};
use noirc_errors::Location;
use std::collections::{BTreeMap, HashMap};
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
pub struct LocalModuleId(pub Index);

impl LocalModuleId {
    pub fn dummy_id() -> LocalModuleId {
        LocalModuleId(Index::dummy())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ModuleId {
    pub krate: CrateId,
    pub local_id: LocalModuleId,
}

impl ModuleId {
    pub fn dummy_id() -> ModuleId {
        ModuleId { krate: CrateId::dummy_id(), local_id: LocalModuleId::dummy_id() }
    }

    pub fn module(self, def_maps: &DefMaps) -> &ModuleData {
        &def_maps[&self.krate].modules()[self.local_id.0]
    }

    /// Returns this module's parent, if there's any.
    pub fn parent(self, def_maps: &DefMaps) -> Option<ModuleId> {
        let module_data = &def_maps[&self.krate].modules()[self.local_id.0];
        module_data.parent.map(|local_id| ModuleId { krate: self.krate, local_id })
    }
}

pub type DefMaps = BTreeMap<CrateId, CrateDefMap>;

/// Map of all modules and scopes defined within a crate.
///
/// The definitions of the crate are accessible indirectly via the scopes of each module.
#[derive(Debug)]
pub struct CrateDefMap {
    pub(crate) root: LocalModuleId,

    pub(crate) modules: Arena<ModuleData>,

    pub(crate) krate: CrateId,

    /// Maps an external dependency's name to its root module id.
    pub(crate) extern_prelude: BTreeMap<String, ModuleId>,
}

impl CrateDefMap {
    /// Collect all definitions in the crate
    pub fn collect_defs(
        crate_id: CrateId,
        context: &mut Context,
        debug_comptime_in_file: Option<&str>,
        error_on_unused_imports: bool,
    ) -> Vec<(CompilationError, FileId)> {
        // Check if this Crate has already been compiled
        // XXX: There is probably a better alternative for this.
        // Without this check, the compiler will panic as it does not
        // expect the same crate to be processed twice. It would not
        // make the implementation wrong, if the same crate was processed twice, it just makes it slow.
        let mut errors: Vec<(CompilationError, FileId)> = vec![];
        if context.def_map(&crate_id).is_some() {
            return errors;
        }

        // First parse the root file.
        let root_file_id = context.crate_graph[crate_id].root_file_id;
        let (ast, parsing_errors) = context.parsed_file_results(root_file_id);
        let ast = ast.into_sorted();

        // Allocate a default Module for the root, giving it a ModuleId
        let mut modules: Arena<ModuleData> = Arena::default();
        let location = Location::new(Default::default(), root_file_id);
        let root = modules.insert(ModuleData::new(
            None,
            location,
            Vec::new(),
            ast.inner_attributes.clone(),
            false, // is contract
            false, // is struct
        ));

        let def_map = CrateDefMap {
            root: LocalModuleId(root),
            modules,
            krate: crate_id,
            extern_prelude: BTreeMap::new(),
        };

        // Now we want to populate the CrateDefMap using the DefCollector
        errors.extend(DefCollector::collect_crate_and_dependencies(
            def_map,
            context,
            ast,
            root_file_id,
            debug_comptime_in_file,
            error_on_unused_imports,
        ));

        errors.extend(
            parsing_errors.iter().map(|e| (e.clone().into(), root_file_id)).collect::<Vec<_>>(),
        );

        errors
    }

    pub fn root(&self) -> LocalModuleId {
        self.root
    }
    pub fn modules(&self) -> &Arena<ModuleData> {
        &self.modules
    }

    pub fn modules_mut(&mut self) -> &mut Arena<ModuleData> {
        &mut self.modules
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

    /// Go through all modules in this crate, and find all functions in
    /// each module with the #[test] attribute
    pub fn get_all_test_functions<'a>(
        &'a self,
        interner: &'a NodeInterner,
    ) -> impl Iterator<Item = TestFunction> + 'a {
        self.modules.iter().flat_map(|(_, module)| {
            module.value_definitions().filter_map(|id| {
                if let Some(func_id) = id.as_function() {
                    let attributes = interner.function_attributes(&func_id);
                    match attributes.function() {
                        Some(FunctionAttribute::Test(scope)) => {
                            let location = interner.function_meta(&func_id).name.location;
                            Some(TestFunction::new(func_id, scope.clone(), location))
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
    /// each module with the #[export] attribute
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

    /// Go through all modules in this crate, find all `contract ... { ... }` declarations,
    /// and collect them all into a Vec.
    pub fn get_all_contracts(&self, interner: &NodeInterner) -> Vec<Contract> {
        self.modules
            .iter()
            .filter_map(|(id, module)| {
                if module.is_contract {
                    let functions = module
                        .value_definitions()
                        .filter_map(|id| {
                            id.as_function().map(|function_id| {
                                let is_entry_point = interner
                                    .function_attributes(&function_id)
                                    .is_contract_entry_point();
                                ContractFunctionMeta { function_id, is_entry_point }
                            })
                        })
                        .collect();

                    let mut outputs =
                        ContractOutputs { structs: HashMap::new(), globals: HashMap::new() };

                    interner.get_all_globals().iter().for_each(|global_info| {
                        interner.global_attributes(&global_info.id).iter().for_each(|attr| {
                            if let SecondaryAttribute::Abi(tag) = attr {
                                if let Some(tagged) = outputs.globals.get_mut(tag) {
                                    tagged.push(global_info.id);
                                } else {
                                    outputs.globals.insert(tag.to_string(), vec![global_info.id]);
                                }
                            }
                        });
                    });

                    module.type_definitions().for_each(|id| {
                        if let ModuleDefId::TypeId(struct_id) = id {
                            interner.struct_attributes(&struct_id).iter().for_each(|attr| {
                                if let SecondaryAttribute::Abi(tag) = attr {
                                    if let Some(tagged) = outputs.structs.get_mut(tag) {
                                        tagged.push(struct_id);
                                    } else {
                                        outputs.structs.insert(tag.to_string(), vec![struct_id]);
                                    }
                                }
                            });
                        }
                    });

                    let name = self.get_module_path(id, module.parent);
                    Some(Contract { name, location: module.location, functions, outputs })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Find a child module's name by inspecting its parent.
    /// Currently required as modules do not store their own names.
    pub fn get_module_path(&self, child_id: Index, parent: Option<LocalModuleId>) -> String {
        self.get_module_path_with_separator(child_id, parent, ".")
    }

    pub fn get_module_path_with_separator(
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
                .map(|(name, _)| &name.0.contents)
                .expect("Child module was not a child of the given parent module");

            let parent_name = self.get_module_path_with_separator(id.0, parent.parent, separator);
            if parent_name.is_empty() {
                name.to_string()
            } else {
                format!("{parent_name}{separator}{name}")
            }
        } else {
            String::new()
        }
    }
}

/// Specifies a contract function and extra metadata that
/// one can use when processing a contract function.
///
/// One of these is whether the contract function is an entry point.
/// The caller should only type-check these functions and not attempt
/// to create a circuit for them.
pub struct ContractFunctionMeta {
    pub function_id: FuncId,
    /// Indicates whether the function is an entry point
    pub is_entry_point: bool,
}

pub struct ContractOutputs {
    pub structs: HashMap<String, Vec<StructId>>,
    pub globals: HashMap<String, Vec<GlobalId>>,
}

/// A 'contract' in Noir source code with a given name, functions and events.
/// This is not an AST node, it is just a convenient form to return for CrateDefMap::get_all_contracts.
pub struct Contract {
    /// To keep `name` semi-unique, it is prefixed with the names of parent modules via CrateDefMap::get_module_path
    pub name: String,
    pub location: Location,
    pub functions: Vec<ContractFunctionMeta>,
    pub outputs: ContractOutputs,
}

/// Given a FileId, fetch the File, from the FileManager and parse it's content
pub fn parse_file(fm: &FileManager, file_id: FileId) -> (ParsedModule, Vec<ParserError>) {
    let file_source = fm.fetch_file(file_id).expect("File does not exist");
    parse_program(file_source)
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

pub struct TestFunction {
    id: FuncId,
    scope: TestScope,
    location: Location,
}

impl TestFunction {
    fn new(id: FuncId, scope: TestScope, location: Location) -> Self {
        TestFunction { id, scope, location }
    }

    /// Returns the function id of the test function
    pub fn get_id(&self) -> FuncId {
        self.id
    }

    pub fn file_id(&self) -> FileId {
        self.location.file
    }

    /// Returns true if the test function has been specified to fail
    /// This is done by annotating the function with `#[test(should_fail)]`
    /// or `#[test(should_fail_with = "reason")]`
    pub fn should_fail(&self) -> bool {
        match self.scope {
            TestScope::ShouldFailWith { .. } => true,
            TestScope::None => false,
        }
    }

    /// Returns the reason for the test function to fail if specified
    /// by the user.
    pub fn failure_reason(&self) -> Option<&str> {
        match &self.scope {
            TestScope::None => None,
            TestScope::ShouldFailWith { reason } => reason.as_deref(),
        }
    }
}
