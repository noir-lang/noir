use iter_extended::vecmap;
use noirc_errors::Location;
use rustc_hash::FxHashMap as HashMap;
use std::collections::HashSet;

use crate::{
    GenericTypeVars, Shared, Type, TypeBindings,
    graph::CrateId,
    hir::type_check::generics::TraitGenerics,
    hir_def::traits::{NamedType, ResolvedTraitBound, TraitConstraint, TraitImpl},
    node_interner::{ImplSearchErrorKind, TraitId, TraitImplId, TraitImplKind},
};

use super::NodeInterner;

/// An arbitrary number to limit the recursion depth when searching for trait impls.
/// This is needed to stop recursing for cases such as `impl<T> Foo for T where T: Eq`
const IMPL_SEARCH_RECURSION_LIMIT: u32 = 10;

impl NodeInterner {
    /// Returns what the next trait impl id is expected to be.
    pub fn next_trait_impl_id(&mut self) -> TraitImplId {
        let next_id = self.next_trait_implementation_id;
        self.next_trait_implementation_id += 1;
        TraitImplId(next_id)
    }

    /// Gets the trait implementations from the node interner.
    pub fn trait_implementations(&self) -> &HashMap<TraitImplId, Shared<TraitImpl>> {
        &self.trait_implementations
    }

    pub fn try_get_trait_implementation(&self, id: TraitImplId) -> Option<Shared<TraitImpl>> {
        self.trait_implementations.get(&id).cloned()
    }

    pub fn get_trait_implementation(&self, id: TraitImplId) -> Shared<TraitImpl> {
        self.trait_implementations[&id].clone()
    }

    pub fn get_trait_implementations_in_crate(&self, crate_id: CrateId) -> HashSet<TraitImplId> {
        let trait_impls = self.trait_implementations.iter();
        let trait_impls = trait_impls.filter_map(|(id, trait_impl)| {
            if trait_impl.borrow().crate_id == crate_id { Some(*id) } else { None }
        });
        trait_impls.collect()
    }

    /// Adds an "assumed" trait implementation to the currently known trait implementations.
    /// Unlike normal trait implementations, these are only assumed to exist. They often correspond
    /// to `where` clauses in functions where we assume there is some `T: Eq` even though we do
    /// not yet know T. For these cases, we store an impl here so that we assume they exist and
    /// can resolve them. They are then later verified when the function is called, and linked
    /// properly after being monomorphized to the correct variant.
    ///
    /// Returns true on success, or false if there is already an overlapping impl in scope.
    pub fn add_assumed_trait_implementation(
        &mut self,
        object_type: Type,
        trait_id: TraitId,
        trait_generics: TraitGenerics,
    ) -> bool {
        // Make sure there are no overlapping impls
        let existing = self.try_lookup_trait_implementation(
            &object_type,
            trait_id,
            &trait_generics.ordered,
            &trait_generics.named,
        );
        if existing.is_ok() {
            return false;
        }

        let entries = self.trait_implementation_map.entry(trait_id).or_default();
        entries.push((object_type.clone(), TraitImplKind::Assumed { object_type, trait_generics }));
        true
    }

    /// Adds a trait implementation to the list of known implementations.
    pub fn add_trait_implementation(
        &mut self,
        object_type: Type,
        trait_id: TraitId,
        impl_id: TraitImplId,
        impl_generics: GenericTypeVars,
        trait_impl: Shared<TraitImpl>,
    ) -> Result<(), Location> {
        self.trait_implementations.insert(impl_id, trait_impl.clone());

        // Avoid adding error types to impls since they'll conflict with every other type.
        // We don't need to return an error since we expect an error to already be issued when
        // the error type is created.
        if object_type == Type::Error {
            // TODO(audit): return an "expecting other error" here
            return Ok(());
        }

        // Replace each generic with a fresh type variable
        let substitutions = impl_generics
            .into_iter()
            .map(|typevar| {
                let typevar_kind = typevar.kind();
                let typevar_id = typevar.id();
                let substitution = (
                    typevar,
                    typevar_kind.clone(),
                    self.next_type_variable_with_kind(typevar_kind),
                );
                (typevar_id, substitution)
            })
            .collect();

        let instantiated_object_type = object_type.substitute(&substitutions);

        let trait_generics = &trait_impl.borrow().trait_generics;

        // Replace any associated types with fresh type variables so that we match
        // any existing impl regardless of associated types if one already exists.
        // E.g. if we already have an `impl Foo<Bar = i32> for Baz`, we should
        // reject `impl Foo<Bar = u32> for Baz` if it were to be added.
        let associated_types = self.get_associated_types_for_impl(impl_id);

        let associated_types = vecmap(associated_types, |named| {
            let typ = self.next_type_variable();
            NamedType { name: named.name.clone(), typ }
        });

        // Ignoring overlapping `TraitImplKind::Assumed` impls here is perfectly fine.
        // It should never happen since impls are defined at global scope, but even
        // if they were, we should never prevent defining a new impl because a 'where'
        // clause already assumes it exists.
        if let Ok((TraitImplKind::Normal(existing), ..)) = self.try_lookup_trait_implementation(
            &instantiated_object_type,
            trait_id,
            trait_generics,
            &associated_types,
        ) {
            let existing_impl = self.get_trait_implementation(existing);
            let existing_impl = existing_impl.borrow();
            return Err(existing_impl.ident.location());
        }

        for method in &trait_impl.borrow().methods {
            let method_name = self.function_name(method).to_owned();
            self.add_method(&object_type, method_name, *method, Some(trait_id));
        }

        // The object type is generalized so that a generic impl will apply
        // to any type T, rather than just the generic type named T.
        let generalized_object_type = object_type.generalize_from_substitutions(substitutions);

        let entries = self.trait_implementation_map.entry(trait_id).or_default();
        entries.push((generalized_object_type, TraitImplKind::Normal(impl_id)));
        Ok(())
    }

    /// Given a `ObjectType: TraitId` pair, try to find an existing impl that satisfies the
    /// constraint. If an impl cannot be found, this will return a vector of each constraint
    /// in the path to get to the failing constraint. Usually this is just the single failing
    /// constraint, but when where clauses are involved, the failing constraint may be several
    /// constraints deep. In this case, all of the constraints are returned, starting with the
    /// failing one.
    /// If this list of failing constraints is empty, this means type annotations are required.
    /// Returns the list of instantiation bindings as well, which should be stored on the
    /// expression.
    pub(crate) fn lookup_trait_implementation(
        &self,
        object_type: &Type,
        trait_id: TraitId,
        trait_generics: &[Type],
        trait_associated_types: &[NamedType],
    ) -> Result<(TraitImplKind, TypeBindings), ImplSearchErrorKind> {
        let (impl_kind, bindings, instantiation_bindings) = self.try_lookup_trait_implementation(
            object_type,
            trait_id,
            trait_generics,
            trait_associated_types,
        )?;

        Type::apply_type_bindings(bindings);
        Ok((impl_kind, instantiation_bindings))
    }

    /// Similar to `lookup_trait_implementation` but does not apply any type bindings on success.
    /// On error returns either:
    /// - 1+ failing trait constraints, including the original.
    ///   Each constraint after the first represents a `where` clause that was followed.
    /// - 0 trait constraints indicating type annotations are needed to choose an impl.
    pub(crate) fn try_lookup_trait_implementation(
        &self,
        object_type: &Type,
        trait_id: TraitId,
        trait_generics: &[Type],
        trait_associated_types: &[NamedType],
    ) -> Result<(TraitImplKind, TypeBindings, TypeBindings), ImplSearchErrorKind> {
        let mut bindings = TypeBindings::default();
        let (impl_kind, instantiation_bindings) = self.lookup_trait_implementation_helper(
            object_type,
            trait_id,
            trait_generics,
            trait_associated_types,
            &mut bindings,
            IMPL_SEARCH_RECURSION_LIMIT,
        )?;
        Ok((impl_kind, bindings, instantiation_bindings))
    }

    /// Returns the trait implementation if found along with the instantiation bindings for
    /// instantiating that trait impl. Note that this is separate from the passed-in TypeBindings
    /// which can be bound via `Type::apply_type_bindings` if needed. Instantiation bindings should
    /// be stored as such but not bound, lest the original named generics in trait impls get bound
    /// over.
    ///
    /// On error returns either:
    /// - 1+ failing trait constraints, including the original.
    ///   Each constraint after the first represents a `where` clause that was followed.
    /// - 0 trait constraints indicating type annotations are needed to choose an impl.
    fn lookup_trait_implementation_helper(
        &self,
        object_type: &Type,
        trait_id: TraitId,
        trait_generics: &[Type],
        trait_associated_types: &[NamedType],
        type_bindings: &mut TypeBindings,
        recursion_limit: u32,
    ) -> Result<(TraitImplKind, TypeBindings), ImplSearchErrorKind> {
        let make_constraint = || {
            let ordered = trait_generics.to_vec();
            let named = trait_associated_types.to_vec();
            TraitConstraint {
                typ: object_type.clone(),
                trait_bound: ResolvedTraitBound {
                    trait_id,
                    trait_generics: TraitGenerics { ordered, named },
                    location: Location::dummy(),
                },
            }
        };

        let nested_error = || ImplSearchErrorKind::Nested(vec![make_constraint()]);

        // Prevent infinite recursion when looking for impls
        if recursion_limit == 0 {
            return Err(nested_error());
        }

        let object_type = object_type.substitute(type_bindings);

        // If the object type isn't known, just return an error saying type annotations are needed.
        if object_type.is_bindable() {
            return Err(ImplSearchErrorKind::TypeAnnotationsNeededOnObjectType);
        }

        let impls = self.trait_implementation_map.get(&trait_id).ok_or_else(nested_error)?;

        let mut matching_impls = Vec::new();
        let mut where_clause_error = None;

        for (existing_object_type, impl_kind) in impls {
            let (existing_object_type, instantiation_bindings) =
                existing_object_type.instantiate(self);

            let mut fresh_bindings = type_bindings.clone();

            if object_type.try_unify(&existing_object_type, &mut fresh_bindings).is_err() {
                continue;
            }

            let impl_trait_generics = match impl_kind {
                TraitImplKind::Normal(id) => {
                    let shared_impl = self.get_trait_implementation(*id);
                    let shared_impl = shared_impl.borrow();
                    let named = self.get_associated_types_for_impl(*id).to_vec();
                    let ordered = shared_impl.trait_generics.clone();
                    TraitGenerics { named, ordered }
                }
                TraitImplKind::Assumed { trait_generics, .. } => trait_generics.clone(),
            };

            let generics_unify = trait_generics.iter().zip(&impl_trait_generics.ordered).all(
                |(trait_generic, impl_generic)| {
                    let impl_generic = impl_generic.substitute(&instantiation_bindings);
                    trait_generic.try_unify(&impl_generic, &mut fresh_bindings).is_ok()
                },
            );

            if !generics_unify {
                continue;
            }

            if let TraitImplKind::Normal(impl_id) = impl_kind {
                let trait_impl = self.get_trait_implementation(*impl_id);
                let trait_impl = trait_impl.borrow();

                if let Err(error) = self.validate_where_clause(
                    &trait_impl.where_clause,
                    &mut fresh_bindings,
                    &instantiation_bindings,
                    recursion_limit,
                ) {
                    // Only keep the first errors we get from a failing where clause
                    if where_clause_error.is_none() {
                        where_clause_error = Some(error);
                    }
                    continue;
                }
            }

            // Match associated types by name, not position
            let associated_types_unify = trait_associated_types.iter().all(|trait_generic| {
                // Find the matching impl generic by name
                let Some(impl_generic) = impl_trait_generics
                    .named
                    .iter()
                    .find(|impl_g| impl_g.name.as_str() == trait_generic.name.as_str())
                else {
                    // If the impl doesn't have this associated type, it doesn't match
                    return false;
                };
                let impl_generic2 = impl_generic.typ.force_substitute(&instantiation_bindings);
                trait_generic.typ.try_unify(&impl_generic2, &mut fresh_bindings).is_ok()
            });
            if !associated_types_unify {
                continue;
            }

            let constraint = TraitConstraint {
                typ: existing_object_type,
                trait_bound: ResolvedTraitBound {
                    trait_id,
                    trait_generics: impl_trait_generics,
                    location: Location::dummy(),
                },
            };
            matching_impls.push((
                impl_kind.clone(),
                fresh_bindings,
                instantiation_bindings,
                constraint,
            ));
        }

        if matching_impls.len() == 1 {
            let (impl_, fresh_bindings, instantiation_bindings, _) = matching_impls.pop().unwrap();
            *type_bindings = fresh_bindings;
            Ok((impl_, instantiation_bindings))
        } else if matching_impls.is_empty() {
            let mut errors = match where_clause_error {
                Some((_, ImplSearchErrorKind::Nested(errors))) => errors,
                Some((constraint, _other)) => vec![constraint],
                None => vec![],
            };
            errors.push(make_constraint());
            Err(ImplSearchErrorKind::Nested(errors))
        } else {
            let impls = vecmap(matching_impls, |(_, _, _, constraint)| {
                let name = &self.get_trait(constraint.trait_bound.trait_id).name;
                format!("{}: {name}{}", constraint.typ, constraint.trait_bound.trait_generics)
            });
            Err(ImplSearchErrorKind::MultipleMatching(impls))
        }
    }

    /// Verifies that each constraint in the given where clause is valid.
    /// If an impl cannot be found for any constraint, the erroring constraint is returned.
    fn validate_where_clause(
        &self,
        where_clause: &[TraitConstraint],
        type_bindings: &mut TypeBindings,
        instantiation_bindings: &TypeBindings,
        recursion_limit: u32,
    ) -> Result<(), (TraitConstraint, ImplSearchErrorKind)> {
        for constraint in where_clause {
            // Instantiation bindings are generally safe to force substitute into the same type.
            // This is needed here to undo any bindings done to trait methods by monomorphization.
            // Otherwise, an impl for any (A, B) could get narrowed to only an impl for e.g. (u8, u16).
            let constraint_type =
                constraint.typ.force_substitute(instantiation_bindings).substitute(type_bindings);

            let trait_generics =
                vecmap(&constraint.trait_bound.trait_generics.ordered, |generic| {
                    generic.force_substitute(instantiation_bindings).substitute(type_bindings)
                });

            let trait_associated_types =
                vecmap(&constraint.trait_bound.trait_generics.named, |generic| {
                    let typ = generic.typ.force_substitute(instantiation_bindings);
                    NamedType { name: generic.name.clone(), typ: typ.substitute(type_bindings) }
                });

            // We can ignore any associated types on the constraint since those should not affect
            // which impl we choose.
            self.lookup_trait_implementation_helper(
                &constraint_type,
                constraint.trait_bound.trait_id,
                &trait_generics,
                &trait_associated_types,
                // Use a fresh set of type bindings here since the constraint_type originates from
                // our impl list, which we don't want to bind to.
                type_bindings,
                recursion_limit - 1,
            )
            .map_err(|error| (constraint.clone(), error))?;
        }

        Ok(())
    }

    pub(crate) fn trait_constraint_string(
        &self,
        object_type: &Type,
        trait_id: TraitId,
        trait_generics: &[Type],
        trait_associated_types: &[NamedType],
    ) -> String {
        let name = self.get_trait(trait_id).name.to_string();
        let mut generics = vecmap(trait_generics, |t| format!("{t:?}")).join(", ");
        let associated =
            vecmap(trait_associated_types, |t| format!("{}: {:?}", t.name, t.typ)).join(", ");

        if !generics.is_empty() && !associated.is_empty() {
            generics += ", ";
            generics += &associated;
        }

        if !generics.is_empty() {
            generics = format!("<{generics}>");
        }
        format!("{object_type:?}: {name}{generics}")
    }

    /// Removes all TraitImplKind::Assumed from the list of known impls for the given trait
    pub fn remove_assumed_trait_implementations_for_trait(&mut self, trait_id: TraitId) {
        self.remove_assumed_trait_implementations_for_trait_and_parents(trait_id, HashSet::new());
    }

    fn remove_assumed_trait_implementations_for_trait_and_parents(
        &mut self,
        trait_id: TraitId,
        mut visited_trait_ids: HashSet<TraitId>,
    ) {
        let entries = self.trait_implementation_map.entry(trait_id).or_default();
        entries.retain(|(_, kind)| matches!(kind, TraitImplKind::Normal(_)));

        // Also remove assumed implementations for the parent traits, if any
        if let Some(trait_bounds) =
            self.try_get_trait(trait_id).map(|the_trait| the_trait.trait_bounds.clone())
        {
            for parent_trait_bound in trait_bounds {
                // Avoid looping forever in case there are cycles
                if visited_trait_ids.contains(&parent_trait_bound.trait_id) {
                    continue;
                }
                visited_trait_ids.insert(parent_trait_bound.trait_id);

                self.remove_assumed_trait_implementations_for_trait_and_parents(
                    parent_trait_bound.trait_id,
                    visited_trait_ids.clone(),
                );
            }
        }
    }
}
