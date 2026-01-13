use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        dom::DominatorTree,
        instruction::{Instruction, InstructionId},
        value::{Value, ValueId},
    },
    opt::pure::Purity,
};
use rustc_hash::FxHashMap as HashMap;

/// HashMap from `(Instruction, side_effects_enabled_var)` to the results of the instruction.
/// Stored as a two-level map to avoid cloning Instructions during the `.get` call.
///
/// The `side_effects_enabled_var` is optional because we only use them when `Instruction::requires_acir_gen_predicate`
/// is true _and_ the constraint information is also taken into account.
///
/// In addition to each result, the original BasicBlockId is stored as well. This allows us
/// to deduplicate instructions across blocks as long as the new block dominates the original.
#[derive(Default)]
pub(super) struct InstructionResultCache(
    HashMap<Instruction, HashMap<Option<ValueId>, ResultCache>>,
);

impl InstructionResultCache {
    /// Get a cached result if it can be used in this context.
    pub(super) fn get(
        &self,
        dfg: &DataFlowGraph,
        dom: &mut DominatorTree,
        id: InstructionId,
        instruction: &Instruction,
        predicate: Option<ValueId>,
        block: BasicBlockId,
    ) -> Option<CacheResult> {
        let results_for_instruction = self.0.get(instruction)?;

        let cached_results = results_for_instruction.get(&predicate)?.get(
            block,
            dom,
            instruction.has_side_effects(dfg),
        );

        cached_results.filter(|results| {
            // This is a hacky solution to https://github.com/noir-lang/noir/issues/9477
            // We explicitly check that the cached result values are of the same type as expected by the instruction
            // being checked against the cache and reject if they differ.
            if let CacheResult::Cached { results, .. } = results {
                let old_results = dfg.instruction_results(id);

                results.len() == old_results.len()
                    && old_results
                        .iter()
                        .zip(results.iter())
                        .all(|(old, new)| dfg.type_of_value(*old) == dfg.type_of_value(*new))
            } else {
                true
            }
        })
    }

    pub(super) fn cache(
        &mut self,
        dom: &mut DominatorTree,
        instruction: Instruction,
        predicate: Option<ValueId>,
        block: BasicBlockId,
        results: Vec<ValueId>,
    ) {
        self.0
            .entry(instruction)
            .or_default()
            .entry(predicate)
            .or_default()
            .cache(block, dom, results);
    }

    pub(super) fn remove(
        &mut self,
        instruction: &Instruction,
    ) -> Option<HashMap<Option<ValueId>, ResultCache>> {
        self.0.remove(instruction)
    }

    /// Remove previously cached instructions that created arrays,
    /// if the current instruction is such that it could modify that array.
    pub(super) fn remove_possibly_mutated_cached_make_arrays(
        &mut self,
        instruction: &Instruction,
        dfg: &DataFlowGraph,
    ) {
        use Instruction::{ArraySet, Call, MakeArray, Store};

        /// Recursively remove from the cache any array values.
        fn go(
            dfg: &DataFlowGraph,
            cached_instruction_results: &mut InstructionResultCache,
            value: &ValueId,
        ) {
            // We expect globals to be immutable, so we can cache those results indefinitely.
            if dfg.is_global(*value) {
                return;
            };

            // We only care about arrays and vectors. (`Store` can act on non-array values as well)
            if !dfg.type_of_value(*value).is_array() {
                return;
            };

            // Look up the original instruction that created the value, which is the cache key.
            let instruction = match &dfg[*value] {
                Value::Instruction { instruction, .. } => &dfg[*instruction],
                _ => return,
            };

            // Remove the creator instruction from the cache.
            if matches!(instruction, MakeArray { .. } | Call { .. }) {
                cached_instruction_results.remove(instruction);
            }

            // For arrays, we also want to invalidate the values, because multi-dimensional arrays
            // can be passed around, and through them their sub-arrays might be modified.
            if let MakeArray { elements, .. } = instruction {
                for elem in elements {
                    go(dfg, cached_instruction_results, elem);
                }
            }
        }

        let mut remove_if_array = |value| go(dfg, self, value);

        // Should we consider calls to vector_push_back and similar to be mutating operations as well?
        match instruction {
            Store { value, .. } | ArraySet { array: value, .. } => {
                // If we write to a value, it's not safe for reuse, as its value has changed since its creation.
                remove_if_array(value);
            }
            Call { arguments, func } if dfg.runtime().is_brillig() => {
                // If we pass a value to a function, it might get modified, making it unsafe for reuse after the call.
                let Value::Function(func_id) = &dfg[*func] else { return };
                if matches!(dfg.purity_of(*func_id), None | Some(Purity::Impure)) {
                    // Arrays passed to functions might be mutated by them if there are no `inc_rc` instructions
                    // placed *before* the call to protect them. Currently we don't track the ref count in this
                    // context, so be conservative and do not reuse any array shared with a callee.
                    // In ACIR we don't track refcounts, so it should be fine.
                    for arg in arguments {
                        remove_if_array(arg);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Records the results of all duplicate [`Instruction`]s along with the blocks in which they sit.
///
/// For more information see [`InstructionResultCache`].
#[derive(Default, Debug)]
pub(super) struct ResultCache {
    result: Option<(BasicBlockId, Vec<ValueId>)>,
}
impl ResultCache {
    /// Records that an `Instruction` in block `block` produced the result values `results`.
    fn cache(&mut self, block: BasicBlockId, dom: &mut DominatorTree, results: Vec<ValueId>) {
        let overwrite = match self.result {
            None => true,
            Some((origin, _)) => origin != block && dom.dominates(block, origin),
        };

        if overwrite {
            self.result = Some((block, results));
        }
    }

    /// Returns a set of [`ValueId`]s produced from a copy of this [`Instruction`] which sits
    /// within a block which dominates `block`.
    ///
    /// We require that the cached instruction's block dominates `block` in order to avoid
    /// cycles causing issues (e.g. two instructions being replaced with the results of each other
    /// such that neither instruction exists anymore.)
    pub(super) fn get(
        &self,
        block: BasicBlockId,
        dom: &mut DominatorTree,
        has_side_effects: bool,
    ) -> Option<CacheResult> {
        self.result.as_ref().and_then(|(origin, results)| {
            if dom.dominates(*origin, block) {
                Some(CacheResult::Cached { results })
            } else if !has_side_effects {
                // Insert a copy of this instruction in the common dominator
                let dominator = dom.common_dominator(*origin, block);
                Some(CacheResult::NeedToHoistToCommonBlock { dominator })
            } else {
                None
            }
        })
    }
}

#[derive(Debug)]
pub(super) enum CacheResult<'a> {
    /// The result of an earlier instruction can be readily reused, because it was found
    /// in a block that dominates the one where the current instruction is. We can drop
    /// the current instruction and redefine its results in terms of the existing values.
    Cached {
        /// The value IDs we can reuse.
        results: &'a [ValueId],
    },
    /// We found an identical instruction in a non-dominating block, so we cannot directly
    /// reuse its results, because they are not visible in the current block. However, we
    /// can hoist the instruction into the common dominator, and deduplicate later.
    NeedToHoistToCommonBlock {
        /// The common dominator where we can hoist the current instruction.
        dominator: BasicBlockId,
    },
}
