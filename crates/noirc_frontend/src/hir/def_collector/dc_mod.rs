use fm::FileId;
use noirc_errors::{CollectedErrors, CustomDiagnostic, DiagnosableError};

use crate::{Ident, ParsedModule, StructType};

use super::{
    dc_crate::{DefCollector, UnresolvedFunctions},
    errors::DefCollectorErrorKind,
};
use crate::hir::def_map::{parse_file, LocalModuleId, ModuleData, ModuleId, ModuleOrigin};
use crate::hir::resolution::import::ImportDirective;
use crate::hir::Context;

/// Given a module collect all definitions into ModuleData
struct ModCollector<'a> {
    pub(crate) def_collector: &'a mut DefCollector,
    pub(crate) ast: ParsedModule,
    pub(crate) file_id: FileId,
    pub(crate) module_id: LocalModuleId,
}

/// Walk a module and collect it's definitions
///
/// ast.modules and ast.imports will be empty afterwards!
pub fn collect_defs<'a>(
    def_collector: &'a mut DefCollector,
    mut ast: ParsedModule,
    file_id: FileId,
    module_id: LocalModuleId,
    context: &mut Context,
    errors: &mut Vec<CollectedErrors>,
) {
    let modules = std::mem::take(&mut ast.module_decls);
    let imports = std::mem::take(&mut ast.imports);
    let mut collector = ModCollector {
        def_collector,
        ast,
        file_id,
        module_id,
    };

    // First resolve the module declarations
    for decl in modules {
        collector.parse_module_declaration(context, &decl, errors)
    }

    // Then add the imports to defCollector to resolve once all modules in the hierarchy have been resolved
    for import in imports {
        collector
            .def_collector
            .collected_imports
            .push(ImportDirective {
                module_id: collector.module_id,
                path: import.path,
                alias: import.alias,
            });
    }

    let mut errors_in_same_file = collector.collect_structs(context);
    errors_in_same_file.extend(collector.collect_functions(context));
    collector.collect_impls(context);

    if !errors_in_same_file.is_empty() {
        errors.push(CollectedErrors {
            file_id: collector.file_id,
            errors: errors_in_same_file,
        });
    }
}

impl<'a> ModCollector<'a> {
    fn collect_impls(&mut self, context: &mut Context) {
        for r#impl in self.ast.impls.iter() {
            let mut unresolved_functions = UnresolvedFunctions {
                file_id: self.file_id,
                functions: Vec::new(),
            };

            for method in r#impl.methods.iter() {
                let func_id = context.def_interner.push_empty_fn();
                unresolved_functions.push_fn(self.module_id, func_id, method.clone());
            }

            let key = (r#impl.type_path.clone(), self.module_id);
            let methods = self.def_collector.collected_impls.entry(key).or_default();
            methods.push(unresolved_functions);
        }
    }

    fn collect_functions(&mut self, context: &mut Context) -> Vec<CustomDiagnostic> {
        let mut errors = vec![];

        let mut unresolved_functions = UnresolvedFunctions {
            file_id: self.file_id,
            functions: Vec::new(),
        };

        for function in self.ast.functions.clone() {
            let name = function.name_ident().clone();

            // First create dummy function in the DefInterner
            // So that we can get a FuncId
            let func_id = context.def_interner.push_empty_fn();

            // Now link this func_id to a crate level map with the noir function and the module id
            // Encountering a NoirFunction, we retrieve it's module_data to get the namespace
            // Once we have lowered it to a HirFunction, we retrieve it's Id from the DefInterner
            // and replace it
            // With this method we iterate each function in the Crate and not each module
            // This may not be great because we have to pull the module_data for each function
            unresolved_functions.push_fn(self.module_id, func_id, function);

            // Add function to scope/ns of the module
            let result = self.def_collector.def_map.modules[self.module_id.0]
                .scope
                .define_func_def(name, func_id);

            if let Err((first_def, second_def)) = result {
                errors.push(
                    DefCollectorErrorKind::DuplicateFunction {
                        first_def,
                        second_def,
                    }
                    .to_diagnostic(),
                );
            }
        }

        self.def_collector
            .collected_functions
            .push(unresolved_functions);

        errors
    }

    /// Collect any struct definitions declared within the ast.
    /// Returns a vector of errors if any structs were already defined.
    fn collect_structs(&mut self, context: &mut Context) -> Vec<CustomDiagnostic> {
        let mut errors = vec![];

        for struct_definition in self.ast.types.iter() {
            let id = context.next_struct_id();
            let typ = StructType::new(id, struct_definition.clone());

            // Add the struct to scope so its path can be looked up later
            let result = self.def_collector.def_map.modules[self.module_id.0]
                .scope
                .define_struct_def(typ.name.clone(), id);

            if let Err((first_def, second_def)) = result {
                let err = DefCollectorErrorKind::DuplicateFunction {
                    first_def,
                    second_def,
                };

                errors.push(err.to_diagnostic());
            }

            // And store the TypeId -> StructType mapping somewhere it is reachable
            self.def_collector.collected_types.insert(id, typ);
        }

        errors
    }

    /// Search for a module named `mod_name`
    /// Parse it, add it as a child to the parent module in which it was declared
    /// and then collect all definitions of the child module
    fn parse_module_declaration(
        &mut self,
        context: &mut Context,
        mod_name: &Ident,
        errors: &mut Vec<CollectedErrors>,
    ) {
        let child_file_id = match context
            .file_manager
            .resolve_path(self.file_id, &mod_name.0.contents)
        {
            Ok(child_file_id) => child_file_id,
            Err(_) => {
                let err = DefCollectorErrorKind::UnresolvedModuleDecl {
                    mod_name: mod_name.clone(),
                };

                errors.push(CollectedErrors {
                    file_id: self.file_id,
                    errors: vec![err.to_diagnostic()],
                });
                return;
            }
        };

        // Parse the AST for the module we just found and then recursively look for it's defs
        let ast = parse_file(&mut context.file_manager, child_file_id, errors);

        // Add module into def collector and get a ModuleId
        match self.push_child_module(mod_name, child_file_id) {
            Err(mut more_errors) => errors.append(&mut more_errors),
            Ok(child_mod_id) => collect_defs(
                self.def_collector,
                ast,
                child_file_id,
                child_mod_id,
                context,
                errors,
            ),
        }
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
        // To find out where the module was declared, you need to check its parent
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
        };
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
