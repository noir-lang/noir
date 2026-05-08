//! Function metadata and function body elaboration.
//!
//! This module handles the definition and elaboration of functions:
//! - Collect and resolve function metadata (i.e., signatures with type information). Type information includes:
//!   - Generics, parameters, trait constraints which will all be resolved while collecting function metas.
//!   - This metadata is also used for elaboration of impls and trait impls
//! - Second stage elaboration strategy of function bodies and their return type.
//!   - Shared strategy for all types of functions (standalone, impl, trait impl)

use std::collections::HashSet;

use iter_extended::vecmap;
use itertools::Itertools;
use noirc_errors::Location;

use crate::{
    Kind, ResolvedGeneric, Type, TypeVariable,
    ast::{
        BlockExpression, FunctionKind, Ident, IdentOrQuotedType, NoirFunction, Param,
        UnresolvedGeneric, UnresolvedGenerics, UnresolvedTraitConstraint, UnresolvedType,
        UnresolvedTypeData,
    },
    elaborator::{
        UnstableFeature, lints,
        types::{WildcardAllowed, WildcardDisallowedContext},
    },
    hir::{
        def_collector::dc_crate::{ImplMap, UnresolvedFunctions, UnresolvedTraitImpl},
        def_map::LocalModuleId,
        resolution::errors::ResolverError,
        type_check::TypeCheckError,
    },
    hir_def::{
        expr::HirIdent,
        function::{FuncMeta, FunctionBody, HirFunction},
        stmt::HirPattern,
        traits::TraitConstraint,
    },
    node_interner::{
        DefinitionKind, DependencyId, FuncId, FunctionModifiers, TraitId, TraitImplId,
    },
    shared::Visibility,
};

use super::Elaborator;

type ResolvedParametersInfo = (Vec<(HirPattern, Type, Visibility)>, Vec<Type>, Vec<HirIdent>);

/// Captures everything needed to lazily resolve a function's metadata.
///
/// Function metas are registered with this elaborator-side context up-front, then
/// resolved lazily when a meta is first read (or eagerly drained at the end of
/// elaboration). This lets meta resolution happen *after* trait impls are bound
/// and globals can elaborate, and lets forward references between metas, globals,
/// and other functions resolve in any order.
pub(super) struct UnresolvedFunctionMeta {
    pub(super) func: NoirFunction,
    pub(super) local_module: LocalModuleId,
    pub(super) self_type: Option<Type>,
    /// Generics in scope from an enclosing `impl<...>` or trait impl, already
    /// resolved. The function's own generics are added during meta resolution.
    pub(super) outer_generics: Vec<ResolvedGeneric>,
    pub(super) current_trait: Option<TraitId>,
    pub(super) current_trait_impl: Option<TraitImplId>,
    pub(super) extra_trait_constraints: Vec<(TraitConstraint, Location)>,
}

impl Elaborator<'_> {
    /// Registers all functions, impl methods, and trait impl methods for *lazy*
    /// metadata resolution. Each entry captures the elaborator context (self_type,
    /// outer generics, trait/impl ids) needed to resolve the meta later. Trait
    /// impls are also prepared here so that `<Object as Trait>::Type` references
    /// inside any signature can be looked up during meta resolution.
    ///
    /// The metas are not actually resolved here — they are resolved on demand.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn register_function_metas(
        &mut self,
        functions: &mut [UnresolvedFunctions],
        impls: &mut ImplMap,
        trait_impls: &mut [UnresolvedTraitImpl],
    ) {
        // Prepare all trait impls, so we can refer to `<Object as Trait>::Type` in function signatures.
        let trait_constraints_and_generics = vecmap(trait_impls.iter_mut(), |trait_impl| {
            self.prepare_trait_impl_for_function_meta_definition(trait_impl)
        });

        // Register metas for regular functions
        for function_set in functions {
            self.register_function_metas_for_functions(function_set, &[]);
        }

        // Register metas for impl functions
        for ((self_type, local_module), function_sets) in impls {
            self.register_function_metas_for_impl(self_type, *local_module, function_sets);
        }

        // Register metas for trait impl functions
        for (trait_impl, (trait_constraints, generics)) in
            trait_impls.iter_mut().zip_eq(trait_constraints_and_generics)
        {
            self.register_function_metas_for_trait_impl(trait_impl, trait_constraints, generics);
        }
    }

    /// Registers each function in the set as an unresolved meta. The functions are
    /// not iterated by `define_function_meta` here — they are resolved lazily.
    #[tracing::instrument(level = "trace", skip_all)]
    fn register_function_metas_for_functions(
        &mut self,
        function_set: &UnresolvedFunctions,
        extra_constraints: &[(TraitConstraint, Location)],
    ) {
        for (local_module, id, func) in &function_set.functions {
            self.unresolved_function_metas.insert(
                *id,
                UnresolvedFunctionMeta {
                    func: func.clone(),
                    local_module: *local_module,
                    self_type: None,
                    outer_generics: Vec::new(),
                    current_trait: None,
                    current_trait_impl: None,
                    extra_trait_constraints: extra_constraints.to_vec(),
                },
            );
        }
    }

    /// Registers each impl method as an unresolved meta, resolving the impl's
    /// self type and generics so that they're captured for later meta resolution.
    #[tracing::instrument(level = "trace", skip_all)]
    fn register_function_metas_for_impl(
        &mut self,
        self_type: &UnresolvedType,
        local_module: LocalModuleId,
        function_sets: &mut Vec<(UnresolvedGenerics, Location, UnresolvedFunctions)>,
    ) {
        self.local_module = Some(local_module);

        for (generics, _, function_set) in function_sets {
            // Prepare the impl: adds the impl generics to scope so the self type can
            // reference them, then resolve the self type.
            self.add_generics(generics);

            let wildcard_allowed = WildcardAllowed::No(WildcardDisallowedContext::ImplType);
            let self_type = self.resolve_type(self_type.clone(), wildcard_allowed);
            function_set.self_type = Some(self_type.clone());

            let outer_generics = self.generics.clone();
            for (method_module, id, func) in &function_set.functions {
                self.unresolved_function_metas.insert(
                    *id,
                    UnresolvedFunctionMeta {
                        func: func.clone(),
                        local_module: *method_module,
                        self_type: Some(self_type.clone()),
                        outer_generics: outer_generics.clone(),
                        current_trait: None,
                        current_trait_impl: None,
                        extra_trait_constraints: Vec::new(),
                    },
                );
            }

            self.generics.clear();
        }
    }

    /// Registers each trait impl method as an unresolved meta, capturing the trait
    /// impl's self type, generics, and trait/impl ids for later resolution.
    #[tracing::instrument(level = "trace", skip_all)]
    fn register_function_metas_for_trait_impl(
        &mut self,
        trait_impl: &UnresolvedTraitImpl,
        new_generics_trait_constraints: Vec<(TraitConstraint, Location)>,
        generics: Vec<ResolvedGeneric>,
    ) {
        let self_type = trait_impl.methods.self_type.clone();
        for (method_module, id, func) in &trait_impl.methods.functions {
            self.unresolved_function_metas.insert(
                *id,
                UnresolvedFunctionMeta {
                    func: func.clone(),
                    local_module: *method_module,
                    self_type: self_type.clone(),
                    outer_generics: generics.clone(),
                    current_trait: trait_impl.trait_id,
                    current_trait_impl: trait_impl.impl_id,
                    extra_trait_constraints: new_generics_trait_constraints.clone(),
                },
            );
        }
    }

    /// Returns whether `func_id` is a method on a trait impl, looking through both
    /// resolved and yet-to-be-resolved metas. Useful when callers only need to
    /// distinguish trait-impl methods without forcing full meta resolution (which
    /// may not be possible if other borrows are live).
    pub(crate) fn function_is_trait_impl_method(&self, func_id: FuncId) -> bool {
        if let Some(info) = self.unresolved_function_metas.get(&func_id) {
            return info.current_trait_impl.is_some();
        }
        self.interner.try_function_meta(&func_id).is_some_and(|meta| meta.trait_impl.is_some())
    }

    /// If `func_id` was registered but its meta hasn't been resolved yet, resolve
    /// it now under the registered context. This is the lazy entry point used by
    /// callers that need a function's meta before the end-of-elaboration drain.
    ///
    /// A no-op for: functions already resolved, functions that aren't in the side
    /// map (e.g. trait method definitions and default impl methods, which are
    /// resolved eagerly elsewhere), and re-entrant calls for a function currently
    /// being resolved (the `remove` returns `None`, breaking cycles — the cycle
    /// will surface as an error from later phases like dependency-cycle detection).
    pub(crate) fn define_function_meta_if_undefined(&mut self, func_id: FuncId) {
        let Some(info) = self.unresolved_function_metas.remove(&func_id) else {
            return;
        };
        self.resolve_unresolved_function_meta(func_id, info);
    }

    /// Lazy-aware accessor for a function's [FuncMeta]. Resolves the meta first
    /// if it's still deferred, then borrows it from the interner. This is the
    /// preferred way to read a function meta during elaboration — equivalent to
    /// `define_function_meta_if_undefined(id)` followed by
    /// `interner.function_meta(&id)`, but in one call.
    pub(crate) fn function_meta(&mut self, func_id: FuncId) -> &FuncMeta {
        self.define_function_meta_if_undefined(func_id);
        self.interner.function_meta(&func_id)
    }

    /// Mutable counterpart of [Self::function_meta]. Resolves the meta first if
    /// deferred, then returns a mutable reference for callers that mutate the
    /// stored meta (e.g. `function_def_disable`).
    pub(crate) fn function_meta_mut(&mut self, func_id: FuncId) -> &mut FuncMeta {
        self.define_function_meta_if_undefined(func_id);
        self.interner.function_meta_mut(&func_id)
    }

    /// Drains unresolved metas, skipping the given set. Used to keep some
    /// signatures deferred (typically top-level free functions whose parameter
    /// or return types might mention items generated by attributes that have
    /// not yet run) while still draining everything else (impl methods, trait
    /// impl methods, attributed top-level functions) so that later phases like
    /// global elaboration and method lookup see fully resolved metas.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn drain_unresolved_function_metas_skipping(&mut self, skip: &HashSet<FuncId>) {
        let to_resolve: Vec<FuncId> = self
            .unresolved_function_metas
            .keys()
            .copied()
            .filter(|id| !skip.contains(id))
            .collect();
        for func_id in to_resolve {
            if let Some(info) = self.unresolved_function_metas.remove(&func_id) {
                self.resolve_unresolved_function_meta(func_id, info);
            }
        }
    }

    /// Sets up the elaborator context that was captured at registration time, then
    /// runs `define_function_meta`. Restores the prior context on the way out so
    /// callers (which may themselves be mid-resolution) aren't disturbed.
    fn resolve_unresolved_function_meta(&mut self, func_id: FuncId, info: UnresolvedFunctionMeta) {
        let UnresolvedFunctionMeta {
            mut func,
            local_module,
            self_type,
            outer_generics,
            current_trait,
            current_trait_impl,
            extra_trait_constraints,
        } = info;

        let prev_local_module = self.local_module;
        let prev_self_type = self.self_type.take();
        let prev_generics = std::mem::replace(&mut self.generics, outer_generics);
        let prev_current_trait = self.current_trait.take();
        let prev_current_trait_impl = self.current_trait_impl.take();
        // `define_function_meta` unconditionally clears `current_item` on the way
        // out. When meta resolution is triggered lazily from inside another
        // elaboration (e.g. while elaborating a comptime function's body), the
        // caller's `current_item` would be lost, which then breaks
        // [Self::in_comptime_context] for the rest of that elaboration.
        let prev_current_item = self.current_item;

        self.local_module = Some(local_module);
        self.self_type = self_type;
        self.current_trait = current_trait;
        self.current_trait_impl = current_trait_impl;

        // The `trait_id` argument to `define_function_meta` represents the trait
        // that *defines* this method (set for trait method declarations,
        // recorded as `meta.trait_id`). Trait impl methods record their impl on
        // `meta.trait_impl` and use `current_trait` purely for context — they
        // must pass `None` here so `meta.trait_id` stays None.
        let defining_trait = if current_trait_impl.is_some() { None } else { current_trait };
        self.recover_generics(|this| {
            this.define_function_meta(&mut func, func_id, defining_trait, &extra_trait_constraints);
        });

        self.local_module = prev_local_module;
        self.self_type = prev_self_type;
        self.generics = prev_generics;
        self.current_trait = prev_current_trait;
        self.current_trait_impl = prev_current_trait_impl;
        self.current_item = prev_current_item;
    }

    /// Extracts and stores metadata from a function definition.
    ///
    /// This resolves the function's signature including generics, parameters, return type,
    /// and trait constraints. The function body is stored unresolved and will be elaborated
    /// later by [Self::elaborate_function].
    ///
    /// Prerequisite: any implicit generics from enclosing impls have already been added
    /// to scope via [Self::add_generics].
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn define_function_meta(
        &mut self,
        func: &mut NoirFunction,
        func_id: FuncId,
        trait_id: Option<TraitId>,
        extra_trait_constraints: &[(TraitConstraint, Location)],
    ) {
        self.scopes.start_function();
        self.current_item = Some(DependencyId::Function(func_id));
        let old_comptime_value =
            std::mem::replace(&mut self.in_comptime_context, func.def.is_comptime);

        let location = func.name_ident().location();
        let id = self.interner.function_definition_id(func_id);
        let name_ident = HirIdent::non_trait_method(id, location);

        // Add generics to scope
        let (mut generics, associated_generics_trait_constraints) =
            self.add_function_generics_to_scope(&func.def.generics, &mut func.def.where_clause);

        // Setup trait constraints
        for (extra_constraint, location) in extra_trait_constraints {
            let bound = &extra_constraint.trait_bound;
            self.add_trait_bound_to_scope(*location, &extra_constraint.typ, bound);
        }

        let mut trait_constraints =
            self.resolve_trait_constraints_and_add_to_scope(&func.def.where_clause);

        // Add constraints for parent traits that have associated types.
        let (parent_generics, parent_constraints) =
            self.add_parent_associated_type_constraints(&trait_constraints);
        generics.extend(parent_generics);
        trait_constraints.extend(parent_constraints);

        let mut extra_trait_constraints =
            vecmap(extra_trait_constraints, |(constraint, _)| constraint.clone());
        extra_trait_constraints.extend(associated_generics_trait_constraints);

        // Resolve parameters
        let (parameters, parameter_types, parameter_idents) =
            self.resolve_function_parameters(func, &mut generics, &mut trait_constraints);

        // Resolve return type
        let wildcard_allowed = WildcardAllowed::No(WildcardDisallowedContext::FunctionReturn);
        let return_type = Box::new(self.use_type(func.return_type(), wildcard_allowed));

        let is_crate_root = self.is_at_crate_root();
        let is_entry_point = func.is_entry_point(self.is_function_in_contract(), is_crate_root);
        // Temporary allow vectors for contract functions, until contracts are re-factored.
        if !func.attributes().has_contract_library_method() {
            let output = true;
            if let Err(err) = Self::check_if_type_is_valid_for_program(
                &return_type,
                is_entry_point || func.is_test_or_fuzz(),
                func.has_inline_attribute(),
                output,
                func.return_type().location,
            ) {
                self.push_err(err);
            }
        }

        // Build function type
        let mut typ = Type::Function(
            parameter_types,
            return_type,
            Box::new(Type::Unit),
            func.def.is_unconstrained,
        );
        if !generics.is_empty() {
            typ = Type::Forall(generics, Box::new(typ));
        }
        self.interner.push_definition_type(name_ident.id, typ.clone());

        // Set up metadata to place on main FuncMeta structure
        let direct_generics = func.def.generics.iter();
        let direct_generics = direct_generics
            .filter_map(|generic| {
                generic.ident().ident().and_then(|name| self.find_generic(name.as_str())).cloned()
            })
            .collect();

        let statements = std::mem::take(&mut func.def.body.statements);
        let body = BlockExpression { statements };

        let struct_id = if let Some(Type::DataType(struct_type, _)) = &self.self_type {
            Some(struct_type.borrow().id)
        } else {
            None
        };

        // Remove the traits assumed by `resolve_trait_constraints` from scope
        self.remove_trait_constraints_from_scope(
            trait_constraints.iter().chain(extra_trait_constraints.iter()),
        );

        let meta = FuncMeta {
            name: name_ident,
            kind: func.kind,
            location,
            typ,
            direct_generics,
            all_generics: self.generics.clone(),
            type_id: struct_id,
            trait_id,
            trait_impl: self.current_trait_impl,
            enum_variant_index: None,
            parameters: parameters.into(),
            parameter_idents,
            return_type: func.def.return_type.clone(),
            return_visibility: func.def.return_visibility,
            return_visibility_location: func.def.return_visibility_location,
            has_body: !func.def.body.is_empty(),
            trait_constraints,
            extra_trait_constraints,
            is_entry_point,
            has_inline_attribute: func.has_inline_attribute(),
            source_crate: self.crate_id,
            source_module: self.local_module(),
            function_body: FunctionBody::Unresolved(func.kind, body, func.def.location),
            self_type: self.self_type.clone(),
            source_file: location.file,
        };

        self.interner.push_fn_meta(meta, func_id);
        self.scopes.end_function();
        self.current_item = None;
        self.in_comptime_context = old_comptime_value;
    }

    /// Adds function generics and associated generics (from where clause) to scope.
    ///
    /// Returns (generics, associated_generics_trait_constraints) where generics contains
    /// both associated and explicit generics in the correct order (associated first, then explicit function generics).
    #[tracing::instrument(level = "trace", skip_all)]
    fn add_function_generics_to_scope(
        &mut self,
        func_generics: &UnresolvedGenerics,
        where_clause: &mut [UnresolvedTraitConstraint],
    ) -> (Vec<TypeVariable>, Vec<TraitConstraint>) {
        self.add_generics(func_generics);

        let func_generics = vecmap(&self.generics, |generic| generic.type_var.clone());

        let associated_generics = self.desugar_trait_constraints(where_clause);

        let mut generics = Vec::with_capacity(associated_generics.len());
        let mut associated_generics_trait_constraints = Vec::new();

        for (associated_generic, bounds) in associated_generics {
            for bound in bounds {
                let typ = Type::TypeVariable(associated_generic.type_var.clone());
                let location = associated_generic.location;
                self.add_trait_bound_to_scope(location, &typ, &bound);
                associated_generics_trait_constraints
                    .push(TraitConstraint { typ, trait_bound: bound });
            }

            generics.push(associated_generic.type_var);
        }

        // We put associated generics first, as they are implicit and implicit generics
        // come before explicit generics (see `Type::instantiate_with`).
        generics.extend(func_generics);

        (generics, associated_generics_trait_constraints)
    }

    fn is_function_in_contract(&self) -> bool {
        if self.self_type.is_some() {
            // Without this, impl methods can accidentally be placed in contracts.
            // See: https://github.com/noir-lang/noir/issues/3254
            false
        } else {
            self.in_contract()
        }
    }

    /// True if the `pub` keyword is allowed on parameters in this function
    /// `pub` on function parameters is only allowed for entry point functions
    fn pub_allowed(&self, func: &NoirFunction, in_contract: bool, is_crate_root: bool) -> bool {
        func.is_entry_point(in_contract, is_crate_root) || func.attributes().is_foldable()
    }

    /// Resolves function parameters and validates their types for entry points.
    ///
    /// Returns (parameters, parameter_types, parameter_idents) where generics and
    /// trait_constraints may be extended due to `impl Trait` desugaring.
    #[tracing::instrument(level = "trace", skip_all)]
    fn resolve_function_parameters(
        &mut self,
        func: &NoirFunction,
        generics: &mut Vec<TypeVariable>,
        trait_constraints: &mut Vec<TraitConstraint>,
    ) -> ResolvedParametersInfo {
        let is_crate_root = self.is_at_crate_root();
        let is_entry_point = func.is_entry_point(self.is_function_in_contract(), is_crate_root);
        let is_test_or_fuzz = func.is_test_or_fuzz();

        let has_inline_attribute = func.has_inline_attribute();
        let is_pub_allowed = self.pub_allowed(func, self.is_function_in_contract(), is_crate_root);

        let mut parameters = Vec::new();
        let mut parameter_types = Vec::new();
        let mut parameter_idents = Vec::new();
        let mut parameter_names_in_list = rustc_hash::FxHashMap::default();
        let wildcard_allowed = WildcardAllowed::No(WildcardDisallowedContext::FunctionParameter);

        // Seed the parameter names with those from the constant generic parameter list, so that any function parameter
        // that has the same name is reported as a duplicate. This is because their precedence is not obvious.
        for generic in &func.def.generics {
            let UnresolvedGeneric::Numeric { ident: IdentOrQuotedType::Ident(ident), .. } = generic
            else {
                continue;
            };
            parameter_names_in_list.insert(ident.as_string().clone(), ident.location());
        }

        for Param { visibility, visibility_location, pattern, typ, location: _ } in
            func.parameters().iter().cloned()
        {
            self.run_lint(|_| {
                lints::unnecessary_pub_argument(
                    func,
                    visibility,
                    visibility_location,
                    is_pub_allowed,
                )
                .map(Into::into)
            });
            self.run_lint(|_| {
                lints::databus_on_non_entry_point(
                    func,
                    visibility,
                    visibility_location,
                    is_entry_point,
                )
                .map(Into::into)
            });
            let type_location = typ.location;
            let typ = match typ.typ {
                UnresolvedTypeData::TraitAsType(path, args) => {
                    self.use_unstable_feature(UnstableFeature::TraitAsType, path.location);
                    self.desugar_impl_trait_arg(path, args, generics, trait_constraints)
                }
                // Function parameters have Kind::Normal
                _ => self.resolve_type_with_kind(typ, &Kind::Normal, wildcard_allowed),
            };

            let output = false;
            if let Err(err) = Self::check_if_type_is_valid_for_program(
                &typ,
                is_entry_point || is_test_or_fuzz,
                has_inline_attribute,
                output,
                type_location,
            ) {
                self.push_err(err);
            }

            if is_entry_point || is_test_or_fuzz {
                self.mark_type_as_used(&typ);
            }

            let pattern = self.elaborate_pattern_and_store_ids(
                pattern,
                typ.clone(),
                DefinitionKind::Local(None),
                &mut parameter_idents,
                true, // warn_if_unused
                true, // warn_if_not_mutated
                &mut parameter_names_in_list,
            );

            parameters.push((pattern, typ.clone(), visibility));
            parameter_types.push(typ);
        }

        (parameters, parameter_types, parameter_idents)
    }

    /// Only sized types are valid to be used as main's parameters or the parameters to a contract
    /// function. If the given type is not sized (e.g. contains a vector or NamedGeneric type), an
    /// error is issued.
    fn check_if_type_is_valid_for_program(
        typ: &Type,
        is_entry_point: bool,
        has_inline_attribute: bool,
        output: bool,
        location: Location,
    ) -> Result<(), TypeCheckError> {
        if is_entry_point && let Some(invalid_type) = typ.program_validity(output) {
            return Err(TypeCheckError::InvalidTypeForEntryPoint { invalid_type, location });
        }

        if has_inline_attribute
            && !output
            && let Some(invalid_type) = typ.non_inlined_function_input_validity()
        {
            return Err(TypeCheckError::InvalidTypeForEntryPoint { invalid_type, location });
        }

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip_all)]
    fn run_function_lints(&mut self, func: &FuncMeta, modifiers: &FunctionModifiers) {
        self.run_lint(|_| lints::inlining_attributes(func, modifiers).map(Into::into));
        self.run_lint(|_| lints::no_predicates_on_entry_point(func, modifiers).map(Into::into));
        self.run_lint(|_| lints::missing_pub(func, modifiers).map(Into::into));
        self.run_lint(|_| {
            let pub_allowed = func.is_entry_point || modifiers.attributes.is_foldable();
            lints::unnecessary_pub_return(func, modifiers, pub_allowed).map(Into::into)
        });
        self.run_lint(|_| lints::oracle_not_marked_unconstrained(func, modifiers).map(Into::into));
        self.run_lint(|_| lints::oracle_returns_multiple_vectors(func, modifiers).map(Into::into));
        self.run_lint(|_| lints::oracle_returns_reference(func, modifiers).map(Into::into));
        self.run_lint(|_| {
            lints::oracle_returns_vector_with_nested_array(func, modifiers).map(Into::into)
        });
        self.run_lint(|elaborator| {
            lints::low_level_function_outside_stdlib(modifiers, elaborator.crate_id).map(Into::into)
        });
        self.run_lint(|elaborator| {
            lints::oracle_name_clashes_with_stdlib(modifiers, elaborator.crate_id).map(Into::into)
        });
        self.run_lint(|_| lints::check_varargs(func, modifiers).map(Into::into));
    }

    /// Elaborates a function's body and performs type checking.
    ///
    /// This is the second pass of function elaboration that processes the function body,
    /// resolves all expressions and statements, performs type checking, and verifies
    /// trait constraints. The function metadata must already be defined by [Self::define_function_meta].
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn elaborate_function(&mut self, id: FuncId) {
        let func_meta = self.interner.func_meta.get_mut(&id);
        let func_meta =
            func_meta.expect("FuncMetas should be declared before a function is elaborated");

        let (kind, body, body_location) = match func_meta.take_body() {
            FunctionBody::Unresolved(kind, body, location) => (kind, body, location),
            FunctionBody::Resolved => return,
            // Do not error for the still-resolving case. If there is a dependency cycle,
            // the dependency cycle check will find it later on.
            FunctionBody::Resolving => return,
        };

        let func_meta = func_meta.clone();

        assert_eq!(
            self.crate_id, func_meta.source_crate,
            "Functions in other crates should be already elaborated"
        );

        self.local_module = Some(func_meta.source_module);
        self.self_type = func_meta.self_type.clone();
        self.current_trait_impl = func_meta.trait_impl;
        self.current_trait = func_meta.trait_id;
        self.reset_lvalue_index_counter();

        self.scopes.start_function();
        let old_item = self.current_item.replace(DependencyId::Function(id));

        self.trait_bounds = func_meta.all_trait_constraints().cloned().collect();
        self.push_function_context();

        // Lints and visibility must be separately from function meta resolution as comptime attribute
        // may possibly update a function's modifiers.
        let modifiers = self.interner.function_modifiers(&id).clone();
        self.run_function_lints(&func_meta, &modifiers);
        let name = Ident::new(
            self.interner.definition_name(func_meta.name.id).to_string(),
            func_meta.name.location,
        );
        self.check_function_visibility(&func_meta, &modifiers, &name, func_meta.location);

        self.introduce_generics_into_scope(func_meta.all_generics.clone());

        // The DefinitionIds for each parameter were already created in define_function_meta
        // so we need to reintroduce the same IDs into scope here.
        for parameter in &func_meta.parameter_idents {
            let name = self.interner.definition_name(parameter.id).to_owned();
            let warn_if_unused = !(func_meta.trait_impl.is_some() && name == "self");
            let warn_if_not_mutated = false;
            // We allow shadowing here because there's no outer scope to shadow
            // (duplicate parameter names were already checked in `resolve_function_parameters`)
            let allow_shadowing = true;
            self.add_existing_variable_to_scope(
                name,
                parameter.clone(),
                warn_if_unused,
                warn_if_not_mutated,
                allow_shadowing,
            );
        }

        self.add_trait_constraints_to_scope(func_meta.all_trait_constraints(), func_meta.location);

        let (hir_func, body_type) = match kind {
            FunctionKind::Builtin
            | FunctionKind::LowLevel
            | FunctionKind::TraitFunctionWithoutBody => {
                if !body.statements.is_empty() {
                    self.push_err(ResolverError::BuiltinWithBody {
                        location: func_meta.name.location,
                    });
                }
                (HirFunction::empty(), Type::Error)
            }
            FunctionKind::Oracle => {
                if !body.statements.is_empty() {
                    self.push_err(ResolverError::OracleWithBody {
                        location: func_meta.name.location,
                    });
                }
                (HirFunction::empty(), Type::Error)
            }
            FunctionKind::Normal => {
                let return_type = func_meta.return_type();
                let (block, body_type) = self.elaborate_block(body, Some(return_type));
                let expr_id = self.interner.push_expr_full(block, body_location, body_type.clone());
                (HirFunction::unchecked_from_expr(expr_id), body_type)
            }
        };

        // Don't verify the return type for builtin functions & trait function declarations
        if !func_meta.is_stub() {
            self.type_check_function_body(body_type, &func_meta, hir_func.as_expr());
        }

        // Default any type variables that still need defaulting and
        // verify any remaining trait constraints arising from the function body.
        // This is done before trait impl search since leaving them bindable can lead to errors
        // when multiple impls are available. Instead we default first to choose the Field or u64 impl.
        self.check_and_pop_function_context();

        self.remove_trait_constraints_from_scope(func_meta.all_trait_constraints());

        let func_scope_tree = self.scopes.end_function();

        // The arguments to low-level and oracle functions are always unused so we do not produce warnings for them.
        if !func_meta.is_stub() {
            self.check_for_unused_variables_in_scope_tree(&func_scope_tree);
            self.check_for_unnecessary_mut_variables_in_scope_tree(&func_scope_tree);
        }

        // Check that the body can return without calling the function.
        if let FunctionKind::Normal = kind {
            self.run_lint(|elaborator| {
                lints::unbounded_recursion(
                    elaborator.interner,
                    id,
                    || elaborator.interner.definition_name(func_meta.name.id),
                    func_meta.name.location,
                    hir_func.as_expr(),
                )
                .map(Into::into)
            });
        }

        let meta = self
            .interner
            .func_meta
            .get_mut(&id)
            .expect("FuncMetas should be declared before a function is elaborated");

        meta.function_body = FunctionBody::Resolved;

        self.trait_bounds.clear();
        self.interner.update_fn(id, hir_func);
        self.current_item = old_item;
    }
}
