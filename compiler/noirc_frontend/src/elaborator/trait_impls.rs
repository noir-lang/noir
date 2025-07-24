use crate::{
    Kind, NamedGeneric, ResolvedGeneric,
    ast::{Ident, UnresolvedType, UnresolvedTypeData, UnresolvedTypeExpression},
    graph::CrateId,
    hir::def_collector::{
        dc_crate::{CompilationError, UnresolvedTraitImpl},
        errors::DefCollectorErrorKind,
    },
    node_interner::TraitImplId,
};
use crate::{
    Type,
    hir::def_collector::errors::DuplicateType,
    hir_def::traits::{TraitConstraint, TraitFunction},
    node_interner::{FuncId, TraitId},
};

use rustc_hash::FxHashSet as HashSet;

use super::Elaborator;

impl Elaborator<'_> {
    pub(super) fn collect_trait_impl_methods(
        &mut self,
        trait_id: TraitId,
        trait_impl: &mut UnresolvedTraitImpl,
        trait_impl_where_clause: &[TraitConstraint],
    ) {
        self.local_module = trait_impl.module_id;

        let impl_id = trait_impl.impl_id.expect("impl_id should be set in define_function_metas");

        // In this Vec methods[i] corresponds to trait.methods[i]. If the impl has no implementation
        // for a particular method, the default implementation will be added at that slot.
        let mut ordered_methods = Vec::new();

        // check whether the trait implementation is in the same crate as either the trait or the type
        self.check_trait_impl_crate_coherence(trait_id, trait_impl);

        // set of function ids that have a corresponding method in the trait
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
                        this.define_function_meta(&mut default_impl_clone, func_id, None, &[]);
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
        for (
            ResolvedGeneric { type_var: trait_fn_generic, .. },
            ResolvedGeneric { name, type_var: impl_fn_generic, .. },
        ) in method.direct_generics.iter().zip(&override_meta.direct_generics)
        {
            let trait_fn_kind = trait_fn_generic.kind();
            let arg = Type::NamedGeneric(NamedGeneric {
                type_var: impl_fn_generic.clone(),
                name: name.clone(),
                implicit: false,
            });
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
        self.local_module = trait_impl.module_id;

        let object_crate = match &trait_impl.resolved_object_type {
            Some(Type::DataType(struct_type, _)) => struct_type.borrow().id.krate(),
            _ => CrateId::Dummy,
        };

        let the_trait = self.interner.get_trait(trait_id);
        if self.crate_id != the_trait.crate_id && self.crate_id != object_crate {
            self.push_err(DefCollectorErrorKind::TraitImplOrphaned {
                location: trait_impl.object_type.location,
            });
        }
    }

    pub(super) fn take_unresolved_associated_types(
        &mut self,
        trait_impl: &mut UnresolvedTraitImpl,
    ) -> Vec<(Ident, UnresolvedType, Kind)> {
        let mut associated_types = Vec::new();
        for (name, typ, expr) in trait_impl.associated_constants.drain(..) {
            let wildcard_allowed = false;
            let resolved_type = self.resolve_type(typ, wildcard_allowed);
            let kind = Kind::Numeric(Box::new(resolved_type));
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
            associated_types.push((name, typ, Kind::Any));
        }
        associated_types
    }
}
