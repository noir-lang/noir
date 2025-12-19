//! The goal of the constant folding optimization pass is to propagate any constants forwards into
//! later [`Instruction`]s to maximize the impact of [compile-time simplifications][crate::ssa::ir::dfg::simplify::simplify()].
//!
//! The pass works as follows:
//! - Re-insert each instruction in order to apply the instruction simplification performed
//!   by the [`DataFlowGraph`] automatically as new instructions are pushed.
//! - Check whether any input values have been constrained to be equal to a value of a simpler form
//!   by a [constrain instruction][Instruction::Constrain]. If so, replace the input value with the simpler form.
//! - Check whether the instruction [`can_be_deduplicated`]
//!   by duplicate instruction earlier in the same block.
//!
//! These operations are done in parallel so that they can each benefit from each other
//! without the need for multiple passes.
//!
//! This is the only pass which removes duplicated pure [`Instruction`]s however and so is needed when
//! different blocks are merged, i.e. after the [`flatten_cfg`][super::flatten_cfg] pass.
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Empty,
};

use acvm::{FieldElement, acir::AcirField};
use iter_extended::vecmap;

use crate::ssa::{
    interpreter::{Interpreter, InterpreterOptions},
    ir::{
        basic_block::BasicBlockId,
        dfg::DataFlowGraph,
        dom::DominatorTree,
        function::{Function, FunctionId},
        instruction::{Instruction, InstructionId},
        types::NumericType,
        value::{Value, ValueId, ValueMapping},
    },
    opt::pure::Purity,
    ssa_gen::Ssa,
    visit_once_priority_queue::VisitOncePriorityQueue,
};
use rustc_hash::FxHashMap as HashMap;

mod interpret;
mod result_cache;
mod simplification_cache;

use interpret::try_interpret_call;
use result_cache::{CacheResult, InstructionResultCache};
use simplification_cache::{ConstraintSimplificationCache, SimplificationCache};

pub const DEFAULT_MAX_ITER: usize = 5;

/// Maximum number of SSA instructions to execute during inlining a constant Brillig call.
///
/// The number is based on some experimentation to limit a tight loop to ~100ms.
const DEFAULT_INTERPRETER_STEP_LIMIT: usize = 10_000_000;

impl Ssa {
    /// Performs constant folding on each instruction.
    ///
    /// It will not look at constraints to inform simplifications
    /// based on the stated equivalence of two instructions.
    ///
    /// See [`constant_folding`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn fold_constants(mut self, max_iter: usize) -> Ssa {
        for function in self.functions.values_mut() {
            function.constant_fold(false, max_iter, &mut None);
        }
        self
    }

    /// Performs constant folding on each instruction.
    ///
    /// Also uses constraint information to inform more optimizations.
    ///
    /// See [`constant_folding`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn fold_constants_using_constraints(mut self, max_iter: usize) -> Ssa {
        for function in self.functions.values_mut() {
            function.constant_fold(true, max_iter, &mut None);
        }
        self
    }

    /// Performs constant folding on each instruction while also replacing calls to brillig functions
    /// with all constant arguments by trying to evaluate those calls.
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn fold_constants_with_brillig(mut self, max_iter: usize) -> Ssa {
        // Collect all brillig functions so that later we can find them when processing a call instruction
        let mut brillig_functions: BTreeMap<FunctionId, Function> = BTreeMap::new();
        for (func_id, func) in &self.functions {
            if func.runtime().is_brillig() {
                let cloned_function = Function::clone_with_id(*func_id, func);
                brillig_functions.insert(*func_id, cloned_function);
            };
        }
        let mut interpreter = if brillig_functions.is_empty() {
            None
        } else {
            let mut interpreter = Interpreter::new_from_functions(
                &brillig_functions,
                InterpreterOptions {
                    no_foreign_calls: true,
                    step_limit: Some(DEFAULT_INTERPRETER_STEP_LIMIT),
                    ..Default::default()
                },
                std::io::empty(),
            );
            // Interpret globals once so that we do not have to repeat this computation on every Brillig call.
            interpreter.interpret_globals().expect("ICE: Interpreter failed to interpret globals");
            Some(interpreter)
        };

        for function in self.functions.values_mut() {
            function.constant_fold(false, max_iter, &mut interpreter);
        }

        self
    }
}

impl Function {
    /// The structure of this pass is simple:
    /// Go through each block and re-insert all instructions.
    pub(crate) fn constant_fold(
        &mut self,
        use_constraint_info: bool,
        max_iter: usize,
        interpreter: &mut Option<Interpreter<Empty>>,
    ) {
        let mut dom = DominatorTree::with_function(self);
        let mut context = Context::new(use_constraint_info);

        context.enqueue(&dom, [self.entry_block()]);

        for _ in 0..max_iter {
            while let Some(block) = context.block_queue.pop_front() {
                context.fold_constants_in_block(&mut self.dfg, &mut dom, block, interpreter);
                context.enqueue(&dom, self.dfg[block].successors());
            }

            #[cfg(debug_assertions)]
            constant_folding_post_check(&context, &self.dfg);

            // Rebuild the cache and deduplicate the blocks we hoisted into with the origins.
            let blocks_to_revisit = context.blocks_to_revisit;

            // Create a fresh context, so values cached towards the end are not visible to blocks during a revisit.
            // For example reusing the cache could be problematic when using constraint info, as it could make the
            // original content simplify out based on its own prior assertion of a value being a constant.
            context = Context::new(use_constraint_info);
            context.enqueue(&dom, blocks_to_revisit);
        }
    }
}

#[cfg(debug_assertions)]
fn constant_folding_post_check(context: &Context, dfg: &DataFlowGraph) {
    assert!(
        context.values_to_replace.value_types_are_consistent(dfg),
        "Constant folding should not map a ValueId to another of a different type"
    );
}

struct Context {
    /// Keeps track of visited blocks and blocks to visit.
    /// Prioritizes them based on their Reverse Post Order rank, which ensures
    /// that we see them in a consistent order even during restarts.
    block_queue: VisitOncePriorityQueue<u32, BasicBlockId>,

    /// Blocks which we hoisted instructions into. We can make another folding iteration
    /// starting from these blocks and revisiting all their descendants to:
    /// 1. Deduplicate the original instruction we found in the cache
    /// 2. Unlock further instructions that can be hoisted after deduplication.
    blocks_to_revisit: BTreeSet<BasicBlockId>,

    /// Whether to use [constraints][Instruction::Constrain] to inform simplifications later on in the program.
    ///
    /// For example, this allows simplifying the instructions below to determine that `v2 == Field 3` without
    /// laying down constraints for the addition:
    ///
    /// ```ssa
    /// constrain v1 == Field 0
    /// v2 = add v1, Field 2
    /// ```
    use_constraint_info: bool,

    /// Contains sets of values which are constrained to be equivalent to each other.
    ///
    /// The mapping's structure is `side_effects_enabled_var => (constrained_value => simplified_value)`.
    ///
    /// We partition the maps of constrained values according to the side-effects flag at the point
    /// at which the values are constrained. This prevents constraints which are only sometimes enforced
    /// being used to modify the rest of the program.
    constraint_simplification_mappings: ConstraintSimplificationCache,

    /// Cache of instructions along with their outputs which are safe to reuse.
    ///
    /// See [`can_be_deduplicated`] for more information
    cached_instruction_results: InstructionResultCache,

    /// Maps pre-folded ValueIds to the new ValueIds obtained by re-inserting the instruction.
    values_to_replace: ValueMapping,
}

impl Context {
    fn new(use_constraint_info: bool) -> Self {
        Self {
            use_constraint_info,
            block_queue: Default::default(),
            constraint_simplification_mappings: Default::default(),
            cached_instruction_results: Default::default(),
            values_to_replace: Default::default(),
            blocks_to_revisit: Default::default(),
        }
    }

    fn enqueue(&mut self, dom: &DominatorTree, blocks: impl IntoIterator<Item = BasicBlockId>) {
        for block in blocks {
            let rank = dom.reverse_post_order_idx(block).unwrap();
            self.block_queue.push(rank, block);
        }
    }

    fn fold_constants_in_block(
        &mut self,
        dfg: &mut DataFlowGraph,
        dom: &mut DominatorTree,
        block_id: BasicBlockId,
        interpreter: &mut Option<Interpreter<Empty>>,
    ) {
        let instructions = dfg[block_id].take_instructions();

        // Default side effect condition variable with an enabled state.
        let mut side_effects_enabled_var =
            dfg.make_constant(FieldElement::one(), NumericType::bool());

        for instruction_id in instructions {
            let instruction = &mut dfg[instruction_id];

            instruction.replace_values(&self.values_to_replace);

            self.fold_constants_into_instruction(
                dfg,
                dom,
                block_id,
                instruction_id,
                &mut side_effects_enabled_var,
                interpreter,
            );
        }

        // Map the block terminator, resolving any values in the terminator with the
        // internal value mapping generated by this pass.
        dfg.replace_values_in_block_terminator(block_id, &self.values_to_replace);
        dfg.data_bus.replace_values(&self.values_to_replace);

        // Map a terminator in place, replacing any ValueId in the terminator with the
        // resolved version of that value id from the simplification cache's internal value mapping.
        // We need this in addition to the value replacement above in order to take advantage
        // of constraints that may have advised simplifications.
        // The value mapping (`self.values_to_replace`) only maps old instruction results to new instruction results.
        // However, constraints do not have "results" like other instructions, thus are not included in `self.values_to_replace`.
        // To take advantage of constraint simplification we need to still resolve its cache.
        let mut terminator = dfg[block_id].take_terminator();
        let constraint_simplification_cache =
            self.constraint_simplification_mappings.get(side_effects_enabled_var);
        let mut resolve_cache =
            |value| resolve_cache(block_id, dom, constraint_simplification_cache, value);

        terminator.map_values_mut(&mut resolve_cache);
        dfg[block_id].set_terminator(terminator);
        dfg.data_bus.map_values_mut(resolve_cache);
    }

    fn fold_constants_into_instruction(
        &mut self,
        dfg: &mut DataFlowGraph,
        dom: &mut DominatorTree,
        block: BasicBlockId,
        id: InstructionId,
        side_effects_enabled_var: &mut ValueId,
        interpreter: &mut Option<Interpreter<Empty>>,
    ) {
        let constraint_simplification_mapping =
            self.constraint_simplification_mappings.get(*side_effects_enabled_var);

        let instruction =
            Self::resolve_instruction(id, block, dfg, dom, constraint_simplification_mapping);

        let old_results = dfg.instruction_results(id).to_vec();
        let mut target_block = block;

        // If a copy of this instruction exists earlier in the block, then reuse the previous results.
        let runtime_is_brillig = dfg.runtime().is_brillig();
        let predicate = self.cache_predicate(*side_effects_enabled_var, &instruction, dfg);
        if let Some(cache_result) =
            self.cached_instruction_results.get(dfg, dom, id, &instruction, predicate, block)
        {
            match cache_result {
                CacheResult::Cached { results: cached, .. } => {
                    // We track whether we may mutate `MakeArray` instructions before we deduplicate
                    // them but we still need to issue an extra inc_rc in case they're mutated afterward.
                    //
                    // This also applies to calls that return arrays.
                    if runtime_is_brillig {
                        Self::increase_rc(id, cached, block, dfg);
                    }

                    self.values_to_replace.batch_insert(&old_results, cached);

                    return;
                }
                CacheResult::NeedToHoistToCommonBlock { dominator } => {
                    // During revisits we can visit a block which dominates something we already cached instructions from,
                    // if we restarted from a hoist point that this block also dominates. Most likely it is pointless to
                    // schedule a revisit of *this* block after again, because something must have prevented this instruction
                    // from being reused already (e.g. an array mutation).
                    if dominator != block {
                        self.blocks_to_revisit.insert(dominator);
                    }

                    // Just change the block to insert in the common dominator instead.
                    // This will only move the current instance of the instruction right now.
                    // When constant folding is run a second time later on, it'll catch
                    // that the previous instance can be deduplicated to this instance.
                    // Another effect is going to be that the cache should be updated to
                    // point at the dominator, so subsequent blocks can use the result.
                    target_block = dominator;
                }
            }
        };

        // First try to inline a call to a brillig function with all constant arguments.
        let new_results = if runtime_is_brillig {
            Self::push_instruction(id, instruction.clone(), &old_results, target_block, dfg)
        } else {
            // We only want to try to inline Brillig calls for Brillig entry points (functions called from an ACIR runtime).
            try_interpret_call(&instruction, target_block, dfg, interpreter.as_mut())
                // Otherwise, try inserting the instruction again to apply any optimizations using the newly resolved inputs.
                .unwrap_or_else(|| {
                    Self::push_instruction(id, instruction.clone(), &old_results, target_block, dfg)
                })
        };
        // If the target_block is distinct than the original block
        // that means that the current instruction is not added in the original block
        // so it is deduplicated by the one in the target block.
        // In case it refers to an array that is mutated, we need to increment
        // its reference count.
        if target_block != block && runtime_is_brillig {
            Self::increase_rc(id, &new_results, block, dfg);
        }

        self.values_to_replace.batch_insert(&old_results, &new_results);

        self.cache_instruction(
            &instruction,
            new_results,
            dfg,
            dom,
            *side_effects_enabled_var,
            target_block,
        );

        // If we just inserted an `Instruction::EnableSideEffectsIf`, we need to update `side_effects_enabled_var`
        // so that we use the correct set of constrained values in future.
        if let Instruction::EnableSideEffectsIf { condition } = instruction {
            *side_effects_enabled_var = condition;
        };
    }

    fn increase_rc(
        id: InstructionId,
        results: &[ValueId],
        block: BasicBlockId,
        dfg: &mut DataFlowGraph,
    ) {
        let instruction = &dfg[id];
        if matches!(instruction, Instruction::MakeArray { .. } | Instruction::Call { .. }) {
            let call_stack = dfg.get_instruction_call_stack_id(id);
            for value in results {
                let value_type = dfg.type_of_value(*value);
                if value_type.is_array() {
                    let inc_rc = Instruction::IncrementRc { value: *value };
                    dfg.insert_instruction_and_results(inc_rc, block, None, call_stack);
                }
            }
        }
    }

    /// Fetches an [`Instruction`] by its [`InstructionId`] and fully resolves its inputs.
    fn resolve_instruction(
        instruction_id: InstructionId,
        block: BasicBlockId,
        dfg: &DataFlowGraph,
        dom: &mut DominatorTree,
        constraint_simplification_mapping: Option<&HashMap<ValueId, SimplificationCache>>,
    ) -> Instruction {
        let mut instruction = dfg[instruction_id].clone();

        // Resolve any inputs to ensure that we're comparing like-for-like instructions.
        instruction.map_values_mut(|value_id| {
            resolve_cache(block, dom, constraint_simplification_mapping, value_id)
        });
        instruction
    }

    /// Pushes a new [`Instruction`] into the [`DataFlowGraph`] which applies any optimizations
    /// based on newly resolved values for its inputs.
    ///
    /// This may result in the [`Instruction`] being optimized away or replaced with a constant value.
    fn push_instruction(
        id: InstructionId,
        instruction: Instruction,
        old_results: &[ValueId],
        block: BasicBlockId,
        dfg: &mut DataFlowGraph,
    ) -> Vec<ValueId> {
        let ctrl_typevars = instruction
            .requires_ctrl_typevars()
            .then(|| vecmap(old_results, |result| dfg.type_of_value(*result)));

        let call_stack = dfg.get_instruction_call_stack_id(id);
        let results = dfg.insert_instruction_and_results_if_simplified(
            instruction,
            block,
            ctrl_typevars,
            call_stack,
            Some(id),
        );
        let new_results = results.results().to_vec();
        // Optimizations while inserting the instruction should not change the number of results.
        assert_eq!(old_results.len(), new_results.len());

        new_results
    }

    #[allow(clippy::too_many_arguments)]
    fn cache_instruction(
        &mut self,
        instruction: &Instruction,
        instruction_results: Vec<ValueId>,
        dfg: &DataFlowGraph,
        dom: &mut DominatorTree,
        side_effects_enabled_var: ValueId,
        block: BasicBlockId,
    ) {
        if self.use_constraint_info {
            // If the instruction was a constraint, then create a link between the two `ValueId`s
            // to map from the more complex to the simpler value.
            if let Instruction::Constrain(lhs, rhs, _) = instruction {
                // These `ValueId`s should be fully resolved now.
                self.constraint_simplification_mappings.cache(
                    dfg,
                    side_effects_enabled_var,
                    block,
                    *lhs,
                    *rhs,
                );
            }
        }

        // If we have an array get whose value is from an array set on the same array at the same index,
        // we can simplify that array get to the value of the previous array set.
        //
        // For example:
        // v3 = array_set v0, index v1, value v2
        // v4 = array_get v3, index v1 -> Field
        //
        // We know that `v4` can be simplified to `v2`.
        // Thus, even if the index is dynamic (meaning the array get would have side effects),
        // we can simplify the operation when we take into account the predicate.
        if let Instruction::ArraySet { index, value, .. } = instruction {
            let predicate = self.use_constraint_info.then_some(side_effects_enabled_var);

            let array_get = Instruction::ArrayGet { array: instruction_results[0], index: *index };

            // If we encounter an array_get for this address, we know what the result will be.
            self.cached_instruction_results.cache(dom, array_get, predicate, block, vec![*value]);
        }

        self.cached_instruction_results
            .remove_possibly_mutated_cached_make_arrays(instruction, dfg);

        // If the instruction doesn't have side-effects and if it won't interact with enable_side_effects during acir_gen,
        // we cache the results so we can reuse them if the same instruction appears again later in the block.
        // Others have side effects representing failure, which are implicit in the ACIR code and can also be deduplicated.
        let can_be_deduplicated = can_be_deduplicated(instruction, dfg);

        let use_constraint_info = self.use_constraint_info;
        let is_make_array = matches!(instruction, Instruction::MakeArray { .. });

        let cache_instruction = || {
            let predicate = self.cache_predicate(side_effects_enabled_var, instruction, dfg);
            // If we see this make_array again, we can reuse the current result.
            self.cached_instruction_results.cache(
                dom,
                instruction.clone(),
                predicate,
                block,
                instruction_results,
            );
        };

        match can_be_deduplicated {
            CanBeDeduplicated::Always => cache_instruction(),
            CanBeDeduplicated::UnderSamePredicate if use_constraint_info => cache_instruction(),
            // We also allow deduplicating MakeArray instructions that we have tracked which haven't been mutated.
            _ if is_make_array => cache_instruction(),

            CanBeDeduplicated::UnderSamePredicate | CanBeDeduplicated::Never => {}
        }
    }

    /// Returns the predicate value to be used when looking up this [`Instruction`] in the cache.
    ///
    /// We sometimes remove the predicate in situations where an instruction is infallible as it allows us to
    /// deduplicate more aggressively.
    fn cache_predicate(
        &self,
        side_effects_enabled_var: ValueId,
        instruction: &Instruction,
        dfg: &DataFlowGraph,
    ) -> Option<ValueId> {
        let use_predicate =
            self.use_constraint_info && instruction.requires_acir_gen_predicate(dfg);
        use_predicate.then_some(side_effects_enabled_var)
    }
}

// Alternate between resolving `value_id` in the `dfg` and checking to see if the resolved value
// has been constrained to be equal to some simpler value in the current block.
//
// This allows us to reach a stable final `ValueId` for each instruction input as we add more
// constraints to the cache.
fn resolve_cache(
    block: BasicBlockId,
    dom: &mut DominatorTree,
    cache: Option<&HashMap<ValueId, SimplificationCache>>,
    value_id: ValueId,
) -> ValueId {
    match cache.and_then(|cache| cache.get(&value_id)) {
        Some(simplification_cache) => {
            if let Some(simplified) = simplification_cache.get(block, dom) {
                resolve_cache(block, dom, cache, simplified)
            } else {
                value_id
            }
        }
        None => value_id,
    }
}

#[derive(Debug)]
enum CanBeDeduplicated {
    /// This instruction has no side effects so we can substitute the results for those of the same instruction elsewhere.
    Always,
    /// This instruction has some side effects such as potentially fallible constraints which could halt execution.
    ///
    /// This means that if this instruction passes under a given predicate, we can reuse its results across all
    /// later instances of this instruction under the same predicate.
    UnderSamePredicate,
    /// This instruction has side effects which prevent all deduplication.
    ///
    /// An example is `EnableSideEffects` where a "duplicate" of this instruction has an important effect on later instructions
    /// which is not implied by the existence of the original `EnableSideEffects` instruction. For example:
    ///
    /// ```ssa
    /// enable_side_effects u1 1
    /// enable_side_effects u1 0
    /// enable_side_effects u1 1 <-- deduplicating this instruction results in side effects being disabled rather than enabled.
    /// ```
    Never,
}

/// Indicates if the instruction can be safely replaced with the results of another instruction with the same inputs.
/// If `deduplicate_with_predicate` is set, we assume we're deduplicating with the instruction
/// and its predicate, rather than just the instruction. Setting this means instructions that
/// rely on predicates can be deduplicated as well.
///
/// Some instructions get the predicate attached to their inputs by `handle_instruction_side_effects` in `flatten_cfg`.
/// These can be deduplicated because they implicitly depend on the predicate, not only when the caller uses the
/// predicate variable as a key to cache results. However, to avoid tight coupling between passes, we make the deduplication
/// conditional on whether the caller wants the predicate to be taken into account or not.
fn can_be_deduplicated(instruction: &Instruction, dfg: &DataFlowGraph) -> CanBeDeduplicated {
    use Instruction::*;

    match instruction {
        // These either have side-effects or interact with memory
        EnableSideEffectsIf { .. }
        | Allocate
        | Load { .. }
        | Store { .. }
        | IncrementRc { .. }
        | DecrementRc { .. } => CanBeDeduplicated::Never,

        Call { func, .. } => {
            let purity = match dfg[*func] {
                Value::Intrinsic(intrinsic) => Some(intrinsic.purity()),
                Value::Function(id) => dfg.purity_of(id),
                _ => None,
            };
            match purity {
                Some(Purity::Pure) => CanBeDeduplicated::Always,
                Some(Purity::PureWithPredicate) => CanBeDeduplicated::UnderSamePredicate,
                Some(Purity::Impure) => CanBeDeduplicated::Never,
                None => CanBeDeduplicated::Never,
            }
        }

        // We can deduplicate these instructions if we know the predicate is also the same.
        Constrain(..) | ConstrainNotEqual(..) | RangeCheck { .. } => {
            CanBeDeduplicated::UnderSamePredicate
        }

        // Noop instructions can always be deduplicated, although they're more likely to be
        // removed entirely.
        Noop => CanBeDeduplicated::Always,

        // These instructions can always be deduplicated
        Cast(_, _) | Not(_) | Truncate { .. } => CanBeDeduplicated::Always,
        IfElse { then_value, .. } => {
            if dfg.type_of_value(*then_value).is_array() && dfg.runtime().is_brillig() {
                CanBeDeduplicated::Never
            } else {
                CanBeDeduplicated::Always
            }
        }
        // Arrays can be mutated in unconstrained code so code that handles this case must
        // take care to track whether the array was possibly mutated or not before
        // deduplicating. Since we don't know if the containing pass checks for this, we
        // can only assume these are safe to deduplicate in constrained code.
        MakeArray { .. } => {
            if dfg.runtime().is_acir() {
                CanBeDeduplicated::Always
            } else {
                CanBeDeduplicated::Never
            }
        }

        // These can have different behavior depending on the EnableSideEffectsIf context.
        // Replacing them with a similar instruction potentially enables replacing an instruction
        // with one that was disabled. See
        // https://github.com/noir-lang/noir/pull/4716#issuecomment-2047846328.
        Binary(_) | ArrayGet { .. } | ArraySet { .. } => {
            if instruction.requires_acir_gen_predicate(dfg) {
                CanBeDeduplicated::UnderSamePredicate
            } else {
                CanBeDeduplicated::Always
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        assert_ssa_snapshot,
        ssa::{
            Ssa,
            interpreter::value::Value,
            ir::{types::NumericType, value::ValueMapping},
            opt::{
                assert_normalized_ssa_equals, assert_pass_does_not_affect_execution,
                assert_ssa_does_not_change, constant_folding::DEFAULT_MAX_ITER,
            },
        },
    };

    // Do just 1 iteration in tests where we want to minimize the expected changes in the SSA.
    const MIN_ITER: usize = 1;

    #[test]
    fn simple_constant_fold() {
        // After constructing this IR, we set the value of v0 to 2.
        // The expected return afterwards should be 9.
        let src = "
            acir(inline) fn main f0 {
              b0(v0: Field):
                v1 = add v0, Field 1
                v2 = mul v1, Field 3
                return v2
            }
            ";
        let mut ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main_mut();

        let entry_block = main.entry_block();
        let instructions = main.dfg[entry_block].instructions();
        assert_eq!(instructions.len(), 2); // The final return is not counted

        let v0 = main.parameters()[0];
        let two = main.dfg.make_constant(2_u128.into(), NumericType::NativeField);

        let mut values_to_replace = ValueMapping::default();
        values_to_replace.insert(v0, two);
        main.dfg.replace_values_in_block(entry_block, &values_to_replace);

        let expected = "
            acir(inline) fn main f0 {
              b0(v0: Field):
                return Field 9
            }
            ";
        let ssa = ssa.fold_constants(MIN_ITER);
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn redundant_truncation() {
        // After constructing this IR, we set the value of v1 to 2^8.
        // The expected return afterwards should be v2.
        let src = "
            acir(inline) fn main f0 {
              b0(v0: u16, v1: u16):
                v2 = div v0, v1
                v3 = truncate v2 to 8 bits, max_bit_size: 16
                return v3
            }
            ";
        let mut ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main_mut();

        let entry_block = main.entry_block();
        let instructions = main.dfg[entry_block].instructions();
        assert_eq!(instructions.len(), 2); // The final return is not counted

        let v1 = main.parameters()[1];

        // Note that this constant guarantees that `v0/constant < 2^8`. We then do not need to truncate the result.
        let constant = 2_u128.pow(8);
        let constant = main.dfg.make_constant(constant.into(), NumericType::unsigned(16));

        let mut values_to_replace = ValueMapping::default();
        values_to_replace.insert(v1, constant);
        main.dfg.replace_values_in_block(entry_block, &values_to_replace);

        let expected = "
            acir(inline) fn main f0 {
              b0(v0: u16, v1: u16):
                v3 = div v0, u16 256
                return v3
            }
            ";

        let ssa = ssa.fold_constants(MIN_ITER);
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn non_redundant_truncation() {
        // After constructing this IR, we set the value of v1 to 2^8 - 1.
        // This should not result in the truncation being removed.
        let src = "
            acir(inline) fn main f0 {
              b0(v0: u16, v1: u16):
                v2 = div v0, v1
                v3 = truncate v2 to 8 bits, max_bit_size: 16
                return v3
            }
            ";
        let mut ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main_mut();

        let entry_block = main.entry_block();
        let instructions = main.dfg[entry_block].instructions();
        assert_eq!(instructions.len(), 2); // The final return is not counted

        let v1 = main.parameters()[1];

        // Note that this constant does not guarantee that `v0/constant < 2^8`. We must then truncate the result.
        let constant = 2_u128.pow(8) - 1;
        let constant = main.dfg.make_constant(constant.into(), NumericType::unsigned(16));

        let mut values_to_replace = ValueMapping::default();
        values_to_replace.insert(v1, constant);
        main.dfg.replace_values_in_block(entry_block, &values_to_replace);

        let expected = "
            acir(inline) fn main f0 {
              b0(v0: u16, v1: u16):
                v3 = div v0, u16 255
                v4 = truncate v3 to 8 bits, max_bit_size: 16
                return v4
            }
            ";

        let ssa = ssa.fold_constants(MIN_ITER);
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn arrays_elements_are_updated() {
        // After constructing this IR, we run constant folding with no expected benefit, but to
        // ensure that all new values ids are correctly propagated.
        let src = "
            acir(inline) fn main f0 {
              b0(v0: Field):
                v2 = add v0, Field 1
                v3 = make_array [v2] : [Field; 1]
                return v3
            }
            ";
        assert_ssa_does_not_change(src, |ssa| ssa.fold_constants(MIN_ITER));
    }

    #[test]
    fn instruction_deduplication() {
        // After constructing this IR, we run constant folding which should replace the second cast
        // with a reference to the results to the first. This then allows us to optimize away
        // the constrain instruction as both inputs are known to be equal.
        //
        // The first cast instruction is retained and will be removed in the dead instruction elimination pass.
        let src = "
            acir(inline) fn main f0 {
              b0(v0: u16):
                v1 = cast v0 as u32
                v2 = cast v0 as u32
                constrain v1 == v2
                return
            }
            ";
        let expected = "
            acir(inline) fn main f0 {
              b0(v0: u16):
                v1 = cast v0 as u32
                return
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants(MIN_ITER);
        assert_normalized_ssa_equals(ssa, expected);
    }

    // TODO: https://github.com/noir-lang/noir/issues/9767
    #[test]
    fn constant_fold_duplicated_field_divisions() {
        // We should remove the duplicated field inversions here.
        let src = "
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            v1 = div Field 1, v0
            v2 = div Field 1, v0
            return
        }";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants(MIN_ITER);

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            v2 = div Field 1, v0
            v3 = div Field 1, v0
            return
        }
        ");
    }

    #[test]
    fn constant_index_array_access_deduplication() {
        // After constructing this IR, we run constant folding which should replace the second constant-index array get
        // with a reference to the results to the first. This then allows us to optimize away
        // the constrain instruction as both inputs are known to be equal.
        let src = "
            acir(inline) fn main f0 {
              b0(v0: [Field; 4], v1: u32, v2: bool, v3: bool):
                enable_side_effects v2
                v4 = array_get v0, index u32 0 -> Field
                v5 = array_get v0, index v1 -> Field
                enable_side_effects v3
                v6 = array_get v0, index u32 0 -> Field
                v7 = array_get v0, index v1 -> Field
                constrain v4 == v6
                return
            }
            ";
        let expected = "
            acir(inline) fn main f0 {
              b0(v0: [Field; 4], v1: u32, v2: bool, v3: bool):
                enable_side_effects v2
                v5 = array_get v0, index u32 0 -> Field
                v6 = array_get v0, index v1 -> Field
                enable_side_effects v3
                v7 = array_get v0, index v1 -> Field
                return
            }
            ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants(MIN_ITER);
        assert_normalized_ssa_equals(ssa, expected);
    }

    // Regression for #4600
    #[test]
    fn array_get_regression() {
        // We want to make sure after constant folding both array_gets remain since they are
        // under different enable_side_effects_if contexts and thus one may be disabled while
        // the other is not. If one is removed, it is possible e.g. v4 is replaced with v2 which
        // is disabled (only gets from index 0) and thus returns the wrong result.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u32):
            enable_side_effects v0
            v4 = make_array [Field 0, Field 1] : [Field; 2]
            v5 = array_get v4, index v1 -> Field
            v6 = not v0
            enable_side_effects v6
            v7 = array_get v4, index v1 -> Field
            return
        }
        ";
        assert_ssa_does_not_change(src, |ssa| ssa.fold_constants(MIN_ITER));
    }

    #[test]
    fn deduplicate_instructions_with_predicates() {
        let src = "
            acir(inline) fn main f0 {
              b0(v0: u1, v1: u1, v2: [Field; 2]):
                enable_side_effects v0
                v6 = array_get v2, index u32 0 -> u32
                v7 = array_set v2, index u32 1, value u32 2
                v8 = array_get v7, index u32 0 -> u32
                constrain v6 == v8
                enable_side_effects v1
                v9 = array_get v2, index u32 0 -> u32
                v10 = array_set v2, index u32 1, value u32 2
                v11 = array_get v10, index u32 0 -> u32
                constrain v9 == v11
                enable_side_effects v0
                v12 = array_get v2, index u32 0 -> u32
                v13 = array_set v2, index u32 1, value u32 2
                v14 = array_get v13, index u32 0 -> u32
                constrain v12 == v14
                return
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let main = ssa.main();
        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 15);

        let ssa = ssa.fold_constants_using_constraints(MIN_ITER);

        // The `array_get` instruction after `enable_side_effects v1` is deduplicated
        // with the one under `enable_side_effects v0` because it doesn't require a predicate,
        // but the `array_set` is not, because it does require a predicate, and the subsequent
        // `array_get` uses a different input, so it's not a duplicate of anything.
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u1, v2: [Field; 2]):
            enable_side_effects v0
            v4 = array_get v2, index u32 0 -> u32
            v7 = array_set v2, index u32 1, value u32 2
            v8 = array_get v2, index u32 0 -> u32
            constrain v4 == v8
            enable_side_effects v1
            v9 = array_set v2, index u32 1, value u32 2
            v10 = array_get v2, index u32 0 -> u32
            constrain v4 == v10
            enable_side_effects v0
            return
        }
        ");
    }

    #[test]
    fn constant_array_deduplication() {
        // Here we're checking a situation where two identical arrays are being initialized twice and being assigned separate `ValueId`s.
        // This would result in otherwise identical instructions not being deduplicated.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u64):
            v1 = make_array [v0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0] : [u64; 25]
            v2 = make_array [v0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0] : [u64; 25]
            v5 = call keccakf1600(v1) -> [u64; 25]
            v6 = call keccakf1600(v2) -> [u64; 25]
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        let main = ssa.main();
        let instructions = main.dfg[main.entry_block()].instructions();
        let starting_instruction_count = instructions.len();
        assert_eq!(starting_instruction_count, 4);

        let ssa = ssa.fold_constants(MIN_ITER);

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u64):
            v2 = make_array [v0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0] : [u64; 25]
            inc_rc v2
            v4 = call keccakf1600(v2) -> [u64; 25]
            inc_rc v4
            return
        }
        ");
    }

    #[test]
    fn deduplicate_across_blocks() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            v1 = not v0
            jmp b1()
          b1():
            v2 = not v0
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.fold_constants_using_constraints(MIN_ITER);
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1):
            v1 = not v0
            jmp b1()
          b1():
            return v1
        }
        "
        );
    }

    #[test]
    fn deduplicate_across_non_dominated_blocks() {
        let src = "
            brillig(inline) fn main f0 {
              b0(v0: u32):
                v2 = lt u32 1000, v0
                jmpif v2 then: b1, else: b2
              b1():
                v4 = shl v0, u32 1
                v5 = lt v0, v4
                constrain v5 == u1 1
                jmp b2()
              b2():
                v7 = lt u32 1000, v0
                jmpif v7 then: b3, else: b4
              b3():
                v8 = shl v0, u32 1
                v9 = lt v0, v8
                constrain v9 == u1 1
                jmp b4()
              b4():
                return
            }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants_using_constraints(MIN_ITER);

        // v4 has been hoisted, although:
        // - v5 has not yet been removed since it was encountered earlier in the program
        // - v8 hasn't been recognized as a duplicate of v6 yet since they still reference v4 and
        //   v5 respectively
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v2 = lt u32 1000, v0
            v4 = shl v0, u32 1
            jmpif v2 then: b1, else: b2
          b1():
            v5 = shl v0, u32 1
            v6 = lt v0, v5
            constrain v6 == u1 1
            jmp b2()
          b2():
            jmpif v2 then: b3, else: b4
          b3():
            v8 = lt v0, v4
            constrain v8 == u1 1
            jmp b4()
          b4():
            return
        }
        ");
    }

    #[test]
    fn increment_rc_on_make_array_deduplication() {
        let src = "
            brillig(inline) fn main f0 {
              b0(v0: u32):
                v2 = lt u32 1000, v0
                jmpif v2 then: b1, else: b2
              b1():
                v4 = make_array [u1 0] : [u1; 1]
                v5 = array_get v4, index u32 0 -> u1
                jmp b3(v5)
              b2():
                v7 = make_array [u1 0] : [u1; 1]
                v8 = array_get v7, index u32 0 -> u1
                jmp b3(v8)
              b3(v9: u1):
                constrain v9 == u1 0
                jmp b4()
              b4():
                return
            }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants_using_constraints(MIN_ITER);

        // v7 has been replaced by a v5, and its reference count is increased
        // v6 is not yet replaced but will be in a subsequent constant folding run
        assert_ssa_snapshot!(ssa, @r"
            brillig(inline) fn main f0 {
              b0(v0: u32):
                v3 = lt u32 1000, v0
                v5 = make_array [u1 0] : [u1; 1]
                jmpif v3 then: b1, else: b2
              b1():
                inc_rc v5
                jmp b3(u1 0)
              b2():
                v6 = make_array [u1 0] : [u1; 1]
                jmp b3(u1 0)
              b3(v1: u1):
                constrain v1 == u1 0
                jmp b4()
              b4():
                return
            }
        ");
    }

    #[test]
    fn repeatedly_hoist_and_deduplicate() {
        // Repeating the same block 3x times.
        let src = "
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: i8):
            v2 = allocate -> &mut i8
            store i8 0 at v2
            jmpif v0 then: b1, else: b2
          b1():
            v5 = unchecked_mul v1, i8 127
            v6 = cast v5 as u16
            v7 = truncate v6 to 8 bits, max_bit_size: 16
            v8 = cast v7 as i8
            store v8 at v2
            jmp b2()
          b2():
            jmpif v0 then: b3, else: b4
          b3():
            v9 = unchecked_mul v1, i8 127
            v10 = cast v9 as u16
            v11 = truncate v10 to 8 bits, max_bit_size: 16
            v12 = cast v11 as i8
            store v12 at v2
            jmp b4()
          b4():
            jmpif v0 then: b5, else: b6
          b5():
            v13 = unchecked_mul v1, i8 127
            v14 = cast v13 as u16
            v15 = truncate v14 to 8 bits, max_bit_size: 16
            v16 = cast v15 as i8
            store v16 at v2
            jmp b6()
          b6():
            v17 = load v2 -> i8
            return v17
          }
        ";

        let mut ssa = Ssa::from_str(src).unwrap();

        // First demonstrate what happens if we don't revisit.
        ssa.main_mut().constant_fold(false, 1, &mut None);

        // 1. v9 is a duplicate of v5 -> hoisted to b0
        // 2. v13 is a duplicate of v9 -> immediately deduplicated because it's now in b0
        // 3. v14 is a duplicate of v10 -> hoisted to b2
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: i8):
            v2 = allocate -> &mut i8
            store i8 0 at v2
            v5 = unchecked_mul v1, i8 127
            jmpif v0 then: b1, else: b2
          b1():
            v6 = unchecked_mul v1, i8 127
            v7 = cast v6 as u16
            v8 = truncate v7 to 8 bits, max_bit_size: 16
            v9 = cast v8 as i8
            store v9 at v2
            jmp b2()
          b2():
            v10 = cast v5 as u16
            jmpif v0 then: b3, else: b4
          b3():
            v11 = cast v5 as u16
            v12 = truncate v11 to 8 bits, max_bit_size: 16
            v13 = cast v12 as i8
            store v13 at v2
            jmp b4()
          b4():
            jmpif v0 then: b5, else: b6
          b5():
            v14 = truncate v10 to 8 bits, max_bit_size: 16
            v15 = cast v14 as i8
            store v15 at v2
            jmp b6()
          b6():
            v16 = load v2 -> i8
            return v16
        }
        ");

        // Now with revisit.
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants(DEFAULT_MAX_ITER);

        // All duplicates hoisted into b0.
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: i8):
            v2 = allocate -> &mut i8
            store i8 0 at v2
            v5 = unchecked_mul v1, i8 127
            v6 = cast v5 as u16
            v7 = truncate v6 to 8 bits, max_bit_size: 16
            v8 = cast v7 as i8
            jmpif v0 then: b1, else: b2
          b1():
            store v8 at v2
            jmp b2()
          b2():
            jmpif v0 then: b3, else: b4
          b3():
            store v8 at v2
            jmp b4()
          b4():
            jmpif v0 then: b5, else: b6
          b5():
            store v8 at v2
            jmp b6()
          b6():
            v9 = load v2 -> i8
            return v9
        }
        ");
    }

    #[test]
    fn avoid_unmapped_instructions_during_revisit() {
        // This SSA is based on the `lambda_from_array` integration test, with the Noir code simplified to,
        // and then some extra blocks inserted manually:
        //
        // unconstrained fn main(x: u32) -> pub (str<3>, str<3>) {
        //     let a = lambdas_in_array_literal(x - 1);
        //     let b = lambdas_in_array_literal(x);
        //     (a, b)
        // }
        // unconstrained fn lambdas_in_array_literal(x: u32) -> str<3> {
        //     let xs = [|| "ABC", || "DEF"];
        //     (xs[x])()
        // }
        let src = r#"
          brillig(inline) predicate_pure fn main f0 {
            b0(v0: u32):
              v8 = sub v0, u32 1
              v13 = make_array [Field 2, Field 3, Field 4, Field 5] : [(Field, Field); 2]
              v15 = lt v8, u32 2
              constrain v15 == u1 1, "Index out of bounds"
              v17 = unchecked_mul v8, u32 2
              v18 = unchecked_add v17, u32 1
              v19 = array_get v13, index v18 -> Field
              v20 = eq v19, Field 2
              jmpif v20 then: b1, else: b2
            b1():
              v32 = make_array b"ABC"
              jmp b3(v32)
            b2():
              v21 = eq v19, Field 3
              jmpif v21 then: b4, else: b5
            b3(v1: [u8; 3]):
              v33 = make_array [Field 2, Field 3, Field 4, Field 5] : [(Field, Field); 2]
              v34 = lt v0, u32 2
              constrain v34 == u1 1, "Index out of bounds"
              v35 = unchecked_mul v0, u32 2
              v36 = unchecked_add v35, u32 1
              v37 = array_get v33, index v36 -> Field
              v38 = eq v37, Field 2
              jmpif v38 then: b6, else: b7
            b4():
              v31 = make_array b"ABC"
              jmp b8(v31)
            b5():
              v22 = eq v19, Field 4
              jmpif v22 then: b9, else: b10
            b6():
              v44 = make_array b"ABC"
              jmp b11(v44)
            b7():
              v39 = eq v37, Field 3
              jmpif v39 then: b12, else: b13
            b8(v2: [u8; 3]):
              jmp b3(v2)
            b9():
              v27 = make_array b"DEF"
              jmp b19()
            b19():
              inc_rc v27
              jmp b14(v27)
            b10():
              constrain v19 == Field 5
              v26 = make_array b"DEF"
              jmp b20()
            b20():
              inc_rc v26
              jmp b14(v26)
            b11(v3: [u8; 3]):
              return v1, v3
            b12():
              v43 = make_array b"ABC"
              jmp b15(v43)
            b13():
              v40 = eq v37, Field 4
              jmpif v40 then: b16, else: b17
            b14(v4: [u8; 3]):
              jmp b8(v4)
            b15(v5: [u8; 3]):
              jmp b11(v5)
            b16():
              v42 = make_array b"DEF"
              jmp b18(v42)
            b17():
              constrain v37 == Field 5
              v41 = make_array b"DEF"
              jmp b18(v41)
            b18(v6: [u8; 3]):
              jmp b15(v6)
          }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants(DEFAULT_MAX_ITER);

        // The hoisting of "DEF" will happen in multiple stages:
        // * Appears first in b9
        // * Duplicate of b9 in b10 -> hoisted from b10 to b5
        // * Duplicate of b5 in b16 -> hoisted from b16 to b0
        // * Duplicate of b0 in b17 -> reused from b0
        // * Find the common dominator of [b10, b5, b16, b0, b17]
        // * Start another loop from b0
        // The crucial bit is that b9 and b10 has to be revisited as well, as they contain a reuse from b5,
        // which needs to be updated to point at b0 instead, otherwise trying to normalize the IDs will panic.
        // Also b19 and b20: they refer to values in b9 and b10, and so if we revisit those and update the IDs
        // after hoisting from b5 to b0, we also have to revisit their successors, even though they did not
        // interact with the cache per-se, we have to run resolution again.

        // All make_array hoisted into b0
        assert_ssa_snapshot!(ssa, @r#"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v8 = sub v0, u32 1
            v13 = make_array [Field 2, Field 3, Field 4, Field 5] : [(Field, Field); 2]
            v15 = lt v8, u32 2
            constrain v15 == u1 1, "Index out of bounds"
            v17 = unchecked_mul v8, u32 2
            v18 = unchecked_add v17, u32 1
            v19 = array_get v13, index v18 -> Field
            v20 = eq v19, Field 2
            v24 = make_array b"ABC"
            v28 = make_array b"DEF"
            jmpif v20 then: b1, else: b2
          b1():
            inc_rc v24
            jmp b3(v24)
          b2():
            v29 = eq v19, Field 3
            jmpif v29 then: b4, else: b5
          b3(v1: [u8; 3]):
            inc_rc v13
            v31 = lt v0, u32 2
            constrain v31 == u1 1, "Index out of bounds"
            v32 = unchecked_mul v0, u32 2
            v33 = unchecked_add v32, u32 1
            v34 = array_get v13, index v33 -> Field
            v35 = eq v34, Field 2
            jmpif v35 then: b6, else: b7
          b4():
            inc_rc v24
            jmp b8(v24)
          b5():
            v30 = eq v19, Field 4
            inc_rc v28
            jmpif v30 then: b9, else: b11
          b6():
            inc_rc v24
            jmp b13(v24)
          b7():
            v36 = eq v34, Field 3
            jmpif v36 then: b14, else: b15
          b8(v2: [u8; 3]):
            jmp b3(v2)
          b9():
            inc_rc v28
            jmp b10()
          b10():
            inc_rc v28
            jmp b16(v28)
          b11():
            constrain v19 == Field 5
            inc_rc v28
            jmp b12()
          b12():
            inc_rc v28
            jmp b16(v28)
          b13(v3: [u8; 3]):
            return v1, v3
          b14():
            inc_rc v24
            jmp b17(v24)
          b15():
            v37 = eq v34, Field 4
            jmpif v37 then: b18, else: b19
          b16(v4: [u8; 3]):
            jmp b8(v4)
          b17(v5: [u8; 3]):
            jmp b13(v5)
          b18():
            inc_rc v28
            jmp b20(v28)
          b19():
            constrain v34 == Field 5
            inc_rc v28
            jmp b20(v28)
          b20(v6: [u8; 3]):
            jmp b17(v6)
        }
        "#);
    }

    #[test]
    fn revisit_block_which_dominates_cache() {
        // This test demonstrates how we can, during a follow-up iteration,
        // visit blocks in an order where we see a cached instruction in
        // an origin that the current block dominates.
        let src = r#"
          brillig(inline) predicate_pure fn main f0 {
            b0(v0: u1):
              v1 = make_array [u8 0]: [u8; 1] // this array appears multiple times
              v2 = allocate -> &mut [u8; 1]
              store v1 at v2                  // removes v1 from the cache
              jmp b1(u32 0)
            b1(v3: u32):                      // loop header
              v4 = make_array [u8 0]: [u8; 1] // cannot be deduplicated with v1, it's not in the cache
              v5 = array_set v4, index u32 0, value u8 1  // removes v3 from the cache
              v6 = lt v3, u32 5
              jmpif v6 then: b2, else: b6     // iterate the body or exit
            b2():                             // loop body
              jmpif v0 then: b3, else: b4     // if-then-else with then and else sharing instructions
            b3():
              v7 = make_array [u8 0]: [u8; 1] // v3 not in cache; stays in place
              jmp b5()
            b4():
              v8 = make_array [u8 0]: [u8; 1] // duplicate of v7; hoisted into b2 which dominates b3 and b4
              jmp b5()
            b5():
              v9 = unchecked_add v3, u32 1
              jmp b1(v9)                      // loop back-edge
            b6():
              return
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants(DEFAULT_MAX_ITER);

        // In the 2nd iteration we will restart from b2, which we hoisted into,
        // and revisit b3, b4 and b5, then its successor b1, which will see the
        // make_array now exists in b2.
        // Because we used `array_set` in this example, constant folding figured
        // out that it can create a new array with the updated contents, so we
        // could actually deduplicate the one in b2 with that in b1, but currently
        // we decided we won't be rescheduling a visit to b1, so b2 is not visited
        // again to see this opportunity.

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            v3 = make_array [u8 0] : [u8; 1]
            v4 = allocate -> &mut [u8; 1]
            store v3 at v4
            jmp b1(u32 0)
          b1(v1: u32):
            v6 = make_array [u8 0] : [u8; 1]
            v8 = make_array [u8 1] : [u8; 1]
            v10 = lt v1, u32 5
            jmpif v10 then: b2, else: b6
          b2():
            v11 = make_array [u8 0] : [u8; 1]
            jmpif v0 then: b3, else: b4
          b3():
            inc_rc v11
            jmp b5()
          b4():
            inc_rc v11
            jmp b5()
          b5():
            v13 = unchecked_add v1, u32 1
            jmp b1(v13)
          b6():
            return
        }
        ");
    }

    #[test]
    fn start_revisit_from_common_dominator() {
        // This test simulates a phenomenon from `tests::previous_kernel_validator_builder::validate_common::private_log_length_exceeds_max__private_tail`
        // in private_kernel_lib in aztec-packages/noir-projects/noir-protocol-circuits where starting a second iteration from multiple blocks
        // lead to instructions going unmapped, resulting in:
        //   internal error: entered unreachable code: Unmapped value with id v12: Instruction { instruction: Id(8), position: 0, typ: Array([Numeric(Unsigned { bit_size: 8 })], 1) }
        //                   from instruction: MakeArray { elements: [Id(4)], typ: Array([Numeric(Unsigned { bit_size: 8 })], 1) }, from function: f0
        //
        // Say have a CFG like this:
        //                                 b7
        //                                /  \
        //     b1 - b2 - b3 - b4 - b5 - b6    b9
        //    /                           \  /  \         b18
        //   /                             b8   \        /   \
        // b0                                   b16 - b17     b20
        //   \     b11                          /        \   /
        //    \   /   \                        /          b19
        //     b10     b13 - b14 - b15 -------/
        //        \   /
        //         b12
        //
        // The idea to provoke the error is as follows:
        // * First iteration:
        //   * We will have 3 hoists of the different make_arrays, going into b6, b10 and b17.
        //   * We have `make_array [u8 0]` appear in b11, b7 and b16, but they don't get deduplicated because:
        //     * We visit b11 first, but the array is removed from the cache in b15, because it's stored into a ref
        //     * Then we visit b16 where it's created, but immediately removed from the cache because it's stored
        //     * Finally we visit b7 because of the long delay getting from b1 to b6, and nothing happens as it's not cached.
        // * Second iteration:
        //   * We start from b6, b10 and b17, as none of them dominate each other
        //   * We reach b7 first this time and cache the instruction
        //   * We reach b11 next, and hoist the instruction into b0
        //   * Due to the delay getting from b13 to b15, it remains cached in b0
        //   * We reach b16 via b9 and reuse the results from b0
        //   * We reach b15 and remove the instruction from the cache, but it's too late, b16 was changed.
        //   * We don't revisit b17 after updating b16, so the `inc_rc` which referred to the one in b16
        //     is not updated to point at b0, and leads to the error during normalization.

        let src = r#"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: u1):
            jmpif v0 then: b1, else: b10
          b1():
            jmp b2()
          b2():
            jmp b3()
          b3():
            jmp b4()
          b4():
            jmp b5()
          b5():
            jmp b6()
          b6():
            jmpif v1 then: b7, else: b8
          b7():
            v2 = make_array [u8 0] : [u8; 1]
            v3 = make_array [u8 2] : [u8; 1]
            jmp b9()
          b8():
            v4 = make_array [u8 2] : [u8; 1]
            jmp b9()
          b9():
            jmp b16()
          b10():
            jmpif v1 then: b11, else: b12
          b11():
            v5 = make_array [u8 0] : [u8; 1]
            v7 = make_array [u8 1] : [u8; 1]
            jmp b13()
          b12():
            v8 = make_array [u8 1] : [u8; 1]
            jmp b13()
          b13():
            jmp b14()
          b14():
            jmp b15()
          b15():
            v6 = allocate -> &mut [u8; 1]
            store v5 at v6
            jmp b16()
          b16():
            v9 = make_array [u8 0] : [u8; 1]
            v10 = allocate -> &mut [u8; 1]
            store v9 at v10
            jmp b17()
          b17():
            inc_rc v9
            jmpif v1 then: b18, else: b19
          b18():
            v11 = make_array [u8 3] : [u8; 1]
            jmp b20()
          b19():
            v12 = make_array [u8 3] : [u8; 1]
            jmp b20()
          b20():
            return
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants(2);

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: u1, v1: u1):
            v3 = make_array [u8 0] : [u8; 1]
            jmpif v0 then: b1, else: b10
          b1():
            jmp b2()
          b2():
            jmp b3()
          b3():
            jmp b4()
          b4():
            jmp b5()
          b5():
            jmp b6()
          b6():
            v8 = make_array [u8 2] : [u8; 1]
            jmpif v1 then: b7, else: b8
          b7():
            v9 = make_array [u8 0] : [u8; 1]
            inc_rc v8
            jmp b9()
          b8():
            inc_rc v8
            jmp b9()
          b9():
            jmp b16()
          b10():
            v5 = make_array [u8 1] : [u8; 1]
            jmpif v1 then: b11, else: b12
          b11():
            inc_rc v3
            inc_rc v5
            jmp b13()
          b12():
            inc_rc v5
            jmp b13()
          b13():
            jmp b14()
          b14():
            jmp b15()
          b15():
            v6 = allocate -> &mut [u8; 1]
            store v3 at v6
            jmp b16()
          b16():
            inc_rc v3
            v10 = allocate -> &mut [u8; 1]
            store v3 at v10
            jmp b17()
          b17():
            inc_rc v3
            v12 = make_array [u8 3] : [u8; 1]
            jmpif v1 then: b18, else: b19
          b18():
            inc_rc v12
            jmp b20()
          b19():
            inc_rc v12
            jmp b20()
          b20():
            return
        }
        ");
    }

    #[test]
    fn inlines_brillig_call_without_arguments() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v0 = call f1() -> Field
                return v0
            }

            brillig(inline) fn one f1 {
              b0():
                v0 = add Field 2, Field 3
                return v0
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.fold_constants_with_brillig(MIN_ITER);
        let ssa = ssa.remove_unreachable_functions();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            return Field 5
        }
        ");
    }

    #[test]
    fn inlines_brillig_call_with_two_field_arguments() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v0 = call f1(Field 2, Field 3) -> Field
                return v0
            }

            brillig(inline) fn one f1 {
              b0(v0: Field, v1: Field):
                v2 = add v0, v1
                return v2
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.fold_constants_with_brillig(MIN_ITER);
        let ssa = ssa.remove_unreachable_functions();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            return Field 5
        }
        ");
    }

    #[test]
    fn inlines_brillig_call_with_two_i32_arguments() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v0 = call f1(i32 2, i32 3) -> i32
                return v0
            }

            brillig(inline) fn one f1 {
              b0(v0: i32, v1: i32):
                v2 = unchecked_add v0, v1
                v3 = truncate v2 to 32 bits, max_bit_size: 33
                return v3
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.fold_constants_with_brillig(MIN_ITER);
        let ssa = ssa.remove_unreachable_functions();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            return i32 5
        }
        ");
    }

    #[test]
    fn inlines_brillig_call_with_array_return() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v0 = call f1(Field 2, Field 3, Field 4) -> [Field; 3]
                return v0
            }

            brillig(inline) fn one f1 {
              b0(v0: Field, v1: Field, v2: Field):
                v3 = make_array [v0, v1, v2] : [Field; 3]
                return v3
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.fold_constants_with_brillig(MIN_ITER);
        let ssa = ssa.remove_unreachable_functions();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v3 = make_array [Field 2, Field 3, Field 4] : [Field; 3]
            return v3
        }
        ");
    }

    #[test]
    fn inlines_brillig_call_with_composite_array_return() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v0 = call f1(Field 2, i32 3, Field 4, i32 5) -> [(Field, i32); 2]
                return v0
            }

            brillig(inline) fn one f1 {
              b0(v0: Field, v1: i32, v2: Field, v3: i32):
                v4 = make_array [v0, v1, v2, v3] : [(Field, i32); 2]
                return v4
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.fold_constants_with_brillig(MIN_ITER);
        let ssa = ssa.remove_unreachable_functions();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v4 = make_array [Field 2, i32 3, Field 4, i32 5] : [(Field, i32); 2]
            return v4
        }
        ");
    }

    #[test]
    fn inlines_brillig_call_with_array_arguments() {
        let src = "
            acir(inline) fn main f0 {
              b0():
                v0 = make_array [Field 2, Field 3] : [Field; 2]
                v1 = call f1(v0) -> Field
                return v1
            }

            brillig(inline) fn one f1 {
              b0(v0: [Field; 2]):
                inc_rc v0
                v2 = array_get v0, index u32 0 -> Field
                v4 = array_get v0, index u32 1 -> Field
                v5 = add v2, v4
                dec_rc v0
                return v5
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants_with_brillig(MIN_ITER);
        let ssa = ssa.remove_unreachable_functions();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v2 = make_array [Field 2, Field 3] : [Field; 2]
            return Field 5
        }
        ");
    }

    #[test]
    fn inlines_brillig_call_with_entry_point_globals() {
        let src = "
        g0 = Field 2

        acir(inline) fn main f0 {
          b0():
            v1 = call f1() -> Field
            return v1
        }

        brillig(inline) fn one f1 {
          b0():
            v1 = add g0, Field 3
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.fold_constants_with_brillig(MIN_ITER);
        let ssa = ssa.remove_unreachable_functions();
        assert_ssa_snapshot!(ssa, @r"
        g0 = Field 2

        acir(inline) fn main f0 {
          b0():
            return Field 5
        }
        ");
    }

    #[test]
    fn inlines_brillig_call_with_non_entry_point_globals() {
        let src = "
        g0 = Field 2

        acir(inline) fn main f0 {
          b0():
            v1 = call f1() -> Field
            return v1
        }

        brillig(inline) fn entry_point f1 {
          b0():
            v1 = call f2() -> Field
            return v1
        }

        brillig(inline) fn one f2 {
          b0():
            v1 = add g0, Field 3
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.fold_constants_with_brillig(MIN_ITER);
        let ssa = ssa.remove_unreachable_functions();
        assert_ssa_snapshot!(ssa, @r"
        g0 = Field 2

        acir(inline) fn main f0 {
          b0():
            return Field 5
        }
        ");
    }

    #[test]
    fn does_not_use_cached_constrain_in_block_that_is_not_dominated() {
        // Here v1 in b2 was incorrectly determined to be equal to `Field 1` in the past
        // because of the constrain in b1. However, b2 is not dominated by b1 so this
        // assumption is not valid.
        let src = "
            brillig(inline) fn main f0 {
              b0(v0: Field, v1: Field):
                v3 = eq v0, Field 0
                jmpif v3 then: b1, else: b2
              b1():
                v5 = eq v1, Field 1
                constrain v1 == Field 1
                jmp b2()
              b2():
                v6 = eq v1, Field 0
                constrain v1 == Field 0
                return
            }
            ";
        assert_ssa_does_not_change(src, |ssa| ssa.fold_constants_using_constraints(MIN_ITER));
    }

    #[test]
    fn does_not_hoist_constrain_to_common_ancestor() {
        let src = "
            brillig(inline) fn main f0 {
              b0(v0: Field, v1: Field):
                v2 = eq v0, Field 0
                jmpif v2 then: b1, else: b2
              b1():
                constrain v1 == Field 1
                jmp b2()
              b2():
                v3 = eq v0, Field 1
                jmpif v3 then: b3, else: b4
              b3():
                constrain v1 == Field 1 // This was incorrectly hoisted to b0 but this condition is not valid when going b0 -> b2 -> b4
                jmp b4()
              b4():
                return
            }
            ";
        assert_ssa_does_not_change(src, |ssa| ssa.fold_constants_using_constraints(MIN_ITER));
    }

    #[test]
    fn does_not_hoist_sub_to_common_ancestor() {
        let src = "
            acir(inline) fn main f0 {
              b0(v0: u32):
                v2 = eq v0, u32 0
                jmpif v2 then: b4, else: b1
              b1():
                v3 = eq v0, u32 1
                jmpif v3 then: b3, else: b2
              b2():
                jmp b5()
              b3():
                v5 = sub v0, u32 1 // We can't hoist this because v0 is zero here and it will lead to an underflow
                jmp b5()
              b4():
                v4 = sub v0, u32 1
                jmp b5()
              b5():
                return
            }
            ";
        assert_ssa_does_not_change(src, |ssa| ssa.fold_constants_using_constraints(MIN_ITER));
    }

    #[test]
    fn deduplicates_side_effecting_intrinsics() {
        let src = "
        // After EnableSideEffectsIf removal:
        brillig(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: u1):
            v7 = call to_be_radix(v0, u32 256) -> [u8; 1]    // `a.to_be_radix(256)`;
            inc_rc v7
            v8 = call to_be_radix(v0, u32 256) -> [u8; 1]    // duplicate load of `a`
            inc_rc v8
            v9 = cast v2 as Field                            // `if c { a.to_be_radix(256) }`
            v10 = mul v0, v9                                 // attaching `c` to `a`
            v11 = call to_be_radix(v10, u32 256) -> [u8; 1]  // calling `to_radix(c * a)`
            inc_rc v11
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.fold_constants_using_constraints(MIN_ITER);
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: u1):
            v5 = call to_be_radix(v0, u32 256) -> [u8; 1]
            inc_rc v5
            inc_rc v5
            inc_rc v5
            v6 = cast v2 as Field
            v7 = mul v0, v6
            v8 = call to_be_radix(v7, u32 256) -> [u8; 1]
            inc_rc v8
            return
        }
        ");
    }

    #[test]
    fn array_get_from_array_set_with_different_predicates() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: u32, v2: Field):
            enable_side_effects u1 0
            v4 = array_set v0, index v1, value v2
            enable_side_effects u1 1
            v6 = array_get v4, index v1 -> Field
            return v6
        }
        ";
        assert_ssa_does_not_change(src, |ssa| ssa.fold_constants_using_constraints(MIN_ITER));
    }

    #[test]
    fn array_get_from_array_set_same_predicates() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: u32, v2: Field):
            enable_side_effects u1 1
            v4 = array_set v0, index v1, value v2
            v6 = array_get v4, index v1 -> Field
            return v6
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.fold_constants_using_constraints(MIN_ITER);
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: u32, v2: Field):
            enable_side_effects u1 1
            v4 = array_set v0, index v1, value v2
            return v2
        }
        ");
    }

    #[test]
    fn pure_call_is_deduplicated() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v1 = call f1(v0) -> Field
            v2 = call f1(v0) -> Field
            constrain v1 == Field 0
            constrain v2 == Field 0
            return
        }
        acir(inline) fn foo f1 {
          b0(v0: Field):
            return v0
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis().fold_constants_using_constraints(MIN_ITER);
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            v2 = call f1(v0) -> Field
            constrain v2 == Field 0
            return
        }
        acir(inline) pure fn foo f1 {
          b0(v0: Field):
            return v0
        }
        ");
    }

    #[test]
    fn does_not_deduplicate_field_divisions_under_different_predicates() {
        // Regression test for https://github.com/noir-lang/noir/issues/7283
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: u1):
            enable_side_effects v2
            v3 = div v1, v0
            v4 = mul v3, v0
            v5 = not v2
            enable_side_effects v5
            v6 = div v1, v0
            return
        }
        ";
        assert_ssa_does_not_change(src, |ssa| ssa.fold_constants(MIN_ITER));
    }

    #[test]
    fn does_not_deduplicate_unsigned_divisions_under_different_predicates() {
        // Regression test for https://github.com/noir-lang/noir/issues/7283
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u32, v2: u1):
            enable_side_effects v2
            v3 = div v1, v0
            v4 = not v2
            enable_side_effects v4
            v5 = div v1, v0
            return
        }
        ";
        assert_ssa_does_not_change(src, |ssa| ssa.fold_constants(MIN_ITER));
    }

    #[test]
    fn does_not_deduplicate_signed_divisions_under_different_predicates() {
        // Regression test for https://github.com/noir-lang/noir/issues/7283
        let src = "
        acir(inline) fn main f0 {
          b0(v0: i32, v1: i32, v2: u1):
            enable_side_effects v2
            v3 = div v1, v0
            v4 = not v2
            enable_side_effects v4
            v5 = div v1, v0
            return
        }
        ";
        assert_ssa_does_not_change(src, |ssa| ssa.fold_constants(MIN_ITER));
    }

    #[test]
    fn does_not_deduplicate_unsigned_division_by_zero_constant() {
        // Regression test for https://github.com/noir-lang/noir/issues/7283
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u32, v2: u1):
            enable_side_effects v2
            v4 = div v1, u32 0
            v5 = not v2
            enable_side_effects v5
            v6 = div v1, u32 0
            return
        }
        ";
        assert_ssa_does_not_change(src, |ssa| ssa.fold_constants(MIN_ITER));
    }

    #[test]
    fn does_not_duplicate_unsigned_division_by_non_zero_constant() {
        // Regression test for https://github.com/noir-lang/noir/issues/7836
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u32, v2: u1):
            enable_side_effects v2
            v4 = div v1, u32 2
            v5 = not v2
            enable_side_effects v5
            v6 = div v1, u32 2
            return
        }
        ";
        assert_ssa_does_not_change(src, |ssa| ssa.fold_constants(MIN_ITER));
    }

    #[test]
    fn do_not_inline_brillig_overflow() {
        // Regression test for https://github.com/noir-lang/noir/issues/9694
        // The call can be constant
        let src = "
            acir(inline) predicate_pure fn main f0 {
            b0():
                v2 = call f1(u1 0) -> u1
                return v2
            }
            brillig(inline) predicate_pure fn func_5 f1 {
            b0(v0: u1):
                v2 = shl v0, u1 1
                return v2
            }
        ";
        assert_ssa_does_not_change(src, |ssa| ssa.fold_constants_using_constraints(MIN_ITER));
    }

    #[test]
    fn does_not_deduplicate_calls_to_functions_which_differ_in_return_value_types() {
        // We have a few intrinsics which have a generic return value (generally for array lengths), we want
        // to avoid deduplicating these.
        //
        // This is not an issue for user code as these functions will be monomorphized whereas intrinsics haven't been.
        let src = "
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            v1 = call to_le_radix(v0, u32 256) -> [u8; 2]
            v2 = call to_le_radix(v0, u32 256) -> [u8; 3]
            v3 = call to_le_radix(v0, u32 256) -> [u8; 3]
            v4 = call to_le_radix(v0, u32 256) -> [u8; 2]
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        // These intrinsic calls can only be deduplicated when using constraints.
        let ssa = ssa.fold_constants_using_constraints(MIN_ITER);

        // Only the first one is cached at the moment.
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: Field):
            v3 = call to_le_radix(v0, u32 256) -> [u8; 2]
            v4 = call to_le_radix(v0, u32 256) -> [u8; 3]
            v5 = call to_le_radix(v0, u32 256) -> [u8; 3]
            inc_rc v3
            return
        }
        ");
    }

    #[test]
    fn constant_fold_terminator_argument_from_constrain() {
        // The only instructions advising simplifications for v0 are
        // constrain instructions. We want to make sure that those simplifications
        // are still used for any terminator arguments.
        let src = "
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: Field, v1: Field):
            v5 = eq v0, Field 1
            constrain v0 == Field 1
            v7 = eq v1, Field 0
            constrain v1 == Field 0
            v8 = truncate v0 to 32 bits, max_bit_size: 254
            v9 = cast v8 as u32
            v11 = eq v9, u32 0
            jmpif v11 then: b1, else: b2
          b1():
            v13 = add v0, Field 1
            jmp b3(v0, v13)
          b2():
            v12 = add v0, Field 1
            jmp b3(v12, v0)
          b3(v2: Field, v3: Field):
            v14 = add v0, Field 1
            v15 = eq v2, v14
            constrain v2 == v14
            v16 = eq v3, v0
            constrain v3 == v0
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants_using_constraints(MIN_ITER);

        // The terminators of b1 and b2 should now have constant arguments
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: Field, v1: Field):
            v5 = eq v0, Field 1
            constrain v0 == Field 1
            v7 = eq v1, Field 0
            constrain v1 == Field 0
            jmpif u1 0 then: b1, else: b2
          b1():
            jmp b3(Field 1, Field 2)
          b2():
            jmp b3(Field 2, Field 1)
          b3(v2: Field, v3: Field):
            v10 = eq v2, Field 2
            constrain v2 == Field 2
            v11 = eq v3, Field 1
            constrain v3 == Field 1
            return
        }
        ");
    }

    #[test]
    fn functions_returning_arrays_inc_rc_while_deduplicating() {
        // Regression test for an issue discovered in https://github.com/AztecProtocol/aztec-packages/pull/14492
        // Previously no `inc_rc` was being generated when deduplicating the calls to `f1`,
        // resulting in both references mutating the same array as opposed to having their own copies.
        let src = r#"
        brillig(inline) impure fn constructor f0 {
          b0():
            v8 = make_array [Field 0, Field 0, Field 0] : [Field; 3]
            v23 = call f1() -> [Field; 4]
            v26 = allocate -> &mut [Field; 3]
            store v8 at v26
            v27 = allocate -> &mut [Field; 4]
            store v23 at v27
            v28 = allocate -> &mut u32
            store u32 0 at v28
            call f2(v26, v27, v28, Field 13)
            call f2(v26, v27, v28, Field 0)
            call f2(v26, v27, v28, Field 1)
            v42 = load v26 -> [Field; 3]
            v36 = load v28 -> u32
            call f4(v42, v27, v36)
            v31 = call f1() -> [Field; 4]
            v35 = allocate -> &mut [Field; 4]
            store v31 at v35
            call f4(v8, v35, u32 0)
            return v35
        }
        brillig(inline) predicate_pure fn new f1 {
          b0():
            v7 = make_array [Field 0, Field 0, Field 0, Field 55340232221128654848] : [Field; 4]
            return v7
        }
        brillig(inline) impure fn absorb f2 {
          b0(v4: &mut [Field; 3], v5: &mut [Field; 4], v6: &mut u32, v8: Field):
            v13 = load v6 -> u32
            v14 = load v4 -> [Field; 3]
            v15 = load v5 -> [Field; 4]
            v17 = lt v13, u32 3
            constrain v17 == u1 1, "Index out of bounds"
            v19 = array_set v14, index v13, value v8
            v21 = add v13, u32 1
            store v19 at v4
            store v15 at v5
            store v21 at v6
            return
        }
        brillig(inline) impure fn perform_duplex f4 {
          b0(v4: [Field; 3], v5: &mut [Field; 4], v18: u32):
            jmp b1(u32 0)
          b1(v8: u32):
            v10 = lt v8, u32 3
            jmpif v10 then: b2, else: b3
          b2():
            v19 = lt v8, v18
            jmpif v19 then: b4, else: b5
          b3():
            v11 = load v5 -> [Field; 4]
            inc_rc v11
            v14 = call poseidon2_permutation(v11) -> [Field; 4]
            store v14 at v5
            return
          b4():
            v20 = load v5 -> [Field; 4]
            v21 = array_get v20, index v8 -> Field
            v23 = array_get v4, index v8 -> Field
            v24 = add v21, v23
            v27 = array_set v20, index v8, value v24
            store v27 at v5
            jmp b5()
          b5():
            v29 = unchecked_add v8, u32 1
            jmp b1(v29)
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();

        let (_, execution_result) = assert_pass_does_not_affect_execution(ssa, vec![], |ssa| {
            ssa.fold_constants_using_constraints(MIN_ITER)
        });

        assert!(execution_result.is_ok());
    }

    // Regression for #9451
    #[test]
    fn do_not_deduplicate_call_with_inc_rc() {
        // This test ensures that a function which mutates an array pointer is marked impure.
        // This protects against future deduplication passes incorrectly assuming purity.
        // The increasing RC numbers reflect the current expectation that the RC of the
        // original array does not get decremented when a copy is made.
        let src = r#"
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v3 = make_array [Field 1, Field 2] : [Field; 2]
            v5 = call array_refcount(v3) -> u32
            constrain v5 == u32 1
            v8 = call f1(v3) -> [Field; 2]
            v9 = call array_refcount(v3) -> u32
            constrain v9 == u32 2
            v11 = call f1(v3) -> [Field; 2]
            v12 = call array_refcount(v3) -> u32
            constrain v12 == u32 3
            inc_rc v3
            v15 = array_set v3, index v0, value Field 9
            return v3, v15
        }
        brillig(inline) fn mutator f1 {
          b0(v0: [Field; 2]):
            inc_rc v0
            v3 = array_set v0, index u32 0, value Field 5
            return v3
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let inputs = vec![Value::from_constant(1_u32.into(), NumericType::unsigned(32)).unwrap()];

        let (ssa, _) =
            assert_pass_does_not_affect_execution(ssa, inputs.clone(), |ssa| ssa.purity_analysis());

        let (_, execution_result) = assert_pass_does_not_affect_execution(ssa, inputs, |ssa| {
            ssa.fold_constants_using_constraints(MIN_ITER)
        });

        assert!(execution_result.is_ok());
    }

    #[test]
    fn do_not_deduplicate_call_with_array_set_brillig() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v2 = make_array [Field 1, Field 2] : [Field; 2]
            call f1(v2, Field 9)
            v7 = array_set v2, index v0, value Field 7
            call f1(v2, Field 9)
            v9 = array_get v2, index v0 -> Field
            constrain v9 == Field 9
            return
        }
        brillig(inline) fn mutator f1 {
          b0(v0: [Field; 2], v1: Field):
            v3 = array_set v0, index u32 0, value v1
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let inputs = vec![Value::from_constant(0_u32.into(), NumericType::unsigned(32)).unwrap()];

        let (ssa, _) =
            assert_pass_does_not_affect_execution(ssa, inputs.clone(), |ssa| ssa.purity_analysis());

        let (_, execution_result) = assert_pass_does_not_affect_execution(ssa, inputs, |ssa| {
            ssa.fold_constants_using_constraints(MIN_ITER)
        });

        assert!(execution_result.is_ok());
    }
}
