use std::borrow::Cow;

use crate::ssa::ir::{
    function::Function,
    instruction::{Instruction, InstructionId},
    value::ValueId,
};

use super::alias_set::AliasSet;

/// A `Block` acts as a per-block context for the mem2reg pass.
/// Most notably, it contains the current alias set thought to track each
/// reference value if known, and it contains the expected ReferenceValue
/// for each ValueId. When a block is finished, the final values of these
/// are expected to match the values held by each ValueId at the very end
/// of a block.
#[derive(Debug, Default, Clone)]
pub(super) struct Block {
    /// Map each ValueId which may contain nested references to the references it may contain.
    pub(super) containers: im::OrdMap<ValueId, Container>,

    /// Each alias a ValueId which is a reference type may have. These will always
    /// refer to other values of the same type.
    pub(super) aliases: im::OrdMap<ValueId, AliasSet>,

    /// Each allocate instruction result (and some reference block parameters)
    /// will map to a Reference value which tracks whether the last value stored
    /// to the reference is known.
    pub(super) references: im::OrdMap<ValueId, ValueId>,

    /// The last instance of a `Store` instruction to each address in this block
    pub(super) last_stores: im::OrdMap<ValueId, InstructionId>,
}

/// A `Container` is stored for any ValueId whose type may contain references.
/// This is currently true for arrays/slices of references and nested references.
/// Currently, a container only holds onto an alias set of references it may contain.
pub(super) type Container = AliasSet;

impl Block {
    /// If the given reference id points to a known value, return the value
    pub(super) fn get_known_value(&self, address: ValueId) -> Option<ValueId> {
        self.references.get(&address).copied()
    }

    /// If the given address is known, set its value to `value`.
    pub(super) fn set_known_value(&mut self, address: ValueId, value: ValueId) {
        self.set_value(address, Some(value));
    }

    pub(super) fn set_unknown(&mut self, address: ValueId) {
        self.set_value(address, None);
    }

    /// Helper to retrieve a container for a given value which contains references.
    /// Compared to `self.containers.get` this returns `unknown` instead of None for values not in the map.
    pub(super) fn get_container(&self, container_id: ValueId) -> Cow<AliasSet> {
        self.containers.get(&container_id)
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(AliasSet::unknown()))
    }

    fn set_value(&mut self, address: ValueId, value: Option<ValueId>) {
        let aliases = self.aliases.entry(address).or_default();

        if aliases.is_unknown() {
            // uh-oh, we don't know at all what this reference refers to, could be anything.
            // Now we have to invalidate every reference we know of
            self.invalidate_all_references();
        } else if let Some(alias) = aliases.single_alias() {
            self.set_reference_value(alias, value);
        } else {
            // >= 1 alias. We're not sure which it refers to so we have to conservatively
            // invalidate all references it may refer to. If there is exactly is exactly
            // 1 alias, its value becomes known on the call to `set_reference_value` below.
            for alias in aliases.iter() {
                self.references.remove(&alias);
            }
        }

        // We always know address points to value
        self.set_reference_value(address, value);
    }

    fn set_reference_value(&mut self, address: ValueId, value: Option<ValueId>) {
        if let Some(value) = value {
            self.references.insert(address, value);
        } else {
            self.references.remove(&address);
        }
    }

    /// Inserts a new reference, aliased to itself only
    pub(super) fn insert_fresh_reference(&mut self, address: ValueId) {
        self.aliases.insert(address, AliasSet::known(address));
    }

    fn invalidate_all_references(&mut self) {
        self.references.clear();
        self.last_stores.clear();
    }

    pub(super) fn unify(mut self, other: &Self) -> Self {
        self.containers = Self::unify_alias_sets(&self.containers, &other.containers);
        self.aliases = Self::unify_alias_sets(&self.aliases, &other.aliases);

        // for (expression, new_aliases) in &other.aliases {
        //     // If nothing would change, then don't call `.entry(...).and_modify(...)` as it involves creating more `Arc`s.
        //     if let Some(aliases) = self.aliases.get(expression) {
        //         if !aliases.should_unify(new_aliases) {
        //             continue;
        //         }
        //     }
        //     self.aliases
        //         .entry(*expression)
        //         .and_modify(|aliases| aliases.unify(new_aliases))
        //         .or_insert_with(|| new_aliases.clone());
        // }

        // Keep only the references present in both maps.
        let mut intersection = im::OrdMap::new();
        for (value_id, reference) in &other.references {
            if let Some(existing) = self.references.get(value_id) {
                if reference == existing {
                    intersection.insert(*value_id, *reference);
                }
            }
        }
        self.references = intersection;

        self
    }

    /// Unify two maps of alias sets by taking the intersection of both.
    fn unify_alias_sets(map1: &im::OrdMap<ValueId, AliasSet>, map2: &im::OrdMap<ValueId, AliasSet>) -> im::OrdMap<ValueId, AliasSet> {
        let mut intersection = im::OrdMap::new();
        for (value_id, other_set) in map2 {
            if let Some(existing) = map1.get(value_id) {
                if !existing.is_unknown() && !other_set.is_unknown() {
                    let mut new_set = existing.clone();
                    new_set.unify(other_set);
                    intersection.insert(*value_id, new_set);
                }
            }
        }
        intersection
    }

    /// Forget the last store to an address and all of its aliases, to eliminate them
    /// from the candidates for removal at the end.
    ///
    /// Note that this only affects this block: when we merge blocks we clear the
    /// last stores anyway, we don't inherit them from predecessors, so if one
    /// block stores to an address and a descendant block loads it, this mechanism
    /// does not affect the candidacy of the last store in the predecessor block.
    fn keep_last_stores_for(&mut self, address: ValueId, function: &Function) {
        self.keep_last_store(address, function);

        for alias in self.get_aliases_for_value(address).into_owned().iter() {
            self.keep_last_store(alias, function);
        }
    }

    /// Forget the last store to an address, to remove it from the set of instructions
    /// which are candidates for removal at the end. Also marks the values in the last
    /// store as used, now that we know we want to keep them.
    fn keep_last_store(&mut self, address: ValueId, function: &Function) {
        if let Some(instruction) = self.last_stores.remove(&address) {
            // Whenever we decide we want to keep a store instruction, we also need
            // to go through its stored value and mark that used as well.
            match &function.dfg[instruction] {
                Instruction::Store { value, .. } => self.mark_value_used(*value, function),
                other => {
                    unreachable!("last_store held an id of a non-store instruction: {other:?}")
                }
            }
        }
    }

    /// Adds `alias` as an alias of `reference` and vice-versa.
    pub(super) fn add_alias(&mut self, reference: ValueId, alias: ValueId) {
        if let Some(aliases) = self.aliases.get_mut(&reference) {
            aliases.insert(alias);
        }
        if let Some(aliases) = self.aliases.get_mut(&alias) {
            aliases.insert(reference);
        }
    }

    /// Mark a value (for example an address we loaded) as used by forgetting the last store instruction,
    /// which removes it from the candidates for removal.
    pub(super) fn mark_value_used(&mut self, value: ValueId, function: &Function) {
        self.keep_last_stores_for(value, function);

        // We must do a recursive check for arrays since they're the only Values which may contain
        // other ValueIds.
        if let Some((array, _)) = function.dfg.get_array_constant(value) {
            for value in array {
                self.mark_value_used(value, function);
            }
        }
    }

    /// Collect all aliases used by the given value list
    pub(super) fn collect_all_aliases(
        &self,
        values: impl IntoIterator<Item = ValueId>,
    ) -> AliasSet {
        let mut aliases = AliasSet::known_empty();
        for value in values {
            aliases.unify(&self.get_aliases_for_value(value));
        }
        aliases
    }

    pub(super) fn get_aliases_for_value(&self, value: ValueId) -> Cow<AliasSet> {
        if let Some(aliases) = self.aliases.get(&value) {
            return Cow::Borrowed(aliases);
        }

        Cow::Owned(AliasSet::unknown())
    }
}
