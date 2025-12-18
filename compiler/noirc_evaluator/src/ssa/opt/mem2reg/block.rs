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
    /// Maps a ValueId to the Expression it represents.
    /// Multiple ValueIds can map to the same Expression, e.g.
    /// dereferences to the same allocation.
    pub(super) expressions: im::OrdMap<ValueId, Expression>,

    /// Each expression is tracked as to how many aliases it
    /// may have. If there is only 1, we can attempt to optimize
    /// out any known loads to that alias. Note that "alias" here
    /// includes the original reference as well.
    pub(super) aliases: im::OrdMap<Expression, AliasSet>,

    /// Each allocate instruction result (and some reference block parameters)
    /// will map to a Reference value which tracks whether the last value stored
    /// to the reference is known.
    pub(super) references: im::OrdMap<ValueId, ValueId>,

    /// The last instance of a `Store` instruction to each address in this block
    pub(super) last_stores: im::OrdMap<ValueId, InstructionId>,

    // The last instance of a `Load` instruction to each address in this block
    pub(super) last_loads: im::OrdMap<ValueId, InstructionId>,
}

/// An `Expression` here is used to represent a canonical key
/// into the aliases map since otherwise two dereferences of the
/// same address will be given different ValueIds.
#[derive(Debug, Copy, Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub(super) enum Expression {
    Dereference(ValueId),
    ArrayElement(ValueId),
    Other(ValueId),
}

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

    fn set_value(&mut self, address: ValueId, value: Option<ValueId>) {
        let expression = self.expressions.entry(address).or_insert(Expression::Other(address));
        let aliases = self.aliases.entry(*expression).or_default();

        if aliases.is_unknown() {
            // uh-oh, we don't know at all what this reference refers to, could be anything.
            // Now we have to invalidate every reference we know of
            self.invalidate_all_references();
        } else if let Some(alias) = aliases.single_alias() {
            // We always know address points to value
            self.set_reference_value(alias, value);
        } else {
            // More than one alias. We're not sure which it refers to so we have to
            // conservatively invalidate all references it may refer to.
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

    fn invalidate_all_references(&mut self) {
        self.references.clear();
        self.last_stores.clear();
    }

    pub(super) fn unify(mut self, other: &Self) -> Self {
        for (value_id, expression) in &other.expressions {
            if let Some(existing) = self.expressions.get(value_id) {
                assert_eq!(existing, expression, "Expected expressions for {value_id} to be equal");
            } else {
                self.expressions.insert(*value_id, *expression);
            }
        }

        for (expression, new_aliases) in &other.aliases {
            // If nothing would change, then don't call `.entry(...).and_modify(...)` as it involves creating more `Arc`s.
            if let Some(aliases) = self.aliases.get(expression) {
                if !aliases.should_unify(new_aliases) {
                    continue;
                }
            }
            self.aliases
                .entry(*expression)
                .and_modify(|aliases| aliases.unify(new_aliases))
                .or_insert_with(|| new_aliases.clone());
        }

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

        // Keep only the last loads present in both maps, if they map to the same InstructionId
        let mut intersection = im::OrdMap::new();
        for (value_id, instruction) in &other.last_loads {
            if let Some(existing) = self.last_loads.get(value_id) {
                if existing == instruction {
                    intersection.insert(*value_id, *instruction);
                }
            }
        }
        self.last_loads = intersection;

        self
    }

    /// Remember that `result` is the result of dereferencing `address`. This is important to
    /// track aliasing when references are stored within other references.
    pub(super) fn remember_dereference(
        &mut self,
        function: &Function,
        address: ValueId,
        result: ValueId,
    ) {
        if function.dfg.value_is_reference(result) {
            if let Some(known_address) = self.get_known_value(address) {
                self.expressions.insert(result, Expression::Other(known_address));
            } else {
                let expression = Expression::Dereference(address);
                self.expressions.insert(result, expression);
                // No known aliases to insert for this expression... can we find an alias
                // even if we don't have a known address? If not we'll have to invalidate all
                // known references if this reference is ever stored to.
            }
        }
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

        for alias in (*self.get_aliases_for_value(address)).clone().iter() {
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
                Instruction::Store { value, .. } => {
                    self.mark_value_used(*value, function);
                }
                other => {
                    unreachable!("last_store held an id of a non-store instruction: {other:?}")
                }
            }
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

    /// Collect all aliases used by the given value vector
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
        if let Some(expression) = self.expressions.get(&value) {
            if let Some(aliases) = self.aliases.get(expression) {
                return Cow::Borrowed(aliases);
            }
        }

        Cow::Owned(AliasSet::unknown())
    }

    pub(super) fn set_last_load(&mut self, address: ValueId, instruction: InstructionId) {
        let aliases = self.get_aliases_for_value(address);
        if !aliases.is_unknown() {
            self.last_loads.insert(address, instruction);
        }
    }

    pub(super) fn keep_last_load_for(&mut self, address: ValueId) {
        self.last_loads.remove(&address);

        if let Some(expr) = self.expressions.get(&address) {
            if let Some(aliases) = self.aliases.get(expr) {
                for alias in aliases.iter() {
                    self.last_loads.remove(&alias);
                }
            }
        }
    }
}
