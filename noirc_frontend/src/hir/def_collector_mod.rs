use fm::FileId;

use crate::{NoirFunction, Program};

use super::{
    crate_def_map::ModuleId,
    def_collector_crate::{CollectedErrors, UnresolvedFunctions},
    resolution::import::ImportDirective,
};

use super::{
    crate_def_map::{parse_root_file, LocalModuleId, ModuleData, ModuleOrigin},
    def_collector_crate::DefCollector,
    Context,
};

/// Given a module collect all definitions into ModuleData
pub struct ModCollector<'a> {
    pub(crate) def_collector: &'a mut DefCollector,
    pub(crate) ast: Program,
    pub(crate) file_id: FileId,
    pub(crate) module_id: LocalModuleId,
}

impl<'a> ModCollector<'a> {
    /// Walk a module and collect it's definitions
    pub fn collect_defs(&mut self, context: &mut Context) -> Result<(), Vec<CollectedErrors>> {
        // First resolve the module declarations
        // XXX: to avoid clone, possibly destructure the AST and pass in `self` for mod collector instead of `&mut self`
        // Alternatively, pass in the AST as a reference
        for decl in self.ast.module_decls.clone() {
            self.parse_module_declaration(context, &decl)?
        }

        // Then add the imports to defCollector to resolve once all modules in the hierarchy have been resolved
        for import in self.ast.imports.clone() {
            self.def_collector.collected_imports.push(ImportDirective {
                module_id: self.module_id,
                path: import.path,
                alias: import.alias,
            });
        }

        // Then add functions to functionArena
        let mut unresolved_functions = UnresolvedFunctions {
            file_id: self.file_id,
            functions: Vec::new(),
        };
        for function in self.ast.functions.clone() {
            let name = function.name().to_owned();
            let nf: NoirFunction = function.into();

            // First create dummy function in the DefInterner
            // So that we can get a FuncId
            let func_id = context.def_interner.push_empty_fn();

            // Now link this func_id to a crate level map with the noir function and the module id
            // Encountering a NoirFunction, we retrieve it's module_data to get the namespace
            // Once we have lowered it to a HirFunction, we retrieve it's Id from the DefInterner
            // and replace it
            // With this method we iterate each function in the Crate and not each module
            // This may not be great because we have to pull the module_data for each function
            unresolved_functions.push_fn(self.module_id, func_id, nf);

            // Add function to scope/ns of the module
            self.def_collector.def_map.modules[self.module_id.0]
                .scope
                .define_func_def(name, func_id);
        }
        self.def_collector
            .collected_functions
            .push(unresolved_functions);

        Ok(())
    }
    /// Search for a module named `mod_name`
    /// Parse it, add it as a child to the parent module in which it was declared
    /// and then collect all definitions of the child module
    fn parse_module_declaration(
        &mut self,
        context: &mut Context,
        mod_name: &str,
    ) -> Result<(), Vec<CollectedErrors>> {
        let new_file_id = context
            .file_manager()
            .resolve_path(self.file_id, mod_name)
            .unwrap();

        // Parse the AST for the module we just found and then recursively look for it's defs
        let ast = parse_root_file(context.file_manager(), new_file_id)?;

        // Add module into def collector and get a ModuleId
        let new_mod_id = self.push_child_module(mod_name, new_file_id);

        ModCollector {
            def_collector: self.def_collector,
            ast,
            file_id: new_file_id,
            module_id: new_mod_id,
        }
        .collect_defs(context)
    }

    pub fn push_child_module(&mut self, mod_name: &str, file_id: FileId) -> LocalModuleId {
        // Create a new default module
        let module_id = self
            .def_collector
            .def_map
            .modules
            .insert(ModuleData::default());

        let modules = &mut self.def_collector.def_map.modules;

        // Update the child module to reference the parent
        modules[module_id].parent = Some(self.module_id);

        // Update the origin of the child module
        // Note: We do not support inline modules
        // Also note that the FileId is where this module is defined and not declared
        // To fnd out where the module was declared, you need to check its parent
        modules[module_id].origin = ModuleOrigin::File(file_id);

        // Update the parent module to reference the child
        modules[self.module_id.0]
            .children
            .insert(mod_name.to_owned(), LocalModuleId(module_id));

        // Add this child module into the scope of the parent module as a module definition
        // module definitions are definitions which can only exist at the module level.
        // ModuleDefinitionIds can be used across crates since they contain the CrateId
        let mod_id = ModuleId {
            krate: self.def_collector.def_map.krate,
            local_id: LocalModuleId(module_id),
        }
        .into();
        modules[self.module_id.0]
            .scope
            .define_module_def(mod_name.to_owned(), mod_id);

        LocalModuleId(module_id)
    }
}
