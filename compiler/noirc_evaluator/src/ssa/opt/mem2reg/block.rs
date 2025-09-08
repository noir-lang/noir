use std::borrow::Cow;

use fxhash::FxHashSet;
use noirc_frontend::hir_def::function;
use petgraph::graph::NodeIndex;

use crate::ssa::ir::{
    dfg::DataFlowGraph,
    function::Function,
    instruction::{Instruction, InstructionId},
    value::ValueId,
};

use super::{alias_graph::AliasGraph, alias_set::AliasSet};

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
    pub(super) alias_graph: AliasGraph,

    /// The last instance of a `Store` instruction to each address in this block
    /// This is a set to keep track of stores from previous blocks.
    pub(super) last_stores: im::OrdMap<ValueId, FxHashSet<InstructionId>>,
}

/// A `Container` is stored for any ValueId whose type may contain references.
/// This is currently true for arrays/slices of references and nested references.
/// Currently, a container only holds onto an alias set of references it may contain.
pub(super) type Container = AliasSet;

impl Block {
    /// If the given reference id points to a known value, return the value
    pub(super) fn get_known_value(&self, address: ValueId) -> Option<ValueId> {
        self.alias_graph.get_known_value(address)
    }

    /// If the given address is known, set its value to `value`.
    pub(super) fn set_known_value(&mut self, address: ValueId, value: ValueId) {
        self.alias_graph.store_to_reference(address, value);
    }

    pub(super) fn set_unknown(&mut self, address: ValueId) {
        self.alias_graph.invalidate(address);
    }

    /// Helper to retrieve a container for a given value which contains references.
    /// Compared to `self.containers.get` this returns `unknown` instead of None for values not in the map.
    pub(super) fn get_container(&self, container_id: ValueId) -> Cow<AliasSet> {
        self.containers
            .get(&container_id)
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(AliasSet::unknown()))
    }

    /// Inserts a new reference, aliased to itself only
    pub(super) fn insert_fresh_reference(
        &mut self,
        address: ValueId,
        dfg: &DataFlowGraph,
    ) -> NodeIndex {
        self.alias_graph.new_reference(address, dfg.type_of_value(address))
    }

    /// Inserts a reference derived from the given existing references
    pub(super) fn insert_derived_reference(
        &mut self,
        address: ValueId,
        derived_from: &[ValueId],
        dfg: &DataFlowGraph,
    ) {
        self.alias_graph.new_derived_reference(address, derived_from, dfg.type_of_value(address));
    }

    pub(super) fn insert_derived_reference_from_alias_set(
        &mut self,
        address: ValueId,
        aliases: &AliasSet,
        dfg: &DataFlowGraph,
    ) {
        self.alias_graph.new_derived_reference_from_alias_set(
            address,
            aliases,
            dfg.type_of_value(address),
        );
    }

    pub(super) fn unify(mut self, other: &Self, dfg: &DataFlowGraph) -> Self {
        self.containers = Self::unify_alias_sets(&self.containers, &other.containers);
        self.alias_graph = self.alias_graph.unify(&other.alias_graph, dfg);
        for (address, last_stores) in &other.last_stores {
            self.last_stores.entry(*address).or_default().extend(last_stores.iter().copied());
        }
        self
    }

    /// Unify two maps of alias sets by taking the intersection of both.
    fn unify_alias_sets(
        map1: &im::OrdMap<ValueId, AliasSet>,
        map2: &im::OrdMap<ValueId, AliasSet>,
    ) -> im::OrdMap<ValueId, AliasSet> {
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
    fn keep_last_stores_for(&mut self, address: ValueId, function: &Function, instructions_to_keep: &mut FxHashSet<InstructionId>) {
        self.keep_last_stores(address, function, instructions_to_keep);

        let aliases = self.get_aliases_for_value(address);
        assert!(!aliases.is_unknown());
        for alias in self.get_aliases_for_value(address).iter() {
            self.keep_last_stores(alias, function, instructions_to_keep);
        }
    }

    /// Forget the last store to an address, to remove it from the set of instructions
    /// which are candidates for removal at the end. Also marks the values in the last
    /// store as used, now that we know we want to keep them.
    fn keep_last_stores(&mut self, address: ValueId, function: &Function, instructions_to_keep: &mut FxHashSet<InstructionId>) {
        if let Some(instructions) = self.last_stores.remove(&address) {
            for instruction in instructions {
                instructions_to_keep.insert(instruction);

                // Whenever we decide we want to keep a store instruction, we also need
                // to go through its stored value and mark that used as well.
                match &function.dfg[instruction] {
                    Instruction::Store { value, .. } => self.mark_value_used(*value, function, instructions_to_keep),
                    other => {
                        unreachable!("last_store held an id of a non-store instruction: {other:?}")
                    }
                }
            }
        }
    }

    /// Mark a value (for example an address we loaded) as used by forgetting the last store instruction,
    /// which removes it from the candidates for removal.
    pub(super) fn mark_value_used(&mut self, value: ValueId, function: &Function, instructions_to_keep: &mut FxHashSet<InstructionId>) {
        if function.dfg.type_of_value(value).contains_reference() {
            self.keep_last_stores_for(value, function, instructions_to_keep);

            // We must do a recursive check for arrays since they're the only Values which may contain
            // other ValueIds.
            if let Some(contained) = self.containers.get(&value) {
                assert!(!contained.is_unknown());
                for value in contained.clone().iter() {
                    self.mark_value_used(value, function, instructions_to_keep);
                }
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

    pub(super) fn get_aliases_for_value(&self, value: ValueId) -> AliasSet {
        self.alias_graph.possible_aliases(value)
    }
}
