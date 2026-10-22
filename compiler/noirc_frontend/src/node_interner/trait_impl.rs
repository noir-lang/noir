use iter_extended::vecmap;
use itertools::Itertools;
use noirc_errors::Location;
use rustc_hash::FxHashMap as HashMap;
use std::{cell::Cell, collections::HashSet};

use crate::{
    GenericTypeVars, Kind, Shared, Type, TypeBindings,
    graph::CrateId,
    hir::{
        def_collector::dc_crate::CompilationError,
        type_check::{TypeCheckError, generics::TraitGenerics},
    },
    hir_def::traits::{Impl, NamedType, ResolvedTraitBound, TraitConstraint, TraitImpl},
    node_interner::{FuncId, ImplId, ImplSearchErrorKind, TraitId, TraitImplId, TraitImplKind},
};

use super::NodeInterner;

/// An arbitrary number to limit the recursion depth when searching for trait impls.
/// This is needed to stop recursing for cases such as `impl<T> Foo for T where T: Eq`
const IMPL_SEARCH_RECURSION_LIMIT: u32 = 10;

/// The most impls the overlap check tries for one pair of impls. Past it, the impls are taken not
/// to overlap, as when a where clause can't be decided.
const OVERLAP_SEARCH_BUDGET: u32 = 10_000;

/// A bound `object: Trait<ordered, named>` without unknowns that an impl search has found an impl
/// for.
type SolvedBound = (Type, TraitId, Vec<Type>, Vec<NamedType>);

/// Modes that affect the behavior of [`NodeInterner::try_lookup_trait_implementation`].
#[derive(Clone, Copy)]
pub(crate) enum TraitLookupMode {
    /// Does not look up implementations for bindable object types, but matches any [`TraitImplKind`].
    /// Answers only once the answer can't change: an impl whose where clause can't be checked yet
    /// (it needs a type that isn't known yet) might still apply, so a single other matching impl
    /// is not chosen over it.
    Default,
    /// Looks up implementation for bindable object types, and matches only [`TraitImplKind::Assumed`].
    /// The returned bindings are not expected to be applied.
    SelfAssumedOnly,
    /// Like [Default](TraitLookupMode::Default), but skips every [`TraitImplKind::Assumed`] impl.
    /// Used to normalize an associated-type projection over a rigid object type: the real impl
    /// is the single ground truth, and any `where` clause hypothesis for the same type is
    /// discharged by it rather than competing with it.
    IgnoreAssumed,
}

/// The assumed impls [`NodeInterner::take_assumed_trait_implementations`] set aside.
pub struct AssumedTraitImplementations(HashMap<TraitId, Vec<(Type, TraitImplKind)>>);

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

    /// Reserves the next [ImplId] for an inherent (non-trait) `impl` block.
    pub fn next_impl_id(&mut self) -> ImplId {
        let next_id = self.next_impl_id;
        self.next_impl_id += 1;
        ImplId(next_id)
    }

    /// Records a resolved inherent `impl` block under its previously-reserved [ImplId].
    pub fn add_impl(&mut self, id: ImplId, impl_: Impl) {
        self.impls.insert(id, impl_);
    }

    pub fn get_impl(&self, id: ImplId) -> &Impl {
        &self.impls[&id]
    }

    pub fn get_impls_in_crate(&self, crate_id: CrateId) -> HashSet<ImplId> {
        self.impls
            .iter()
            .filter_map(|(id, impl_)| (impl_.crate_id == crate_id).then_some(*id))
            .collect()
    }

    /// Adds an "assumed" trait implementation to the currently known trait implementations.
    /// Unlike normal trait implementations, these are only assumed to exist. They often correspond
    /// to `where` clauses in functions where we assume there is some `T: Eq` even though we do
    /// not yet know T. For these cases, we store an impl here so that we assume they exist and
    /// can resolve them. They are then later verified when the function is called, and linked
    /// properly after being monomorphized to the correct variant.
    ///
    /// Returns Ok(true) on success, or Ok(false) if there is already an overlapping impl in scope.
    pub fn add_assumed_trait_implementation(
        &mut self,
        object_type: Type,
        trait_id: TraitId,
        trait_generics: TraitGenerics,
    ) -> Result<bool, ImplSearchErrorKind> {
        // Make sure there are no overlapping impls
        let existing = self.try_lookup_trait_implementation(
            &object_type,
            trait_id,
            &trait_generics.ordered,
            &trait_generics.named,
            TraitLookupMode::Default,
        );
        match existing {
            // No impl applies yet, including when the trait has none at all (one declared
            // later in the program).
            Err(ImplSearchErrorKind::NoMatching(_))
            | Err(ImplSearchErrorKind::NoImplFound(_))
            | Err(ImplSearchErrorKind::TypeAnnotationsNeededOnObjectType) => {
                // The incoming bound may describe an associated type inherited from a parent (or
                // grandparent) trait, supplied as a fresh variable that does not unify with the
                // rigid `<T as Trait>::Assoc` placeholder already stored on an existing assumed
                // entry (hence the lookup found no match). Rather than register a second,
                // divergent impl for the same object type, reconcile the associated types into
                // the existing entry. Only entries with matching ordered generics are merged, so
                // genuinely distinct bounds are never collapsed.
                if self.merge_named_generics_into_assumed_impl(
                    trait_id,
                    &object_type,
                    &trait_generics.ordered,
                    &trait_generics.named,
                ) {
                    return Ok(true);
                }
                self.mark_trait_as_having_impls(trait_id);
                let entries = self.assumed_trait_implementations.entry(trait_id).or_default();
                entries.push((
                    object_type.clone(),
                    TraitImplKind::Assumed { object_type, trait_generics },
                ));
                Ok(true)
            }
            Ok(_) => {
                // A parent trait constraint may provide fresh variables for associated types we
                // already have an assumed entry for; reconcile them so both share one binding.
                // As above, only an entry with matching ordered generics describes the same
                // bound: `T: Foo<u8>` and `T: Foo<u32>` are separate bounds that each keep their
                // own associated types.
                if self.merge_named_generics_into_assumed_impl(
                    trait_id,
                    &object_type,
                    &trait_generics.ordered,
                    &trait_generics.named,
                ) {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Err(
                error @ (ImplSearchErrorKind::MultipleMatching(_)
                | ImplSearchErrorKind::RecursionLimitReached),
            ) => Err(error),
        }
    }

    /// Reconcile fresh associated-type bindings with an existing assumed impl entry for
    /// `trait_id` whose object type equals `object_type`, so that a single assumed impl is kept
    /// per object type and trait. This keeps the fresh per-function variables used to resolve
    /// `T::Assoc` syntax in sync with the variables stored in the assumed impl used during
    /// method-call resolution.
    ///
    /// For each associated type, the incoming binding overwrites the existing one *unless* doing
    /// so would replace a usable binding with a rigid `<T as Trait>::Assoc` placeholder — such a
    /// placeholder carries no extra information and would otherwise leave the associated type
    /// unresolvable. This keeps the most-resolved binding regardless of registration order.
    ///
    /// A bound is identified by its object type *and* its ordered generics, so only an entry
    /// whose ordered generics equal `ordered` is merged: `T: Foo<u8>` and `T: Foo<u32>` are
    /// distinct bounds on one trait and must not be collapsed into each other.
    ///
    /// Returns true if such an entry was found and reconciled; the caller then avoids registering
    /// a duplicate entry.
    fn merge_named_generics_into_assumed_impl(
        &mut self,
        trait_id: TraitId,
        object_type: &Type,
        ordered: &[Type],
        named: &[NamedType],
    ) -> bool {
        if named.is_empty() {
            return false;
        }
        let Some(entries) = self.assumed_trait_implementations.get_mut(&trait_id) else {
            return false;
        };
        for (_, impl_kind) in entries.iter_mut() {
            if let TraitImplKind::Assumed {
                object_type: existing_obj,
                trait_generics: existing_generics,
            } = impl_kind
                && *existing_obj == *object_type
            {
                if existing_generics.ordered.as_slice() != ordered {
                    // A genuinely distinct bound (different ordered generics) — keep looking
                    // rather than collapsing it into this entry.
                    continue;
                }

                for new_named in named {
                    for existing_named in &mut existing_generics.named {
                        if existing_named.name.as_str() != new_named.name.as_str() {
                            continue;
                        }
                        // A bare type variable is the placeholder a supertrait bound stores for
                        // an associated item it leaves out; it carries no information, so a
                        // rigid `<T as Trait>::Assoc` replaces it.
                        let would_downgrade_to_placeholder =
                            matches!(new_named.typ, Type::NamedGeneric(_))
                                && !matches!(
                                    existing_named.typ,
                                    Type::NamedGeneric(_) | Type::TypeVariable(_)
                                );
                        if !would_downgrade_to_placeholder {
                            existing_named.typ = new_named.typ.clone();
                        }
                    }
                }
                return true;
            }
        }
        false
    }

    /// Adds a prepared trait implementation.
    ///
    /// This is called before the normal implementation is ready, so we can look up the
    /// associated types while defining the meta-data for other functions and trait methods.
    pub fn add_prepared_trait_implementation(
        &mut self,
        object_type: Type,
        trait_id: TraitId,
        impl_id: TraitImplId,
        impl_generics: GenericTypeVars,
        where_clause: Vec<TraitConstraint>,
        location: Location,
    ) {
        self.trait_impl_where_clauses.insert(impl_id, where_clause);

        if matches!(object_type, Type::Error) {
            // If we stored a prepared impl for Error, it would later unify with anything,
            // leading to potentially unexpected duplications with the real prepared impl.
            return;
        }

        // Stored generalized over the impl's generics, as a finished impl is, so that each lookup
        // instantiates them.
        let generalized_object_type = Type::Forall(impl_generics, Box::new(object_type));
        let entries = self.trait_implementation_map.entry(trait_id).or_default();
        entries.push((generalized_object_type, TraitImplKind::Prepared(impl_id, location)));
    }

    /// Adds a trait implementation to the list of known implementations.
    pub fn add_trait_implementation(
        &mut self,
        object_type: Type,
        trait_id: TraitId,
        impl_id: TraitImplId,
        trait_impl: Shared<TraitImpl>,
        location: Location,
    ) -> Result<Result<(), Location>, CompilationError> {
        self.trait_implementations.insert(impl_id, trait_impl.clone());
        self.trait_implementations_by_trait_id.entry(trait_id).or_default().push(impl_id);

        // Avoid adding error types to impls since they'll conflict with every other type.
        // We don't need to return an error since we expect an error to already be issued when
        // the error type is created.
        if object_type == Type::Error {
            return Err(TypeCheckError::expecting_other_error(
                "collect_trait_impl: missing trait type",
                location,
            )
            .into());
        }

        let impl_generics = trait_impl.borrow().generics.clone();
        let (instantiated_object_type, substitutions) =
            object_type.substitute_type_vars_with_fresh_type_vars(&impl_generics, self);

        // Associated types don't take part: `impl Foo<Bar = u32> for Baz` overlaps
        // `impl Foo<Bar = i32> for Baz`.
        let ordered_generics = vecmap(&self.get_trait_generics_for_impl(impl_id).ordered, |typ| {
            typ.substitute(&substitutions)
        });

        // Remove any prepared implementation for this impl.
        self.remove_prepared_trait_implementation(trait_id, impl_id);

        let mut where_clause = self.get_where_clause_for_impl(impl_id).to_vec();
        for constraint in &mut where_clause {
            constraint.apply_instantiation_bindings(&substitutions);
        }
        if let Some(existing) = self.overlapping_impl(
            &instantiated_object_type,
            trait_id,
            &ordered_generics,
            &where_clause,
        ) {
            let existing_impl = self.get_trait_implementation(existing);
            let existing_impl = existing_impl.borrow();
            return Ok(Err(existing_impl.ident.location()));
        }

        // The object type is generalized so that a generic impl will apply
        // to any type T, rather than just the generic type named T.
        for method in &trait_impl.borrow().methods {
            let method_name = self.function_name(method).to_owned();
            self.add_method(&object_type, method_name, *method, Some(impl_id))?;
        }

        let generalized_object_type = object_type.generalize_from_substitutions(substitutions);

        let entries = self.trait_implementation_map.entry(trait_id).or_default();
        entries.push((generalized_object_type, TraitImplKind::Normal(impl_id)));

        Ok(Ok(()))
    }

    /// An impl already added for `trait_id` that applies to some type the impl with header
    /// `object_type: trait_id<trait_generics>` and where clause `where_clause` (both instantiated
    /// with unknowns for its generics) applies to as well.
    ///
    /// Two impls apply to the same type when their headers unify and both where clauses can hold
    /// for it together: `impl<T> Foo for T where T: A` and `impl<T> Foo for T where T: B`
    /// overlap if some type implements both `A` and `B`.
    fn overlapping_impl(
        &self,
        object_type: &Type,
        trait_id: TraitId,
        trait_generics: &[Type],
        where_clause: &[TraitConstraint],
    ) -> Option<TraitImplId> {
        let impls = self.trait_implementation_map.get(&trait_id)?;
        impls.iter().find_map(|(existing_type, impl_kind)| {
            let TraitImplKind::Normal(existing) = impl_kind else {
                return None;
            };
            let mut bindings = TypeBindings::default();
            let (_, instantiation_bindings) = self.unify_with_impl_header(
                object_type,
                trait_generics,
                existing_type,
                impl_kind,
                &mut bindings,
            )?;
            let mut obligations = where_clause.to_vec();
            obligations.extend(self.instantiated_where_clause(*existing, &instantiation_bindings));
            let budget = Cell::new(OVERLAP_SEARCH_BUDGET);
            self.constraints_may_hold_together(
                &obligations,
                &bindings,
                IMPL_SEARCH_RECURSION_LIMIT,
                &budget,
            )
            .then_some(*existing)
        })
    }

    /// Instantiates the impl entry `impl_type`/`impl_kind` with unknowns for its generics and
    /// unifies its object type and trait arguments with `object_type` and `trait_generics`,
    /// recording the bindings in `bindings`. Returns the impl's instantiated object type and its
    /// instantiation bindings, or `None` if its header doesn't match.
    ///
    /// The trait's associated types are left to the caller, as each has its own rule: the impl
    /// search requires every one it is given to match the impl's, `constraints_may_hold_together`
    /// skips one the impl doesn't name, and the overlap check ignores them (impls that differ only
    /// in an associated type overlap).
    fn unify_with_impl_header(
        &self,
        object_type: &Type,
        trait_generics: &[Type],
        impl_type: &Type,
        impl_kind: &TraitImplKind,
        bindings: &mut TypeBindings,
    ) -> Option<(Type, TypeBindings)> {
        let (impl_type, instantiation_bindings) = impl_type.instantiate(self);
        object_type.try_unify(&impl_type, bindings).ok()?;
        let impl_generics = &self.impl_trait_generics(impl_kind).ordered;
        let generics_unify =
            trait_generics.iter().zip_eq(impl_generics).all(|(generic, impl_generic)| {
                generic
                    .try_unify(&impl_generic.substitute(&instantiation_bindings), bindings)
                    .is_ok()
            });
        generics_unify.then_some((impl_type, instantiation_bindings))
    }

    /// The where clause of `impl_id`, in terms of the unknowns `instantiation_bindings` gives its
    /// generics.
    fn instantiated_where_clause(
        &self,
        impl_id: TraitImplId,
        instantiation_bindings: &TypeBindings,
    ) -> Vec<TraitConstraint> {
        let where_clause = self.trait_impl_where_clauses.get(&impl_id).into_iter().flatten();
        vecmap(where_clause, |constraint| {
            let mut constraint = constraint.clone();
            constraint.apply_instantiation_bindings(instantiation_bindings);
            constraint
        })
    }

    /// The trait arguments the impl entry `impl_kind` implements its trait at.
    fn impl_trait_generics<'a>(&'a self, impl_kind: &'a TraitImplKind) -> &'a TraitGenerics {
        match impl_kind {
            TraitImplKind::Normal(id) | TraitImplKind::Prepared(id, _) => {
                self.get_trait_generics_for_impl(*id)
            }
            TraitImplKind::Assumed { trait_generics, .. } => trait_generics,
        }
    }

    /// Whether some choice of the unknowns in `constraints` (beyond those `bindings` decides)
    /// gives every constraint an impl, searching the declared impls for each one in turn.
    /// Gives up, answering no, past `recursion_limit` nested where clauses or once `budget`
    /// impls have been tried.
    fn constraints_may_hold_together(
        &self,
        constraints: &[TraitConstraint],
        bindings: &TypeBindings,
        recursion_limit: u32,
        budget: &Cell<u32>,
    ) -> bool {
        if constraints.is_empty() {
            return true;
        }
        if recursion_limit == 0 {
            return false;
        }
        // The constraint with the fewest impls to try, preferring one on a known type: a
        // constraint on an unknown type is tried against every impl of its trait, each of which
        // may nest further.
        let impl_count = |constraint: &TraitConstraint| {
            self.trait_implementation_map.get(&constraint.trait_bound.trait_id).map_or(0, Vec::len)
        };
        let index = (0..constraints.len())
            .min_by_key(|index| {
                let constraint = &constraints[*index];
                let on_unknown = constraint.typ.substitute(bindings).is_bindable();
                (on_unknown, impl_count(constraint))
            })
            .expect("constraints is not empty");
        let constraint = &constraints[index];
        let rest = constraints[..index].iter().chain(&constraints[index + 1..]);
        let bound = &constraint.trait_bound;
        let Some(impls) = self.trait_implementation_map.get(&bound.trait_id) else {
            return false;
        };
        impls.iter().any(|(impl_type, impl_kind)| {
            if budget.get() == 0 {
                return false;
            }
            budget.set(budget.get() - 1);
            let (TraitImplKind::Normal(impl_id) | TraitImplKind::Prepared(impl_id, _)) = impl_kind
            else {
                return false;
            };
            let mut candidate = bindings.clone();
            let Some((_, instantiation_bindings)) = self.unify_with_impl_header(
                &constraint.typ,
                &bound.trait_generics.ordered,
                impl_type,
                impl_kind,
                &mut candidate,
            ) else {
                return false;
            };
            let impl_generics = self.get_trait_generics_for_impl(*impl_id);
            let named_unify = bound.trait_generics.named.iter().all(|named| {
                impl_generics.named.iter().find(|other| other.name == named.name).is_none_or(
                    |other| {
                        let other = other.typ.substitute(&instantiation_bindings);
                        named.typ.try_unify(&other, &mut candidate).is_ok()
                    },
                )
            });
            if !named_unify {
                return false;
            }
            let mut obligations = self.instantiated_where_clause(*impl_id, &instantiation_bindings);
            obligations.extend(rest.clone().cloned());
            self.constraints_may_hold_together(
                &obligations,
                &candidate,
                recursion_limit - 1,
                budget,
            )
        })
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
    ///
    /// The bindings the search itself produced are committed to the shared HIR, which is what
    /// type checking wants and what a pass over an already-elaborated program does not. Such a
    /// pass should call [`Self::try_lookup_trait_implementation`], which hands those bindings back
    /// instead.
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
            TraitLookupMode::Default,
        )?;

        Type::apply_type_bindings(bindings);
        Ok((impl_kind, instantiation_bindings))
    }

    /// Like [`Self::lookup_trait_implementation`] but ignores [`TraitImplKind::Assumed`] impls.
    /// Intended for normalizing an associated-type projection whose object type is rigid,
    /// where a `where` clause hypothesis must not compete with the real impl that discharges it.
    pub(crate) fn lookup_trait_implementation_ignoring_assumed(
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
            TraitLookupMode::IgnoreAssumed,
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
        trait_lookup_mode: TraitLookupMode,
    ) -> Result<(TraitImplKind, TypeBindings, TypeBindings), ImplSearchErrorKind> {
        let mut bindings = TypeBindings::default();
        let mut solved = HashSet::default();
        let (impl_kind, instantiation_bindings) = self.lookup_trait_implementation_helper(
            object_type,
            trait_id,
            trait_generics,
            trait_associated_types,
            &mut bindings,
            trait_lookup_mode,
            IMPL_SEARCH_RECURSION_LIMIT,
            &mut solved,
        )?;
        Ok((impl_kind, bindings, instantiation_bindings))
    }

    /// Remove the [`TraitImplKind::Prepared`] entry for the given impl, if one exists.
    pub(crate) fn remove_prepared_trait_implementation(
        &mut self,
        trait_id: TraitId,
        impl_id: TraitImplId,
    ) {
        let entries = self.trait_implementation_map.entry(trait_id).or_default();
        entries
            .retain(|(_, kind)| !matches!(kind, TraitImplKind::Prepared(id, _) if *id == impl_id));
    }

    /// Returns the trait implementation if found along with the instantiation bindings for
    /// instantiating that trait impl. Note that this is separate from the passed-in `TypeBindings`
    /// which can be bound via `Type::apply_type_bindings` if needed. Instantiation bindings should
    /// be stored as such but not bound, lest the original named generics in trait impls get bound
    /// over.
    ///
    /// On error returns either:
    /// - 1+ failing trait constraints, including the original.
    ///   Each constraint after the first represents a `where` clause that was followed.
    /// - 0 trait constraints indicating type annotations are needed to choose an impl.
    ///
    /// `solved` holds the where-clause bounds without unknowns this search has already found an
    /// impl for, so that a bound reached through several impls is solved once.
    #[allow(clippy::too_many_arguments)]
    fn lookup_trait_implementation_helper(
        &self,
        object_type: &Type,
        trait_id: TraitId,
        trait_generics: &[Type],
        trait_associated_types: &[NamedType],
        type_bindings: &mut TypeBindings,
        mode: TraitLookupMode,
        recursion_limit: u32,
        solved: &mut HashSet<SolvedBound>,
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

        // Prevent infinite recursion when looking for impls
        if recursion_limit == 0 {
            return Err(ImplSearchErrorKind::RecursionLimitReached);
        }

        // If the object type isn't known, just return an error saying type annotations are needed.
        // However if we are looking up a parent trait constraint on self inside a trait definition,
        // we must allow the assumed implementation we added on the self type variable to be found.
        let object_type = object_type.substitute(type_bindings);
        let is_bindable = object_type.is_bindable();

        // An integer literal's unknown type only stands for an integer type, so its impls are
        // candidates, as in Rust.
        if is_bindable
            && matches!(mode, TraitLookupMode::Default | TraitLookupMode::IgnoreAssumed)
            && !is_integer_type_variable(&object_type)
        {
            return Err(ImplSearchErrorKind::TypeAnnotationsNeededOnObjectType);
        }

        let impls = self
            .trait_implementation_map
            .get(&trait_id)
            .ok_or_else(|| ImplSearchErrorKind::NoImplFound(vec![make_constraint()]))?;
        let assumed_impls = self.assumed_trait_implementations.get(&trait_id);
        let impls = impls.iter().chain(assumed_impls.into_iter().flatten());

        let mut matching_impls = Vec::new();
        let mut where_clause_error = None;
        let mut where_clause_undecided = false;

        for (existing_object_type, impl_kind) in impls {
            let skip = match mode {
                TraitLookupMode::Default => false,
                TraitLookupMode::SelfAssumedOnly => {
                    !matches!(impl_kind, TraitImplKind::Assumed { .. })
                }
                TraitLookupMode::IgnoreAssumed => {
                    matches!(impl_kind, TraitImplKind::Assumed { .. })
                }
            };
            if skip {
                continue;
            }

            let mut fresh_bindings = type_bindings.clone();
            let Some((existing_object_type, instantiation_bindings)) = self.unify_with_impl_header(
                &object_type,
                trait_generics,
                existing_object_type,
                impl_kind,
                &mut fresh_bindings,
            ) else {
                continue;
            };
            let impl_trait_generics = self.impl_trait_generics(impl_kind).clone();

            let where_clause = match impl_kind {
                TraitImplKind::Normal(impl_id) | TraitImplKind::Prepared(impl_id, _) => {
                    self.trait_impl_where_clauses.get(impl_id)
                }
                TraitImplKind::Assumed { .. } => None,
            };
            if let Some(where_clause) = where_clause
                && let Err(error) = self.validate_where_clause(
                    where_clause,
                    &mut fresh_bindings,
                    &instantiation_bindings,
                    recursion_limit,
                    solved,
                )
            {
                if error.1.is_undecided() {
                    where_clause_undecided = true;
                }
                // Only keep the first errors we get from a failing where clause
                if where_clause_error.is_none() {
                    where_clause_error = Some(error);
                }
                continue;
            }

            // Match associated types by name, not position
            let associated_types_unify = trait_associated_types.iter().all(|trait_generic| {
                // Find the matching impl generic by name
                let Some(named_impl_generic) = impl_trait_generics
                    .named
                    .iter()
                    .find(|impl_g| impl_g.name.as_str() == trait_generic.name.as_str())
                else {
                    // If the impl doesn't have this associated type, it doesn't match
                    return false;
                };

                let impl_generic = named_impl_generic.typ.substitute(&instantiation_bindings);

                trait_generic.typ.try_unify(&impl_generic, &mut fresh_bindings).is_ok()
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

        // An impl whose where clause can't be decided yet might still apply, so neither a single
        // other match nor no match at all is an answer yet.
        if matching_impls.len() <= 1 && where_clause_undecided {
            Err(ImplSearchErrorKind::TypeAnnotationsNeededOnObjectType)
        } else if matching_impls.len() == 1 {
            let (impl_, fresh_bindings, instantiation_bindings, _) = matching_impls.pop().unwrap();
            *type_bindings = fresh_bindings;
            Ok((impl_, instantiation_bindings))
        } else if is_bindable {
            Err(ImplSearchErrorKind::TypeAnnotationsNeededOnObjectType)
        } else if matching_impls.is_empty() {
            let mut errors = match where_clause_error {
                Some((_, ImplSearchErrorKind::NoImplFound(errors))) => errors,
                Some((constraint, _other)) => vec![constraint],
                None => vec![],
            };
            errors.push(make_constraint());
            Err(ImplSearchErrorKind::NoMatching(errors))
        } else {
            // An assumed impl is a `where` clause hypothesis that an impl exists for the object
            // type. When the query is rigid (no unbound type variables in the object type or
            // ordered generics) and a real impl also matches, it discharges that hypothesis
            // rather than competing with it: by coherence it is the unique ground truth, so
            // prefer it over the assumed candidates.
            //
            // Otherwise the candidates may bind a variable differently - an assumed
            // `Wrapper<T>: Foo` and a real `impl Foo for Wrapper<u32>` both match `Wrapper<_>`.
            // As in Rust, a single assumed candidate wins: the `where` clause is what the
            // function was written against, so `_` becomes `T`. Several assumed candidates are
            // ambiguous.
            let query_is_rigid = !object_type.contains_unbound_type_variable()
                && trait_generics.iter().all(|generic| {
                    !generic.substitute(type_bindings).contains_unbound_type_variable()
                });

            // A `where` clause is what the function was written against, so a single one that
            // matches decides the query, as in Rust. For a rigid query it only does when it
            // names the function's generics (`where u8: Tr<Out = T>`); one naming none
            // (`where u8: Tr`) only repeats what the single real impl says.
            let decides = |kind: &TraitImplKind| match kind {
                TraitImplKind::Assumed { trait_generics, .. } => {
                    !query_is_rigid
                        || trait_generics
                            .named
                            .iter()
                            .any(|named| mentions_generic_parameter(&named.typ))
                }
                TraitImplKind::Normal(_) | TraitImplKind::Prepared(..) => false,
            };
            let deciding: Vec<usize> =
                matching_impls.iter().positions(|(kind, ..)| decides(kind)).collect();
            let real_impls: Vec<usize> = matching_impls
                .iter()
                .positions(|(kind, ..)| !matches!(kind, TraitImplKind::Assumed { .. }))
                .collect();
            let selected = match (&deciding[..], &real_impls[..]) {
                ([index], _) => Some(*index),
                ([], [index]) if query_is_rigid => Some(*index),
                _ => None,
            };
            if let Some(index) = selected {
                let (impl_, fresh_bindings, instantiation_bindings, _) =
                    matching_impls.swap_remove(index);
                *type_bindings = fresh_bindings;
                Ok((impl_, instantiation_bindings))
            } else {
                let impls = vecmap(matching_impls, |(_, _, _, constraint)| {
                    let name = &self.get_trait(constraint.trait_bound.trait_id).name;
                    format!("{}: {name}{}", constraint.typ, constraint.trait_bound.trait_generics)
                });
                Err(ImplSearchErrorKind::MultipleMatching(impls))
            }
        }
    }

    /// The bound declared at `location` in some impl's where clause, as written.
    pub(crate) fn declared_impl_where_clause_bound(
        &self,
        location: Location,
    ) -> Option<&TraitConstraint> {
        self.trait_impl_where_clauses
            .values()
            .flatten()
            .find(|bound| bound.trait_bound.location == location)
    }

    /// Whether the where clause of the impl `method` belongs to can hold for a receiver or path
    /// of type `typ`: false only when one of its bounds is known to fail for it. A method of an
    /// impl that can't apply isn't a candidate for a call.
    pub(crate) fn method_impl_where_clause_may_hold(&self, method: FuncId, typ: &Type) -> bool {
        let meta = self.function_meta(&method);
        let (impl_type, generics, where_clause) = if let Some(impl_id) = meta.trait_impl {
            let Some(trait_impl) = self.try_get_trait_implementation(impl_id) else {
                return true;
            };
            let Some(where_clause) = self.trait_impl_where_clauses.get(&impl_id) else {
                return true;
            };
            let trait_impl = trait_impl.borrow();
            (trait_impl.typ.clone(), trait_impl.generics.clone(), where_clause.clone())
        } else if let Some(impl_id) = meta.impl_id {
            let impl_ = self.get_impl(impl_id);
            let generics = vecmap(&impl_.generics, |generic| generic.type_var.clone());
            (impl_.typ.clone(), generics, impl_.where_clause.clone())
        } else {
            return true;
        };
        if where_clause.is_empty() {
            return true;
        }
        let (impl_type, instantiation_bindings) =
            impl_type.substitute_type_vars_with_fresh_type_vars(&generics, self);
        let mut bindings = TypeBindings::default();
        if !impl_type.try_unify_receiver(typ, &mut bindings) {
            return true;
        }
        match self.validate_where_clause(
            &where_clause,
            &mut bindings,
            &instantiation_bindings,
            IMPL_SEARCH_RECURSION_LIMIT,
            &mut HashSet::default(),
        ) {
            Ok(()) => true,
            Err((_, error)) => error.is_undecided(),
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
        solved: &mut HashSet<SolvedBound>,
    ) -> Result<(), (TraitConstraint, ImplSearchErrorKind)> {
        // An undecided bound doesn't hide a later one that fails: the where clause fails then.
        let mut undecided = None;
        for constraint in where_clause {
            let constraint_type =
                constraint.typ.substitute(instantiation_bindings).substitute(type_bindings);

            let trait_generics =
                vecmap(&constraint.trait_bound.trait_generics.ordered, |generic| {
                    generic.substitute(instantiation_bindings).substitute(type_bindings)
                });

            let trait_associated_types =
                vecmap(&constraint.trait_bound.trait_generics.named, |generic| {
                    let typ = generic.typ.substitute(instantiation_bindings);
                    NamedType { name: generic.name.clone(), typ: typ.substitute(type_bindings) }
                });

            let key = (
                constraint_type.clone(),
                constraint.trait_bound.trait_id,
                trait_generics.clone(),
                trait_associated_types.clone(),
            );
            let is_rigid = !constraint_type.contains_unbound_type_variable()
                && trait_generics.iter().all(|typ| !typ.contains_unbound_type_variable())
                && trait_associated_types
                    .iter()
                    .all(|named| !named.typ.contains_unbound_type_variable());
            if is_rigid && solved.contains(&key) {
                continue;
            }

            // We can ignore any associated types on the constraint since those should not affect
            // which impl we choose.
            let result = self
                .lookup_trait_implementation_helper(
                    &constraint_type,
                    constraint.trait_bound.trait_id,
                    &trait_generics,
                    &trait_associated_types,
                    // Use a fresh set of type bindings here since the constraint_type originates from
                    // our impl list, which we don't want to bind to.
                    type_bindings,
                    TraitLookupMode::Default,
                    recursion_limit - 1,
                    solved,
                )
                .map_err(|error| {
                    let mut constraint = constraint.clone();
                    constraint.typ = constraint_type.clone();
                    constraint.trait_bound.trait_generics.ordered = trait_generics.clone();
                    constraint.trait_bound.trait_generics.named = trait_associated_types.clone();
                    (constraint, error)
                });
            match result {
                Err(error) if error.1.is_undecided() => {
                    undecided.get_or_insert(error);
                    continue;
                }
                Err(error) => return Err(error),
                Ok(_) => (),
            }
            if is_rigid {
                solved.insert(key);
            }
        }

        undecided.map_or(Ok(()), Err)
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

    /// Gives `trait_id` an entry in `trait_implementation_map`. Lookup reports a trait without
    /// one as [`ImplSearchErrorKind::NoImplFound`], which is for a trait nothing was ever
    /// registered for, assumed impls included.
    fn mark_trait_as_having_impls(&mut self, trait_id: TraitId) {
        self.trait_implementation_map.entry(trait_id).or_default();
    }

    /// Removes every assumed impl, returning them for [`Self::restore_assumed_trait_implementations`].
    pub fn take_assumed_trait_implementations(&mut self) -> AssumedTraitImplementations {
        AssumedTraitImplementations(std::mem::take(&mut self.assumed_trait_implementations))
    }

    /// Puts back the assumed impls [`Self::take_assumed_trait_implementations`] removed.
    pub fn restore_assumed_trait_implementations(&mut self, assumed: AssumedTraitImplementations) {
        self.assumed_trait_implementations = assumed.0;
    }

    /// Removes all `TraitImplKind::Assumed` from the list of known impls for the given trait, and
    /// for every trait that a bound on it implies: its parent traits and the traits bounding its
    /// associated types. These are the assumed impls `Elaborator::add_trait_bound_to_scope` adds
    /// alongside the bound itself.
    pub fn remove_assumed_trait_implementations_for_trait(&mut self, trait_id: TraitId) {
        self.remove_assumed_trait_implementations_for_trait_and_implied(
            trait_id,
            &mut HashSet::new(),
        );
    }

    fn remove_assumed_trait_implementations_for_trait_and_implied(
        &mut self,
        trait_id: TraitId,
        visited_trait_ids: &mut HashSet<TraitId>,
    ) {
        // Avoid looping forever in case there are cycles
        if !visited_trait_ids.insert(trait_id) {
            return;
        }
        self.mark_trait_as_having_impls(trait_id);
        self.assumed_trait_implementations.remove(&trait_id);

        // Also remove assumed implementations for the traits a bound on this one implies
        let implied_trait_ids: Vec<TraitId> = self
            .try_get_trait(trait_id)
            .map(|the_trait| {
                let parents = the_trait.parent_bounds().map(|bound| bound.trait_id());
                let associated_type_bounds =
                    the_trait.associated_type_bounds.values().flatten().map(|bound| bound.trait_id);
                parents.chain(associated_type_bounds).collect()
            })
            .unwrap_or_default();
        for implied_trait_id in implied_trait_ids {
            self.remove_assumed_trait_implementations_for_trait_and_implied(
                implied_trait_id,
                visited_trait_ids,
            );
        }
    }
}

/// Whether `typ` mentions a generic parameter (`T`, not an associated item like
/// `<T as Foo>::Out`).
fn mentions_generic_parameter(typ: &Type) -> bool {
    let mut found = false;
    typ.visit(&mut |typ| {
        found |= matches!(typ, Type::NamedGeneric(generic) if !generic.is_associated());
        !found
    });
    found
}

/// Whether `typ` is an integer literal's still unknown type.
fn is_integer_type_variable(typ: &Type) -> bool {
    match typ.follow_bindings() {
        Type::TypeVariable(variable) => {
            matches!(variable.kind().as_ref(), Kind::IntegerOrField | Kind::Integer)
        }
        _ => false,
    }
}
