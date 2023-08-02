use std::fmt::format;

use fm::FileId;
use noirc_errors::FileDiagnostic;

use crate::{
    graph::CrateId,
    hir::def_collector::dc_crate::{UnresolvedStruct, UnresolvedTrait},
    node_interner::{StructId, TraitId},
    parser::SubModule,
    Ident, LetStatement, NoirFunction, NoirStruct, NoirTrait, NoirTypeAlias, ParsedModule,
    TraitConstraint, TraitImpl, TraitImplItem, TraitItem, TypeImpl, UnresolvedType,
};

use super::{
    dc_crate::{DefCollector, UnresolvedFunctions, UnresolvedGlobal, UnresolvedTypeAlias},
    errors::DefCollectorErrorKind,
};
use crate::hir::def_map::{
    parse_file, LocalModuleId, ModuleData, ModuleDefId, ModuleId, ModuleOrigin,
};
use crate::hir::resolution::import::ImportDirective;
use crate::hir::Context;

/// Given a module collect all definitions into ModuleData
struct ModCollector<'a> {
    pub(crate) def_collector: &'a mut DefCollector,
    pub(crate) file_id: FileId,
    pub(crate) module_id: LocalModuleId,
}

/// Walk a module and collect its definitions.
///
/// This performs the entirety of the definition collection phase of the name resolution pass.
pub fn collect_defs(
    def_collector: &mut DefCollector,
    ast: ParsedModule,
    file_id: FileId,
    module_id: LocalModuleId,
    crate_id: CrateId,
    context: &mut Context,
    errors: &mut Vec<FileDiagnostic>,
) {
    let mut collector = ModCollector { def_collector, file_id, module_id };

    // First resolve the module declarations
    for decl in ast.module_decls {
        collector.parse_module_declaration(context, &decl, crate_id, errors);
    }

    collector.collect_submodules(context, crate_id, ast.submodules, file_id, errors);

    // Then add the imports to defCollector to resolve once all modules in the hierarchy have been resolved
    for import in ast.imports {
        collector.def_collector.collected_imports.push(ImportDirective {
            module_id: collector.module_id,
            path: import.path,
            alias: import.alias,
        });
    }

    collector.collect_globals(context, ast.globals, errors);

    collector.collect_traits(ast.traits, crate_id, errors);

    collector.collect_structs(ast.types, crate_id, errors);

    collector.collect_type_aliases(context, ast.type_aliases, errors);

    collector.collect_functions(context, ast.functions, errors);

    collector.collect_trait_impls(context, ast.trait_impls, errors);

    collector.collect_impls(context, ast.impls);
}

fn check_trait_method_implementation_generics(
    _generics: &Vec<Ident>,
    _noir_function: &NoirFunction,
    _trait_name: &String,
) -> Result<(), DefCollectorErrorKind> {
    // TODO
    Ok(())
}

fn check_trait_method_implementation_parameters(
    parameters: &Vec<(Ident, UnresolvedType)>,
    noir_function: &NoirFunction,
    trait_name: &String,
) -> Result<(), DefCollectorErrorKind> {
    if noir_function.def.parameters.len() != parameters.len() {
        return Err(DefCollectorErrorKind::SimpleError {
            primary_message: format!("Mismatch signature [Number of parameters] of method with name `{}` that implemetns trait `{}`", noir_function.name(), trait_name),
            secondary_message: "".to_string(),
            span: noir_function.name_ident().span(),
        });
    }
    let mut count = 0;
    for (pattern, typ, _abi_vis) in &noir_function.def.parameters {
        let (expected_name, expected_type) = &parameters[count];
        if pattern.name_ident().0.contents != expected_name.0.contents {
            // we allow different namings of parameters
        }
        if typ != expected_type {
            return Err(DefCollectorErrorKind::SimpleError {
                primary_message: format!(
                    "Mismatch signature of method {} that implemtns trait {}",
                    noir_function.name(),
                    trait_name,
                ),
                secondary_message: format!(
                    "`{}: {}` expected",
                    pattern.name_ident().0.contents,
                    expected_type.to_string(),
                ),
                span: pattern.name_ident().span(),
            });
        }
        count = count + 1;
    }
    Ok(())
}

fn check_trait_method_implementation_trait_constains(
    _where_clause: &Vec<TraitConstraint>,
    _noir_function: &NoirFunction,
    _trait_name: &String,
) -> Result<(), DefCollectorErrorKind> {
    // TODO
    Ok(())
}

fn check_trait_method_implementation_return_type(
    return_type: &UnresolvedType,
    noir_function: &NoirFunction,
    trait_name: &String,
) -> Result<(), DefCollectorErrorKind> {
    if return_type != &noir_function.return_type() {
        Err(DefCollectorErrorKind::SimpleError {
            primary_message: format!(
                "mismatch return type of method with name {} that implemetns trait {}",
                noir_function.name(),
                trait_name
            ),
            secondary_message: "".to_string(),
            span: noir_function.name_ident().span(),
        })
    } else {
        Ok(())
    }
}

fn check_trait_method_implementation(
    r#trait: &NoirTrait,
    noir_function: &NoirFunction,
) -> Result<(), DefCollectorErrorKind> {
    for item in &r#trait.items {
        match item {
            TraitItem::Function {
                name,
                generics,
                parameters,
                return_type,
                where_clause,
                body: _,
            } => {
                if name.0.contents == noir_function.def.name.0.contents {
                    // name matches, check for parameters, return type and where clause
                    check_trait_method_implementation_generics(
                        generics,
                        &noir_function,
                        &r#trait.name.0.contents,
                    )?;
                    check_trait_method_implementation_parameters(
                        parameters,
                        &noir_function,
                        &r#trait.name.0.contents,
                    )?;
                    check_trait_method_implementation_trait_constains(
                        where_clause,
                        &noir_function,
                        &r#trait.name.0.contents,
                    )?;
                    check_trait_method_implementation_return_type(
                        return_type,
                        &noir_function,
                        &r#trait.name.0.contents,
                    )?;
                    return Ok(());
                }
            }
            _ => {}
        }
    }

    Err(DefCollectorErrorKind::SimpleError {
        primary_message: format!(
            "method with name {} is not part of trait {}, therefore it can't be implemented",
            noir_function.name(),
            r#trait.name.0.contents
        ),
        secondary_message: "".to_string(),
        span: noir_function.name_ident().span(),
    })
}

impl<'a> ModCollector<'a> {
    fn collect_globals(
        &mut self,
        context: &mut Context,
        globals: Vec<LetStatement>,
        errors: &mut Vec<FileDiagnostic>,
    ) {
        for global in globals {
            let name = global.pattern.name_ident().clone();

            // First create dummy function in the DefInterner
            // So that we can get a StmtId
            let stmt_id = context.def_interner.push_empty_global();

            // Add the statement to the scope so its path can be looked up later
            let result =
                self.def_collector.def_map.modules[self.module_id.0].declare_global(name, stmt_id);

            if let Err((first_def, second_def)) = result {
                let err = DefCollectorErrorKind::DuplicateGlobal { first_def, second_def };
                errors.push(err.into_file_diagnostic(self.file_id));
            }

            self.def_collector.collected_globals.push(UnresolvedGlobal {
                file_id: self.file_id,
                module_id: self.module_id,
                stmt_id,
                stmt_def: global,
            });
        }
    }

    fn collect_impls(&mut self, context: &mut Context, impls: Vec<TypeImpl>) {
        for r#impl in impls {
            let mut unresolved_functions =
                UnresolvedFunctions { file_id: self.file_id, functions: Vec::new() };

            for method in r#impl.methods {
                let func_id = context.def_interner.push_empty_fn();
                context.def_interner.push_function_definition(method.name().to_owned(), func_id);
                unresolved_functions.push_fn(self.module_id, func_id, method);
            }

            let key = (r#impl.object_type, self.module_id);
            let methods = self.def_collector.collected_impls.entry(key).or_default();
            methods.push((r#impl.generics, r#impl.type_span, unresolved_functions));
        }
    }

    fn collect_trait_impls(
        &mut self,
        context: &mut Context,
        impls: Vec<TraitImpl>,
        errors: &mut Vec<FileDiagnostic>,
    ) {
        for r#impl in impls {
            let mut unresolved_functions =
                UnresolvedFunctions { file_id: self.file_id, functions: Vec::new() };
            let trait_name = r#impl.trait_name.clone();
            let module = &self.def_collector.def_map.modules[self.module_id.0];
            match module.find_name(&trait_name).types {
                Some((module_def_id, _visibility)) => match module_def_id {
                    ModuleDefId::TraitId(trait_id) => {
                        for item in r#impl.items {
                            match item {
                                TraitImplItem::Function(noir_function) => {
                                    if let Some(unresolved_trait) =
                                        self.def_collector.collected_traits.get(&trait_id)
                                    {
                                        match check_trait_method_implementation(
                                            &unresolved_trait.trait_def,
                                            &noir_function,
                                        ) {
                                            Ok(()) => {
                                                let func_id = context.def_interner.push_empty_fn();
                                                context.def_interner.push_function_definition(
                                                    noir_function.name().to_owned(),
                                                    func_id,
                                                );
                                                unresolved_functions.push_fn(
                                                    self.module_id,
                                                    func_id,
                                                    noir_function,
                                                );
                                            }
                                            Err(error) => {
                                                errors
                                                    .push(error.into_file_diagnostic(self.file_id));
                                            }
                                        }
                                    } else {
                                        // ??
                                    }
                                }
                                TraitImplItem::Constant(_name, _typ, _value) => {
                                    // TODO: Implement this
                                }
                                TraitImplItem::Type { name: _, alias: _ } => {
                                    // TODO: Implement this
                                }
                            }
                        }
                        let key = (r#impl.object_type, self.module_id);
                        let methods = self.def_collector.collected_impls.entry(key).or_default();
                        methods.push((
                            r#impl.impl_generics,
                            r#impl.object_type_span,
                            unresolved_functions,
                        ));
                    }
                    _ => {
                        let error = DefCollectorErrorKind::SimpleError {
                            primary_message: format!(
                                "{} is not a trait, therefore it can't be implemented",
                                trait_name
                            ),
                            secondary_message: "".to_string(),
                            span: trait_name.span(),
                        };
                        errors.push(error.into_file_diagnostic(self.file_id));
                    }
                },
                None => {
                    let error = DefCollectorErrorKind::SimpleError {
                        primary_message: format!("Trait {} not found", trait_name),
                        secondary_message: "".to_string(),
                        span: trait_name.span(),
                    };
                    errors.push(error.into_file_diagnostic(self.file_id));
                }
            }
        }
    }

    fn collect_functions(
        &mut self,
        context: &mut Context,
        functions: Vec<NoirFunction>,
        errors: &mut Vec<FileDiagnostic>,
    ) {
        let mut unresolved_functions =
            UnresolvedFunctions { file_id: self.file_id, functions: Vec::new() };

        for function in functions {
            let name = function.name_ident().clone();

            // First create dummy function in the DefInterner
            // So that we can get a FuncId
            let func_id = context.def_interner.push_empty_fn();
            context.def_interner.push_function_definition(name.0.contents.clone(), func_id);

            // Now link this func_id to a crate level map with the noir function and the module id
            // Encountering a NoirFunction, we retrieve it's module_data to get the namespace
            // Once we have lowered it to a HirFunction, we retrieve it's Id from the DefInterner
            // and replace it
            // With this method we iterate each function in the Crate and not each module
            // This may not be great because we have to pull the module_data for each function
            unresolved_functions.push_fn(self.module_id, func_id, function);

            // Add function to scope/ns of the module
            let result = self.def_collector.def_map.modules[self.module_id.0]
                .declare_function(name, func_id);

            if let Err((first_def, second_def)) = result {
                let error = DefCollectorErrorKind::DuplicateFunction { first_def, second_def };
                errors.push(error.into_file_diagnostic(self.file_id));
            }
        }

        self.def_collector.collected_functions.push(unresolved_functions);
    }

    /// Collect any struct definitions declared within the ast.
    /// Returns a vector of errors if any structs were already defined.
    fn collect_structs(
        &mut self,
        types: Vec<NoirStruct>,
        krate: CrateId,
        errors: &mut Vec<FileDiagnostic>,
    ) {
        for struct_definition in types {
            let name = struct_definition.name.clone();

            // Create the corresponding module for the struct namespace
            let id = match self.push_child_module(&name, self.file_id, false, false, errors) {
                Some(local_id) => StructId(ModuleId { krate, local_id }),
                None => continue,
            };

            // Add the struct to scope so its path can be looked up later
            let result =
                self.def_collector.def_map.modules[self.module_id.0].declare_struct(name, id);

            if let Err((first_def, second_def)) = result {
                let err = DefCollectorErrorKind::DuplicateTypeDef { first_def, second_def };
                errors.push(err.into_file_diagnostic(self.file_id));
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

    /// Collect any type aliases definitions declared within the ast.
    /// Returns a vector of errors if any type aliases were already defined.
    fn collect_type_aliases(
        &mut self,
        context: &mut Context,
        type_aliases: Vec<NoirTypeAlias>,
        errors: &mut Vec<FileDiagnostic>,
    ) {
        for type_alias in type_aliases {
            let name = type_alias.name.clone();

            // And store the TypeId -> TypeAlias mapping somewhere it is reachable
            let unresolved = UnresolvedTypeAlias {
                file_id: self.file_id,
                module_id: self.module_id,
                type_alias_def: type_alias,
            };

            let type_alias_id = context.def_interner.push_type_alias(&unresolved);

            // Add the type alias to scope so its path can be looked up later
            let result = self.def_collector.def_map.modules[self.module_id.0]
                .declare_type_alias(name, type_alias_id);

            if let Err((first_def, second_def)) = result {
                let err = DefCollectorErrorKind::DuplicateFunction { first_def, second_def };
                errors.push(err.into_file_diagnostic(self.file_id));
            }

            self.def_collector.collected_type_aliases.insert(type_alias_id, unresolved);
        }
    }

    /// Collect any traits definitions declared within the ast.
    /// Returns a vector of errors if any traits were already defined.
    fn collect_traits(
        &mut self,
        traits: Vec<NoirTrait>,
        krate: CrateId,
        errors: &mut Vec<FileDiagnostic>,
    ) {
        for trait_definition in traits {
            let name = trait_definition.name.clone();

            // Create the corresponding module for the trait namespace
            let id = match self.push_child_module(&name, self.file_id, false, false, errors) {
                Some(local_id) => TraitId(ModuleId { krate, local_id }),
                None => continue,
            };

            // Add the trait to scope so its path can be looked up later
            let result =
                self.def_collector.def_map.modules[self.module_id.0].declare_trait(name, id);

            if let Err((first_def, second_def)) = result {
                let err = DefCollectorErrorKind::DuplicateTraitDef { first_def, second_def };
                errors.push(err.into_file_diagnostic(self.file_id));
            }

            // And store the TypeId -> TraitType mapping somewhere it is reachable
            let unresolved = UnresolvedTrait {
                file_id: self.file_id,
                module_id: self.module_id,
                trait_def: trait_definition,
            };
            self.def_collector.collected_traits.insert(id, unresolved);
        }
    }

    fn collect_submodules(
        &mut self,
        context: &mut Context,
        crate_id: CrateId,
        submodules: Vec<SubModule>,
        file_id: FileId,
        errors: &mut Vec<FileDiagnostic>,
    ) {
        for submodule in submodules {
            if let Some(child) = self.push_child_module(
                &submodule.name,
                file_id,
                true,
                submodule.is_contract,
                errors,
            ) {
                collect_defs(
                    self.def_collector,
                    submodule.contents,
                    file_id,
                    child,
                    crate_id,
                    context,
                    errors,
                );
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
        errors: &mut Vec<FileDiagnostic>,
    ) {
        let child_file_id =
            match context.file_manager.resolve_path(self.file_id, &mod_name.0.contents) {
                Ok(child_file_id) => child_file_id,
                Err(_) => {
                    let err =
                        DefCollectorErrorKind::UnresolvedModuleDecl { mod_name: mod_name.clone() };
                    errors.push(err.into_file_diagnostic(self.file_id));
                    return;
                }
            };

        // Parse the AST for the module we just found and then recursively look for it's defs
        let ast = parse_file(&mut context.file_manager, child_file_id, errors);

        // Add module into def collector and get a ModuleId
        if let Some(child_mod_id) =
            self.push_child_module(mod_name, child_file_id, true, false, errors)
        {
            collect_defs(
                self.def_collector,
                ast,
                child_file_id,
                child_mod_id,
                crate_id,
                context,
                errors,
            );
        }
    }

    /// Add a child module to the current def_map.
    /// On error this returns None and pushes to `errors`
    fn push_child_module(
        &mut self,
        mod_name: &Ident,
        file_id: FileId,
        add_to_parent_scope: bool,
        is_contract: bool,
        errors: &mut Vec<FileDiagnostic>,
    ) -> Option<LocalModuleId> {
        let parent = Some(self.module_id);
        let new_module = ModuleData::new(parent, ModuleOrigin::File(file_id), is_contract);
        let module_id = self.def_collector.def_map.modules.insert(new_module);

        let modules = &mut self.def_collector.def_map.modules;

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

            if let Err((first_def, second_def)) =
                modules[self.module_id.0].declare_child_module(mod_name.to_owned(), mod_id)
            {
                let err = DefCollectorErrorKind::DuplicateModuleDecl { first_def, second_def };
                errors.push(err.into_file_diagnostic(self.file_id));
                return None;
            }
        }

        Some(LocalModuleId(module_id))
    }
}
