use std::borrow::{Borrow, Cow};

use itertools::Itertools;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        dom::DominatorTree,
        instruction::{Instruction, InstructionId},
        types::Type,
        value::{Value, ValueId},
    },
    opt::pure::Purity,
};
use rustc_hash::FxHashMap as HashMap;

#[derive(Debug, Eq)]
struct CacheKeyRef<'a>(Cow<'a, Instruction>);

#[derive(Debug, Eq, PartialEq, Hash)]
struct CacheKey(CacheKeyRef<'static>);

impl From<Instruction> for CacheKey {
    fn from(value: Instruction) -> Self {
        Self(CacheKeyRef::from(value))
    }
}

impl From<Instruction> for CacheKeyRef<'_> {
    fn from(value: Instruction) -> Self {
        Self(Cow::Owned(value))
    }
}

impl<'a> From<&'a Instruction> for CacheKeyRef<'a> {
    fn from(value: &'a Instruction) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl PartialEq for CacheKeyRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self.0.as_ref(), other.0.as_ref()) {
            (Instruction::Constrain(lhs1, lhs2, _), Instruction::Constrain(rhs1, rhs2, _))
            | (
                Instruction::ConstrainNotEqual(lhs1, rhs1, _),
                Instruction::ConstrainNotEqual(lhs2, rhs2, _),
            ) => lhs1 == rhs1 && lhs2 == rhs2,

            (
                Instruction::RangeCheck {
                    value: lhs_value,
                    max_bit_size: lhs_max_bit_size,
                    assert_message: _,
                },
                Instruction::RangeCheck {
                    value: rhs_value,
                    max_bit_size: rhs_max_bit_size,
                    assert_message: _,
                },
            ) => lhs_value == rhs_value && lhs_max_bit_size == rhs_max_bit_size,

            (a, b) => a == b,
        }
    }
}

impl std::hash::Hash for CacheKeyRef<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self.0.as_ref() {
            Instruction::Constrain(a, b, _) | Instruction::ConstrainNotEqual(a, b, _) => {
                a.hash(state);
                b.hash(state);
            }
            Instruction::RangeCheck { value, max_bit_size, assert_message: _ } => {
                value.hash(state);
                max_bit_size.hash(state);
            }
            other => other.hash(state),
        }
    }
}

impl<'a> Borrow<CacheKeyRef<'a>> for CacheKey {
    fn borrow(&self) -> &CacheKeyRef<'a> {
        &self.0
    }
}

impl AsRef<Instruction> for CacheKey {
    fn as_ref(&self) -> &Instruction {
        self.0.0.as_ref()
    }
}

/// HashMap from `(Instruction, side_effects_enabled_var)` to the results of the instruction.
/// Stored as a two-level map to avoid cloning Instructions during the `.get` call.
///
/// The `side_effects_enabled_var` is optional because we only use them when `Instruction::requires_acir_gen_predicate`
/// is true _and_ the constraint information is also taken into account.
///
/// In addition to each result, the original `BasicBlockId` is stored as well. This allows us
/// to deduplicate instructions across blocks as long as the new block dominates the original.
#[derive(Default)]
pub(super) struct InstructionResultCache(HashMap<CacheKey, HashMap<Option<ValueId>, ResultCache>>);

impl InstructionResultCache {
    /// Get a cached result if it can be used in this context.
    pub(super) fn get(
        &self,
        dfg: &DataFlowGraph,
        dom: &DominatorTree,
        id: InstructionId,
        instruction: &Instruction,
        predicate: Option<ValueId>,
        block: BasicBlockId,
    ) -> Option<CacheResult> {
        let results_for_instruction = self.0.get(&CacheKeyRef::from(instruction))?;

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
                        .zip_eq(results.iter())
                        .all(|(old, new)| dfg.type_of_value(*old) == dfg.type_of_value(*new))
            } else {
                true
            }
        })
    }

    pub(super) fn cache(
        &mut self,
        dom: &DominatorTree,
        instruction: Instruction,
        predicate: Option<ValueId>,
        block: BasicBlockId,
        results: Vec<ValueId>,
    ) {
        self.0
            .entry(CacheKey::from(instruction))
            .or_default()
            .entry(predicate)
            .or_default()
            .cache(block, dom, results);
    }

    pub(super) fn remove(
        &mut self,
        instruction: &Instruction,
    ) -> Option<HashMap<Option<ValueId>, ResultCache>> {
        self.0.remove(&CacheKeyRef::from(instruction))
    }

    /// Remove all cached MakeArray instructions that produce the given type.
    /// Used when we encounter a mutation of an array value that we can't trace back
    /// to a specific instruction (e.g. block parameters), so we must conservatively
    /// invalidate all cached MakeArrays that could be the source.
    fn remove_make_arrays_of_type(&mut self, typ: &Type) {
        self.0.retain(|instruction, _| {
            !matches!(instruction.as_ref(), Instruction::MakeArray { typ: make_array_typ, .. } if make_array_typ == typ)
        });
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
            }

            let value_type = dfg.type_of_value(*value);

            // We only care about arrays and vectors. (`Store` can act on non-array values as well)
            if !value_type.is_array() {
                return;
            }

            // Look up the original instruction that created the value, which is the cache key.
            let instruction = match &dfg[*value] {
                Value::Instruction { instruction, .. } => &dfg[*instruction],
                _ => {
                    // If we can't trace back to a creating instruction (e.g. block parameters),
                    // conservatively remove all cached MakeArrays of the same type since any
                    // of them could be the source of this value.
                    cached_instruction_results.remove_make_arrays_of_type(&value_type);
                    return;
                }
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

        match instruction {
            // A mutable `array_set` writes through its input array's backing store in place rather
            // than producing a fresh array, so the input is no longer safe to reuse. In ACIR the
            // `mutable` flag is only set by `mutable_array_set_optimization`, which runs after
            // constant folding; keying off the flag (rather than the runtime) keeps this invalidation
            // correct even if that ordering ever changes. In Brillig in-place mutation happens under
            // copy-on-write regardless of the flag, which the runtime-guarded arm below handles.
            ArraySet { array: value, mutable: true, .. } => {
                remove_if_array(value);
            }
            // In Brillig, `array_set`/`store` may mutate their array operand in place under
            // copy-on-write, so the operand is not safe for reuse. In ACIR these are value-semantic:
            // a non-mutable `array_set` produces a fresh array, and a `store` cannot lead to in-place
            // mutation because the only source of mutable sets (`mutable_array_set_optimization`)
            // runs after mem2reg has removed all loads/stores, so the two never coexist.
            Store { value, .. } | ArraySet { array: value, .. } if dfg.runtime().is_brillig() => {
                remove_if_array(value);
            }
            Call { arguments, func } if dfg.runtime().is_brillig() => {
                // Arrays passed to a callee might be mutated by it if there are no `inc_rc` instructions
                // placed *before* the call to protect them. Currently we don't track the ref count in this
                // context, so be conservative and do not reuse any array shared with such a callee.
                // In ACIR we don't track refcounts, so it should be fine.
                let mutates_arguments = match &dfg[*func] {
                    // A non-pure user-defined function may mutate its array arguments in place.
                    Value::Function(func_id) => {
                        matches!(dfg.purity_of(*func_id), None | Some(Purity::Impure))
                    }
                    // The vector mutators (`push`/`pop`/`insert`/`remove`) write through their
                    // vector argument when its copy-on-write reference count is 1, even though they
                    // are otherwise "pure". Treat them like an impure call so a later identical
                    // array-producing instruction is not deduplicated against the now-mutated value.
                    Value::Intrinsic(intrinsic) => intrinsic.unsafe_for_clone_elision_in_brillig(),
                    _ => false,
                };
                if mutates_arguments {
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
    fn cache(&mut self, block: BasicBlockId, dom: &DominatorTree, results: Vec<ValueId>) {
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
        dom: &DominatorTree,
        has_side_effects: bool,
    ) -> Option<CacheResult> {
        let (origin, results) = self.result.as_ref()?;
        if dom.dominates(*origin, block) {
            Some(CacheResult::Cached { dominator: *origin, results })
        } else if !has_side_effects {
            // Insert a copy of this instruction in the common dominator
            let dominator = dom.common_dominator(*origin, block);
            Some(CacheResult::NeedToHoistToCommonBlock { dominator })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub(super) enum CacheResult<'a> {
    /// The result of an earlier instruction can be readily reused, because it was found
    /// in a block that dominates the one where the current instruction is. We can drop
    /// the current instruction and redefine its results in terms of the existing values.
    Cached {
        dominator: BasicBlockId,
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
