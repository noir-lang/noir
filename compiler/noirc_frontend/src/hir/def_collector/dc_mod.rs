use core::str;
use std::path::Path;
use std::rc::Rc;
use std::vec;

use acvm::{AcirField, FieldElement};
use fm::{FileId, FileManager, FILE_EXTENSION};
use noirc_errors::{Location, Span};
use num_bigint::BigUint;
use num_traits::Num;
use rustc_hash::FxHashMap as HashMap;

use crate::ast::{
    Documented, Expression, FunctionDefinition, Ident, ItemVisibility, LetStatement,
    ModuleDeclaration, NoirFunction, NoirStruct, NoirTrait, NoirTraitImpl, NoirTypeAlias, Pattern,
    TraitImplItemKind, TraitItem, TypeImpl, UnresolvedType, UnresolvedTypeData,
};
use crate::hir::resolution::errors::ResolverError;
use crate::node_interner::{ModuleAttributes, NodeInterner, ReferenceId, StructId};
use crate::token::SecondaryAttribute;
use crate::usage_tracker::{UnusedItem, UsageTracker};
use crate::{
    graph::CrateId,
    hir::def_collector::dc_crate::{UnresolvedStruct, UnresolvedTrait},
    node_interner::{FunctionModifiers, TraitId, TypeAliasId},
    parser::{SortedModule, SortedSubModule},
};
use crate::{Generics, Kind, ResolvedGeneric, Type, TypeVariable};

use super::dc_crate::CollectedItems;
use super::dc_crate::ModuleAttribute;
use super::{
    dc_crate::{
        CompilationError, DefCollector, UnresolvedFunctions, UnresolvedGlobal, UnresolvedTraitImpl,
        UnresolvedTypeAlias,
    },
    errors::{DefCollectorErrorKind, DuplicateType},
};
use crate::hir::def_map::{CrateDefMap, LocalModuleId, ModuleData, ModuleId, MAIN_FUNCTION};
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
) -> Vec<(CompilationError, FileId)> {
    let mut collector = ModCollector { def_collector, file_id, module_id };
    let mut errors: Vec<(CompilationError, FileId)> = vec![];

    // First resolve the module declarations
    for decl in ast.module_decls {
        errors.extend(
            collector.parse_module_declaration(context, decl, crate_id, file_id, module_id),
        );
    }

    errors.extend(collector.collect_submodules(
        context,
        crate_id,
        module_id,
        ast.submodules,
        file_id,
    ));

    // Then add the imports to defCollector to resolve once all modules in the hierarchy have been resolved
    for import in ast.imports {
        collector.def_collector.imports.push(ImportDirective {
            visibility: import.visibility,
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

    errors.extend(collector.collect_trait_impls(context, ast.trait_impls, crate_id));

    errors.extend(collector.collect_impls(context, ast.impls, crate_id));

    collector.collect_attributes(
        ast.inner_attributes,
        file_id,
        module_id,
        file_id,
        module_id,
        true,
    );

    errors
}

impl<'a> ModCollector<'a> {
    fn collect_attributes(
        &mut self,
        attributes: Vec<SecondaryAttribute>,
        file_id: FileId,
        module_id: LocalModuleId,
        attribute_file_id: FileId,
        attribute_module_id: LocalModuleId,
        is_inner: bool,
    ) {
        for attribute in attributes {
            self.def_collector.items.module_attributes.push(ModuleAttribute {
                file_id,
                module_id,
                attribute_file_id,
                attribute_module_id,
                attribute,
                is_inner,
            });
        }
    }

    fn collect_globals(
        &mut self,
        context: &mut Context,
        globals: Vec<(Documented<LetStatement>, ItemVisibility)>,
        crate_id: CrateId,
    ) -> Vec<(CompilationError, fm::FileId)> {
        let mut errors = vec![];
        for (global, visibility) in globals {
            let (global, error) = collect_global(
                &mut context.def_interner,
                &mut self.def_collector.def_map,
                &mut context.usage_tracker,
                global,
                visibility,
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

    fn collect_impls(
        &mut self,
        context: &mut Context,
        impls: Vec<TypeImpl>,
        krate: CrateId,
    ) -> Vec<(CompilationError, FileId)> {
        let mut errors = Vec::new();
        let module_id = ModuleId { krate, local_id: self.module_id };

        for r#impl in impls {
            collect_impl(
                &mut context.def_interner,
                &mut self.def_collector.items,
                r#impl,
                self.file_id,
                module_id,
                &mut errors,
            );
        }

        errors
    }

    fn collect_trait_impls(
        &mut self,
        context: &mut Context,
        impls: Vec<NoirTraitImpl>,
        krate: CrateId,
    ) -> Vec<(CompilationError, FileId)> {
        let mut errors = Vec::new();

        for mut trait_impl in impls {
            let trait_name = trait_impl.trait_name.clone();

            let (mut unresolved_functions, associated_types, associated_constants) =
                collect_trait_impl_items(
                    &mut context.def_interner,
                    &mut trait_impl,
                    krate,
                    self.file_id,
                    self.module_id,
                );

            let module = ModuleId { krate, local_id: self.module_id };

            for (_, func_id, noir_function) in &mut unresolved_functions.functions {
                if noir_function.def.attributes.is_test_function() {
                    let error = DefCollectorErrorKind::TestOnAssociatedFunction {
                        span: noir_function.name_ident().span(),
                    };
                    errors.push((error.into(), self.file_id));
                }

                if noir_function.def.attributes.has_export() {
                    let error = DefCollectorErrorKind::ExportOnAssociatedFunction {
                        span: noir_function.name_ident().span(),
                    };
                    errors.push((error.into(), self.file_id));
                }

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
                associated_constants,
                associated_types,

                // These last fields are filled later on
                trait_id: None,
                impl_id: None,
                resolved_object_type: None,
                resolved_generics: Vec::new(),
                resolved_trait_generics: Vec::new(),
            };

            self.def_collector.items.trait_impls.push(unresolved_trait_impl);
        }

        errors
    }

    fn collect_functions(
        &mut self,
        context: &mut Context,
        functions: Vec<Documented<NoirFunction>>,
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
            let Some(func_id) = collect_function(
                &mut context.def_interner,
                &mut self.def_collector.def_map,
                &mut context.usage_tracker,
                &function.item,
                module,
                self.file_id,
                function.doc_comments,
                &mut errors,
            ) else {
                continue;
            };

            // Now link this func_id to a crate level map with the noir function and the module id
            // Encountering a NoirFunction, we retrieve it's module_data to get the namespace
            // Once we have lowered it to a HirFunction, we retrieve it's Id from the DefInterner
            // and replace it
            // With this method we iterate each function in the Crate and not each module
            // This may not be great because we have to pull the module_data for each function
            unresolved_functions.push_fn(self.module_id, func_id, function.item);
        }

        self.def_collector.items.functions.push(unresolved_functions);
        errors
    }

    /// Collect any struct definitions declared within the ast.
    /// Returns a vector of errors if any structs were already defined,
    /// or if a struct has duplicate fields in it.
    fn collect_structs(
        &mut self,
        context: &mut Context,
        types: Vec<Documented<NoirStruct>>,
        krate: CrateId,
    ) -> Vec<(CompilationError, FileId)> {
        let mut definition_errors = vec![];
        for struct_definition in types {
            if let Some((id, the_struct)) = collect_struct(
                &mut context.def_interner,
                &mut self.def_collector.def_map,
                &mut context.usage_tracker,
                struct_definition,
                self.file_id,
                self.module_id,
                krate,
                &mut definition_errors,
            ) {
                self.def_collector.items.types.insert(id, the_struct);
            }
        }
        definition_errors
    }

    /// Collect any type aliases definitions declared within the ast.
    /// Returns a vector of errors if any type aliases were already defined.
    fn collect_type_aliases(
        &mut self,
        context: &mut Context,
        type_aliases: Vec<Documented<NoirTypeAlias>>,
        krate: CrateId,
    ) -> Vec<(CompilationError, FileId)> {
        let mut errors: Vec<(CompilationError, FileId)> = vec![];
        for type_alias in type_aliases {
            let doc_comments = type_alias.doc_comments;
            let type_alias = type_alias.item;
            let name = type_alias.name.clone();
            let visibility = type_alias.visibility;

            // And store the TypeId -> TypeAlias mapping somewhere it is reachable
            let unresolved = UnresolvedTypeAlias {
                file_id: self.file_id,
                module_id: self.module_id,
                type_alias_def: type_alias,
            };

            let resolved_generics = Context::resolve_generics(
                &context.def_interner,
                &unresolved.type_alias_def.generics,
                &mut errors,
                self.file_id,
            );

            let type_alias_id =
                context.def_interner.push_type_alias(&unresolved, resolved_generics);

            context.def_interner.set_doc_comments(ReferenceId::Alias(type_alias_id), doc_comments);

            // Add the type alias to scope so its path can be looked up later
            let result = self.def_collector.def_map.modules[self.module_id.0].declare_type_alias(
                name.clone(),
                visibility,
                type_alias_id,
            );

            let parent_module_id = ModuleId { krate, local_id: self.module_id };
            context.usage_tracker.add_unused_item(
                parent_module_id,
                name.clone(),
                UnusedItem::TypeAlias(type_alias_id),
                visibility,
            );

            if let Err((first_def, second_def)) = result {
                let err = DefCollectorErrorKind::Duplicate {
                    typ: DuplicateType::Function,
                    first_def,
                    second_def,
                };
                errors.push((err.into(), self.file_id));
            }

            self.def_collector.items.type_aliases.insert(type_alias_id, unresolved);

            if context.def_interner.is_in_lsp_mode() {
                let parent_module_id = ModuleId { krate, local_id: self.module_id };
                let name = name.to_string();
                context.def_interner.register_type_alias(
                    type_alias_id,
                    name,
                    visibility,
                    parent_module_id,
                );
            }
        }
        errors
    }

    /// Collect any traits definitions declared within the ast.
    /// Returns a vector of errors if any traits were already defined.
    fn collect_traits(
        &mut self,
        context: &mut Context,
        traits: Vec<Documented<NoirTrait>>,
        krate: CrateId,
    ) -> Vec<(CompilationError, FileId)> {
        let mut errors: Vec<(CompilationError, FileId)> = vec![];
        for trait_definition in traits {
            let doc_comments = trait_definition.doc_comments;
            let trait_definition = trait_definition.item;
            let name = trait_definition.name.clone();

            // Create the corresponding module for the trait namespace
            let trait_id = match self.push_child_module(
                context,
                &name,
                ItemVisibility::Public,
                Location::new(name.span(), self.file_id),
                Vec::new(),
                Vec::new(),
                false,
                false, // is contract
                false, // is struct
            ) {
                Ok(module_id) => TraitId(ModuleId { krate, local_id: module_id.local_id }),
                Err(error) => {
                    errors.push((error.into(), self.file_id));
                    continue;
                }
            };

            context.def_interner.set_doc_comments(ReferenceId::Trait(trait_id), doc_comments);

            // Add the trait to scope so its path can be looked up later
            let visibility = trait_definition.visibility;
            let result = self.def_collector.def_map.modules[self.module_id.0].declare_trait(
                name.clone(),
                visibility,
                trait_id,
            );

            let parent_module_id = ModuleId { krate, local_id: self.module_id };
            context.usage_tracker.add_unused_item(
                parent_module_id,
                name.clone(),
                UnusedItem::Trait(trait_id),
                visibility,
            );

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
            let mut associated_types = Generics::new();

            for trait_item in &trait_definition.items {
                match &trait_item.item {
                    TraitItem::Function {
                        name,
                        generics,
                        parameters,
                        return_type,
                        where_clause,
                        body,
                        is_unconstrained,
                        visibility: _,
                        is_comptime,
                    } => {
                        let func_id = context.def_interner.push_empty_fn();
                        if !method_ids.contains_key(&name.0.contents) {
                            method_ids.insert(name.to_string(), func_id);
                        }

                        let location = Location::new(name.span(), self.file_id);
                        let modifiers = FunctionModifiers {
                            name: name.to_string(),
                            visibility: trait_definition.visibility,
                            // TODO(Maddiaa): Investigate trait implementations with attributes see: https://github.com/noir-lang/noir/issues/2629
                            attributes: crate::token::Attributes::empty(),
                            is_unconstrained: *is_unconstrained,
                            generic_count: generics.len(),
                            is_comptime: *is_comptime,
                            name_location: location,
                        };

                        context
                            .def_interner
                            .push_function_definition(func_id, modifiers, trait_id.0, location);

                        let referenced = ReferenceId::Function(func_id);
                        context.def_interner.add_definition_location(referenced, Some(trait_id.0));

                        if !trait_item.doc_comments.is_empty() {
                            context.def_interner.set_doc_comments(
                                ReferenceId::Function(func_id),
                                trait_item.doc_comments.clone(),
                            );
                        }

                        match self.def_collector.def_map.modules[trait_id.0.local_id.0]
                            .declare_function(name.clone(), ItemVisibility::Public, func_id)
                        {
                            Ok(()) => {
                                if let Some(body) = body {
                                    let impl_method =
                                        NoirFunction::normal(FunctionDefinition::normal(
                                            name,
                                            *is_unconstrained,
                                            generics,
                                            parameters,
                                            body.clone(),
                                            where_clause.clone(),
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
                    TraitItem::Constant { name, typ, default_value: _ } => {
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
                            .declare_global(name.clone(), ItemVisibility::Public, global_id)
                        {
                            let error = DefCollectorErrorKind::Duplicate {
                                typ: DuplicateType::TraitAssociatedConst,
                                first_def,
                                second_def,
                            };
                            errors.push((error.into(), self.file_id));
                        } else {
                            let type_variable_id = context.def_interner.next_type_variable_id();
                            let typ = self.resolve_associated_constant_type(typ, &mut errors);

                            associated_types.push(ResolvedGeneric {
                                name: Rc::new(name.to_string()),
                                type_var: TypeVariable::unbound(
                                    type_variable_id,
                                    Kind::numeric(typ),
                                ),
                                span: name.span(),
                            });
                        }
                    }
                    TraitItem::Type { name } => {
                        if let Err((first_def, second_def)) = self.def_collector.def_map.modules
                            [trait_id.0.local_id.0]
                            .declare_type_alias(
                                name.clone(),
                                ItemVisibility::Public,
                                TypeAliasId::dummy_id(),
                            )
                        {
                            let error = DefCollectorErrorKind::Duplicate {
                                typ: DuplicateType::TraitAssociatedType,
                                first_def,
                                second_def,
                            };
                            errors.push((error.into(), self.file_id));
                        } else {
                            let type_variable_id = context.def_interner.next_type_variable_id();
                            associated_types.push(ResolvedGeneric {
                                name: Rc::new(name.to_string()),
                                type_var: TypeVariable::unbound(type_variable_id, Kind::Normal),
                                span: name.span(),
                            });
                        }
                    }
                }
            }

            let resolved_generics = Context::resolve_generics(
                &context.def_interner,
                &trait_definition.generics,
                &mut errors,
                self.file_id,
            );

            let unresolved = UnresolvedTrait {
                file_id: self.file_id,
                module_id: self.module_id,
                crate_id: krate,
                trait_def: trait_definition,
                method_ids,
                fns_with_default_impl: unresolved_functions,
            };
            context.def_interner.push_empty_trait(
                trait_id,
                &unresolved,
                resolved_generics,
                associated_types,
            );

            if context.def_interner.is_in_lsp_mode() {
                let parent_module_id = ModuleId { krate, local_id: self.module_id };
                context.def_interner.register_trait(
                    trait_id,
                    name.to_string(),
                    visibility,
                    parent_module_id,
                );
            }

            self.def_collector.items.traits.insert(trait_id, unresolved);
        }
        errors
    }

    fn collect_submodules(
        &mut self,
        context: &mut Context,
        crate_id: CrateId,
        parent_module_id: LocalModuleId,
        submodules: Vec<Documented<SortedSubModule>>,
        file_id: FileId,
    ) -> Vec<(CompilationError, FileId)> {
        let mut errors: Vec<(CompilationError, FileId)> = vec![];
        for submodule in submodules {
            let mut doc_comments = submodule.doc_comments;
            let submodule = submodule.item;

            match self.push_child_module(
                context,
                &submodule.name,
                submodule.visibility,
                Location::new(submodule.name.span(), file_id),
                submodule.outer_attributes.clone(),
                submodule.contents.inner_attributes.clone(),
                true,
                submodule.is_contract,
                false, // is struct
            ) {
                Ok(child) => {
                    self.collect_attributes(
                        submodule.outer_attributes,
                        file_id,
                        child.local_id,
                        file_id,
                        parent_module_id,
                        false,
                    );

                    if !(doc_comments.is_empty()
                        && submodule.contents.inner_doc_comments.is_empty())
                    {
                        doc_comments.extend(submodule.contents.inner_doc_comments.clone());

                        context
                            .def_interner
                            .set_doc_comments(ReferenceId::Module(child), doc_comments);
                    }

                    errors.extend(collect_defs(
                        self.def_collector,
                        submodule.contents,
                        file_id,
                        child.local_id,
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
    #[allow(clippy::too_many_arguments)]
    fn parse_module_declaration(
        &mut self,
        context: &mut Context,
        mod_decl: Documented<ModuleDeclaration>,
        crate_id: CrateId,
        parent_file_id: FileId,
        parent_module_id: LocalModuleId,
    ) -> Vec<(CompilationError, FileId)> {
        let mut doc_comments = mod_decl.doc_comments;
        let mod_decl = mod_decl.item;

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
        let ast = ast.into_sorted();

        errors.extend(
            parsing_errors.iter().map(|e| (e.clone().into(), child_file_id)).collect::<Vec<_>>(),
        );

        // Add module into def collector and get a ModuleId
        match self.push_child_module(
            context,
            &mod_decl.ident,
            mod_decl.visibility,
            Location::new(Span::empty(0), child_file_id),
            mod_decl.outer_attributes.clone(),
            ast.inner_attributes.clone(),
            true,
            false, // is contract
            false, // is struct
        ) {
            Ok(child_mod_id) => {
                self.collect_attributes(
                    mod_decl.outer_attributes,
                    child_file_id,
                    child_mod_id.local_id,
                    parent_file_id,
                    parent_module_id,
                    false,
                );

                // Track that the "foo" in `mod foo;` points to the module "foo"
                context.def_interner.add_module_reference(child_mod_id, location);

                if !(doc_comments.is_empty() && ast.inner_doc_comments.is_empty()) {
                    doc_comments.extend(ast.inner_doc_comments.clone());

                    context
                        .def_interner
                        .set_doc_comments(ReferenceId::Module(child_mod_id), doc_comments);
                }

                errors.extend(collect_defs(
                    self.def_collector,
                    ast,
                    child_file_id,
                    child_mod_id.local_id,
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
    #[allow(clippy::too_many_arguments)]
    fn push_child_module(
        &mut self,
        context: &mut Context,
        mod_name: &Ident,
        visibility: ItemVisibility,
        mod_location: Location,
        outer_attributes: Vec<SecondaryAttribute>,
        inner_attributes: Vec<SecondaryAttribute>,
        add_to_parent_scope: bool,
        is_contract: bool,
        is_struct: bool,
    ) -> Result<ModuleId, DefCollectorErrorKind> {
        push_child_module(
            &mut context.def_interner,
            &mut self.def_collector.def_map,
            self.module_id,
            mod_name,
            visibility,
            mod_location,
            outer_attributes,
            inner_attributes,
            add_to_parent_scope,
            is_contract,
            is_struct,
        )
    }

    fn resolve_associated_constant_type(
        &self,
        typ: &UnresolvedType,
        errors: &mut Vec<(CompilationError, FileId)>,
    ) -> Type {
        match &typ.typ {
            UnresolvedTypeData::FieldElement => Type::FieldElement,
            UnresolvedTypeData::Integer(sign, bits) => Type::Integer(*sign, *bits),
            _ => {
                let span = typ.span;
                let error = ResolverError::AssociatedConstantsMustBeNumeric { span };
                errors.push((error.into(), self.file_id));
                Type::Error
            }
        }
    }
}

/// Add a child module to the current def_map.
/// On error this returns None and pushes to `errors`
#[allow(clippy::too_many_arguments)]
fn push_child_module(
    interner: &mut NodeInterner,
    def_map: &mut CrateDefMap,
    parent: LocalModuleId,
    mod_name: &Ident,
    visibility: ItemVisibility,
    mod_location: Location,
    outer_attributes: Vec<SecondaryAttribute>,
    inner_attributes: Vec<SecondaryAttribute>,
    add_to_parent_scope: bool,
    is_contract: bool,
    is_struct: bool,
) -> Result<ModuleId, DefCollectorErrorKind> {
    // Note: the difference between `location` and `mod_location` is:
    // - `mod_location` will point to either the token "foo" in `mod foo { ... }`
    //   if it's an inline module, or the first char of a the file if it's an external module.
    // - `location` will always point to the token "foo" in `mod foo` regardless of whether
    //   it's inline or external.
    // Eventually the location put in `ModuleData` is used for codelenses about `contract`s,
    // so we keep using `location` so that it continues to work as usual.
    let location = Location::new(mod_name.span(), mod_location.file);
    let new_module = ModuleData::new(
        Some(parent),
        location,
        outer_attributes,
        inner_attributes,
        is_contract,
        is_struct,
    );

    let module_id = def_map.modules.insert(new_module);
    let modules = &mut def_map.modules;

    // Update the parent module to reference the child
    modules[parent.0].children.insert(mod_name.clone(), LocalModuleId(module_id));

    let mod_id = ModuleId { krate: def_map.krate, local_id: LocalModuleId(module_id) };

    // Add this child module into the scope of the parent module as a module definition
    // module definitions are definitions which can only exist at the module level.
    // ModuleDefinitionIds can be used across crates since they contain the CrateId
    //
    // We do not want to do this in the case of struct modules (each struct type corresponds
    // to a child module containing its methods) since the module name should not shadow
    // the struct name.
    if add_to_parent_scope {
        if let Err((first_def, second_def)) =
            modules[parent.0].declare_child_module(mod_name.to_owned(), visibility, mod_id)
        {
            let err = DefCollectorErrorKind::Duplicate {
                typ: DuplicateType::Module,
                first_def,
                second_def,
            };
            return Err(err);
        }

        interner.add_module_attributes(
            mod_id,
            ModuleAttributes {
                name: mod_name.0.contents.clone(),
                location: mod_location,
                parent: Some(parent),
                visibility,
            },
        );

        if interner.is_in_lsp_mode() {
            interner.register_module(mod_id, visibility, mod_name.0.contents.clone());
        }
    }

    Ok(mod_id)
}

#[allow(clippy::too_many_arguments)]
pub fn collect_function(
    interner: &mut NodeInterner,
    def_map: &mut CrateDefMap,
    usage_tracker: &mut UsageTracker,
    function: &NoirFunction,
    module: ModuleId,
    file: FileId,
    doc_comments: Vec<String>,
    errors: &mut Vec<(CompilationError, FileId)>,
) -> Option<crate::node_interner::FuncId> {
    if let Some(field) = function.attributes().get_field_attribute() {
        if !is_native_field(&field) {
            return None;
        }
    }

    let module_data = &mut def_map.modules[module.local_id.0];

    let is_test = function.def.attributes.is_test_function();
    let is_entry_point_function = if module_data.is_contract {
        function.attributes().is_contract_entry_point()
    } else {
        function.name() == MAIN_FUNCTION
    };
    let has_export = function.def.attributes.has_export();

    let name = function.name_ident().clone();
    let func_id = interner.push_empty_fn();
    let visibility = function.def.visibility;
    let location = Location::new(function.span(), file);
    interner.push_function(func_id, &function.def, module, location);
    if interner.is_in_lsp_mode() && !function.def.is_test() {
        interner.register_function(func_id, &function.def);
    }

    if !is_test && !is_entry_point_function && !has_export {
        let item = UnusedItem::Function(func_id);
        usage_tracker.add_unused_item(module, name.clone(), item, visibility);
    }

    interner.set_doc_comments(ReferenceId::Function(func_id), doc_comments);

    // Add function to scope/ns of the module
    let result = def_map.modules[module.local_id.0].declare_function(name, visibility, func_id);
    if let Err((first_def, second_def)) = result {
        let error = DefCollectorErrorKind::Duplicate {
            typ: DuplicateType::Function,
            first_def,
            second_def,
        };
        errors.push((error.into(), file));
    }
    Some(func_id)
}

#[allow(clippy::too_many_arguments)]
pub fn collect_struct(
    interner: &mut NodeInterner,
    def_map: &mut CrateDefMap,
    usage_tracker: &mut UsageTracker,
    struct_definition: Documented<NoirStruct>,
    file_id: FileId,
    module_id: LocalModuleId,
    krate: CrateId,
    definition_errors: &mut Vec<(CompilationError, FileId)>,
) -> Option<(StructId, UnresolvedStruct)> {
    let doc_comments = struct_definition.doc_comments;
    let struct_definition = struct_definition.item;

    check_duplicate_field_names(&struct_definition, file_id, definition_errors);

    let name = struct_definition.name.clone();

    let unresolved = UnresolvedStruct { file_id, module_id, struct_def: struct_definition };

    let resolved_generics = Context::resolve_generics(
        interner,
        &unresolved.struct_def.generics,
        definition_errors,
        file_id,
    );

    // Create the corresponding module for the struct namespace
    let location = Location::new(name.span(), file_id);
    let id = match push_child_module(
        interner,
        def_map,
        module_id,
        &name,
        ItemVisibility::Public,
        location,
        Vec::new(),
        Vec::new(),
        false, // add to parent scope
        false, // is contract
        true,  // is struct
    ) {
        Ok(module_id) => {
            interner.new_struct(&unresolved, resolved_generics, krate, module_id.local_id, file_id)
        }
        Err(error) => {
            definition_errors.push((error.into(), file_id));
            return None;
        }
    };

    interner.set_doc_comments(ReferenceId::Struct(id), doc_comments);

    for (index, field) in unresolved.struct_def.fields.iter().enumerate() {
        if !field.doc_comments.is_empty() {
            interner
                .set_doc_comments(ReferenceId::StructMember(id, index), field.doc_comments.clone());
        }
    }

    // Add the struct to scope so its path can be looked up later
    let visibility = unresolved.struct_def.visibility;
    let result = def_map.modules[module_id.0].declare_struct(name.clone(), visibility, id);

    let parent_module_id = ModuleId { krate, local_id: module_id };

    if !unresolved.struct_def.is_abi() {
        usage_tracker.add_unused_item(
            parent_module_id,
            name.clone(),
            UnusedItem::Struct(id),
            visibility,
        );
    }

    if let Err((first_def, second_def)) = result {
        let error = DefCollectorErrorKind::Duplicate {
            typ: DuplicateType::TypeDefinition,
            first_def,
            second_def,
        };
        definition_errors.push((error.into(), file_id));
    }

    if interner.is_in_lsp_mode() {
        interner.register_struct(id, name.to_string(), visibility, parent_module_id);
    }

    Some((id, unresolved))
}

pub fn collect_impl(
    interner: &mut NodeInterner,
    items: &mut CollectedItems,
    r#impl: TypeImpl,
    file_id: FileId,
    module_id: ModuleId,
    errors: &mut Vec<(CompilationError, FileId)>,
) {
    let mut unresolved_functions =
        UnresolvedFunctions { file_id, functions: Vec::new(), trait_id: None, self_type: None };

    for (method, _) in r#impl.methods {
        let doc_comments = method.doc_comments;
        let mut method = method.item;

        if method.def.attributes.is_test_function() {
            let error = DefCollectorErrorKind::TestOnAssociatedFunction {
                span: method.name_ident().span(),
            };
            errors.push((error.into(), file_id));
            continue;
        }
        if method.def.attributes.has_export() {
            let error = DefCollectorErrorKind::ExportOnAssociatedFunction {
                span: method.name_ident().span(),
            };
            errors.push((error.into(), file_id));
        }

        let func_id = interner.push_empty_fn();
        method.def.where_clause.extend(r#impl.where_clause.clone());
        let location = Location::new(method.span(), file_id);
        interner.push_function(func_id, &method.def, module_id, location);
        unresolved_functions.push_fn(module_id.local_id, func_id, method);
        interner.set_doc_comments(ReferenceId::Function(func_id), doc_comments);
    }

    let key = (r#impl.object_type, module_id.local_id);
    let methods = items.impls.entry(key).or_default();
    methods.push((r#impl.generics, r#impl.type_span, unresolved_functions));
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

type AssociatedTypes = Vec<(Ident, UnresolvedType)>;
type AssociatedConstants = Vec<(Ident, UnresolvedType, Expression)>;

/// Returns a tuple of (methods, associated types, associated constants)
pub(crate) fn collect_trait_impl_items(
    interner: &mut NodeInterner,
    trait_impl: &mut NoirTraitImpl,
    krate: CrateId,
    file_id: FileId,
    local_id: LocalModuleId,
) -> (UnresolvedFunctions, AssociatedTypes, AssociatedConstants) {
    let mut unresolved_functions =
        UnresolvedFunctions { file_id, functions: Vec::new(), trait_id: None, self_type: None };

    let mut associated_types = Vec::new();
    let mut associated_constants = Vec::new();

    let module = ModuleId { krate, local_id };

    for item in std::mem::take(&mut trait_impl.items) {
        match item.item.kind {
            TraitImplItemKind::Function(mut impl_method) => {
                // Regardless of what visibility was on the source code, treat it as public
                // (a warning is produced during parsing for this)
                impl_method.def.visibility = ItemVisibility::Public;

                let func_id = interner.push_empty_fn();
                let location = Location::new(impl_method.span(), file_id);
                interner.push_function(func_id, &impl_method.def, module, location);
                interner.set_doc_comments(ReferenceId::Function(func_id), item.doc_comments);
                unresolved_functions.push_fn(local_id, func_id, impl_method);
            }
            TraitImplItemKind::Constant(name, typ, expr) => {
                associated_constants.push((name, typ, expr));
            }
            TraitImplItemKind::Type { name, alias } => {
                associated_types.push((name, alias));
            }
        }
    }

    (unresolved_functions, associated_types, associated_constants)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn collect_global(
    interner: &mut NodeInterner,
    def_map: &mut CrateDefMap,
    usage_tracker: &mut UsageTracker,
    global: Documented<LetStatement>,
    visibility: ItemVisibility,
    file_id: FileId,
    module_id: LocalModuleId,
    crate_id: CrateId,
) -> (UnresolvedGlobal, Option<(CompilationError, FileId)>) {
    let doc_comments = global.doc_comments;
    let global = global.item;

    let name = global.pattern.name_ident().clone();
    let is_abi = global.attributes.iter().any(|attribute| attribute.is_abi());

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
    let result = def_map.modules[module_id.0].declare_global(name.clone(), visibility, global_id);

    // Globals marked as ABI don't have to be used.
    if !is_abi {
        let parent_module_id = ModuleId { krate: crate_id, local_id: module_id };
        usage_tracker.add_unused_item(
            parent_module_id,
            name,
            UnusedItem::Global(global_id),
            visibility,
        );
    }

    let error = result.err().map(|(first_def, second_def)| {
        let err =
            DefCollectorErrorKind::Duplicate { typ: DuplicateType::Global, first_def, second_def };
        (err.into(), file_id)
    });

    interner.set_doc_comments(ReferenceId::Global(global_id), doc_comments);

    let global = UnresolvedGlobal { file_id, module_id, global_id, stmt_def: global, visibility };
    (global, error)
}

fn check_duplicate_field_names(
    struct_definition: &NoirStruct,
    file: FileId,
    definition_errors: &mut Vec<(CompilationError, FileId)>,
) {
    let mut seen_field_names = std::collections::HashSet::new();
    for field in &struct_definition.fields {
        let field_name = &field.item.name;

        if seen_field_names.insert(field_name) {
            continue;
        }

        let previous_field_name = *seen_field_names.get(field_name).unwrap();
        definition_errors.push((
            DefCollectorErrorKind::DuplicateField {
                first_def: previous_field_name.clone(),
                second_def: field_name.clone(),
            }
            .into(),
            file,
        ));
    }
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
