use super::{
    crate_graph::CrateId, def_collector_crate::DefCollector, lower::node_interner::FuncId, Context,
};

use crate::{parser::Program, Parser};
use arena::{Arena, Index};
use fm::{FileId, FileManager};
use noirc_errors::{CollectedErrors, DiagnosableError};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
// XXX: Ultimately, we want to constrain an index to be of a certain type just like in RA
/// Lets first check if this is offered by any external crate
pub struct LocalModuleId(pub Index);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
    // XXX: We will need to pass in a CrateManager to give access to the other crates
    // Each crate gives itself a LocalModuleId, independently.
    //
    pub fn collect_defs(
        crate_id: CrateId,
        context: &mut Context,
    ) -> Result<(), Vec<CollectedErrors>> {
        // Check if this Crate has already been compiled
        // XXX: There is probably a better alternative for this.
        // Without this check, the compiler will panic as it does not
        // expect the same crate to be processed twice. It does not
        // make the implementation wrong, it just makes it slow.
        if context.def_map(crate_id).is_some() {
            return Ok(());
        }

        let root_file_id = context.crate_graph[crate_id].root_file_id;

        // First parse the root file into an AST
        let ast = parse_root_file(&mut context.file_manager(), root_file_id)?;

        // Allocate a default Module for the root
        let mut modules: Arena<ModuleData> = Arena::default();
        let root = modules.insert(ModuleData::default());

        let def_map = CrateDefMap {
            root: LocalModuleId(root),
            modules,
            krate: crate_id,
            extern_prelude: HashMap::new(),
        };

        // Now we want to populate the CrateDefMap using the DefCollector
        //
        DefCollector::collect(def_map, context, ast, root_file_id)
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

    // Find the main function for this module
    pub fn main_function(&self) -> Option<FuncId> {
        const MAIN_FUNCTION: &str = "main";

        let root_module = &self.modules()[self.root.0];

        root_module.scope.find_func_with_name(MAIN_FUNCTION)
    }

    pub fn root_file_id(&self) -> FileId {
        let root_module = &self.modules()[self.root.0];
        root_module.origin.into()
    }
}
/// Fetch the crate root and parse the file
pub fn parse_root_file(
    fm: &mut FileManager,
    root_file_id: FileId,
) -> Result<Program, Vec<CollectedErrors>> {
    let file = fm.fetch_file(root_file_id);
    let mut parser = Parser::from_src(file.get_source());
    match parser.parse_program() {
        Ok(prog) => Ok(prog),
        Err(errs) => {
            let root_file_errs = CollectedErrors {
                file_id: root_file_id,
                errors: errs.into_iter().map(|err| err.to_diagnostic()).collect(),
            };

            Err(vec![root_file_errs])
        }
    }
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

#[derive(Default, Debug, PartialEq, Eq)]
pub struct ModuleData {
    pub parent: Option<LocalModuleId>,
    pub children: HashMap<String, LocalModuleId>,
    pub scope: ItemScope,

    pub origin: ModuleOrigin,
}

// This works exactly the same as in r-a, just simplified
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct PerNs {
    pub types: Option<(ModuleDefId, Visibility)>,
    pub values: Option<(ModuleDefId, Visibility)>,
}

impl PerNs {
    pub fn types(t: ModuleDefId) -> PerNs {
        PerNs {
            types: Some((t, Visibility::Public)),
            values: None,
        }
    }

    pub fn take_types(self) -> Option<ModuleDefId> {
        self.types.map(|it| it.0)
    }

    pub fn take_values(self) -> Option<ModuleDefId> {
        self.values.map(|it| it.0)
    }

    pub fn iter_defs(self) -> impl Iterator<Item = ModuleDefId> {
        self.types
            .map(|it| it.0)
            .into_iter()
            .chain(self.values.map(|it| it.0).into_iter())
    }

    pub fn iter_items(self) -> impl Iterator<Item = (ModuleDefId, Visibility)> {
        self.types
            .map(|it| it)
            .into_iter()
            .chain(self.values.map(|it| it).into_iter())
    }

    pub fn is_none(&self) -> bool {
        self.types.is_none() && self.values.is_none()
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct ItemScope {
    types: HashMap<String, (ModuleDefId, Visibility)>,
    values: HashMap<String, (ModuleDefId, Visibility)>,

    defs: Vec<ModuleDefId>,
}

impl ItemScope {
    pub fn add_definition(&mut self, name: String, mod_def: ModuleDefId) {
        self.add_item_to_namespace(name, mod_def).unwrap();
        self.defs.push(mod_def);
    }

    pub fn add_item_to_namespace(
        &mut self,
        name: String,
        mod_def: ModuleDefId,
    ) -> Result<(), String> {
        let old_value = match &mod_def {
            ModuleDefId::ModuleId(_) => self
                .types
                .insert(name.clone(), (mod_def, Visibility::Public)),
            ModuleDefId::FunctionId(_) => self
                .values
                .insert(name.clone(), (mod_def, Visibility::Public)),
        };
        match old_value {
            None => Ok(()),
            Some(_) => {
                // XXX: If a module has the same function name twice, this error will trigger or module def.
                // Not an ice, but a user defined error
                Err(name)
            }
        }
    }

    pub fn define_module_def(&mut self, name: String, mod_id: ModuleId) {
        self.add_definition(name, mod_id.into())
    }
    pub fn define_func_def(&mut self, name: String, local_id: FuncId) {
        self.add_definition(name, local_id.into())
    }

    pub fn find_module_with_name(&self, mod_name: &str) -> Option<&ModuleId> {
        let (module_def, _) = self.types.get(mod_name)?;
        match module_def {
            ModuleDefId::ModuleId(id) => Some(id),
            _ => None,
        }
    }
    pub fn find_func_with_name(&self, func_name: &str) -> Option<FuncId> {
        let (module_def, _) = self.values.get(func_name)?;
        match module_def {
            ModuleDefId::FunctionId(id) => Some(*id),
            _ => None,
        }
    }
    pub fn find_name(&self, name: &str) -> PerNs {
        PerNs {
            types: self.types.get(name).cloned(),
            values: self.values.get(name).cloned(),
        }
    }

    pub fn definitions(&self) -> Vec<ModuleDefId> {
        self.defs.clone()
    }
    pub fn types(&self) -> &HashMap<String, (ModuleDefId, Visibility)> {
        &self.types
    }
    pub fn values(&self) -> &HashMap<String, (ModuleDefId, Visibility)> {
        &self.values
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Visibility {
    Public,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ModuleOrigin {
    CrateRoot(FileId),
    File(FileId),
}

impl Into<FileId> for ModuleOrigin {
    fn into(self) -> FileId {
        match self {
            ModuleOrigin::CrateRoot(file_id) => file_id,
            ModuleOrigin::File(file_id) => file_id,
        }
    }
}

impl Default for ModuleOrigin {
    fn default() -> Self {
        ModuleOrigin::CrateRoot(FileId::default())
    }
}
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ModuleDefId {
    ModuleId(ModuleId),
    FunctionId(FuncId),
}

impl ModuleDefId {
    pub fn as_function(&self) -> Option<FuncId> {
        if let ModuleDefId::FunctionId(func_id) = self {
            return Some(*func_id);
        }
        return None;
    }
    // XXX: We are still allocating fro error reporting even though strings are stored in binary
    // It is a minor performance issue, which can be addressed by having the error reporting, not allocate
    pub fn as_str(&self) -> &'static str {
        match self {
            ModuleDefId::FunctionId(_) => "function",
            ModuleDefId::ModuleId(_) => "module",
        }
    }
}

impl Into<ModuleDefId> for ModuleId {
    fn into(self) -> ModuleDefId {
        ModuleDefId::ModuleId(self)
    }
}
impl Into<ModuleDefId> for FuncId {
    fn into(self) -> ModuleDefId {
        ModuleDefId::FunctionId(self)
    }
}
