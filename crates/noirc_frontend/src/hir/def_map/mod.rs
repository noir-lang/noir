use crate::graph::CrateId;
use crate::hir::def_collector::dc_crate::DefCollector;
use crate::hir::Context;
use crate::node_interner::{FuncId, NodeInterner};
use crate::parser::{parse_program, ParsedModule};
use crate::token::Attribute;
use arena::{Arena, Index};
use fm::{FileId, FileManager};
use noirc_errors::FileDiagnostic;
use std::collections::HashMap;

mod module_def;
pub use module_def::*;
mod item_scope;
pub use item_scope::*;
mod module_data;
pub use module_data::*;
mod namespace;
pub use namespace::*;
// XXX: Ultimately, we want to constrain an index to be of a certain type just like in RA
/// Lets first check if this is offered by any external crate
/// XXX: RA has made this a crate on crates.io
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct LocalModuleId(pub Index);

impl LocalModuleId {
    pub fn dummy_id() -> LocalModuleId {
        LocalModuleId(Index::from_raw_parts(std::usize::MAX, std::u64::MAX))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ModuleId {
    pub krate: CrateId,
    pub local_id: LocalModuleId,
}

#[derive(Debug)]
pub struct CrateDefMap {
    pub(crate) root: LocalModuleId,

    pub(crate) modules: Arena<ModuleData>,

    pub(crate) krate: CrateId,

    pub(crate) extern_prelude: HashMap<String, ModuleId>,
}

impl CrateDefMap {
    /// Collect all definitions in the crate
    pub fn collect_defs(
        crate_id: CrateId,
        context: &mut Context,
        errors: &mut Vec<FileDiagnostic>,
    ) {
        // Check if this Crate has already been compiled
        // XXX: There is probably a better alternative for this.
        // Without this check, the compiler will panic as it does not
        // expect the same crate to be processed twice. It would not
        // make the implementation wrong, if the same crate was processed twice, it just makes it slow.
        if context.def_map(crate_id).is_some() {
            return;
        }

        // First parse the root file.
        let root_file_id = context.crate_graph[crate_id].root_file_id;
        let ast = parse_file(&mut context.file_manager, root_file_id, errors);

        // Allocate a default Module for the root, giving it a ModuleId
        let mut modules: Arena<ModuleData> = Arena::default();
        let root = modules.insert(ModuleData::default());

        // Set the origin of the root module
        modules[root].origin = ModuleOrigin::CrateRoot(root_file_id);

        let def_map = CrateDefMap {
            root: LocalModuleId(root),
            modules,
            krate: crate_id,
            extern_prelude: HashMap::new(),
        };

        // Now we want to populate the CrateDefMap using the DefCollector
        DefCollector::collect(def_map, context, ast, root_file_id, errors);
    }

    pub fn root(&self) -> LocalModuleId {
        self.root
    }
    pub fn modules(&self) -> &Arena<ModuleData> {
        &self.modules
    }
    pub fn krate(&self) -> CrateId {
        self.krate
    }

    /// Find the main function for this crate
    pub fn main_function(&self) -> Option<FuncId> {
        const MAIN_FUNCTION: &str = "main";

        let root_module = &self.modules()[self.root.0];

        // This function accepts an Ident, so we attach a dummy span to
        // "main". Equality is implemented only on the contents.
        root_module.scope.find_func_with_name(&MAIN_FUNCTION.into())
    }

    pub fn root_file_id(&self) -> FileId {
        let root_module = &self.modules()[self.root.0];
        root_module.origin.into()
    }

    pub fn module_file_id(&self, module_id: LocalModuleId) -> FileId {
        self.modules[module_id.0].origin.file_id()
    }

    /// Go through all modules in this crate, and find all functions in
    /// each module with the #[test] attribute
    pub fn get_all_test_functions<'a>(
        &'a self,
        interner: &'a NodeInterner,
    ) -> impl Iterator<Item = FuncId> + 'a {
        self.modules.iter().flat_map(|(_, module)| {
            let functions = module.scope.values().values().filter_map(|(id, _)| id.as_function());
            functions.filter(|id| interner.function_meta(id).attributes == Some(Attribute::Test))
        })
    }
}

/// Given a FileId, fetch the File, from the FileManager and parse it's content
pub fn parse_file(
    fm: &mut FileManager,
    file_id: FileId,
    all_errors: &mut Vec<FileDiagnostic>,
) -> ParsedModule {
    let file = fm.fetch_file(file_id);
    let (program, errors) = parse_program(file.get_source());
    all_errors.extend(errors.into_iter().map(|error| error.in_file(file_id)));
    program
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
