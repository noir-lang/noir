use std::path::Path;
use std::vec;

use acvm::{AcirField, FieldElement};
use fm::{FileId, FileManager, FILE_EXTENSION};
use noirc_errors::{Location, Span};
use num_bigint::BigUint;
use num_traits::Num;
use rustc_hash::FxHashMap as HashMap;

use crate::ast::{
    FunctionDefinition, Ident, ItemVisibility, LetStatement, ModuleDeclaration, NoirFunction,
    NoirStruct, NoirTrait, NoirTraitImpl, NoirTypeAlias, Pattern, TraitImplItem, TraitItem,
    TypeImpl,
};
use crate::macros_api::NodeInterner;
use crate::node_interner::{ModuleAttributes, ReferenceId};
use crate::{
    graph::CrateId,
    hir::def_collector::dc_crate::{UnresolvedStruct, UnresolvedTrait},
    macros_api::MacroProcessor,
    node_interner::{FunctionModifiers, TraitId, TypeAliasId},
    parser::{SortedModule, SortedSubModule},
};

use super::{
    dc_crate::{
        CompilationError, DefCollector, UnresolvedFunctions, UnresolvedGlobal, UnresolvedTraitImpl,
        UnresolvedTypeAlias,
    },
    errors::{DefCollectorErrorKind, DuplicateType},
};
use crate::hir::def_map::{CrateDefMap, LocalModuleId, ModuleData, ModuleId};
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
    ast: SortedModule,
    file_id: FileId,
    module_id: LocalModuleId,
    crate_id: CrateId,
    context: &mut Context,
    macro_processors: &[&dyn MacroProcessor],
) -> Vec<(CompilationError, FileId)> {
    let mut collector = ModCollector { def_collector, file_id, module_id };
    let mut errors: Vec<(CompilationError, FileId)> = vec![];

    // First resolve the module declarations
    for decl in ast.module_decls {
        errors.extend(collector.parse_module_declaration(
            context,
            &decl,
            crate_id,
            macro_processors,
        ));
    }

    errors.extend(collector.collect_submodules(
        context,
        crate_id,
        ast.submodules,
        file_id,
        macro_processors,
    ));

    // Then add the imports to defCollector to resolve once all modules in the hierarchy have been resolved
    for import in ast.imports {
        collector.def_collector.imports.push(ImportDirective {
            module_id: collector.module_id,
            path: import.path,
            alias: import.alias,
            is_prelude: false,
        });
    }

    errors.extend(collector.collect_globals(context, ast.globals, crate_id));

    errors.extend(collector.collect_traits(context, ast.traits, crate_id));

    errors.extend(collector.collect_structs(context, ast.types, crate_id));

    errors.extend(collector.collect_type_aliases(context, ast.type_aliases, crate_id));

    errors.extend(collector.collect_functions(context, ast.functions, crate_id));

    collector.collect_trait_impls(context, ast.trait_impls, crate_id);

    collector.collect_impls(context, ast.impls, crate_id);

    errors
}

impl<'a> ModCollector<'a> {
    fn collect_globals(
        &mut self,
        context: &mut Context,
        globals: Vec<LetStatement>,
        crate_id: CrateId,
    ) -> Vec<(CompilationError, fm::FileId)> {
        let mut errors = vec![];
        for global in globals {
            let (global, error) = collect_global(
                &mut context.def_interner,
                &mut self.def_collector.def_map,
                global,
                self.file_id,
                self.module_id,
                crate_id,
            );

            if let Some(error) = error {
                errors.push(error);
            }

            self.def_collector.items.globals.push(global);
        }
        errors
    }

    fn collect_impls(&mut self, context: &mut Context, impls: Vec<TypeImpl>, krate: CrateId) {
        let module_id = ModuleId { krate, local_id: self.module_id };

        for r#impl in impls {
            let mut unresolved_functions = UnresolvedFunctions {
                file_id: self.file_id,
                functions: Vec::new(),
                trait_id: None,
                self_type: None,
            };

            for (mut method, _) in r#impl.methods {
                let func_id = context.def_interner.push_empty_fn();
                method.def.where_clause.extend(r#impl.where_clause.clone());
                let location = Location::new(method.span(), self.file_id);
                context.def_interner.push_function(func_id, &method.def, module_id, location);
                unresolved_functions.push_fn(self.module_id, func_id, method);
            }

            let key = (r#impl.object_type, self.module_id);
            let methods = self.def_collector.items.impls.entry(key).or_default();
            methods.push((r#impl.generics, r#impl.type_span, unresolved_functions));
        }
    }

    fn collect_trait_impls(
        &mut self,
        context: &mut Context,
        impls: Vec<NoirTraitImpl>,
        krate: CrateId,
    ) {
        for mut trait_impl in impls {
            let trait_name = trait_impl.trait_name.clone();

            let mut unresolved_functions = collect_trait_impl_functions(
                &mut context.def_interner,
                &mut trait_impl,
                krate,
                self.file_id,
                self.module_id,
            );

            let module = ModuleId { krate, local_id: self.module_id };

            for (_, func_id, noir_function) in &mut unresolved_functions.functions {
                // Attach any trait constraints on the impl to the function
                noir_function.def.where_clause.append(&mut trait_impl.where_clause.clone());
                let location = Location::new(noir_function.def.span, self.file_id);
                context.def_interner.push_function(*func_id, &noir_function.def, module, location);
            }

            let unresolved_trait_impl = UnresolvedTraitImpl {
                file_id: self.file_id,
                module_id: self.module_id,
                trait_path: trait_name,
                methods: unresolved_functions,
                object_type: trait_impl.object_type,
                generics: trait_impl.impl_generics,
                where_clause: trait_impl.where_clause,
                trait_generics: trait_impl.trait_generics,
                is_comptime: trait_impl.is_comptime,

                // These last fields are filled later on
                trait_id: None,
                impl_id: None,
                resolved_object_type: None,
                resolved_generics: Vec::new(),
                resolved_trait_generics: Vec::new(),
            };

            self.def_collector.items.trait_impls.push(unresolved_trait_impl);
        }
    }

    fn collect_functions(
        &mut self,
        context: &mut Context,
        functions: Vec<NoirFunction>,
        krate: CrateId,
    ) -> Vec<(CompilationError, FileId)> {
        let mut unresolved_functions = UnresolvedFunctions {
            file_id: self.file_id,
            functions: Vec::new(),
            trait_id: None,
            self_type: None,
        };
        let mut errors = vec![];

        let module = ModuleId { krate, local_id: self.module_id };

        for function in functions {
            // check if optional field attribute is compatible with native field
            if let Some(field) = function.attributes().get_field_attribute() {
                if !is_native_field(&field) {
                    continue;
                }
            }

            let name = function.name_ident().clone();
            let func_id = context.def_interner.push_empty_fn();
            let visibility = function.def.visibility;

            // First create dummy function in the DefInterner
            // So that we can get a FuncId
            let location = Location::new(function.span(), self.file_id);
            context.def_interner.push_function(func_id, &function.def, module, location);

            // Now link this func_id to a crate level map with the noir function and the module id
            // Encountering a NoirFunction, we retrieve it's module_data to get the namespace
            // Once we have lowered it to a HirFunction, we retrieve it's Id from the DefInterner
            // and replace it
            // With this method we iterate each function in the Crate and not each module
            // This may not be great because we have to pull the module_data for each function
            unresolved_functions.push_fn(self.module_id, func_id, function);

            // Add function to scope/ns of the module
            let result = self.def_collector.def_map.modules[self.module_id.0]
                .declare_function(name, visibility, func_id);

            if let Err((first_def, second_def)) = result {
                let error = DefCollectorErrorKind::Duplicate {
                    typ: DuplicateType::Function,
                    first_def,
                    second_def,
                };
                errors.push((error.into(), self.file_id));
            }
        }

        self.def_collector.items.functions.push(unresolved_functions);
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
        let mut definition_errors = vec![];
        for struct_definition in types {
            let name = struct_definition.name.clone();

            let unresolved = UnresolvedStruct {
                file_id: self.file_id,
                module_id: self.module_id,
                struct_def: struct_definition,
            };

            let resolved_generics = context.resolve_generics(
                &unresolved.struct_def.generics,
                &mut definition_errors,
                self.file_id,
            );

            // Create the corresponding module for the struct namespace
            let id = match self.push_child_module(
                context,
                &name,
                Location::new(name.span(), self.file_id),
                false,
                false,
            ) {
                Ok(module_id) => context.def_interner.new_struct(
                    &unresolved,
                    resolved_generics,
                    krate,
                    module_id.local_id,
                    self.file_id,
                ),
                Err(error) => {
                    definition_errors.push((error.into(), self.file_id));
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
                definition_errors.push((error.into(), self.file_id));
            }

            // And store the TypeId -> StructType mapping somewhere it is reachable
            self.def_collector.items.types.insert(id, unresolved);

            context.def_interner.add_definition_location(
                ReferenceId::Struct(id),
                Some(ModuleId { krate, local_id: self.module_id }),
            );
        }
        definition_errors
    }

    /// Collect any type aliases definitions declared within the ast.
    /// Returns a vector of errors if any type aliases were already defined.
    fn collect_type_aliases(
        &mut self,
        context: &mut Context,
        type_aliases: Vec<NoirTypeAlias>,
        krate: CrateId,
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

            let resolved_generics = context.resolve_generics(
                &unresolved.type_alias_def.generics,
                &mut errors,
                self.file_id,
            );

            let type_alias_id =
                context.def_interner.push_type_alias(&unresolved, resolved_generics);

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

            self.def_collector.items.type_aliases.insert(type_alias_id, unresolved);

            context.def_interner.add_definition_location(
                ReferenceId::Alias(type_alias_id),
                Some(ModuleId { krate, local_id: self.module_id }),
            );
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
            let trait_id = match self.push_child_module(
                context,
                &name,
                Location::new(name.span(), self.file_id),
                false,
                false,
            ) {
                Ok(module_id) => TraitId(ModuleId { krate, local_id: module_id.local_id }),
                Err(error) => {
                    errors.push((error.into(), self.file_id));
                    continue;
                }
            };

            // Add the trait to scope so its path can be looked up later
            let result =
                self.def_collector.def_map.modules[self.module_id.0].declare_trait(name, trait_id);

            if let Err((first_def, second_def)) = result {
                let error = DefCollectorErrorKind::Duplicate {
                    typ: DuplicateType::Trait,
                    first_def,
                    second_def,
                };
                errors.push((error.into(), self.file_id));
            }

            // Add all functions that have a default implementation in the trait
            let mut unresolved_functions = UnresolvedFunctions {
                file_id: self.file_id,
                functions: Vec::new(),
                trait_id: None,
                self_type: None,
            };

            let mut method_ids = HashMap::default();
            for trait_item in &trait_definition.items {
                match trait_item {
                    TraitItem::Function {
                        name,
                        generics,
                        parameters,
                        return_type,
                        where_clause,
                        body,
                    } => {
                        let func_id = context.def_interner.push_empty_fn();
                        method_ids.insert(name.to_string(), func_id);

                        let location = Location::new(name.span(), self.file_id);
                        let modifiers = FunctionModifiers {
                            name: name.to_string(),
                            visibility: ItemVisibility::Public,
                            // TODO(Maddiaa): Investigate trait implementations with attributes see: https://github.com/noir-lang/noir/issues/2629
                            attributes: crate::token::Attributes::empty(),
                            is_unconstrained: false,
                            generic_count: generics.len(),
                            is_comptime: false,
                            name_location: location,
                        };

                        context
                            .def_interner
                            .push_function_definition(func_id, modifiers, trait_id.0, location);

                        match self.def_collector.def_map.modules[trait_id.0.local_id.0]
                            .declare_function(name.clone(), ItemVisibility::Public, func_id)
                        {
                            Ok(()) => {
                                if let Some(body) = body {
                                    let impl_method =
                                        NoirFunction::normal(FunctionDefinition::normal(
                                            name,
                                            generics,
                                            parameters,
                                            body,
                                            where_clause,
                                            return_type,
                                        ));
                                    unresolved_functions.push_fn(
                                        self.module_id,
                                        func_id,
                                        impl_method,
                                    );
                                }
                            }
                            Err((first_def, second_def)) => {
                                let error = DefCollectorErrorKind::Duplicate {
                                    typ: DuplicateType::TraitAssociatedFunction,
                                    first_def,
                                    second_def,
                                };
                                errors.push((error.into(), self.file_id));
                            }
                        }
                    }
                    TraitItem::Constant { name, .. } => {
                        let global_id = context.def_interner.push_empty_global(
                            name.clone(),
                            trait_id.0.local_id,
                            krate,
                            self.file_id,
                            vec![],
                            false,
                            false,
                        );

                        if let Err((first_def, second_def)) = self.def_collector.def_map.modules
                            [trait_id.0.local_id.0]
                            .declare_global(name.clone(), global_id)
                        {
                            let error = DefCollectorErrorKind::Duplicate {
                                typ: DuplicateType::TraitAssociatedConst,
                                first_def,
                                second_def,
                            };
                            errors.push((error.into(), self.file_id));
                        }
                    }
                    TraitItem::Type { name } => {
                        // TODO(nickysn or alexvitkov): implement context.def_interner.push_empty_type_alias and get an id, instead of using TypeAliasId::dummy_id()
                        if let Err((first_def, second_def)) = self.def_collector.def_map.modules
                            [trait_id.0.local_id.0]
                            .declare_type_alias(name.clone(), TypeAliasId::dummy_id())
                        {
                            let error = DefCollectorErrorKind::Duplicate {
                                typ: DuplicateType::TraitAssociatedType,
                                first_def,
                                second_def,
                            };
                            errors.push((error.into(), self.file_id));
                        }
                    }
                }
            }

            let resolved_generics =
                context.resolve_generics(&trait_definition.generics, &mut errors, self.file_id);

            // And store the TraitId -> TraitType mapping somewhere it is reachable
            let unresolved = UnresolvedTrait {
                file_id: self.file_id,
                module_id: self.module_id,
                crate_id: krate,
                trait_def: trait_definition,
                method_ids,
                fns_with_default_impl: unresolved_functions,
            };
            context.def_interner.push_empty_trait(trait_id, &unresolved, resolved_generics);

            context.def_interner.add_definition_location(
                ReferenceId::Trait(trait_id),
                Some(ModuleId { krate, local_id: self.module_id }),
            );

            self.def_collector.items.traits.insert(trait_id, unresolved);
        }
        errors
    }

    fn collect_submodules(
        &mut self,
        context: &mut Context,
        crate_id: CrateId,
        submodules: Vec<SortedSubModule>,
        file_id: FileId,
        macro_processors: &[&dyn MacroProcessor],
    ) -> Vec<(CompilationError, FileId)> {
        let mut errors: Vec<(CompilationError, FileId)> = vec![];
        for submodule in submodules {
            match self.push_child_module(
                context,
                &submodule.name,
                Location::new(submodule.name.span(), file_id),
                true,
                submodule.is_contract,
            ) {
                Ok(child) => {
                    errors.extend(collect_defs(
                        self.def_collector,
                        submodule.contents,
                        file_id,
                        child.local_id,
                        crate_id,
                        context,
                        macro_processors,
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
        mod_decl: &ModuleDeclaration,
        crate_id: CrateId,
        macro_processors: &[&dyn MacroProcessor],
    ) -> Vec<(CompilationError, FileId)> {
        let mut errors: Vec<(CompilationError, FileId)> = vec![];
        let child_file_id = match find_module(&context.file_manager, self.file_id, &mod_decl.ident)
        {
            Ok(child_file_id) => child_file_id,
            Err(err) => {
                errors.push((err.into(), self.file_id));
                return errors;
            }
        };

        let location = Location { file: self.file_id, span: mod_decl.ident.span() };

        if let Some(old_location) = context.visited_files.get(&child_file_id) {
            let error = DefCollectorErrorKind::ModuleAlreadyPartOfCrate {
                mod_name: mod_decl.ident.clone(),
                span: location.span,
            };
            errors.push((error.into(), location.file));

            let error = DefCollectorErrorKind::ModuleOriginallyDefined {
                mod_name: mod_decl.ident.clone(),
                span: old_location.span,
            };
            errors.push((error.into(), old_location.file));
            return errors;
        }

        context.visited_files.insert(child_file_id, location);

        // Parse the AST for the module we just found and then recursively look for it's defs
        let (ast, parsing_errors) = context.parsed_file_results(child_file_id);
        let mut ast = ast.into_sorted();

        for macro_processor in macro_processors {
            match macro_processor.process_untyped_ast(
                ast.clone(),
                &crate_id,
                child_file_id,
                context,
            ) {
                Ok(processed_ast) => {
                    ast = processed_ast;
                }
                Err((error, file_id)) => {
                    let def_error = DefCollectorErrorKind::MacroError(error);
                    errors.push((def_error.into(), file_id));
                }
            }
        }

        errors.extend(
            parsing_errors.iter().map(|e| (e.clone().into(), child_file_id)).collect::<Vec<_>>(),
        );

        // Add module into def collector and get a ModuleId
        match self.push_child_module(
            context,
            &mod_decl.ident,
            Location::new(Span::empty(0), child_file_id),
            true,
            false,
        ) {
            Ok(child_mod_id) => {
                // Track that the "foo" in `mod foo;` points to the module "foo"
                context.def_interner.add_module_reference(child_mod_id, location);

                errors.extend(collect_defs(
                    self.def_collector,
                    ast,
                    child_file_id,
                    child_mod_id.local_id,
                    crate_id,
                    context,
                    macro_processors,
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
        context: &mut Context,
        mod_name: &Ident,
        mod_location: Location,
        add_to_parent_scope: bool,
        is_contract: bool,
    ) -> Result<ModuleId, DefCollectorErrorKind> {
        let parent = Some(self.module_id);

        // Note: the difference between `location` and `mod_location` is:
        // - `mod_location` will point to either the token "foo" in `mod foo { ... }`
        //   if it's an inline module, or the first char of a the file if it's an external module.
        // - `location` will always point to the token "foo" in `mod foo` regardless of whether
        //   it's inline or external.
        // Eventually the location put in `ModuleData` is used for codelenses about `contract`s,
        // so we keep using `location` so that it continues to work as usual.
        let location = Location::new(mod_name.span(), mod_location.file);
        let new_module = ModuleData::new(parent, location, is_contract);
        let module_id = self.def_collector.def_map.modules.insert(new_module);

        let modules = &mut self.def_collector.def_map.modules;

        // Update the parent module to reference the child
        modules[self.module_id.0].children.insert(mod_name.clone(), LocalModuleId(module_id));

        let mod_id = ModuleId {
            krate: self.def_collector.def_map.krate,
            local_id: LocalModuleId(module_id),
        };

        // Add this child module into the scope of the parent module as a module definition
        // module definitions are definitions which can only exist at the module level.
        // ModuleDefinitionIds can be used across crates since they contain the CrateId
        //
        // We do not want to do this in the case of struct modules (each struct type corresponds
        // to a child module containing its methods) since the module name should not shadow
        // the struct name.
        if add_to_parent_scope {
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

            context.def_interner.add_module_attributes(
                mod_id,
                ModuleAttributes {
                    name: mod_name.0.contents.clone(),
                    location: mod_location,
                    parent: self.module_id,
                },
            );
        }

        Ok(mod_id)
    }
}

fn find_module(
    file_manager: &FileManager,
    anchor: FileId,
    mod_name: &Ident,
) -> Result<FileId, DefCollectorErrorKind> {
    let anchor_path = file_manager
        .path(anchor)
        .expect("File must exist in file manager in order for us to be resolving its imports.")
        .with_extension("");
    let anchor_dir = anchor_path.parent().unwrap();

    // Assuming anchor is called "anchor.nr" and we are looking up a module named "mod_name"...
    // This is "mod_name"
    let mod_name_str = &mod_name.0.contents;

    // If we are in a special name like "main.nr", "lib.nr", "mod.nr" or "{mod_name}.nr",
    // the search starts at the same directory, otherwise it starts in a nested directory.
    let start_dir = if should_check_siblings_for_module(&anchor_path, anchor_dir) {
        anchor_dir
    } else {
        anchor_path.as_path()
    };

    // Check "mod_name.nr"
    let mod_name_candidate = start_dir.join(format!("{mod_name_str}.{FILE_EXTENSION}"));
    let mod_name_result = file_manager.name_to_id(mod_name_candidate.clone());

    // Check "mod_name/mod.nr"
    let mod_nr_candidate = start_dir.join(mod_name_str).join(format!("mod.{FILE_EXTENSION}"));
    let mod_nr_result = file_manager.name_to_id(mod_nr_candidate.clone());

    match (mod_nr_result, mod_name_result) {
        (Some(_), Some(_)) => Err(DefCollectorErrorKind::OverlappingModuleDecls {
            mod_name: mod_name.clone(),
            expected_path: mod_name_candidate.as_os_str().to_string_lossy().to_string(),
            alternative_path: mod_nr_candidate.as_os_str().to_string_lossy().to_string(),
        }),
        (Some(id), None) | (None, Some(id)) => Ok(id),
        (None, None) => Err(DefCollectorErrorKind::UnresolvedModuleDecl {
            mod_name: mod_name.clone(),
            expected_path: mod_name_candidate.as_os_str().to_string_lossy().to_string(),
            alternative_path: mod_nr_candidate.as_os_str().to_string_lossy().to_string(),
        }),
    }
}

/// Returns true if a module's child modules are expected to be in the same directory.
/// Returns false if they are expected to be in a subdirectory matching the name of the module.
fn should_check_siblings_for_module(module_path: &Path, parent_path: &Path) -> bool {
    if let Some(filename) = module_path.file_stem() {
        // This check also means a `main.nr` or `lib.nr` file outside of the crate root would
        // check its same directory for child modules instead of a subdirectory. Should we prohibit
        // `main.nr` and `lib.nr` files outside of the crate root?
        filename == "main"
            || filename == "lib"
            || filename == "mod"
            || Some(filename) == parent_path.file_stem()
    } else {
        // If there's no filename, we arbitrarily return true.
        // Alternatively, we could panic, but this is left to a different step where we
        // ideally have some source location to issue an error.
        true
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "bls12_381")] {
        pub const CHOSEN_FIELD: &str = "bls12_381";
    } else {
        pub const CHOSEN_FIELD: &str = "bn254";
    }
}

fn is_native_field(str: &str) -> bool {
    let big_num = if let Some(hex) = str.strip_prefix("0x") {
        BigUint::from_str_radix(hex, 16)
    } else {
        BigUint::from_str_radix(str, 10)
    };
    if let Ok(big_num) = big_num {
        big_num == FieldElement::modulus()
    } else {
        CHOSEN_FIELD == str
    }
}

pub(crate) fn collect_trait_impl_functions(
    interner: &mut NodeInterner,
    trait_impl: &mut NoirTraitImpl,
    krate: CrateId,
    file_id: FileId,
    local_id: LocalModuleId,
) -> UnresolvedFunctions {
    let mut unresolved_functions =
        UnresolvedFunctions { file_id, functions: Vec::new(), trait_id: None, self_type: None };

    let module = ModuleId { krate, local_id };

    for item in std::mem::take(&mut trait_impl.items) {
        if let TraitImplItem::Function(impl_method) = item {
            let func_id = interner.push_empty_fn();
            let location = Location::new(impl_method.span(), file_id);
            interner.push_function(func_id, &impl_method.def, module, location);
            unresolved_functions.push_fn(local_id, func_id, impl_method);
        }
    }

    unresolved_functions
}

pub(crate) fn collect_global(
    interner: &mut NodeInterner,
    def_map: &mut CrateDefMap,
    global: LetStatement,
    file_id: FileId,
    module_id: LocalModuleId,
    crate_id: CrateId,
) -> (UnresolvedGlobal, Option<(CompilationError, FileId)>) {
    let name = global.pattern.name_ident().clone();

    let global_id = interner.push_empty_global(
        name.clone(),
        module_id,
        crate_id,
        file_id,
        global.attributes.clone(),
        matches!(global.pattern, Pattern::Mutable { .. }),
        global.comptime,
    );

    // Add the statement to the scope so its path can be looked up later
    let result = def_map.modules[module_id.0].declare_global(name, global_id);

    let error = result.err().map(|(first_def, second_def)| {
        let err =
            DefCollectorErrorKind::Duplicate { typ: DuplicateType::Global, first_def, second_def };
        (err.into(), file_id)
    });

    let global = UnresolvedGlobal { file_id, module_id, global_id, stmt_def: global };
    (global, error)
}

#[cfg(test)]
mod find_module_tests {
    use super::*;

    use noirc_errors::Spanned;
    use std::path::{Path, PathBuf};

    fn add_file(file_manager: &mut FileManager, dir: &Path, file_name: &str) -> FileId {
        let mut target_filename = PathBuf::from(&dir);
        for path in file_name.split('/') {
            target_filename = target_filename.join(path);
        }

        file_manager
            .add_file_with_source(&target_filename, "fn foo() {}".to_string())
            .expect("could not add file to file manager and obtain a FileId")
    }

    fn find_module(
        file_manager: &FileManager,
        anchor: FileId,
        mod_name: &str,
    ) -> Result<FileId, DefCollectorErrorKind> {
        let mod_name = Ident(Spanned::from_position(0, 1, mod_name.to_string()));
        super::find_module(file_manager, anchor, &mod_name)
    }

    #[test]
    fn errors_if_cannot_find_file() {
        let dir = PathBuf::new();
        let mut fm = FileManager::new(&PathBuf::new());

        let file_id = add_file(&mut fm, &dir, "my_dummy_file.nr");

        let result = find_module(&fm, file_id, "foo");
        assert!(matches!(result, Err(DefCollectorErrorKind::UnresolvedModuleDecl { .. })));
    }

    #[test]
    fn errors_because_cannot_find_mod_relative_to_main() {
        let dir = PathBuf::new();
        let mut fm = FileManager::new(&dir);

        let main_file_id = add_file(&mut fm, &dir, "main.nr");
        add_file(&mut fm, &dir, "main/foo.nr");

        let result = find_module(&fm, main_file_id, "foo");
        assert!(matches!(result, Err(DefCollectorErrorKind::UnresolvedModuleDecl { .. })));
    }

    #[test]
    fn errors_because_cannot_find_mod_relative_to_lib() {
        let dir = PathBuf::new();
        let mut fm = FileManager::new(&dir);

        let lib_file_id = add_file(&mut fm, &dir, "lib.nr");
        add_file(&mut fm, &dir, "lib/foo.nr");

        let result = find_module(&fm, lib_file_id, "foo");
        assert!(matches!(result, Err(DefCollectorErrorKind::UnresolvedModuleDecl { .. })));
    }

    #[test]
    fn errors_because_cannot_find_sibling_mod_for_regular_name() {
        let dir = PathBuf::new();
        let mut fm = FileManager::new(&dir);

        let foo_file_id = add_file(&mut fm, &dir, "foo.nr");
        add_file(&mut fm, &dir, "bar.nr");

        let result = find_module(&fm, foo_file_id, "bar");
        assert!(matches!(result, Err(DefCollectorErrorKind::UnresolvedModuleDecl { .. })));
    }

    #[test]
    fn cannot_find_module_in_the_same_directory_for_regular_name() {
        let dir = PathBuf::new();
        let mut fm = FileManager::new(&dir);

        let lib_file_id = add_file(&mut fm, &dir, "lib.nr");
        add_file(&mut fm, &dir, "bar.nr");
        add_file(&mut fm, &dir, "foo.nr");

        // `mod bar` from `lib.nr` should find `bar.nr`
        let bar_file_id = find_module(&fm, lib_file_id, "bar").unwrap();

        // `mod foo` from `bar.nr` should fail to find `foo.nr`
        let result = find_module(&fm, bar_file_id, "foo");
        assert!(matches!(result, Err(DefCollectorErrorKind::UnresolvedModuleDecl { .. })));
    }

    #[test]
    fn finds_module_in_sibling_dir_for_regular_name() {
        let dir = PathBuf::new();
        let mut fm = FileManager::new(&dir);

        let sub_dir_file_id = add_file(&mut fm, &dir, "sub_dir.nr");
        add_file(&mut fm, &dir, "sub_dir/foo.nr");

        // `mod foo` from `sub_dir.nr` should find `sub_dir/foo.nr`
        find_module(&fm, sub_dir_file_id, "foo").unwrap();
    }

    #[test]
    fn finds_module_in_sibling_dir_mod_nr_for_regular_name() {
        let dir = PathBuf::new();
        let mut fm = FileManager::new(&dir);

        let sub_dir_file_id = add_file(&mut fm, &dir, "sub_dir.nr");
        add_file(&mut fm, &dir, "sub_dir/foo/mod.nr");

        // `mod foo` from `sub_dir.nr` should find `sub_dir/foo.nr`
        find_module(&fm, sub_dir_file_id, "foo").unwrap();
    }

    #[test]
    fn finds_module_in_sibling_dir_for_special_name() {
        let dir = PathBuf::new();
        let mut fm = FileManager::new(&dir);

        let lib_file_id = add_file(&mut fm, &dir, "lib.nr");
        add_file(&mut fm, &dir, "sub_dir.nr");
        add_file(&mut fm, &dir, "sub_dir/foo.nr");

        // `mod sub_dir` from `lib.nr` should find `sub_dir.nr`
        let sub_dir_file_id = find_module(&fm, lib_file_id, "sub_dir").unwrap();

        // `mod foo` from `sub_dir.nr` should find `sub_dir/foo.nr`
        find_module(&fm, sub_dir_file_id, "foo").unwrap();
    }

    #[test]
    fn finds_mod_dot_nr_for_special_name() {
        let dir = PathBuf::new();
        let mut fm = FileManager::new(&dir);

        let lib_file_id = add_file(&mut fm, &dir, "lib.nr");
        add_file(&mut fm, &dir, "foo/mod.nr");

        // Check that searching "foo" finds the mod.nr file
        find_module(&fm, lib_file_id, "foo").unwrap();
    }

    #[test]
    fn errors_mod_dot_nr_in_same_directory() {
        let dir = PathBuf::new();
        let mut fm = FileManager::new(&dir);

        let lib_file_id = add_file(&mut fm, &dir, "lib.nr");
        add_file(&mut fm, &dir, "mod.nr");

        // Check that searching "foo" does not pick up the mod.nr file
        let result = find_module(&fm, lib_file_id, "foo");
        assert!(matches!(result, Err(DefCollectorErrorKind::UnresolvedModuleDecl { .. })));
    }

    #[test]
    fn errors_if_file_exists_at_both_potential_module_locations_for_regular_name() {
        let dir = PathBuf::new();
        let mut fm = FileManager::new(&dir);

        let foo_file_id = add_file(&mut fm, &dir, "foo.nr");
        add_file(&mut fm, &dir, "foo/bar.nr");
        add_file(&mut fm, &dir, "foo/bar/mod.nr");

        // Check that `mod bar` from `foo` gives an error
        let result = find_module(&fm, foo_file_id, "bar");
        assert!(matches!(result, Err(DefCollectorErrorKind::OverlappingModuleDecls { .. })));
    }

    #[test]
    fn errors_if_file_exists_at_both_potential_module_locations_for_special_name() {
        let dir = PathBuf::new();
        let mut fm = FileManager::new(&dir);

        let lib_file_id = add_file(&mut fm, &dir, "lib.nr");
        add_file(&mut fm, &dir, "foo.nr");
        add_file(&mut fm, &dir, "foo/mod.nr");

        // Check that searching "foo" gives an error
        let result = find_module(&fm, lib_file_id, "foo");
        assert!(matches!(result, Err(DefCollectorErrorKind::OverlappingModuleDecls { .. })));
    }
}
