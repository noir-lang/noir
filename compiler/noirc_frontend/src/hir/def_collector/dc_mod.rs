use std::{collections::HashMap, vec};

use fm::FileId;
use noirc_errors::Location;

use crate::{
    graph::CrateId,
    hir::def_collector::dc_crate::{UnresolvedStruct, UnresolvedTrait},
    node_interner::TraitId,
    parser::SubModule,
    FunctionDefinition, Ident, LetStatement, NoirFunction, NoirStruct, NoirTrait, NoirTraitImpl,
    NoirTypeAlias, ParsedModule, TraitImplItem, TraitItem, TypeImpl,
};

use super::{
    dc_crate::{
        CompilationError, DefCollector, UnresolvedFunctions, UnresolvedGlobal, UnresolvedTraitImpl,
        UnresolvedTypeAlias,
    },
    errors::{DefCollectorErrorKind, DuplicateType},
};
use crate::hir::def_map::{parse_file, LocalModuleId, ModuleData, ModuleId};
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
) -> Vec<(CompilationError, FileId)> {
    let mut collector = ModCollector { def_collector, file_id, module_id };
    let mut errors: Vec<(CompilationError, FileId)> = vec![];
    // First resolve the module declarations
    for decl in ast.module_decls {
        errors.extend(collector.parse_module_declaration(context, &decl, crate_id));
    }

    errors.extend(collector.collect_submodules(context, crate_id, ast.submodules, file_id));

    // Then add the imports to defCollector to resolve once all modules in the hierarchy have been resolved
    for import in ast.imports {
        collector.def_collector.collected_imports.push(ImportDirective {
            module_id: collector.module_id,
            path: import.path,
            alias: import.alias,
        });
    }

    errors.extend(collector.collect_globals(context, ast.globals));

    errors.extend(collector.collect_traits(context, ast.traits, crate_id));

    errors.extend(collector.collect_structs(context, ast.types, crate_id));

    errors.extend(collector.collect_type_aliases(context, ast.type_aliases));

    errors.extend(collector.collect_functions(context, ast.functions, crate_id));

    errors.extend(collector.collect_trait_impls(context, ast.trait_impls, crate_id));

    collector.collect_impls(context, ast.impls, crate_id);

    errors
}

impl<'a> ModCollector<'a> {
    fn collect_globals(
        &mut self,
        context: &mut Context,
        globals: Vec<LetStatement>,
    ) -> Vec<(CompilationError, fm::FileId)> {
        let mut errors = vec![];
        for global in globals {
            let name = global.pattern.name_ident().clone();

            // First create dummy function in the DefInterner
            // So that we can get a StmtId
            let stmt_id = context.def_interner.push_empty_global();

            // Add the statement to the scope so its path can be looked up later
            let result =
                self.def_collector.def_map.modules[self.module_id.0].declare_global(name, stmt_id);

            if let Err((first_def, second_def)) = result {
                let err = DefCollectorErrorKind::Duplicate {
                    typ: DuplicateType::Global,
                    first_def,
                    second_def,
                };
                errors.push((err.into(), self.file_id));
            }

            self.def_collector.collected_globals.push(UnresolvedGlobal {
                file_id: self.file_id,
                module_id: self.module_id,
                stmt_id,
                stmt_def: global,
            });
        }
        errors
    }

    fn collect_impls(&mut self, context: &mut Context, impls: Vec<TypeImpl>, krate: CrateId) {
        let module_id = ModuleId { krate, local_id: self.module_id };

        for r#impl in impls {
            let mut unresolved_functions =
                UnresolvedFunctions { file_id: self.file_id, functions: Vec::new() };

            for method in r#impl.methods {
                let func_id = context.def_interner.push_empty_fn();
                context.def_interner.push_function(func_id, &method.def, module_id);
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
        impls: Vec<NoirTraitImpl>,
        krate: CrateId,
    ) -> Vec<(CompilationError, fm::FileId)> {
        for trait_impl in impls {
            let trait_name = trait_impl.trait_name.clone();

            let unresolved_functions =
                self.collect_trait_impl_function_overrides(context, &trait_impl, krate);

            let module = ModuleId { krate, local_id: self.module_id };

            for (_, func_id, noir_function) in &unresolved_functions.functions {
                context.def_interner.push_function(*func_id, &noir_function.def, module);
            }

            let unresolved_trait_impl = UnresolvedTraitImpl {
                file_id: self.file_id,
                module_id: self.module_id,
                trait_path: trait_name,
                methods: unresolved_functions,
                object_type: trait_impl.object_type,
                trait_id: None, // will be filled later
            };

            self.def_collector.collected_traits_impls.push(unresolved_trait_impl);
        }
        vec![]
    }

    fn collect_trait_impl_function_overrides(
        &mut self,
        context: &mut Context,
        trait_impl: &NoirTraitImpl,
        krate: CrateId,
    ) -> UnresolvedFunctions {
        let mut unresolved_functions =
            UnresolvedFunctions { file_id: self.file_id, functions: Vec::new() };

        let module = ModuleId { krate, local_id: self.module_id };

        for item in &trait_impl.items {
            if let TraitImplItem::Function(impl_method) = item {
                let func_id = context.def_interner.push_empty_fn();
                context.def_interner.push_function(func_id, &impl_method.def, module);
                unresolved_functions.push_fn(self.module_id, func_id, impl_method.clone());
            }
        }

        unresolved_functions
    }

    fn collect_functions(
        &mut self,
        context: &mut Context,
        functions: Vec<NoirFunction>,
        krate: CrateId,
    ) -> Vec<(CompilationError, FileId)> {
        let mut unresolved_functions =
            UnresolvedFunctions { file_id: self.file_id, functions: Vec::new() };
        let mut errors = vec![];

        let module = ModuleId { krate, local_id: self.module_id };

        for function in functions {
            let name = function.name_ident().clone();
            let func_id = context.def_interner.push_empty_fn();

            // First create dummy function in the DefInterner
            // So that we can get a FuncId
            context.def_interner.push_function(func_id, &function.def, module);

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
                let error = DefCollectorErrorKind::Duplicate {
                    typ: DuplicateType::Function,
                    first_def,
                    second_def,
                };
                errors.push((error.into(), self.file_id));
            }
        }

        self.def_collector.collected_functions.push(unresolved_functions);
        errors
    }

    /// Collect any struct definitions declared within the ast.
    /// Returns a vector of errors if any structs were already defined.
    fn collect_structs(
        &mut self,
        context: &mut Context,
        types: Vec<NoirStruct>,
        krate: CrateId,
    ) -> Vec<(CompilationError, FileId)> {
        let mut definiton_errors = vec![];
        for struct_definition in types {
            let name = struct_definition.name.clone();

            let unresolved = UnresolvedStruct {
                file_id: self.file_id,
                module_id: self.module_id,
                struct_def: struct_definition,
            };

            // Create the corresponding module for the struct namespace
            let id = match self.push_child_module(&name, self.file_id, false, false) {
                Ok(local_id) => context.def_interner.new_struct(&unresolved, krate, local_id),
                Err(error) => {
                    definiton_errors.push((error.into(), self.file_id));
                    continue;
                }
            };

            // Add the struct to scope so its path can be looked up later
            let result =
                self.def_collector.def_map.modules[self.module_id.0].declare_struct(name, id);

            if let Err((first_def, second_def)) = result {
                let error = DefCollectorErrorKind::Duplicate {
                    typ: DuplicateType::TypeDefinition,
                    first_def,
                    second_def,
                };
                definiton_errors.push((error.into(), self.file_id));
            }

            // And store the TypeId -> StructType mapping somewhere it is reachable
            self.def_collector.collected_types.insert(id, unresolved);
        }
        definiton_errors
    }

    /// Collect any type aliases definitions declared within the ast.
    /// Returns a vector of errors if any type aliases were already defined.
    fn collect_type_aliases(
        &mut self,
        context: &mut Context,
        type_aliases: Vec<NoirTypeAlias>,
    ) -> Vec<(CompilationError, FileId)> {
        let mut errors: Vec<(CompilationError, FileId)> = vec![];
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
                let err = DefCollectorErrorKind::Duplicate {
                    typ: DuplicateType::Function,
                    first_def,
                    second_def,
                };
                errors.push((err.into(), self.file_id));
            }

            self.def_collector.collected_type_aliases.insert(type_alias_id, unresolved);
        }
        errors
    }

    fn check_for_duplicate_trait_items(
        &mut self,
        trait_items: &Vec<TraitItem>,
    ) -> Vec<(CompilationError, FileId)> {
        let mut errors: Vec<(CompilationError, FileId)> = vec![];

        let mut functions_and_consts: HashMap<&Ident, &TraitItem> = HashMap::new();
        let mut types: HashMap<&Ident, &TraitItem> = HashMap::new();

        for trait_item in trait_items {
            match trait_item {
                TraitItem::Function { name, .. } => {
                    if let Some(prev_item) = functions_and_consts.insert(name, trait_item) {
                        let error = match prev_item {
                            TraitItem::Function { name: prev_name, .. } => DefCollectorErrorKind::Duplicate {
                                typ: DuplicateType::TraitAssociatedFunction,
                                first_def: prev_name.clone(),
                                second_def: name.clone(),
                            },
                            TraitItem::Constant { name: prev_name, .. } => {
                                DefCollectorErrorKind::Duplicate {
                                    typ: DuplicateType::TraitAssociatedItem,
                                    first_def: prev_name.clone(),
                                    second_def: name.clone(),
                                }
                            }
                            TraitItem::Type { .. } => {
                                unreachable!();
                            }
                        };
                        errors.push((error.into(), self.file_id));
                    }
                }
                TraitItem::Constant { name, .. } => {
                    if let Some(prev_item) = functions_and_consts.insert(name, trait_item) {
                        let error = match prev_item {
                            TraitItem::Function { name: prev_name, .. } => DefCollectorErrorKind::Duplicate {
                                typ: DuplicateType::TraitAssociatedItem,
                                first_def: prev_name.clone(),
                                second_def: name.clone(),
                            },
                            TraitItem::Constant { name: prev_name, .. } => {
                                DefCollectorErrorKind::Duplicate {
                                    typ: DuplicateType::TraitAssociatedConst,
                                    first_def: prev_name.clone(),
                                    second_def: name.clone(),
                                }
                            }
                            TraitItem::Type { .. } => {
                                unreachable!();
                            }
                        };
                        errors.push((error.into(), self.file_id));
                    }
                }
                TraitItem::Type { name } => {
                    if let Some(prev_item) = types.insert(name, trait_item) {
                        if let TraitItem::Type { name: prev_name } = prev_item {
                            let error = DefCollectorErrorKind::Duplicate {
                                typ: DuplicateType::TraitAssociatedType,
                                first_def: prev_name.clone(),
                                second_def: name.clone(),
                            };
                            errors.push((error.into(), self.file_id));
                        } else {
                            unreachable!();
                        }
                    }
                }
            }
        }

        errors
    }

    /// Collect any traits definitions declared within the ast.
    /// Returns a vector of errors if any traits were already defined.
    fn collect_traits(
        &mut self,
        context: &mut Context,
        traits: Vec<NoirTrait>,
        krate: CrateId,
    ) -> Vec<(CompilationError, FileId)> {
        let mut errors: Vec<(CompilationError, FileId)> = vec![];
        for trait_definition in traits {
            let name = trait_definition.name.clone();

            // Create the corresponding module for the trait namespace
            let id = match self.push_child_module(&name, self.file_id, false, false) {
                Ok(local_id) => TraitId(ModuleId { krate, local_id }),
                Err(error) => {
                    errors.push((error.into(), self.file_id));
                    continue;
                }
            };

            // Add the trait to scope so its path can be looked up later
            let result =
                self.def_collector.def_map.modules[self.module_id.0].declare_trait(name, id);

            if let Err((first_def, second_def)) = result {
                let error = DefCollectorErrorKind::Duplicate {
                    typ: DuplicateType::Trait,
                    first_def,
                    second_def,
                };
                errors.push((error.into(), self.file_id));
            }

            errors.append(&mut self.check_for_duplicate_trait_items(&trait_definition.items));

            // Add all functions that have a default implementation in the trait
            let mut unresolved_functions =
                UnresolvedFunctions { file_id: self.file_id, functions: Vec::new() };
            for trait_item in &trait_definition.items {
                // TODO(Maddiaa): Investigate trait implementations with attributes see: https://github.com/noir-lang/noir/issues/2629
                if let TraitItem::Function {
                    name,
                    generics,
                    parameters,
                    return_type,
                    where_clause,
                    body: Some(body),
                } = trait_item
                {
                    let func_id = context.def_interner.push_empty_fn();

                    let impl_method = NoirFunction::normal(FunctionDefinition::normal(
                        name,
                        generics,
                        parameters,
                        body,
                        where_clause,
                        return_type,
                    ));
                    unresolved_functions.push_fn(self.module_id, func_id, impl_method);
                }
            }

            // And store the TraitId -> TraitType mapping somewhere it is reachable
            let unresolved = UnresolvedTrait {
                file_id: self.file_id,
                module_id: self.module_id,
                crate_id: krate,
                trait_def: trait_definition,
                fns_with_default_impl: unresolved_functions,
            };
            self.def_collector.collected_traits.insert(id, unresolved);
        }
        errors
    }

    fn collect_submodules(
        &mut self,
        context: &mut Context,
        crate_id: CrateId,
        submodules: Vec<SubModule>,
        file_id: FileId,
    ) -> Vec<(CompilationError, FileId)> {
        let mut errors: Vec<(CompilationError, FileId)> = vec![];
        for submodule in submodules {
            match self.push_child_module(&submodule.name, file_id, true, submodule.is_contract) {
                Ok(child) => {
                    errors.extend(collect_defs(
                        self.def_collector,
                        submodule.contents,
                        file_id,
                        child,
                        crate_id,
                        context,
                    ));
                }
                Err(error) => {
                    errors.push((error.into(), file_id));
                }
            };
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
        crate_id: CrateId,
    ) -> Vec<(CompilationError, FileId)> {
        let mut errors: Vec<(CompilationError, FileId)> = vec![];
        let child_file_id =
            match context.file_manager.find_module(self.file_id, &mod_name.0.contents) {
                Ok(child_file_id) => child_file_id,
                Err(expected_path) => {
                    let mod_name = mod_name.clone();
                    let err =
                        DefCollectorErrorKind::UnresolvedModuleDecl { mod_name, expected_path };
                    errors.push((err.into(), self.file_id));
                    return errors;
                }
            };

        let location = Location { file: self.file_id, span: mod_name.span() };

        if let Some(old_location) = context.visited_files.get(&child_file_id) {
            let error = DefCollectorErrorKind::ModuleAlreadyPartOfCrate {
                mod_name: mod_name.clone(),
                span: location.span,
            };
            errors.push((error.into(), location.file));

            let error2 = DefCollectorErrorKind::ModuleOrignallyDefined {
                mod_name: mod_name.clone(),
                span: old_location.span,
            };
            errors.push((error2.into(), old_location.file));
            return errors;
        }

        context.visited_files.insert(child_file_id, location);

        // Parse the AST for the module we just found and then recursively look for it's defs
        //let ast = parse_file(&context.file_manager, child_file_id, errors);
        let (ast, parsing_errors) = parse_file(&context.file_manager, child_file_id);

        errors.extend(
            parsing_errors.iter().map(|e| (e.clone().into(), child_file_id)).collect::<Vec<_>>(),
        );

        // Add module into def collector and get a ModuleId
        match self.push_child_module(mod_name, child_file_id, true, false) {
            Ok(child_mod_id) => {
                errors.extend(collect_defs(
                    self.def_collector,
                    ast,
                    child_file_id,
                    child_mod_id,
                    crate_id,
                    context,
                ));
            }
            Err(error) => {
                errors.push((error.into(), child_file_id));
            }
        }
        errors
    }

    /// Add a child module to the current def_map.
    /// On error this returns None and pushes to `errors`
    fn push_child_module(
        &mut self,
        mod_name: &Ident,
        file_id: FileId,
        add_to_parent_scope: bool,
        is_contract: bool,
    ) -> Result<LocalModuleId, DefCollectorErrorKind> {
        let parent = Some(self.module_id);
        let location = Location::new(mod_name.span(), file_id);
        let new_module = ModuleData::new(parent, location, is_contract);
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
                let err = DefCollectorErrorKind::Duplicate {
                    typ: DuplicateType::Module,
                    first_def,
                    second_def,
                };
                return Err(err);
            }
        }

        Ok(LocalModuleId(module_id))
    }
}
