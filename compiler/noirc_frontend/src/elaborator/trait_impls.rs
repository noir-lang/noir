//! Trait implementation collection, method matching, and coherence checking.

use crate::{
    Kind, NamedGeneric, ResolvedGeneric, Shared, TypeBindings, TypeVariable,
    ast::{GenericTypeArgs, Ident, UnresolvedType, UnresolvedTypeData, UnresolvedTypeExpression},
    elaborator::{
        PathResolutionMode, WildcardDisallowedContext,
        types::{WildcardAllowed, bind_ordered_generics},
    },
    hir::{
        def_collector::{
            dc_crate::{CompilationError, UnresolvedTraitImpl},
            errors::DefCollectorErrorKind,
        },
        resolution::errors::ResolverError,
        type_check::TypeCheckError,
    },
    hir_def::traits::{NamedType, TraitImpl},
    node_interner::TraitImplId,
};
use crate::{
    Type,
    hir::def_collector::errors::DuplicateType,
    hir_def::traits::{TraitConstraint, TraitFunction},
    node_interner::{FuncId, TraitId},
};

use iter_extended::vecmap;
use noirc_errors::Location;
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;

use super::Elaborator;

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
    /// - `OverlappingImpl`: Another impl already exists for this type/trait combination
    pub(super) fn collect_trait_impl(&mut self, trait_impl: &mut UnresolvedTraitImpl) {
        let previous_local_module = self.local_module.replace(trait_impl.module_id);
        let previous_current_trait_impl =
            std::mem::replace(&mut self.current_trait_impl, trait_impl.impl_id);

        let self_type = trait_impl.methods.self_type.clone();
        let self_type =
            self_type.expect("Expected struct type to be set before collect_trait_impl");

        let previous_self_type = self.self_type.replace(self_type.clone());
        let self_type_location = trait_impl.object_type.location;

        if matches!(self_type, Type::Reference(..)) {
            self.push_err(DefCollectorErrorKind::ReferenceInTraitImpl {
                location: self_type_location,
            });
        }

        self.check_generics_appear_in_types(
            &trait_impl.generics,
            &[&trait_impl.r#trait, &trait_impl.object_type],
            &trait_impl.where_clause,
        );

        if let Some(trait_id) = trait_impl.trait_id {
            let previous_generics =
                std::mem::replace(&mut self.generics, trait_impl.resolved_generics.clone());

            let where_clause =
                self.resolve_trait_constraints_and_add_to_scope(&trait_impl.where_clause);

            // Now solve the actual types of the associated types
            // (before this we only declared them without knowing their type)
            if let Some(trait_impl_id) = trait_impl.impl_id {
                let unresolved_associated_types =
                    std::mem::take(&mut trait_impl.unresolved_associated_types);
                let mut unresolved_associated_types =
                    unresolved_associated_types.into_iter().collect::<HashMap<_, _>>();

                let associated_types =
                    self.interner.get_associated_types_for_impl(trait_impl_id).to_vec();
                for associated_type in &associated_types {
                    let Type::NamedGeneric(named_generic) = &associated_type.typ else {
                        // This can happen if the associated type is specified directly in the impl trait generics,
                        // This can't be done in code, but it could happen with unquoted types.
                        continue;
                    };

                    let Some(unresolved_type) =
                        unresolved_associated_types.remove(&associated_type.name)
                    else {
                        // This too can happen if the associated type is specified directly in the impl trait generics,
                        // like `impl<H> BuildHasher<H = H>`, where `H` is a named generic but its resolution isn't delayed.
                        // This can't be done in code, but it could happen with unquoted types.
                        self.push_err(TypeCheckError::ExpectingOtherError {
                            message: "collect_trait_impl: missing associated type".to_string(),
                            location: trait_impl.object_type.location,
                        });
                        continue;
                    };
                    let wildcard_allowed =
                        WildcardAllowed::No(WildcardDisallowedContext::AssociatedType);
                    let location = unresolved_type.location;
                    let resolved_type = self.resolve_type_with_kind(
                        unresolved_type,
                        &associated_type.typ.kind(),
                        wildcard_allowed,
                    );
                    if let Err(error) = named_generic.type_var.try_bind(
                        resolved_type,
                        &named_generic.type_var.kind(),
                        location,
                    ) {
                        self.push_err(error);
                    }
                }
            }

            let trait_ = self.interner.get_trait(trait_id);

            // If there are bounds on the trait's associated types, check them now
            let associated_type_bounds = &trait_.associated_type_bounds;
            let associated_type_bounds = associated_type_bounds.clone();
            let named_generics =
                self.interner.get_associated_types_for_impl(trait_impl.impl_id.unwrap()).to_vec();
            for named_generic in named_generics {
                let Some(bounds) = associated_type_bounds.get(named_generic.name.as_str()) else {
                    continue;
                };
                let object_type = &named_generic.typ;
                for bound in bounds {
                    if let Err(error) = self.interner.lookup_trait_implementation(
                        object_type,
                        bound.trait_id,
                        &bound.trait_generics.ordered,
                        &bound.trait_generics.named,
                    ) {
                        self.push_trait_constraint_error(
                            object_type,
                            error,
                            named_generic.name.location(),
                        );
                    }
                }
            }

            self.remove_trait_constraints_from_scope(where_clause.iter());

            self.collect_trait_impl_methods(trait_id, trait_impl, &where_clause);

            let location = trait_impl.object_type.location;
            self.declare_methods_on_data_type(Some(trait_id), &mut trait_impl.methods, location);

            let trait_visibility = self.interner.get_trait(trait_id).visibility;

            let methods = trait_impl.methods.function_ids();
            for func_id in &methods {
                if self.interner.set_function_trait(*func_id, self_type.clone(), trait_id).is_some()
                {
                    self.push_err(TypeCheckError::ExpectingOtherError {
                        message: "collect_trait_impl: overlapping function trait".to_string(),
                        location,
                    });
                }

                // A trait impl method has the same visibility as its trait
                let modifiers = self.interner.function_modifiers_mut(func_id);
                modifiers.visibility = trait_visibility;
            }

            let trait_generics = trait_impl.resolved_trait_generics.clone();
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
                    self.push_err(TypeCheckError::ExpectingOtherError {
                        message: "collect_trait_impl: missing trait type".to_string(),
                        location,
                    });
                    Ident::new(trait_impl.r#trait.to_string(), trait_impl.r#trait.location)
                }
            };

            let resolved_trait_impl = Shared::new(TraitImpl {
                ident,
                location,
                typ: self_type.clone(),
                trait_id,
                trait_generics,
                file: trait_impl.file_id,
                crate_id: self.crate_id,
                where_clause,
                methods,
            });

            let generics = vecmap(&self.generics, |generic| generic.type_var.clone());

            match self.interner.add_trait_implementation(
                self_type.clone(),
                trait_id,
                trait_impl.impl_id.expect("ICE: impl_id should be set in define_function_metas"),
                generics,
                resolved_trait_impl,
                location,
            ) {
                Ok(Ok(())) => (),
                Err(error) => self.push_err(error),
                Ok(Err(prev_location)) => {
                    self.push_err(DefCollectorErrorKind::OverlappingImpl {
                        typ: self_type.clone(),
                        location: self_type_location,
                        prev_location,
                    });
                }
            }

            self.generics = previous_generics;
        }

        self.local_module = previous_local_module;
        self.current_trait_impl = previous_current_trait_impl;
        self.self_type = previous_self_type;
    }

    pub(super) fn collect_trait_impl_methods(
        &mut self,
        trait_id: TraitId,
        trait_impl: &mut UnresolvedTraitImpl,
        trait_impl_where_clause: &[TraitConstraint],
    ) {
        let previous_local_module = self.local_module.replace(trait_impl.module_id);

        let impl_id = trait_impl.impl_id.expect("impl_id should be set in define_function_metas");

        // In this Vec methods[i] corresponds to trait.methods[i]. If the impl has no implementation
        // for a particular method, the default implementation will be added at that slot.
        let mut ordered_methods = Vec::new();

        // Check whether the trait implementation is in the same crate as either the trait or the type
        self.check_trait_impl_crate_coherence(trait_id, trait_impl);

        // Set of function ids that have a corresponding method in the trait
        let mut func_ids_in_trait = HashSet::default();

        // Temporarily take ownership of the trait's methods so we can iterate over them
        // while also mutating the interner
        let the_trait = self.interner.get_trait_mut(trait_id);
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
                    // copy 'where' clause from unresolved trait impl
                    let mut default_impl_clone = default_impl.clone();
                    default_impl_clone.def.where_clause.extend(trait_impl.where_clause.clone());

                    let func_id = self.interner.push_empty_fn();
                    let module = self.module_id();
                    let location = default_impl.def.location;
                    self.interner.push_function(func_id, &default_impl.def, module, location);
                    self.recover_generics(|this| {
                        let no_trait_id = None;
                        let no_extra_trait_constraints = &[];
                        this.define_function_meta(
                            &mut default_impl_clone,
                            func_id,
                            no_trait_id,
                            no_extra_trait_constraints,
                        );
                    });
                    func_ids_in_trait.insert(func_id);
                    ordered_methods.push((
                        method.default_impl_module_id,
                        func_id,
                        *default_impl_clone,
                    ));
                } else {
                    self.push_err(DefCollectorErrorKind::TraitMissingMethod {
                        trait_name: self.interner.get_trait(trait_id).name.clone(),
                        method_name: method.name.clone(),
                        trait_impl_location: trait_impl.object_type.location,
                    });
                }
            } else {
                for (_, func_id, _) in &overrides {
                    self.check_where_clause_against_trait(
                        func_id,
                        method,
                        trait_impl_where_clause,
                        &trait_impl.resolved_trait_generics,
                        trait_id,
                        impl_id,
                    );

                    func_ids_in_trait.insert(*func_id);
                }

                if overrides.len() > 1 {
                    self.push_err(DefCollectorErrorKind::Duplicate {
                        typ: DuplicateType::TraitAssociatedItem,
                        first_def: overrides[0].2.name_ident().clone(),
                        second_def: overrides[1].2.name_ident().clone(),
                    });
                }

                ordered_methods.push(overrides[0].clone());
            }
        }

        // Restore the methods that were taken before the for loop
        let the_trait = self.interner.get_trait_mut(trait_id);
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
                self.push_err(error);
            }
        }

        trait_impl.methods.functions = ordered_methods;
        trait_impl.methods.trait_id = Some(trait_id);

        self.local_module = previous_local_module;
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
        let self_type = self.self_type.as_ref().unwrap().clone();

        let mut bindings =
            self.interner.trait_to_impl_bindings(trait_id, impl_id, trait_impl_generics, self_type);

        let override_meta = self.interner.function_meta(func_id);
        // Substitute each generic on the trait function with the corresponding generic on the impl function
        for (ResolvedGeneric { type_var: trait_fn_generic, .. }, impl_fn_resolved_generic) in
            method.direct_generics.iter().zip(&override_meta.direct_generics)
        {
            let trait_fn_kind = trait_fn_generic.kind();
            let arg = impl_fn_resolved_generic.clone().as_named_generic();
            bindings.insert(
                trait_fn_generic.id(),
                (trait_fn_generic.clone(), trait_fn_kind.clone(), arg),
            );
        }

        let mut substituted_method_ids = HashSet::default();
        for method_constraint in method.trait_constraints.iter() {
            let substituted_constraint_type = method_constraint.typ.substitute(&bindings);
            let substituted_trait_generics = method_constraint
                .trait_bound
                .trait_generics
                .map(|generic| generic.substitute(&bindings));

            substituted_method_ids.insert((
                substituted_constraint_type,
                method_constraint.trait_bound.trait_id,
                substituted_trait_generics,
            ));
        }

        for override_trait_constraint in override_meta.trait_constraints.clone() {
            let override_constraint_is_from_impl =
                trait_impl_where_clause.iter().any(|impl_constraint| {
                    impl_constraint.trait_bound.trait_id
                        == override_trait_constraint.trait_bound.trait_id
                });
            if override_constraint_is_from_impl {
                continue;
            }

            let override_trait_generics =
                override_trait_constraint.trait_bound.trait_generics.clone();

            if !substituted_method_ids.contains(&(
                override_trait_constraint.typ.clone(),
                override_trait_constraint.trait_bound.trait_id,
                override_trait_generics,
            )) {
                let the_trait =
                    self.interner.get_trait(override_trait_constraint.trait_bound.trait_id);
                self.push_err(DefCollectorErrorKind::ImplIsStricterThanTrait {
                    constraint_typ: override_trait_constraint.typ,
                    constraint_name: the_trait.name.to_string(),
                    constraint_generics: override_trait_constraint.trait_bound.trait_generics,
                    constraint_location: override_trait_constraint.trait_bound.location,
                    trait_method_name: method.name.to_string(),
                    trait_method_location: method.location,
                });
            }
        }
    }

    fn check_trait_impl_crate_coherence(
        &mut self,
        trait_id: TraitId,
        trait_impl: &UnresolvedTraitImpl,
    ) {
        let previous_local_module = self.local_module.replace(trait_impl.module_id);

        let object_crate = match &trait_impl.resolved_object_type {
            Some(Type::DataType(struct_or_enum_type, _)) => {
                Some(struct_or_enum_type.borrow().id.krate())
            }
            _ => None,
        };

        let the_trait = self.interner.get_trait(trait_id);
        if self.crate_id != the_trait.crate_id && Some(self.crate_id) != object_crate {
            self.push_err(DefCollectorErrorKind::TraitImplOrphaned {
                location: trait_impl.object_type.location,
            });
        }

        self.local_module = previous_local_module;
    }

    pub(super) fn take_unresolved_associated_types(
        &mut self,
        trait_impl: &mut UnresolvedTraitImpl,
    ) -> Vec<(Ident, UnresolvedType, Kind)> {
        let mut associated_types = Vec::new();
        for (name, typ, expr) in trait_impl.associated_constants.drain(..) {
            let wildcard_allowed = WildcardAllowed::No(WildcardDisallowedContext::AssociatedType);
            let resolved_type =
                typ.map(|typ| self.resolve_type(typ, wildcard_allowed)).unwrap_or(Type::Error);
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

    pub(super) fn add_trait_impl_assumed_trait_implementations(
        &mut self,
        impl_id: Option<TraitImplId>,
    ) {
        if let Some(impl_id) = impl_id {
            if let Some(trait_implementation) = self.interner.try_get_trait_implementation(impl_id)
            {
                for trait_constrain in &trait_implementation.borrow().where_clause {
                    let trait_bound = &trait_constrain.trait_bound;
                    self.add_trait_bound_to_scope(
                        trait_bound.location,
                        &trait_constrain.typ,
                        trait_bound,
                        trait_bound.trait_id,
                    );
                }
            }
        }
    }

    pub(super) fn remove_trait_impl_assumed_trait_implementations(
        &mut self,
        impl_id: Option<TraitImplId>,
    ) {
        if let Some(impl_id) = impl_id {
            if let Some(trait_implementation) = self.interner.try_get_trait_implementation(impl_id)
            {
                for trait_constrain in &trait_implementation.borrow().where_clause {
                    self.interner.remove_assumed_trait_implementations_for_trait(
                        trait_constrain.trait_bound.trait_id,
                    );
                }
            }
        }
    }

    pub(super) fn check_trait_impl_where_clause_matches_trait_where_clause(
        &mut self,
        trait_impl: &UnresolvedTraitImpl,
    ) {
        let Some(trait_id) = trait_impl.trait_id else {
            self.push_err(TypeCheckError::ExpectingOtherError {
                message:
                    "check_trait_impl_where_clause_matches_trait_where_clause: missing trait ID"
                        .to_string(),
                location: trait_impl.object_type.location,
            });
            return;
        };

        let Some(the_trait) = self.interner.try_get_trait(trait_id) else {
            self.push_err(TypeCheckError::ExpectingOtherError {
                message: "check_trait_impl_where_clause_matches_trait_where_clause: missing trait"
                    .to_string(),
                location: trait_impl.object_type.location,
            });
            return;
        };

        let impl_trait = the_trait.name.to_string();

        let mut bindings = TypeBindings::default();
        bind_ordered_generics(
            &the_trait.generics,
            &trait_impl.resolved_trait_generics,
            &mut bindings,
        );

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
                self.push_err(TypeCheckError::ExpectingOtherError {
                    message: "check_trait_impl_where_clause_matches_trait_where_clause: missing trait constraint trait".to_string(),
                    location: *error_location,
                });
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
            ) {
                Ok((_, impl_bindings, impl_instantiation_bindings)) => {
                    bindings.extend(impl_bindings);
                    bindings.extend(impl_instantiation_bindings);
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
        &mut self,
        named_generics: &mut [NamedType],
        bindings: &mut TypeBindings,
    ) {
        for named_type in named_generics.iter_mut() {
            match &named_type.typ {
                Type::NamedGeneric(NamedGeneric { type_var, implicit: true, .. })
                    if type_var.borrow().is_unbound() =>
                {
                    let type_var_id = type_var.id();
                    let new_type_var_id = self.interner.next_type_variable_id();
                    let kind = type_var.kind();
                    let new_type_var = TypeVariable::unbound(new_type_var_id, kind.clone());
                    named_type.typ = Type::TypeVariable(new_type_var.clone());
                    bindings.insert(type_var_id, (new_type_var, kind, named_type.typ.clone()));
                }
                _ => (),
            };
        }
    }

    pub(super) fn check_parent_traits_are_implemented(&mut self, trait_impl: &UnresolvedTraitImpl) {
        let Some(trait_id) = trait_impl.trait_id else {
            self.push_err(TypeCheckError::ExpectingOtherError {
                message: "check_parent_traits_are_implemented: missing trait ID".to_string(),
                location: trait_impl.object_type.location,
            });
            return;
        };

        let Some(object_type) = &trait_impl.resolved_object_type else {
            self.push_err(TypeCheckError::ExpectingOtherError {
                message: "check_parent_traits_are_implemented: missing object type".to_string(),
                location: trait_impl.object_type.location,
            });
            return;
        };

        let Some(the_trait) = self.interner.try_get_trait(trait_id) else {
            self.push_err(TypeCheckError::ExpectingOtherError {
                message: "check_parent_traits_are_implemented: missing trait".to_string(),
                location: trait_impl.object_type.location,
            });
            return;
        };

        let impl_trait = the_trait.name.to_string();

        let mut bindings = TypeBindings::default();
        bind_ordered_generics(
            &the_trait.generics,
            &trait_impl.resolved_trait_generics,
            &mut bindings,
        );

        // Note: we only check if the immediate parents are implemented, we don't check recursively.
        // Why? If a parent isn't implemented, we get an error. If a parent is implemented, we'll
        // do the same check for the parent, so this trait's parents parents will be checked, so the
        // recursion is guaranteed.
        //
        // Convert parent trait bounds (ResolvedTraitBound) to TraitConstraints using
        // {Self, ResolvedTraitBound} where Self is the object type being implemented for.
        let constraints: Vec<TraitConstraint> = the_trait
            .trait_bounds
            .iter()
            .map(|bound| TraitConstraint { typ: object_type.clone(), trait_bound: bound.clone() })
            .collect();

        self.check_trait_bounds_are_satisfied(
            constraints,
            &impl_trait,
            &trait_impl.object_type.location,
            &mut bindings,
        );
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
    pub(super) fn prepare_trait_impl_for_function_meta_definition(
        &mut self,
        trait_impl: &mut UnresolvedTraitImpl,
    ) -> Vec<(TraitConstraint, Location)> {
        let previous_local_module = self.local_module.replace(trait_impl.module_id);

        let (trait_id, trait_generics, path_location) =
            self.resolve_trait_impl_trait_path(trait_impl);

        trait_impl.trait_id = trait_id;

        let (constraints, new_generics_trait_constraints) =
            self.setup_trait_impl_generics(trait_impl);

        // The impl ID is needed for registering associated types and later validation checks.
        let impl_id = Some(self.interner.next_trait_impl_id());
        trait_impl.impl_id = impl_id;

        self.resolve_trait_impl_associated_types(
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

        self.local_module = previous_local_module;

        new_generics_trait_constraints
    }

    /// Resolves the trait path from a trait impl declaration.
    /// Returns (trait_id, trait_generics, path_location).
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
                if !trait_generics.named_args.is_empty() {
                    if let Some(trait_id) = trait_id {
                        let associated_types =
                            self.interner.get_trait(trait_id).associated_types.clone();
                        trait_generics.named_args.retain(|(name, typ)| {
                            let associated_type = associated_types.iter().find(|associated_type| {
                                associated_type.name.as_str() == name.as_str()
                            });
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
    fn setup_trait_impl_generics(
        &mut self,
        trait_impl: &mut UnresolvedTraitImpl,
    ) -> (Vec<TraitConstraint>, Vec<(TraitConstraint, Location)>) {
        self.add_generics(&trait_impl.generics);
        trait_impl.resolved_generics = self.generics.clone();

        let new_generics = self.desugar_trait_constraints(&mut trait_impl.where_clause);
        let mut new_generics_trait_constraints = Vec::new();
        for (new_generic, bounds) in new_generics {
            for bound in bounds {
                let typ = Type::TypeVariable(new_generic.type_var.clone());
                let location = new_generic.location;
                self.add_trait_bound_to_scope(location, &typ, &bound, bound.trait_id);
                new_generics_trait_constraints
                    .push((TraitConstraint { typ, trait_bound: bound }, location));
            }
            trait_impl.resolved_generics.push(new_generic.clone());
            self.generics.push(new_generic);
        }

        // We need to resolve the where clause before any associated types to be
        // able to resolve trait as type syntax, eg. `<T as Foo>` in case there
        // is a where constraint for `T: Foo`.
        let constraints = self.resolve_trait_constraints_and_add_to_scope(&trait_impl.where_clause);

        // Attach any trait constraints on the impl to the function
        for (_, _, method) in trait_impl.methods.functions.iter_mut() {
            method.def.where_clause.append(&mut trait_impl.where_clause.clone());
        }

        // Return the constraints along with the new generics trait constraints
        // so they can be removed from scope later
        (constraints, new_generics_trait_constraints)
    }

    /// Resolves associated types for a trait impl and checks for missing generics.
    /// Sets resolved_trait_generics and unresolved_associated_types on trait_impl.
    fn resolve_trait_impl_associated_types(
        &mut self,
        trait_impl: &mut UnresolvedTraitImpl,
        mut trait_generics: GenericTypeArgs,
        trait_id: Option<TraitId>,
        path_location: Location,
    ) {
        // Add each associated type to the list of named type arguments
        let associated_types = self.take_unresolved_associated_types(trait_impl);

        // Put every associated type behind a type variable (inside a NamedGeneric).
        // This way associated types can be referred to even if their actual value (for associated constants)
        // is not known yet. This is to allow associated constants to refer to associated constants
        // in other trait impls.
        let associated_types_behind_type_vars = vecmap(&associated_types, |(name, _typ, kind)| {
            let new_generic_id = self.interner.next_type_variable_id();
            let type_var = TypeVariable::unbound(new_generic_id, kind.clone());
            let typ = type_var.clone().into_named_generic(std::rc::Rc::new(name.to_string()));
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

        trait_impl.resolved_trait_generics = ordered_generics;
        let impl_id = trait_impl.impl_id.expect("impl_id should be set");
        self.interner.set_associated_types_for_impl(impl_id, named_generics);

        trait_impl.unresolved_associated_types = associated_types;
    }

    /// Identical to [Self::resolve_type_or_trait_args_inner] but does not allow
    /// associated types to be elided since trait impls must specify them.
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
