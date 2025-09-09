use crate::ssa::ir::{
    basic_block::BasicBlockId,
    dfg::DataFlowGraph,
    dom::DominatorTree,
    value::{Value, ValueId},
};
use rustc_hash::FxHashMap as HashMap;

/// Records a simplified equivalents of an [`Instruction`][crate::ssa::ir::instruction::Instruction] in the blocks
/// where the constraint that advised the simplification has been encountered.
///
/// For more information see [`ConstraintSimplificationCache`].
#[derive(Default)]
pub(super) struct SimplificationCache {
    /// Simplified expressions where we found them.
    ///
    /// It will always have at least one value because `add` is called
    /// after the default is constructed.
    simplifications: HashMap<BasicBlockId, ValueId>,
}

impl SimplificationCache {
    /// Try to find a simplification in a visible block.
    pub(super) fn get(&self, block: BasicBlockId, dom: &DominatorTree) -> Option<ValueId> {
        // Deterministically walk up the dominator chain until we encounter a block that contains a simplification.
        dom.find_map_dominator(block, |b| self.simplifications.get(&b).cloned())
    }

    /// Add a newly encountered simplification to the cache.
    fn add(&mut self, dfg: &DataFlowGraph, simple: ValueId, block: BasicBlockId) {
        self.simplifications
            .entry(block)
            .and_modify(|existing| {
                // `SimplificationCache` may already hold a simplification in this block
                // so we check whether `simple` is a better simplification than the current one.
                if let Some((_, simpler)) = simplify(dfg, *existing, simple) {
                    *existing = simpler;
                };
            })
            .or_insert(simple);
    }
}

/// HashMap from `(side_effects_enabled_var, Instruction)` to a simplified expression that it can
/// be replaced with based on constraints that testify to their equivalence, stored together
/// with the set of blocks at which this constraint has been observed.
///
/// Only blocks dominated by one in the cache should have access to this information, otherwise
/// we create a sort of time paradox where we replace an instruction with a constant we believe
/// it _should_ equal to, without ever actually producing and asserting the value.
#[derive(Default)]
pub(super) struct ConstraintSimplificationCache(
    HashMap<ValueId, HashMap<ValueId, SimplificationCache>>,
);

impl ConstraintSimplificationCache {
    pub(super) fn cache(
        &mut self,
        dfg: &DataFlowGraph,
        predicate: ValueId,
        block: BasicBlockId,
        lhs: ValueId,
        rhs: ValueId,
    ) {
        if let Some((complex, simple)) = simplify(dfg, lhs, rhs) {
            self.get(predicate).entry(complex).or_default().add(dfg, simple, block);
        }
    }

    /// Get the simplification mapping from complex to simpler instructions,
    /// which all depend on the same side effect condition variable.
    pub(super) fn get(&mut self, predicate: ValueId) -> &mut HashMap<ValueId, SimplificationCache> {
        self.0.entry(predicate).or_default()
    }
}

/// Check if one expression is simpler than the other.
/// Returns `Some((complex, simple))` if a simplification was found, otherwise `None`.
/// Expects the `ValueId`s to be fully resolved.
fn simplify(dfg: &DataFlowGraph, lhs: ValueId, rhs: ValueId) -> Option<(ValueId, ValueId)> {
    match (&dfg[lhs], &dfg[rhs]) {
        // Ignore trivial constraints
        (Value::NumericConstant { .. }, Value::NumericConstant { .. }) => None,

        // Prefer replacing with constants where possible.
        (Value::NumericConstant { .. }, _) => Some((rhs, lhs)),
        (_, Value::NumericConstant { .. }) => Some((lhs, rhs)),
        // Otherwise prefer block parameters over instruction results.
        // This is as block parameters are more likely to be a single witness rather than a full expression.
        (Value::Param { .. }, Value::Instruction { .. }) => Some((rhs, lhs)),
        (Value::Instruction { .. }, Value::Param { .. }) => Some((lhs, rhs)),
        (_, _) => None,
    }
}
