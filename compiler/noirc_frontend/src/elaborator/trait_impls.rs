//! Trait implementation collection, method matching, and coherence checking.

use std::rc::Rc;

use crate::{
    Kind, NamedGeneric, ResolvedGeneric, Shared, TypeBindings, TypeVariable,
    ast::{GenericTypeArgs, Ident, UnresolvedType, UnresolvedTypeData, UnresolvedTypeExpression},
    elaborator::{PathResolutionMode, WildcardDisallowedContext, types::WildcardAllowed},
    hir::{
        def_collector::{
            dc_crate::{CompilationError, UnresolvedTraitImpl},
            errors::DefCollectorErrorKind,
        },
        resolution::errors::ResolverError,
        type_check::TypeCheckError,
    },
    hir_def::traits::{NamedType, ResolvedTraitBound, TraitImpl},
    node_interner::{TraitImplId, TraitLookupMode},
};
use crate::{
    Type,
    hir::def_collector::errors::DuplicateType,
    hir_def::traits::{TraitConstraint, TraitFunction},
    node_interner::{FuncId, TraitId, get_type_method_key},
};

use iter_extended::vecmap;
use noirc_errors::{Located, Location};
use rustc_hash::FxHashSet as HashSet;

use super::deferred::{PendingWhereClauseCheck, UnresolvedImplAssociatedType};
use super::{
    Elaborator,
    item_context::{ImplContext, ItemContext, ModuleContext},
};

impl Elaborator<'_> {
    /// Collects and validates a trait implementation.
    ///
    /// This is the main entry point for processing a trait impl block like:
    /// ```noir
    /// impl<A> MyTrait<A> for MyStruct where A: OtherTrait {
    ///     fn method(&self) { ... }
    /// }
    /// ```
    ///
    /// # Prerequisites
    ///
    /// Before calling this function, [`Self::prepare_trait_impl_for_function_meta_definition`]
    /// must have been called to:
    /// - Resolve the trait path and set `trait_impl.trait_id`
    /// - Set up generics and assign `trait_impl.impl_id`
    /// - Register associated types
    /// - Resolve the self type (`trait_impl.methods.self_type`)
    ///
    /// The function validates and verifies self, generics and associated types,
    /// processes impl methods via [`Self::collect_trait_impl_methods`],
    /// register the trait impl in the interner.
    ///
    /// # Errors
    ///
    /// This function may emit the following errors:
    /// - `ReferenceInTraitImpl`: Self type cannot be a reference
    /// - `TraitMissingMethod`: Required trait method not implemented
    /// - `MethodNotInTrait`: Impl contains method not in trait
    /// - `ImplIsStricterThanTrait`: Method where clause is more restrictive than trait
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn collect_trait_impl(&mut self, trait_impl: &mut UnresolvedTraitImpl) {
        let self_type = trait_impl.methods.self_type.clone();
        let self_type =
            self_type.expect("Expected struct type to be set before collect_trait_impl");

        // The impl is collected in a context of its own: its module, the trait and impl ids and
        // the implementing type as `Self`. The caller's context is reinstated afterwards.
        let context = ItemContext::new(ModuleContext::in_module(trait_impl.module_id)).with_impl(
            ImplContext::in_trait_impl(Some(self_type), trait_impl.trait_id, trait_impl.impl_id),
        );
        self.with_item_context(context, |this| {
            this.collect_trait_impl_in_context(trait_impl);
        });
    }

    /// Does the work of [`Self::collect_trait_impl`].
    ///
    /// Expects the trait impl's own [`ItemContext`] to be installed.
    fn collect_trait_impl_in_context(&mut self, trait_impl: &mut UnresolvedTraitImpl) {
        let self_type_location = trait_impl.object_type.location;

        let self_type = self.item.impl_context.expect_self_type();
        let error: Option<CompilationError> =
            if matches!(self_type.follow_bindings_shallow().as_ref(), Type::Reference(..)) {
                let is_alias = matches!(self_type, Type::Alias(..));
                Some(
                    DefCollectorErrorKind::ReferenceInTraitImpl {
                        is_alias,
                        location: self_type_location,
                    }
                    .into(),
                )
            } else if get_type_method_key(self_type).is_none() {
                Some(if *self_type == Type::Error {
                    TypeCheckError::expecting_other_error(
                        "collect_trait_impl: missing trait type",
                        self_type_location,
                    )
                    .into()
                } else {
                    ResolverError::TypeUnsupportedForTraitImpl {
                        typ: self_type.clone(),
                        location: self_type_location,
                    }
                    .into()
                })
            } else {
                None
            };
        if let Some(error) = error {
            self.push_err(error);
        }

        self.check_generics_appear_in_types(
            &trait_impl.generics,
            &[&trait_impl.r#trait, &trait_impl.object_type],
            &trait_impl.where_clause,
        );

        if let Some(trait_id) = trait_impl.trait_id {
            self.item.generics.set_params(trait_impl.resolved_generics.clone());

            let impl_id =
                trait_impl.impl_id.expect("ICE: impl_id should be set in define_function_metas");
            let where_clause = self.interner.get_where_clause_for_impl(impl_id).to_vec();
            for constraint in &where_clause {
                self.add_resolved_trait_constraint_to_scope(constraint);
            }

            self.resolve_trait_impl_associated_types_if_unresolved(impl_id);

            // If there are bounds on the trait's associated types, check them now, for this impl's
            // `Self` and trait arguments.
            let impl_constraint = TraitConstraint {
                typ: self.item.impl_context.expect_self_type().clone(),
                trait_bound: ResolvedTraitBound {
                    trait_id,
                    trait_generics: self.interner.get_trait_generics_for_impl(impl_id).clone(),
                    location: trait_impl.object_type.location,
                },
            };
            for (constraint, location) in self.associated_type_constraints(&[impl_constraint]) {
                let (object_type, bound) = (&constraint.typ, &constraint.trait_bound);
                if let Err(error) = self.interner.lookup_trait_implementation(
                    object_type,
                    bound.trait_id,
                    &bound.trait_generics.ordered,
                    &bound.trait_generics.named,
                ) {
                    self.push_trait_constraint_error(object_type, error, location);
                }
            }

            self.remove_trait_constraints_from_scope(where_clause.iter());

            self.collect_trait_impl_methods(trait_id, trait_impl, &where_clause);

            let location = trait_impl.object_type.location;
            self.declare_methods_on_data_type(Some(trait_id), &trait_impl.methods, location);

            let trait_visibility = self.interner.get_trait(trait_id).visibility;

            let methods = trait_impl.methods.function_ids();
            for func_id in &methods {
                // Slots inherited from the trait's default method point at the trait's
                // own FuncId, which is shared by every inheriting impl. Skip per-impl
                // bookkeeping for those — `func_id_to_trait` is single-valued and
                // visibility/modifiers were set when the trait method was collected.
                if trait_impl.inherited_default_method_func_ids.contains(func_id) {
                    continue;
                }

                let self_type = self.item.impl_context.expect_self_type().clone();
                if self.interner.set_function_trait(*func_id, self_type, trait_id).is_some() {
                    self.push_err(TypeCheckError::expecting_other_error(
                        "collect_trait_impl: overlapping function trait",
                        location,
                    ));
                }

                // A trait impl method has the same visibility as its trait
                let modifiers = self.interner.function_modifiers_mut(func_id);
                modifiers.visibility = trait_visibility;
            }

            let ident = match &trait_impl.r#trait.typ {
                UnresolvedTypeData::Named(trait_path, _, _) => trait_path.last_ident(),
                UnresolvedTypeData::Resolved(quoted_type_id) => {
                    let typ = self.interner.get_quoted_type(*quoted_type_id);
                    let name = if let Type::TraitAsType(_, name, _) = typ {
                        name.to_string()
                    } else {
                        typ.to_string()
                    };
                    Ident::new(name, trait_impl.r#trait.location)
                }
                _ => {
                    self.push_err(TypeCheckError::expecting_other_error(
                        "collect_trait_impl: missing trait type",
                        location,
                    ));
                    Ident::new(trait_impl.r#trait.to_string(), trait_impl.r#trait.location)
                }
            };

            let resolved_trait_impl = Shared::new(TraitImpl {
                ident,
                location,
                typ: self.item.impl_context.expect_self_type().clone(),
                generics: self.item.generics.type_vars(),
                trait_id,
                file: trait_impl.file_id,
                crate_id: self.crate_id,
                methods,
            });

            if let Err(error) = self.interner.add_trait_implementation(
                self.item.impl_context.expect_self_type().clone(),
                trait_id,
                impl_id,
                resolved_trait_impl,
                location,
            ) {
                self.push_err(error);
            }
        }
    }

    /// Reports `OverlappingImpl` for each of `impl_ids` that overlaps an impl of its trait added
    /// before it. This runs once all the impls elaborated together are collected, so that the
    /// verdict doesn't depend on the order they are collected in.
    pub(super) fn check_overlapping_trait_impls(
        &mut self,
        impl_ids: impl IntoIterator<Item = TraitImplId>,
    ) {
        for impl_id in impl_ids {
            let Some(existing) = self.interner.overlapping_earlier_impl(impl_id) else {
                continue;
            };
            let prev_location =
                self.interner.get_trait_implementation(existing).borrow().ident.location();
            let trait_impl = self.interner.get_trait_implementation(impl_id);
            let trait_impl = trait_impl.borrow();
            self.push_err(DefCollectorErrorKind::OverlappingImpl {
                typ: trait_impl.typ.clone(),
                location: trait_impl.location,
                prev_location,
            });
        }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn collect_trait_impl_methods(
        &mut self,
        trait_id: TraitId,
        trait_impl: &mut UnresolvedTraitImpl,
        trait_impl_where_clause: &[TraitConstraint],
    ) {
        self.in_local_module(trait_impl.module_id, |this| {
            let impl_id =
                trait_impl.impl_id.expect("impl_id should be set in define_function_metas");

            // In this Vec methods[i] corresponds to trait.methods[i]. If the impl has no implementation
            // for a particular method, the default implementation will be added at that slot.
            let mut ordered_methods = Vec::new();

            // Check whether the trait implementation is in the same crate as either the trait or the type
            this.check_trait_impl_crate_coherence(trait_id, trait_impl);

            // Set of function ids that have a corresponding method in the trait
            let mut func_ids_in_trait = HashSet::default();

            // Temporarily take ownership of the trait's methods so we can iterate over them
            // while also mutating the interner
            let the_trait = this.interner.get_trait_mut(trait_id);
            let methods = std::mem::take(&mut the_trait.methods);
            for method in &methods {
                let overrides: Vec<_> = trait_impl
                    .methods
                    .functions
                    .iter()
                    .filter(|(_, _, f)| f.name() == method.name.as_str())
                    .collect();

                if overrides.is_empty() {
                    if let Some(default_impl) = &method.default_impl {
                        // Reuse the trait's own FuncId for the default method instead of cloning
                        // the body into a fresh FuncId per impl. The body has already been
                        // elaborated once at the trait definition (`elaborate_traits` walks
                        // `UnresolvedTrait::fns_with_default_impl`). Sharing the FuncId means
                        // the body is type-checked exactly once, errors are reported once, and
                        // paths in the body resolve from the trait's module rather than each
                        // impl's. Per-call `Self` substitution still happens at the call site
                        // via the instantiation bindings recorded during trait method
                        // resolution, so dispatch needs no extra work.
                        let trait_func_id =
                            this.interner.get_trait(trait_id).method_ids[method.name.as_str()];
                        trait_impl.inherited_default_method_func_ids.insert(trait_func_id);
                        func_ids_in_trait.insert(trait_func_id);
                        ordered_methods.push((
                            method.default_impl_module_id,
                            trait_func_id,
                            *default_impl.clone(),
                        ));
                    } else {
                        this.push_err(DefCollectorErrorKind::TraitMissingMethod {
                            trait_name: this.interner.get_trait(trait_id).name.clone(),
                            method_name: method.name.clone(),
                            trait_impl_location: trait_impl.object_type.location,
                        });
                    }
                } else {
                    let ordered_generics =
                        this.interner.get_ordered_generics_for_impl(impl_id).to_vec();
                    for (_, func_id, _) in &overrides {
                        // Defer the where-clause check until after the post-attribute
                        // drain so that the impl method's meta and the trait method's
                        // `TraitFunction` record are both fully resolved.
                        this.queue_pending_where_clause_check(
                            *func_id,
                            method,
                            trait_impl_where_clause,
                            &ordered_generics,
                            trait_id,
                            impl_id,
                        );

                        func_ids_in_trait.insert(*func_id);
                    }

                    if overrides.len() > 1 {
                        this.push_err(DefCollectorErrorKind::Duplicate {
                            typ: DuplicateType::TraitAssociatedItem,
                            first_def: overrides[0].2.name_ident().clone(),
                            second_def: overrides[1].2.name_ident().clone(),
                        });
                    }

                    ordered_methods.push(overrides[0].clone());
                }
            }

            // Restore the methods that were taken before the for loop
            let the_trait = this.interner.get_trait_mut(trait_id);
            the_trait.set_methods(methods);

            let trait_name = the_trait.name.clone();

            // Emit MethodNotInTrait error for methods in the impl block that
            // don't have a corresponding method signature defined in the trait
            for (_, func_id, func) in &trait_impl.methods.functions {
                if !func_ids_in_trait.contains(func_id) {
                    let trait_name = trait_name.clone();
                    let impl_method = func.name_ident().clone();
                    let error = DefCollectorErrorKind::MethodNotInTrait { trait_name, impl_method };
                    let error: CompilationError = error.into();
                    this.push_err(error);
                }
            }

            trait_impl.methods.functions = ordered_methods;
            trait_impl.methods.trait_id = Some(trait_id);
        });
    }

    /// Issue an error if the impl is stricter than the trait.
    ///
    /// # Example
    ///
    /// ```compile_fail
    /// trait MyTrait { }
    /// trait Foo<T> {
    ///     fn foo<U>();
    /// }
    /// impl<A> Foo<A> for () {
    ///     // Error issued here as `foo` does not have the `MyTrait` constraint
    ///     fn foo<B>() where B: MyTrait {}
    /// }
    /// ```
    /// Records a pending where-clause-against-trait check to run after the
    /// post-attribute drain. Until then, neither the impl method's meta nor
    /// the trait method's [`TraitFunction`] record is guaranteed to be
    /// populated, so we capture only the data needed to look them up later.
    fn queue_pending_where_clause_check(
        &mut self,
        impl_method_func_id: FuncId,
        trait_method: &TraitFunction,
        trait_impl_where_clause: &[TraitConstraint],
        ordered_generics: &[Type],
        trait_id: TraitId,
        impl_id: TraitImplId,
    ) {
        let module_id = self.item.module.local_module();
        self.session.deferred.trait_work.where_clause_checks.push(PendingWhereClauseCheck {
            impl_method_func_id,
            trait_id,
            impl_id,
            module_id,
            trait_method_name: trait_method.name.as_str().to_string(),
            trait_impl_where_clause: trait_impl_where_clause.to_vec(),
            ordered_generics: ordered_generics.to_vec(),
        });
    }

    /// Runs every pending where-clause check queued by
    /// [`Self::queue_pending_where_clause_check`]. Must be called after the
    /// post-attribute drain has resolved every involved meta and after
    /// [`Elaborator::populate_resolved_trait_method_records`] has updated the
    /// trait's `TraitFunction` records.
    pub(super) fn run_pending_where_clause_checks(&mut self) {
        let pending = std::mem::take(&mut self.session.deferred.trait_work.where_clause_checks);
        for check in pending {
            // Re-fetch the trait method's TraitFunction record (now populated)
            // by name, mirroring how `collect_trait_impl_methods` matched it.
            let the_trait = self.interner.get_trait(check.trait_id);
            let Some(trait_method) =
                the_trait.methods.iter().find(|m| m.name.as_str() == check.trait_method_name)
            else {
                continue;
            };
            let trait_method = trait_method.clone();

            let impl_self_type =
                self.interner.get_trait_implementation(check.impl_id).borrow().typ.clone();
            // Each check runs in a context of its own for the impl, as trait impl collection did.
            let context = ItemContext::new(ModuleContext::in_module(check.module_id)).with_impl(
                ImplContext::in_trait_impl(
                    Some(impl_self_type),
                    Some(check.trait_id),
                    Some(check.impl_id),
                ),
            );
            self.with_item_context(context, |this| {
                this.check_where_clause_against_trait(
                    &check.impl_method_func_id,
                    &trait_method,
                    &check.trait_impl_where_clause,
                    &check.ordered_generics,
                    check.trait_id,
                    check.impl_id,
                );
            });
        }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    fn check_where_clause_against_trait(
        &mut self,
        func_id: &FuncId,
        method: &TraitFunction,
        trait_impl_where_clause: &[TraitConstraint],
        trait_impl_generics: &[Type],
        trait_id: TraitId,
        impl_id: TraitImplId,
    ) {
        // First get the general trait to impl bindings.
        // Then we'll need to add the bindings for this specific method.
        let self_type = self.item.impl_context.expect_self_type();

        let mut bindings =
            self.interner.trait_to_impl_bindings(trait_id, impl_id, trait_impl_generics, self_type);

        // The override method's meta may not have been resolved yet — function metas are
        // resolved lazily so they can see fully-bound trait impls and globals. Resolve it
        // now before we read it.
        self.define_function_meta_if_undefined(*func_id);
        let override_meta = self.interner.remove_function_meta(func_id);

        // Substitute each generic on the trait function with the corresponding generic on the impl function
        for (ResolvedGeneric { type_var: trait_fn_generic, .. }, impl_fn_resolved_generic) in
            // Use zip (not zip_eq) since the impl may have a different number of
            // generics than the trait method (which is a user error caught elsewhere).
            method.direct_generics.iter().zip(&override_meta.direct_generics)
        {
            let trait_fn_kind = trait_fn_generic.kind();
            let arg = impl_fn_resolved_generic.clone().into_named_generic(None);

            if self.check_kind(&trait_fn_kind, &arg.kind(), impl_fn_resolved_generic.location) {
                bindings.insert(
                    trait_fn_generic.id(),
                    (trait_fn_generic.clone(), trait_fn_kind.into_owned(), arg),
                );
            }
        }

        self.resolve_trait_method_metas_for(trait_id);
        if let Some(method_id) =
            self.interner.get_trait(trait_id).method_ids.get(method.name.as_str())
            && let Some(method_meta) = self.interner.try_function_meta(method_id)
        {
            bindings
                .extend(super::traits::impl_trait_parameter_bindings(method_meta, &override_meta));
        }
        // An associated type the trait method's bounds leave to a placeholder
        // (`where Self: C, <Self as C>::N: Eq`) is the impl's once `Self` is the impl's type, so
        // the bounds on that placeholder are bounds on the impl's associated type.
        self.bind_placeholders_to_normalized_associated_types(
            method.trait_constraints.iter(),
            &mut bindings,
        );

        // The impl method may only require what the trait method's bounds, for this impl, and
        // the impl's where clause imply: each of its bounds must hold under those assumptions.
        let assumptions = vecmap(&method.trait_constraints, |constraint| {
            let mut constraint = constraint.clone();
            constraint.apply_bindings(&bindings);
            constraint
        });
        self.assume_trait_constraints(assumptions.iter().chain(trait_impl_where_clause));

        let mut solved = TypeBindings::default();
        for override_trait_constraint in &override_meta.trait_constraints {
            if !self.override_bound_holds(override_trait_constraint, &mut solved) {
                let the_trait =
                    self.interner.get_trait(override_trait_constraint.trait_bound.trait_id);
                self.push_err(DefCollectorErrorKind::ImplIsStricterThanTrait {
                    constraint_typ: override_trait_constraint.typ.clone(),
                    constraint_name: the_trait.name.to_string(),
                    constraint_generics: override_trait_constraint
                        .trait_bound
                        .trait_generics
                        .clone(),
                    constraint_location: override_trait_constraint.trait_bound.location,
                    trait_method_name: method.name.to_string(),
                    trait_method_location: method.location,
                });
            }
        }

        self.remove_trait_constraints_from_scope(assumptions.iter().chain(trait_impl_where_clause));
        self.interner.push_fn_meta(override_meta, *func_id);
    }

    /// Whether an impl method's bound holds under the assumptions in scope. An associated type the
    /// bound leaves out is named by a placeholder of its own, which stands for whatever the
    /// assumptions say: `solved` binds the placeholders of the bounds checked so far to it.
    fn override_bound_holds(&self, bound: &TraitConstraint, solved: &mut TypeBindings) -> bool {
        // Its own placeholders not solved by an earlier bound stand for unknowns.
        let mut own = bound.trait_bound.trait_generics.named.clone();
        own.retain(|named| !matches!(&named.typ, Type::NamedGeneric(generic) if solved.contains_key(&generic.type_var.id())));
        self.bind_to_fresh_variable(&mut own, solved);
        let mut constraint = bound.clone();
        constraint.apply_bindings(solved);
        let bound = &constraint.trait_bound;
        let Ok((_, impl_bindings, _)) = self.interner.try_lookup_trait_implementation(
            &constraint.typ,
            bound.trait_id,
            &bound.trait_generics.ordered,
            &bound.trait_generics.named,
            TraitLookupMode::Default,
        ) else {
            return false;
        };
        for (_, _, typ) in solved.values_mut() {
            *typ = typ.substitute(&impl_bindings);
        }
        true
    }

    /// Check that an associated type in a trait impl is not assigned a type that is more
    /// private than the trait itself. Otherwise, a private type would leak through the
    /// trait's public surface.
    #[tracing::instrument(level = "trace", skip_all)]
    fn check_trait_impl_associated_type_visibility(
        &mut self,
        trait_id: TraitId,
        associated_type_name: &Ident,
        resolved_type: &Type,
    ) {
        let the_trait = self.interner.get_trait(trait_id);
        let trait_visibility = the_trait.visibility;
        let item = Ident::from(Located::from(
            associated_type_name.location(),
            format!("{}::{}", the_trait.name, associated_type_name),
        ));
        self.check_type_is_not_more_private_then_item(
            &item,
            trait_visibility,
            resolved_type,
            associated_type_name.location(),
        );
    }

    #[tracing::instrument(level = "trace", skip_all)]
    fn check_trait_impl_crate_coherence(
        &mut self,
        trait_id: TraitId,
        trait_impl: &UnresolvedTraitImpl,
    ) {
        self.in_local_module(trait_impl.module_id, |this| {
            let object_crate = match &trait_impl.resolved_object_type {
                Some(Type::DataType(struct_or_enum_type, _)) => {
                    Some(struct_or_enum_type.borrow().id.krate())
                }
                _ => None,
            };

            let the_trait = this.interner.get_trait(trait_id);
            if this.crate_id != the_trait.crate_id && Some(this.crate_id) != object_crate {
                this.push_err(DefCollectorErrorKind::TraitImplOrphaned {
                    location: trait_impl.object_type.location,
                });
            }
        });
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn take_unresolved_associated_types(
        &mut self,
        trait_impl: &mut UnresolvedTraitImpl,
    ) -> Vec<(Ident, UnresolvedType, Kind)> {
        let mut associated_types = Vec::new();
        for (name, typ, expr) in trait_impl.associated_constants.drain(..) {
            let wildcard_allowed = WildcardAllowed::No(WildcardDisallowedContext::AssociatedType);
            let resolved_type =
                typ.map_or(Type::Error, |typ| self.resolve_type(typ, wildcard_allowed));
            let kind = Kind::numeric(resolved_type);
            let location = expr.location;
            let typ = match UnresolvedTypeExpression::from_expr(expr, location) {
                Ok(expr) => UnresolvedTypeData::Expression(expr).with_location(location),
                Err(error) => {
                    self.push_err(error);
                    UnresolvedTypeData::Error.with_location(location)
                }
            };
            associated_types.push((name, typ, kind));
        }
        for (name, typ) in trait_impl.associated_types.drain(..) {
            let location = name.location();
            let typ = typ.unwrap_or_else(|| UnresolvedTypeData::Error.with_location(location));
            associated_types.push((name, typ, Kind::Normal));
        }
        associated_types
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn add_trait_impl_assumed_trait_implementations(
        &mut self,
        impl_id: Option<TraitImplId>,
    ) {
        if let Some(impl_id) = impl_id
            && self.interner.try_get_trait_implementation(impl_id).is_some()
        {
            for trait_constrain in self.interner.get_where_clause_for_impl(impl_id).to_vec() {
                let trait_bound = &trait_constrain.trait_bound;
                self.add_trait_bound_to_scope(
                    trait_bound.location,
                    &trait_constrain.typ,
                    trait_bound,
                );
            }
        }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn remove_trait_impl_assumed_trait_implementations(
        &mut self,
        impl_id: Option<TraitImplId>,
    ) {
        if let Some(impl_id) = impl_id
            && self.interner.try_get_trait_implementation(impl_id).is_some()
        {
            for trait_constrain in self.interner.get_where_clause_for_impl(impl_id).to_vec() {
                self.interner.remove_assumed_trait_implementations_for_trait(
                    trait_constrain.trait_bound.trait_id,
                );
            }
        }
    }

    /// Verifies that the impl satisfies every obligation the trait's where clause
    /// places on it, including the super-trait bounds that are stored in the trait's
    /// where clause as constraints on `Self`.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn check_trait_impl_where_clause_matches_trait_where_clause(
        &mut self,
        trait_impl: &UnresolvedTraitImpl,
    ) {
        let Some(trait_id) = trait_impl.trait_id else {
            self.push_err(TypeCheckError::expecting_other_error(
                "check_trait_impl_where_clause_matches_trait_where_clause: missing trait ID",
                trait_impl.object_type.location,
            ));
            return;
        };

        let Some(impl_id) = trait_impl.impl_id else {
            self.push_err(TypeCheckError::expecting_other_error(
                "check_trait_impl_where_clause_matches_trait_where_clause: missing impl ID",
                trait_impl.object_type.location,
            ));
            return;
        };

        let Some(the_trait) = self.interner.try_get_trait(trait_id) else {
            self.push_err(TypeCheckError::expecting_other_error(
                "check_trait_impl_where_clause_matches_trait_where_clause: missing trait",
                trait_impl.object_type.location,
            ));
            return;
        };

        let Some(object_type) = &trait_impl.resolved_object_type else {
            self.push_err(TypeCheckError::expecting_other_error(
                "check_trait_impl_where_clause_matches_trait_where_clause: missing object type",
                trait_impl.object_type.location,
            ));
            return;
        };

        let impl_trait = the_trait.name.to_string();
        let impl_generics = self.interner.get_trait_generics_for_impl(impl_id);

        // The where clause, parent bounds included, is written in terms of the trait's `Self`,
        // generics and associated types, so it is checked with this impl's.
        let mut bindings = the_trait.bound_bindings(object_type, impl_generics);

        self.check_trait_bounds_are_satisfied(
            the_trait.where_clause.clone(),
            &impl_trait,
            &trait_impl.object_type.location,
            &mut bindings,
        );
    }

    /// Checks that each trait constraint in the given list is satisfied.
    ///
    /// This is used both for checking:
    /// 1. The trait's where clause constraints are satisfied by the impl
    /// 2. The trait's parent trait bounds are satisfied by the impl
    #[tracing::instrument(level = "trace", skip_all)]
    fn check_trait_bounds_are_satisfied(
        &mut self,
        constraints: Vec<TraitConstraint>,
        impl_trait: &str,
        error_location: &Location,
        bindings: &mut TypeBindings,
    ) {
        for trait_constraint in constraints {
            let Some(trait_constraint_trait) =
                self.interner.try_get_trait(trait_constraint.trait_bound.trait_id)
            else {
                self.push_err(TypeCheckError::expecting_other_error(
                    "check_trait_impl_where_clause_matches_trait_where_clause: missing trait constraint trait",
                    *error_location,
                ));
                continue;
            };
            let trait_constraint_trait_name = trait_constraint_trait.name.to_string();

            let mut trait_constraint = trait_constraint.clone();
            trait_constraint.apply_bindings(bindings);

            let trait_constraint_type = trait_constraint.typ;
            let trait_bound = trait_constraint.trait_bound;

            let mut named_generics = trait_bound.trait_generics.named.clone();

            // If the trait bound is over a trait that has associated types, the ones that
            // aren't explicit will be in `named_generics` as implicitly added ones.
            // If they are unbound, they won't be bound until monomorphization, in which case
            // the below trait implementation lookup will fail (an unbound named generic will
            // never unify in this case). In this case we replace them with fresh type variables
            // so they'll unify (the bindings aren't applied here so this is fine).
            // If they are bound though, we won't replace them as we want to ensure the binding
            // matches.
            //
            // `bindings` is passed here because these implicitly added named generics might
            // have a constraint on them later on and we want to remember what type they ended
            // up being.
            self.bind_to_fresh_variable(&mut named_generics, bindings);

            match self.interner.try_lookup_trait_implementation(
                &trait_constraint_type,
                trait_bound.trait_id,
                &trait_bound.trait_generics.ordered,
                &named_generics,
                TraitLookupMode::Default,
            ) {
                Ok((_, impl_bindings, impl_instantiation_bindings)) => {
                    // The search solves the fresh variables given to the elided associated
                    // types above, which later bounds read through `bindings`.
                    let mut solved = impl_bindings;
                    solved.extend(impl_instantiation_bindings);
                    for (_, _, typ) in bindings.values_mut() {
                        *typ = typ.substitute(&solved);
                    }
                    bindings.extend(solved);
                }
                Err(_) => {
                    let missing_trait =
                        format!("{}{}", trait_constraint_trait_name, trait_bound.trait_generics);
                    self.push_err(ResolverError::TraitNotImplemented {
                        impl_trait: impl_trait.to_string(),
                        missing_trait,
                        type_missing_trait: trait_constraint_type.to_string(),
                        location: *error_location,
                        missing_trait_location: trait_bound.location,
                    });
                }
            }
        }
    }

    // Replace implicitly added unbound named generics with fresh type variables
    fn bind_to_fresh_variable(
        &self,
        named_generics: &mut [NamedType],
        bindings: &mut TypeBindings,
    ) {
        for named_type in named_generics.iter_mut() {
            match &named_type.typ {
                Type::NamedGeneric(NamedGeneric { type_var, implicit: true, .. })
                    if type_var.binding().is_unbound() =>
                {
                    let type_var_id = type_var.id();
                    let new_type_var_id = self.interner.next_type_variable_id();
                    let kind = type_var.kind().into_owned();
                    let new_type_var = TypeVariable::unbound(new_type_var_id, kind.clone());
                    named_type.typ = Type::TypeVariable(new_type_var.clone());
                    bindings.insert(type_var_id, (new_type_var, kind, named_type.typ.clone()));
                }
                _ => (),
            }
        }
    }

    /// Resolves the associated types and constants of a trait impl that aren't resolved yet or being
    /// resolved, binding the placeholder each one has in the impl's trait generics. This runs when
    /// the impl is collected.
    pub(super) fn resolve_trait_impl_associated_types_if_unresolved(
        &mut self,
        impl_id: TraitImplId,
    ) {
        let associated_types = self.interner.get_associated_types_for_impl(impl_id).to_vec();
        for associated_type in associated_types {
            self.resolve_placeholder_if_unresolved(impl_id, &associated_type.typ);
        }
    }

    /// Resolves the associated type or constant `name` of a trait impl if it isn't resolved yet or
    /// being resolved, so that it has its value before it is read. It can be read before the impl
    /// is collected: by another impl (`let N: u32 = <[u8; 3] as C>::N;`), or through a global that
    /// another item of the same impl reads. Only `name` is resolved, as the impl's other items
    /// may themselves depend on whatever is reading it.
    pub(crate) fn resolve_trait_impl_associated_type_if_unresolved(
        &mut self,
        impl_id: TraitImplId,
        name: &str,
    ) {
        if let Some(placeholder) = self.interner.find_associated_type_for_impl(impl_id, name) {
            let placeholder = placeholder.clone();
            self.resolve_placeholder_if_unresolved(impl_id, &placeholder);
        }
    }

    /// Resolves the associated item that `placeholder`, from the impl's trait generics, stands for.
    fn resolve_placeholder_if_unresolved(&mut self, impl_id: TraitImplId, placeholder: &Type) {
        let Type::NamedGeneric(named_generic) = placeholder else {
            return;
        };
        let key = named_generic.type_var.id();
        if let Some(item) = self.session.deferred.trait_impl_associated_types.take(&key) {
            self.resolve_trait_impl_associated_type(impl_id, item);
        }
    }

    fn resolve_trait_impl_associated_type(
        &mut self,
        impl_id: TraitImplId,
        item: UnresolvedImplAssociatedType,
    ) {
        let UnresolvedImplAssociatedType { module_id, self_type, trait_id, generics, name, typ } =
            item;
        let Some(Type::NamedGeneric(named_generic)) =
            self.interner.find_associated_type_for_impl(impl_id, name.as_str()).cloned()
        else {
            // This can happen if the associated type is specified directly in the impl trait generics,
            // This can't be done in code, but it could happen with unquoted types.
            return;
        };

        let impl_context =
            ImplContext::in_trait_impl(Some(self_type), Some(trait_id), Some(impl_id));
        let context = ItemContext::new(ModuleContext::in_module(module_id)).with_impl(impl_context);
        self.with_item_context(context, |this| {
            this.item.generics.set_params(generics);
            for constraint in this.interner.get_where_clause_for_impl(impl_id).to_vec() {
                this.add_resolved_trait_constraint_to_scope(&constraint);
            }

            let wildcard_allowed = WildcardAllowed::No(WildcardDisallowedContext::AssociatedType);
            let location = typ.location;
            let kind = named_generic.type_var.kind();
            let resolved_type = this.resolve_type_with_kind(typ, &kind, wildcard_allowed);
            this.check_trait_impl_associated_type_visibility(trait_id, &name, &resolved_type);
            if let Err(error) = named_generic.type_var.try_bind(resolved_type, &kind, location) {
                this.push_err(error);
            }
        });
    }

    /// Prepares a trait impl for function metadata definition.
    ///
    /// This method handles the setup required for trait impls:
    /// - Resolves the trait path and validates it exists
    /// - Sets up generics including where clause desugaring
    /// - Assigns the impl ID
    /// - Resolves and registers associated types
    /// - Resolves the self type for the impl
    /// - Manages trait constraint scoping
    ///
    /// Returns the new generics trait constraints that were created from desugaring
    /// the where clause. These need to be passed to function meta definition.
    ///
    /// After this preparation, the trait impl is ready for function meta definition.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(super) fn prepare_trait_impl_for_function_meta_definition(
        &mut self,
        trait_impl: &mut UnresolvedTraitImpl,
    ) -> (Vec<(TraitConstraint, Location)>, Vec<ResolvedGeneric>) {
        // No current item is installed so that resolving the self type registers no dependencies.
        let context = ItemContext::new(ModuleContext::in_module(trait_impl.module_id));
        self.with_item_context(context, |this| this.prepare_trait_impl_in_context(trait_impl))
    }

    /// Does the work of [`Self::prepare_trait_impl_for_function_meta_definition`].
    ///
    /// Expects the trait impl's own [`ItemContext`] to be installed; the generics resolved for the
    /// impl are returned rather than left in that context.
    fn prepare_trait_impl_in_context(
        &mut self,
        trait_impl: &mut UnresolvedTraitImpl,
    ) -> (Vec<(TraitConstraint, Location)>, Vec<ResolvedGeneric>) {
        let (trait_id, trait_generics, path_location) =
            self.resolve_trait_impl_trait_path(trait_impl);

        trait_impl.trait_id = trait_id;

        let (constraints, new_generics_trait_constraints) =
            self.setup_trait_impl_generics(trait_impl);

        // The impl ID is needed for registering associated types and later validation checks.
        let impl_id = self.interner.next_trait_impl_id();
        trait_impl.impl_id = Some(impl_id);

        let associated_types = self.resolve_trait_impl_associated_types(
            trait_impl,
            trait_generics,
            trait_id,
            path_location,
        );

        self.remove_trait_constraints_from_scope(
            constraints
                .iter()
                .chain(new_generics_trait_constraints.iter().map(|(constraint, _)| constraint)),
        );

        let wildcard_allowed = WildcardAllowed::No(WildcardDisallowedContext::TraitImplType);
        let unresolved_type = trait_impl.object_type.clone();
        let self_type = self.resolve_type(unresolved_type, wildcard_allowed);

        if let Some(trait_id) = trait_id {
            // An associated type written directly in the impl's trait generics, as in
            // `impl<H> BuildHasher<H = H>`, has no value to resolve. This can't be written in code,
            // but it could come from unquoted types.
            let written = associated_types.iter().map(|(name, _)| name).collect::<HashSet<_>>();
            let unwritten =
                self.interner.get_associated_types_for_impl(impl_id).iter().any(|named| {
                    matches!(named.typ, Type::NamedGeneric(_)) && !written.contains(&named.name)
                });
            if unwritten {
                self.push_err(TypeCheckError::expecting_other_error(
                    "collect_trait_impl: missing associated type",
                    trait_impl.object_type.location,
                ));
            }

            for (name, typ) in associated_types {
                let Some(Type::NamedGeneric(named_generic)) =
                    self.interner.find_associated_type_for_impl(impl_id, name.as_str())
                else {
                    // This can happen if the associated type is specified directly in the impl trait generics,
                    // This can't be done in code, but it could happen with unquoted types.
                    continue;
                };
                self.session.deferred.trait_impl_associated_types.register(
                    named_generic.type_var.id(),
                    UnresolvedImplAssociatedType {
                        module_id: trait_impl.module_id,
                        self_type: self_type.clone(),
                        trait_id,
                        generics: trait_impl.resolved_generics.clone(),
                        name,
                        typ,
                    },
                );
            }

            // The ordered generics and associated types are already stored by `resolve_trait_impl_associated_types`.
            // Now that we have resolved the self-type, register the prepared trait impl for it.
            let impl_generics = self.item.generics.type_vars();
            self.interner.add_prepared_trait_implementation(
                self_type.clone(),
                trait_id,
                impl_id,
                impl_generics,
                constraints.clone(),
                trait_impl.object_type.location,
            );
        }

        trait_impl.methods.self_type = Some(self_type.clone());
        trait_impl.resolved_object_type = Some(self_type);

        // Add trait reference
        if let Some(trait_id) = trait_id {
            let (location, is_self_type_name) = match &trait_impl.r#trait.typ {
                UnresolvedTypeData::Named(trait_path, _, _) => {
                    let trait_name = trait_path.last_ident();
                    (trait_name.location(), trait_name.is_self_type_name())
                }
                _ => (trait_impl.r#trait.location, false),
            };
            self.interner.add_trait_reference(trait_id, location, is_self_type_name);
        }

        let generics = self.item.generics.take_params();

        (new_generics_trait_constraints, generics)
    }

    /// Resolves the trait path from a trait impl declaration.
    /// Returns (`trait_id`, `trait_generics`, `path_location`).
    #[tracing::instrument(level = "trace", skip_all)]
    fn resolve_trait_impl_trait_path(
        &mut self,
        trait_impl: &UnresolvedTraitImpl,
    ) -> (Option<TraitId>, GenericTypeArgs, Location) {
        match &trait_impl.r#trait.typ {
            UnresolvedTypeData::Named(trait_path, trait_generics, _) => {
                let mut trait_generics = trait_generics.clone();
                let location = trait_path.location;
                let trait_path = self.validate_path(trait_path.clone());
                let trait_id = self.resolve_trait_by_path(trait_path);

                // Check and remove and any generic that is specifying an associated item
                if !trait_generics.named_args.is_empty()
                    && let Some(trait_id) = trait_id
                {
                    let associated_types =
                        self.interner.get_trait(trait_id).associated_types.clone();
                    trait_generics.named_args.retain(|(name, typ)| {
                        let associated_type = associated_types
                            .iter()
                            .find(|associated_type| associated_type.name.as_str() == name.as_str());
                        if associated_type.is_some() {
                            let location = name.location().merge(typ.location);
                            self.push_err(
                                ResolverError::AssociatedItemConstraintsNotAllowedInGenerics {
                                    location,
                                },
                            );
                            false
                        } else {
                            true
                        }
                    });
                }

                (trait_id, trait_generics.clone(), location)
            }
            UnresolvedTypeData::Resolved(quoted_type_id) => {
                let typ = self.interner.get_quoted_type(*quoted_type_id);
                let location = trait_impl.r#trait.location;
                let Type::TraitAsType(trait_id, _, trait_generics) = typ else {
                    let found = typ.to_string();
                    self.push_err(ResolverError::ExpectedTrait { location, found });
                    return (None, GenericTypeArgs::default(), location);
                };

                // In order to take associated types into account we turn these resolved generics
                // into unresolved ones, but ones that point to solved types.
                let trait_id = *trait_id;
                let trait_generics = trait_generics.clone();
                let trait_generics = GenericTypeArgs {
                    ordered_args: vecmap(&trait_generics.ordered, |typ| {
                        let quoted_type_id = self.interner.push_quoted_type(typ.clone());
                        let typ = UnresolvedTypeData::Resolved(quoted_type_id);
                        UnresolvedType { typ, location }
                    }),
                    named_args: vecmap(&trait_generics.named, |named_type| {
                        let quoted_type_id = self.interner.push_quoted_type(named_type.typ.clone());
                        let typ = UnresolvedTypeData::Resolved(quoted_type_id);
                        (named_type.name.clone(), UnresolvedType { typ, location })
                    }),
                    kinds: Vec::new(),
                };

                (Some(trait_id), trait_generics, location)
            }
            _ => {
                let location = trait_impl.r#trait.location;
                let found = trait_impl.r#trait.typ.to_string();
                self.push_err(ResolverError::ExpectedTrait { location, found });
                (None, GenericTypeArgs::default(), location)
            }
        }
    }

    /// Sets up generics for a trait impl and processes trait constraints from the where clause.
    /// Returns tuple of (resolved constraints, new generic constraints).
    #[tracing::instrument(level = "trace", skip_all)]
    fn setup_trait_impl_generics(
        &mut self,
        trait_impl: &mut UnresolvedTraitImpl,
    ) -> (Vec<TraitConstraint>, Vec<(TraitConstraint, Location)>) {
        self.add_generics(&trait_impl.generics);
        trait_impl.resolved_generics = self.item.generics.params().to_vec();

        let new_generics = self.desugar_trait_constraints(&mut trait_impl.where_clause);
        for generic in new_generics {
            trait_impl.resolved_generics.push(generic.clone());
            self.item.generics.add_param(generic);
        }

        // We need to resolve the where clause before any associated types to be
        // able to resolve trait as type syntax, eg. `<T as Foo>` in case there
        // is a where constraint for `T: Foo`.
        let constraints = self.resolve_trait_constraints_and_add_to_scope(&trait_impl.where_clause);
        let new_generics_trait_constraints = self.associated_type_constraints(&constraints);

        // Attach any trait constraints on the impl to the function
        for (_, _, method) in &mut trait_impl.methods.functions {
            method.def.where_clause.append(&mut trait_impl.where_clause.clone());
        }

        // Return the constraints along with the new generics trait constraints
        // so they can be removed from scope later
        (constraints, new_generics_trait_constraints)
    }

    /// Resolves the trait generics of a trait impl, with a placeholder for each associated type
    /// and constant, and checks for missing generics. Returns the written value of each
    /// associated item, to resolve later.
    #[tracing::instrument(level = "trace", skip_all)]
    fn resolve_trait_impl_associated_types(
        &mut self,
        trait_impl: &mut UnresolvedTraitImpl,
        mut trait_generics: GenericTypeArgs,
        trait_id: Option<TraitId>,
        path_location: Location,
    ) -> Vec<(Ident, UnresolvedType)> {
        // Add each associated type to the list of named type arguments
        let associated_types = self.take_unresolved_associated_types(trait_impl);

        // Put every associated type behind a type variable (inside a NamedGeneric).
        // This way associated types can be referred to even if their actual value (for associated constants)
        // is not known yet. This is to allow associated constants to refer to associated constants
        // in other trait impls.
        let object = self.unresolved_type_name(&trait_impl.object_type);
        let trait_name = trait_id.map(|id| self.interner.get_trait(id).name.to_string());
        let associated_types_behind_type_vars = vecmap(&associated_types, |(name, _typ, kind)| {
            let new_generic_id = self.interner.next_type_variable_id();
            let type_var = TypeVariable::unbound(new_generic_id, kind.clone());
            let typ = type_var.into_named_generic(
                &Rc::new(name.to_string()),
                trait_name.as_ref().map(|tn| (object.as_str(), tn.as_str())),
            );
            let typ = self.interner.push_quoted_type(typ);
            let typ = UnresolvedTypeData::Resolved(typ).with_location(name.location());
            (name.clone(), typ)
        });

        trait_generics.named_args.extend(associated_types_behind_type_vars);

        let associated_types = vecmap(associated_types, |(name, typ, _kind)| (name, typ));

        let (ordered_generics, named_generics) = trait_id
            .map(|trait_id| {
                // Check for missing generics & associated types for the trait being implemented
                self.resolve_trait_args_from_trait_impl(trait_generics, trait_id, path_location)
            })
            .unwrap_or_default();

        let impl_id = trait_impl.impl_id.expect("impl_id should be set");
        self.interner.set_trait_generics_for_impl(impl_id, ordered_generics, named_generics);

        associated_types
    }

    /// Identical to [`Self::resolve_type_or_trait_args_inner`] but does not allow
    /// associated types to be elided since trait impls must specify them.
    #[tracing::instrument(level = "trace", skip_all)]
    fn resolve_trait_args_from_trait_impl(
        &mut self,
        args: GenericTypeArgs,
        item: TraitId,
        location: Location,
    ) -> (Vec<Type>, Vec<NamedType>) {
        let mode = PathResolutionMode::MarkAsReferenced;
        let allow_implicit_named_args = false;
        let wildcard_allowed = WildcardAllowed::Yes;
        self.resolve_type_or_trait_args_inner(
            args,
            item,
            location,
            allow_implicit_named_args,
            mode,
            wildcard_allowed,
        )
    }
}
