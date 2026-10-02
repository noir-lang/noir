//! Items whose resolution is deferred until something asks for them.
//!
//! Function signatures, struct fields and enum variants may all mention items that a comptime
//! attribute has not generated yet, so the elaborator cannot resolve them in source order. Each
//! kind is instead *registered* up front and *resolved* lazily on first read, with whatever is
//! left drained once the attributes have run.
//!
//! All of them follow the same three rules, which is why they share one type:
//!
//! 1. Resolving a key removes it, so a re-entrant read of a key that is mid-resolution finds
//!    nothing and stops rather than looping.
//! 2. A drain takes a snapshot of what to skip. When `elaborate_items` runs recursively - from a
//!    comptime attribute that generated new items - the parent's still-pending keys live in the
//!    same map, and must stay pending for the parent's own drain.
//! 3. Order is by key, so a drain visits entries in `Id` order and inter-item dependencies that
//!    follow declaration order are respected.

use std::collections::{BTreeMap, HashSet};

use super::{enums::UnresolvedEnumVariants, function::UnresolvedFunctionMeta};
use crate::{
    Type,
    ast::Ident,
    elaborator::structs::UnresolvedStructFields,
    hir::def_map::LocalModuleId,
    hir_def::traits::TraitConstraint,
    node_interner::{FuncId, TraitId, TraitImplId, TypeId},
};

/// Items of one kind registered for deferred resolution, keyed so that a lazy read can find one
/// entry and a drain can walk them all in key order.
pub(crate) struct Deferred<K, V>(BTreeMap<K, V>);

impl<K: Ord + Copy, V> Deferred<K, V> {
    /// Records `value` as needing resolution later.
    pub(crate) fn register(&mut self, key: K, value: V) {
        self.0.insert(key, value);
    }

    /// Removes the entry for `key`, returning it if this is the first time it has been claimed.
    ///
    /// Returning `None` is the ordinary case for a key that was already resolved, and is also what
    /// breaks cycles: a key currently being resolved is no longer in the map, so a read of it from
    /// within its own resolution finds nothing instead of recurring.
    pub(crate) fn take(&mut self, key: &K) -> Option<V> {
        self.0.remove(key)
    }

    /// Removes and returns the lowest-keyed entry, for draining in declaration order.
    pub(crate) fn take_first(&mut self) -> Option<V> {
        self.0.pop_first().map(|(_, value)| value)
    }

    /// The keys registered right now, to be passed back to [`Self::keys_except`] later so that a
    /// nested drain leaves the enclosing one's entries alone.
    pub(crate) fn pending(&self) -> HashSet<K>
    where
        K: std::hash::Hash,
    {
        self.0.keys().copied().collect()
    }

    /// The keys to drain: everything registered except `skip`, in key order.
    ///
    /// Collected rather than borrowed because resolving one entry needs `&mut Elaborator`, and may
    /// itself register or resolve entries here.
    pub(crate) fn keys_except(&self, skip: &HashSet<K>) -> Vec<K>
    where
        K: std::hash::Hash,
    {
        self.0.keys().copied().filter(|key| !skip.contains(key)).collect()
    }
}

impl<K, V> Default for Deferred<K, V> {
    fn default() -> Self {
        Self(BTreeMap::new())
    }
}

#[derive(Default)]
pub(super) struct PendingTraitWork {
    /// Trait method declarations registered with deferred meta resolution. These need
    /// their `TraitFunction` records (in `the_trait.methods`) populated after the
    /// post-attribute drain, since the records are filled with stub types up-front so
    /// `collect_trait_impl` can do name-based matching while the real signatures are
    /// still pending. Each entry is `(trait_id, func_id, name)`.
    pub(super) records: Vec<(TraitId, FuncId, Ident)>,

    /// Trait method declarations without a body whose signature still needs the
    /// `elaborate_function` step run after their meta is defined. We can't run it at
    /// registration time because the meta is deferred.
    pub(super) no_body_func_ids: Vec<FuncId>,

    /// Pending where-clause-against-trait checks deferred from `collect_trait_impl_methods`
    /// so they run after the post-attribute drain (when both the trait method's and the
    /// impl method's metas are fully resolved).
    pub(super) where_clause_checks: Vec<PendingWhereClauseCheck>,
}

#[derive(Clone)]
pub(super) struct PendingWhereClauseCheck {
    pub(super) impl_method_func_id: FuncId,
    pub(super) trait_id: TraitId,
    pub(super) impl_id: TraitImplId,
    pub(super) module_id: LocalModuleId,
    pub(super) trait_method_name: String,
    pub(super) trait_impl_where_clause: Vec<TraitConstraint>,
    pub(super) ordered_generics: Vec<Type>,
}

/// Every kind of deferred work the elaborator owns, in one place so that handing it to a child
/// elaborator, or scoping it to one `elaborate_items` call, moves all of it or none of it.
///
/// Globals are deferred the same way but are not here: they live on the [`Context`] and are shared
/// by reference, because a global resolved by one elaborator must stay resolved for the next.
///
/// [`Context`]: crate::hir::Context
#[derive(Default)]
pub(super) struct DeferredItems {
    /// Function signatures registered before their bodies are elaborated. Registering up front and
    /// resolving on first read is what lets functions, globals and trait associated constants
    /// refer to each other regardless of source order.
    pub(super) function_metas: Deferred<FuncId, UnresolvedFunctionMeta>,

    /// Struct fields, whose types may name items produced by comptime attribute expansion.
    pub(super) struct_fields: Deferred<TypeId, UnresolvedStructFields>,

    /// Enum variants, deferred for the same reason as [`Self::struct_fields`].
    pub(super) enum_variants: Deferred<TypeId, UnresolvedEnumVariants>,

    /// Trait bookkeeping that cannot run until the drains above have resolved the metas it reads.
    pub(super) trait_work: PendingTraitWork,
}
