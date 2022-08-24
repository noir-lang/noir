use fm::FileId;
use noirc_errors::{CollectedErrors, CustomDiagnostic, DiagnosableError};

use crate::{
    graph::CrateId,
    hir::def_collector::dc_crate::UnresolvedStruct,
    node_interner::{StmtId, StructId},
    parser::SubModule,
    Ident, NoirFunction, NoirImpl, NoirStruct, ParsedModule, Statement
};

use super::{
    dc_crate::{DefCollector, UnresolvedFunctions, UnresolvedGlobalConst},
    errors::DefCollectorErrorKind,
};
use crate::hir::def_map::{parse_file, LocalModuleId, ModuleData, ModuleId, ModuleOrigin};
use crate::hir::resolution::import::ImportDirective;
use crate::hir::Context;

/// Given a module collect all definitions into ModuleData
struct ModCollector<'a> {
    pub(crate) def_collector: &'a mut DefCollector,
    pub(crate) file_id: FileId,
    pub(crate) module_id: LocalModuleId,
}

/// Walk a module and collect it's definitions
pub fn collect_defs<'a>(
    def_collector: &'a mut DefCollector,
    ast: &mut ParsedModule,
    file_id: FileId,
    module_id: LocalModuleId,
    crate_id: CrateId,
    context: &mut Context,
    errors: &mut Vec<CollectedErrors>,
) {
    let mut collector = ModCollector { def_collector, file_id, module_id };

    // First resolve the module declarations
    for decl in ast.module_decls.clone() {
        collector.parse_module_declaration(context, &decl, crate_id, errors)
    }

    collector.collect_submodules(context, crate_id, ast.submodules.clone(), file_id, errors);

    // Then add the imports to defCollector to resolve once all modules in the hierarchy have been resolved
    for import in ast.imports.clone() {
        collector.def_collector.collected_imports.push(ImportDirective {
            module_id: collector.module_id,
            path: import.path,
            alias: import.alias,
        });
    }

    collector.collect_structs(ast.types.clone(), crate_id, errors);

    // println!("AST GLOBAL CONSTS: {:?}", ast.global_constants.clone());
    insert_global_constants(&mut ast.functions, ast.global_constants.clone());
    // println!("AST FUNCTIONS after insert global: {:?}", ast.functions.clone());

    let errors_in_same_file = collector.collect_functions(context, ast.functions.clone());

    collector.collect_impls(context, ast.impls.clone());

    if !errors_in_same_file.is_empty() {
        errors.push(CollectedErrors { file_id: collector.file_id, errors: errors_in_same_file });
    }

    for global_const in ast.global_constants.clone() {
        collector.def_collector.collected_consts.push(UnresolvedGlobalConst {
            file_id: collector.file_id,
            module_id: collector.module_id,
            stmt_def: global_const,
        });
    }
}

// NOTE: Possibly do this inside dc_crate where resolution happens to make sure that multiple global_constants are not declared 
fn insert_global_constants(
    functions: &mut Vec<NoirFunction>,
    global_consts: Vec<Statement>,
) {
    for function in functions.into_iter() {
        let mut statements = function.clone().def.body.0;

        for global_const in global_consts.iter() {
            println!("global_const: {:?}", global_const);
            statements.insert(0, global_const.clone());
        }
        function.def.body.0 = statements;
    }
}

impl<'a> ModCollector<'a> {
    fn collect_impls(&mut self, context: &mut Context, impls: Vec<NoirImpl>) {
        for r#impl in impls {
            let mut unresolved_functions =
                UnresolvedFunctions { file_id: self.file_id, functions: Vec::new() };

            for method in r#impl.methods.iter() {
                let func_id = context.def_interner.push_empty_fn();
                unresolved_functions.push_fn(self.module_id, func_id, method.clone());
            }

            let key = (r#impl.type_path.clone(), self.module_id);
            let methods = self.def_collector.collected_impls.entry(key).or_default();
            methods.push(unresolved_functions);
        }
    }

    fn collect_functions(
        &mut self,
        context: &mut Context,
        functions: Vec<NoirFunction>,
    ) -> Vec<CustomDiagnostic> {
        let mut errors = vec![];

        let mut unresolved_functions =
            UnresolvedFunctions { file_id: self.file_id, functions: Vec::new() };

        for function in functions {
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
                    DefCollectorErrorKind::DuplicateFunction { first_def, second_def }
                        .to_diagnostic(),
                );
            }
        }
        // println!("UNRESOLVED FUNCTIONS: {:?}", unresolved_functions.functions);

        self.def_collector.collected_functions.push(unresolved_functions);

        errors
    }

    /// Collect any struct definitions declared within the ast.
    /// Returns a vector of errors if any structs were already defined.
    fn collect_structs(
        &mut self,
        types: Vec<NoirStruct>,
        krate: CrateId,
        errors: &mut Vec<CollectedErrors>,
    ) {
        for struct_definition in types {
            let name = struct_definition.name.clone();

            // Create the corresponding module for the struct namespace
            let id = match self.push_child_module(&name, self.file_id, false) {
                Ok(local_id) => StructId(ModuleId { krate, local_id }),
                Err(mut more_errors) => {
                    errors.append(&mut more_errors);
                    continue;
                }
            };

            // Add the struct to scope so its path can be looked up later
            let result = self.def_collector.def_map.modules[self.module_id.0]
                .scope
                .define_struct_def(name, id);

            if let Err((first_def, second_def)) = result {
                let err = DefCollectorErrorKind::DuplicateFunction { first_def, second_def };

                errors.push(CollectedErrors {
                    file_id: self.file_id,
                    errors: vec![err.to_diagnostic()],
                });
            }

            // And store the TypeId -> StructType mapping somewhere it is reachable
            let unresolved = UnresolvedStruct {
                file_id: self.file_id,
                module_id: self.module_id,
                struct_def: struct_definition,
            };
            self.def_collector.collected_types.insert(id, unresolved);
        }
    }

    fn collect_submodules(
        &mut self,
        context: &mut Context,
        crate_id: CrateId,
        submodules: Vec<SubModule>,
        file_id: FileId,
        errors: &mut Vec<CollectedErrors>,
    ) {
        for mut submodule in submodules {
            match self.push_child_module(&submodule.name, file_id, true) {
                Err(mut more_errors) => errors.append(&mut more_errors),
                Ok(child_mod_id) => collect_defs(
                    self.def_collector,
                    &mut submodule.contents,
                    file_id,
                    child_mod_id,
                    crate_id,
                    context,
                    errors,
                ),
            }
        }
    }

    /// Search for a module named `mod_name`
    /// Parse it, add it as a child to the parent module in which it was declared
    /// and then collect all definitions of the child module
    fn parse_module_declaration(
        &mut self,
        context: &mut Context,
        mod_name: &Ident,
        crate_id: CrateId,
        errors: &mut Vec<CollectedErrors>,
    ) {
        let child_file_id =
            match context.file_manager.resolve_path(self.file_id, &mod_name.0.contents) {
                Ok(child_file_id) => child_file_id,
                Err(_) => {
                    let err =
                        DefCollectorErrorKind::UnresolvedModuleDecl { mod_name: mod_name.clone() };

                    errors.push(CollectedErrors {
                        file_id: self.file_id,
                        errors: vec![err.to_diagnostic()],
                    });
                    return;
                }
            };

        // Parse the AST for the module we just found and then recursively look for it's defs
        let mut ast = parse_file(&mut context.file_manager, child_file_id, errors);

        // Add module into def collector and get a ModuleId
        match self.push_child_module(mod_name, child_file_id, true) {
            Err(mut more_errors) => errors.append(&mut more_errors),
            Ok(child_mod_id) => collect_defs(
                self.def_collector,
                &mut ast,
                child_file_id,
                child_mod_id,
                crate_id,
                context,
                errors,
            ),
        }
    }

    pub fn push_child_module(
        &mut self,
        mod_name: &Ident,
        file_id: FileId,
        add_to_parent_scope: bool,
    ) -> Result<LocalModuleId, Vec<CollectedErrors>> {
        // Create a new default module
        let module_id = self.def_collector.def_map.modules.insert(ModuleData::default());

        let modules = &mut self.def_collector.def_map.modules;

        // Update the child module to reference the parent
        modules[module_id].parent = Some(self.module_id);

        // Update the origin of the child module
        // Also note that the FileId is where this module is defined and not declared
        // To find out where the module was declared, you need to check its parent
        modules[module_id].origin = ModuleOrigin::File(file_id);

        // Update the parent module to reference the child
        modules[self.module_id.0].children.insert(mod_name.clone(), LocalModuleId(module_id));

        // Add this child module into the scope of the parent module as a module definition
        // module definitions are definitions which can only exist at the module level.
        // ModuleDefinitionIds can be used across crates since they contain the CrateId
        //
        // We do not want to do this in the case of struct modules (each struct type corresponds
        // to a child module containing its methods) since the module name should not shadow
        // the struct name.
        if add_to_parent_scope {
            let mod_id = ModuleId {
                krate: self.def_collector.def_map.krate,
                local_id: LocalModuleId(module_id),
            };
            modules[self.module_id.0]
                .scope
                .define_module_def(mod_name.to_owned(), mod_id)
                .map_err(|(first_def, second_def)| {
                    let err = DefCollectorErrorKind::DuplicateModuleDecl { first_def, second_def };

                    vec![CollectedErrors {
                        file_id: self.file_id,
                        errors: vec![err.to_diagnostic()],
                    }]
                })?;
        }

        Ok(LocalModuleId(module_id))
    }
}
