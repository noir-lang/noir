use fm::FileId;
use noirc_errors::{CollectedErrors, DiagnosableError};

use crate::{Ident, NoirFunction, ParsedModule};

use super::{
    dc_crate::{DefCollector, UnresolvedFunctions},
    errors::DefCollectorErrorKind,
};
use crate::hir::def_map::{parse_file, LocalModuleId, ModuleData, ModuleId, ModuleOrigin};
use crate::hir::resolution::import::ImportDirective;
use crate::hir::Context;

/// Given a module collect all definitions into ModuleData
pub struct ModCollector<'a> {
    pub(crate) def_collector: &'a mut DefCollector,
    pub(crate) ast: ParsedModule,
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
            let name = function.name_ident().clone();
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
                .define_func_def(name, func_id)
                .map_err(|(first_def, second_def)| {
                    let err = DefCollectorErrorKind::DuplicateFunction {
                        first_def,
                        second_def,
                    };

                    vec![CollectedErrors {
                        file_id: self.file_id,
                        errors: vec![err.to_diagnostic()],
                    }]
                })?;
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
        mod_name: &Ident,
    ) -> Result<(), Vec<CollectedErrors>> {
        let child_file_id = context
            .file_manager
            .resolve_path(self.file_id, &mod_name.0.contents)
            .map_err(|_| {
                let err = DefCollectorErrorKind::UnresolvedModuleDecl {
                    mod_name: mod_name.clone(),
                };

                vec![CollectedErrors {
                    file_id: self.file_id,
                    errors: vec![err.to_diagnostic()],
                }]
            })?;

        // Parse the AST for the module we just found and then recursively look for it's defs
        let ast = parse_file(&mut context.file_manager, child_file_id)?;

        // Add module into def collector and get a ModuleId
        let child_mod_id = self.push_child_module(mod_name, child_file_id)?;

        ModCollector {
            def_collector: self.def_collector,
            ast,
            file_id: child_file_id,
            module_id: child_mod_id,
        }
        .collect_defs(context)
    }

    pub fn push_child_module(
        &mut self,
        mod_name: &Ident,
        file_id: FileId,
    ) -> Result<LocalModuleId, Vec<CollectedErrors>> {
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
            .insert(mod_name.clone(), LocalModuleId(module_id));

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
            .define_module_def(mod_name.to_owned(), mod_id)
            .map_err(|(first_def, second_def)| {
                let err = DefCollectorErrorKind::DuplicateModuleDecl {
                    first_def,
                    second_def,
                };

                vec![CollectedErrors {
                    file_id: self.file_id,
                    errors: vec![err.to_diagnostic()],
                }]
            })?;

        Ok(LocalModuleId(module_id))
    }
}
