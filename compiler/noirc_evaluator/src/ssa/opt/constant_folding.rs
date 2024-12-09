//! The goal of the constant folding optimization pass is to propagate any constants forwards into
//! later [`Instruction`]s to maximize the impact of [compile-time simplifications][Instruction::simplify()].
//!
//! The pass works as follows:
//! - Re-insert each instruction in order to apply the instruction simplification performed
//!   by the [`DataFlowGraph`] automatically as new instructions are pushed.
//! - Check whether any input values have been constrained to be equal to a value of a simpler form
//!   by a [constrain instruction][Instruction::Constrain]. If so, replace the input value with the simpler form.
//! - Check whether the instruction [can_be_deduplicated][Instruction::can_be_deduplicated()]
//!   by duplicate instruction earlier in the same block.
//!
//! These operations are done in parallel so that they can each benefit from each other
//! without the need for multiple passes.
//!
//! Other passes perform a certain amount of constant folding automatically as they insert instructions
//! into the [`DataFlowGraph`] but this pass can become needed if [`DataFlowGraph::set_value`] or
//! [`DataFlowGraph::set_value_from_id`] are used on a value which enables instructions dependent on the value to
//! now be simplified.
//!
//! This is the only pass which removes duplicated pure [`Instruction`]s however and so is needed when
//! different blocks are merged, i.e. after the [`flatten_cfg`][super::flatten_cfg] pass.
use std::collections::{BTreeMap, HashSet, VecDeque};

use acvm::{
    acir::AcirField,
    brillig_vm::{MemoryValue, VMStatus, VM},
    FieldElement,
};
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use im::Vector;
use iter_extended::vecmap;

use crate::{
    brillig::{
        brillig_gen::gen_brillig_for,
        brillig_ir::{artifact::BrilligParameter, brillig_variable::get_bit_size_from_ssa_type},
        Brillig,
    },
    ssa::{
        ir::{
            basic_block::BasicBlockId,
            dfg::{DataFlowGraph, InsertInstructionResult},
            dom::DominatorTree,
            function::{Function, FunctionId, RuntimeType},
            instruction::{Instruction, InstructionId},
            types::Type,
            value::{Value, ValueId},
        },
        ssa_gen::Ssa,
    },
};
use fxhash::FxHashMap as HashMap;

impl Ssa {
    /// Performs constant folding on each instruction.
    ///
    /// It will not look at constraints to inform simplifications
    /// based on the stated equivalence of two instructions.
    ///
    /// See [`constant_folding`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn fold_constants(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.constant_fold(false, None);
        }
        self
    }

    /// Performs constant folding on each instruction.
    ///
    /// Also uses constraint information to inform more optimizations.
    ///
    /// See [`constant_folding`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn fold_constants_using_constraints(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.constant_fold(true, None);
        }
        self
    }

    /// Performs constant folding on each instruction while also replacing calls to brillig functions
    /// with all constant arguments by trying to evaluate those calls.
    #[tracing::instrument(level = "trace", skip(self, brillig))]
    pub(crate) fn fold_constants_with_brillig(mut self, brillig: &Brillig) -> Ssa {
        // Collect all brillig functions so that later we can find them when processing a call instruction
        let mut brillig_functions: BTreeMap<FunctionId, Function> = BTreeMap::new();
        for (func_id, func) in &self.functions {
            if let RuntimeType::Brillig(..) = func.runtime() {
                let cloned_function = Function::clone_with_id(*func_id, func);
                brillig_functions.insert(*func_id, cloned_function);
            };
        }

        let brillig_info = Some(BrilligInfo { brillig, brillig_functions: &brillig_functions });

        for function in self.functions.values_mut() {
            function.constant_fold(false, brillig_info);
        }

        // It could happen that we inlined all calls to a given brillig function.
        // In that case it's unused so we can remove it. This is what we check next.
        self.remove_unused_brillig_functions(brillig_functions)
    }

    fn remove_unused_brillig_functions(
        mut self,
        mut brillig_functions: BTreeMap<FunctionId, Function>,
    ) -> Ssa {
        // Remove from the above map functions that are called
        for function in self.functions.values() {
            for block_id in function.reachable_blocks() {
                for instruction_id in function.dfg[block_id].instructions() {
                    let instruction = &function.dfg[*instruction_id];
                    let Instruction::Call { func: func_id, arguments: _ } = instruction else {
                        continue;
                    };

                    let func_value = &function.dfg[*func_id];
                    let Value::Function(func_id) = func_value else { continue };

                    brillig_functions.remove(func_id);
                }
            }
        }

        // The ones that remain are never called: let's remove them.
        for (func_id, func) in &brillig_functions {
            // We never want to remove the main function (it could be `unconstrained` or it
            // could have been turned into brillig if `--force-brillig` was given).
            // We also don't want to remove entry points.
            let runtime = func.runtime();
            if self.main_id == *func_id
                || (runtime.is_entry_point() && matches!(runtime, RuntimeType::Acir(_)))
            {
                continue;
            }

            self.functions.remove(func_id);
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
        brillig_info: Option<BrilligInfo>,
    ) {
        let mut context = Context::new(use_constraint_info, brillig_info);
        let mut dom = DominatorTree::with_function(self);
        context.block_queue.push_back(self.entry_block());

        while let Some(block) = context.block_queue.pop_front() {
            if context.visited_blocks.contains(&block) {
                continue;
            }

            context.visited_blocks.insert(block);
            context.fold_constants_in_block(self, &mut dom, block);
        }
    }
}

struct Context<'a> {
    use_constraint_info: bool,
    brillig_info: Option<BrilligInfo<'a>>,
    /// Maps pre-folded ValueIds to the new ValueIds obtained by re-inserting the instruction.
    visited_blocks: HashSet<BasicBlockId>,
    block_queue: VecDeque<BasicBlockId>,

    /// Contains sets of values which are constrained to be equivalent to each other.
    ///
    /// The mapping's structure is `side_effects_enabled_var => (constrained_value => simplified_value)`.
    ///
    /// We partition the maps of constrained values according to the side-effects flag at the point
    /// at which the values are constrained. This prevents constraints which are only sometimes enforced
    /// being used to modify the rest of the program.
    constraint_simplification_mappings: ConstraintSimplificationCache,

    // Cache of instructions without any side-effects along with their outputs.
    cached_instruction_results: InstructionResultCache,
}

#[derive(Copy, Clone)]
pub(crate) struct BrilligInfo<'a> {
    brillig: &'a Brillig,
    brillig_functions: &'a BTreeMap<FunctionId, Function>,
}

/// Records a simplified equivalents of an [`Instruction`] in the blocks
/// where the constraint that advised the simplification has been encountered.
///
/// For more information see [`ConstraintSimplificationCache`].
#[derive(Default)]
struct SimplificationCache {
    /// Simplified expressions where we found them.
    ///
    /// It will always have at least one value because `add` is called
    /// after the default is constructed.
    simplifications: HashMap<BasicBlockId, ValueId>,
}

impl SimplificationCache {
    /// Called with a newly encountered simplification.
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

    /// Try to find a simplification in a visible block.
    fn get(&self, block: BasicBlockId, dom: &DominatorTree) -> Option<ValueId> {
        // Deterministically walk up the dominator chain until we encounter a block that contains a simplification.
        dom.find_map_dominator(block, |b| self.simplifications.get(&b).cloned())
    }
}

/// HashMap from `(side_effects_enabled_var, Instruction)` to a simplified expression that it can
/// be replaced with based on constraints that testify to their equivalence, stored together
/// with the set of blocks at which this constraint has been observed.
///
/// Only blocks dominated by one in the cache should have access to this information, otherwise
/// we create a sort of time paradox where we replace an instruction with a constant we believe
/// it _should_ equal to, without ever actually producing and asserting the value.
type ConstraintSimplificationCache = HashMap<ValueId, HashMap<ValueId, SimplificationCache>>;

/// HashMap from `(Instruction, side_effects_enabled_var)` to the results of the instruction.
/// Stored as a two-level map to avoid cloning Instructions during the `.get` call.
///
/// The `side_effects_enabled_var` is optional because we only use them when `Instruction::requires_acir_gen_predicate`
/// is true _and_ the constraint information is also taken into account.
///
/// In addition to each result, the original BasicBlockId is stored as well. This allows us
/// to deduplicate instructions across blocks as long as the new block dominates the original.
type InstructionResultCache = HashMap<Instruction, HashMap<Option<ValueId>, ResultCache>>;

/// Records the results of all duplicate [`Instruction`]s along with the blocks in which they sit.
///
/// For more information see [`InstructionResultCache`].
#[derive(Default)]
struct ResultCache {
    result: Option<(BasicBlockId, Vec<ValueId>)>,
}

impl<'brillig> Context<'brillig> {
    fn new(use_constraint_info: bool, brillig_info: Option<BrilligInfo<'brillig>>) -> Self {
        Self {
            use_constraint_info,
            brillig_info,
            visited_blocks: Default::default(),
            block_queue: Default::default(),
            constraint_simplification_mappings: Default::default(),
            cached_instruction_results: Default::default(),
        }
    }

    fn fold_constants_in_block(
        &mut self,
        function: &mut Function,
        dom: &mut DominatorTree,
        block: BasicBlockId,
    ) {
        let instructions = function.dfg[block].take_instructions();

        // Default side effect condition variable with an enabled state.
        let mut side_effects_enabled_var =
            function.dfg.make_constant(FieldElement::one(), Type::bool());

        for instruction_id in instructions {
            self.fold_constants_into_instruction(
                function,
                dom,
                block,
                instruction_id,
                &mut side_effects_enabled_var,
            );
        }
        self.block_queue.extend(function.dfg[block].successors());
    }

    fn fold_constants_into_instruction(
        &mut self,
        function: &mut Function,
        dom: &mut DominatorTree,
        mut block: BasicBlockId,
        id: InstructionId,
        side_effects_enabled_var: &mut ValueId,
    ) {
        let constraint_simplification_mapping = self.get_constraint_map(*side_effects_enabled_var);
        let dfg = &mut function.dfg;

        let instruction =
            Self::resolve_instruction(id, block, dfg, dom, constraint_simplification_mapping);

        let old_results = dfg.instruction_results(id).to_vec();

        // If a copy of this instruction exists earlier in the block, then reuse the previous results.
        if let Some(cache_result) =
            self.get_cached(dfg, dom, &instruction, *side_effects_enabled_var, block)
        {
            match cache_result {
                CacheResult::Cached(cached) => {
                    // We track whether we may mutate MakeArray instructions before we deduplicate
                    // them but we still need to issue an extra inc_rc in case they're mutated afterward.
                    if matches!(instruction, Instruction::MakeArray { .. }) {
                        let value = *cached.last().unwrap();
                        let inc_rc = Instruction::IncrementRc { value };
                        let call_stack = dfg.get_call_stack(id);
                        dfg.insert_instruction_and_results(inc_rc, block, None, call_stack);
                    }

                    Self::replace_result_ids(dfg, &old_results, cached);
                    return;
                }
                CacheResult::NeedToHoistToCommonBlock(dominator) => {
                    // Just change the block to insert in the common dominator instead.
                    // This will only move the current instance of the instruction right now.
                    // When constant folding is run a second time later on, it'll catch
                    // that the previous instance can be deduplicated to this instance.
                    block = dominator;
                }
            }
        };

        // First try to inline a call to a brillig function with all constant arguments.
        let new_results = Self::try_inline_brillig_call_with_all_constants(
            &instruction,
            &old_results,
            block,
            dfg,
            self.brillig_info,
        )
        // Otherwise, try inserting the instruction again to apply any optimizations using the newly resolved inputs.
        .unwrap_or_else(|| {
            Self::push_instruction(id, instruction.clone(), &old_results, block, dfg)
        });

        Self::replace_result_ids(dfg, &old_results, &new_results);

        self.cache_instruction(
            instruction.clone(),
            new_results,
            function,
            *side_effects_enabled_var,
            block,
        );

        // If we just inserted an `Instruction::EnableSideEffectsIf`, we need to update `side_effects_enabled_var`
        // so that we use the correct set of constrained values in future.
        if let Instruction::EnableSideEffectsIf { condition } = instruction {
            *side_effects_enabled_var = condition;
        };
    }

    /// Fetches an [`Instruction`] by its [`InstructionId`] and fully resolves its inputs.
    fn resolve_instruction(
        instruction_id: InstructionId,
        block: BasicBlockId,
        dfg: &DataFlowGraph,
        dom: &mut DominatorTree,
        constraint_simplification_mapping: &HashMap<ValueId, SimplificationCache>,
    ) -> Instruction {
        let instruction = dfg[instruction_id].clone();

        // Alternate between resolving `value_id` in the `dfg` and checking to see if the resolved value
        // has been constrained to be equal to some simpler value in the current block.
        //
        // This allows us to reach a stable final `ValueId` for each instruction input as we add more
        // constraints to the cache.
        fn resolve_cache(
            block: BasicBlockId,
            dfg: &DataFlowGraph,
            dom: &mut DominatorTree,
            cache: &HashMap<ValueId, SimplificationCache>,
            value_id: ValueId,
        ) -> ValueId {
            let resolved_id = dfg.resolve(value_id);
            match cache.get(&resolved_id) {
                Some(simplification_cache) => {
                    if let Some(simplified) = simplification_cache.get(block, dom) {
                        resolve_cache(block, dfg, dom, cache, simplified)
                    } else {
                        resolved_id
                    }
                }
                None => resolved_id,
            }
        }

        // Resolve any inputs to ensure that we're comparing like-for-like instructions.
        instruction.map_values(|value_id| {
            resolve_cache(block, dfg, dom, constraint_simplification_mapping, value_id)
        })
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

        let call_stack = dfg.get_call_stack(id);
        let new_results =
            match dfg.insert_instruction_and_results(instruction, block, ctrl_typevars, call_stack)
            {
                InsertInstructionResult::SimplifiedTo(new_result) => vec![new_result],
                InsertInstructionResult::SimplifiedToMultiple(new_results) => new_results,
                InsertInstructionResult::Results(_, new_results) => new_results.to_vec(),
                InsertInstructionResult::InstructionRemoved => vec![],
            };
        // Optimizations while inserting the instruction should not change the number of results.
        assert_eq!(old_results.len(), new_results.len());

        new_results
    }

    fn cache_instruction(
        &mut self,
        instruction: Instruction,
        instruction_results: Vec<ValueId>,
        function: &Function,
        side_effects_enabled_var: ValueId,
        block: BasicBlockId,
    ) {
        if self.use_constraint_info {
            // If the instruction was a constraint, then create a link between the two `ValueId`s
            // to map from the more complex to the simpler value.
            if let Instruction::Constrain(lhs, rhs, _) = instruction {
                // These `ValueId`s should be fully resolved now.
                if let Some((complex, simple)) = simplify(&function.dfg, lhs, rhs) {
                    self.get_constraint_map(side_effects_enabled_var)
                        .entry(complex)
                        .or_default()
                        .add(&function.dfg, simple, block);
                }
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
        if let Instruction::ArraySet { index, value, .. } = &instruction {
            let use_predicate =
                self.use_constraint_info && instruction.requires_acir_gen_predicate(&function.dfg);
            let predicate = use_predicate.then_some(side_effects_enabled_var);

            let array_get = Instruction::ArrayGet { array: instruction_results[0], index: *index };

            self.cached_instruction_results
                .entry(array_get)
                .or_default()
                .entry(predicate)
                .or_default()
                .cache(block, vec![*value]);
        }

        self.remove_possibly_mutated_cached_make_arrays(&instruction, function);

        // If the instruction doesn't have side-effects and if it won't interact with enable_side_effects during acir_gen,
        // we cache the results so we can reuse them if the same instruction appears again later in the block.
        // Others have side effects representing failure, which are implicit in the ACIR code and can also be deduplicated.
        let can_be_deduplicated =
            instruction.can_be_deduplicated(function, self.use_constraint_info);

        // We also allow deduplicating MakeArray instructions that we have tracked which haven't
        // been mutated.
        if can_be_deduplicated || matches!(instruction, Instruction::MakeArray { .. }) {
            let use_predicate =
                self.use_constraint_info && instruction.requires_acir_gen_predicate(&function.dfg);
            let predicate = use_predicate.then_some(side_effects_enabled_var);

            self.cached_instruction_results
                .entry(instruction)
                .or_default()
                .entry(predicate)
                .or_default()
                .cache(block, instruction_results);
        }
    }

    /// Get the simplification mapping from complex to simpler instructions,
    /// which all depend on the same side effect condition variable.
    fn get_constraint_map(
        &mut self,
        side_effects_enabled_var: ValueId,
    ) -> &mut HashMap<ValueId, SimplificationCache> {
        self.constraint_simplification_mappings.entry(side_effects_enabled_var).or_default()
    }

    /// Replaces a set of [`ValueId`]s inside the [`DataFlowGraph`] with another.
    fn replace_result_ids(
        dfg: &mut DataFlowGraph,
        old_results: &[ValueId],
        new_results: &[ValueId],
    ) {
        for (old_result, new_result) in old_results.iter().zip(new_results) {
            dfg.set_value_from_id(*old_result, *new_result);
        }
    }

    /// Get a cached result if it can be used in this context.
    fn get_cached(
        &self,
        dfg: &DataFlowGraph,
        dom: &mut DominatorTree,
        instruction: &Instruction,
        side_effects_enabled_var: ValueId,
        block: BasicBlockId,
    ) -> Option<CacheResult> {
        let results_for_instruction = self.cached_instruction_results.get(instruction)?;
        let predicate = self.use_constraint_info && instruction.requires_acir_gen_predicate(dfg);
        let predicate = predicate.then_some(side_effects_enabled_var);

        results_for_instruction.get(&predicate)?.get(block, dom, instruction.has_side_effects(dfg))
    }

    /// Checks if the given instruction is a call to a brillig function with all constant arguments.
    /// If so, we can try to evaluate that function and replace the results with the evaluation results.
    fn try_inline_brillig_call_with_all_constants(
        instruction: &Instruction,
        old_results: &[ValueId],
        block: BasicBlockId,
        dfg: &mut DataFlowGraph,
        brillig_info: Option<BrilligInfo>,
    ) -> Option<Vec<ValueId>> {
        let evaluation_result = Self::evaluate_const_brillig_call(
            instruction,
            brillig_info?.brillig,
            brillig_info?.brillig_functions,
            dfg,
        );

        match evaluation_result {
            EvaluationResult::NotABrilligCall | EvaluationResult::CannotEvaluate(_) => None,
            EvaluationResult::Evaluated(memory_values) => {
                let mut memory_index = 0;
                let new_results = vecmap(old_results, |old_result| {
                    let typ = dfg.type_of_value(*old_result);
                    Self::new_value_for_type_and_memory_values(
                        typ,
                        block,
                        &memory_values,
                        &mut memory_index,
                        dfg,
                    )
                });
                Some(new_results)
            }
        }
    }

    /// Tries to evaluate an instruction if it's a call that points to a brillig function,
    /// and all its arguments are constant.
    /// We do this by directly executing the function with a brillig VM.
    fn evaluate_const_brillig_call(
        instruction: &Instruction,
        brillig: &Brillig,
        brillig_functions: &BTreeMap<FunctionId, Function>,
        dfg: &mut DataFlowGraph,
    ) -> EvaluationResult {
        let Instruction::Call { func: func_id, arguments } = instruction else {
            return EvaluationResult::NotABrilligCall;
        };

        let func_value = &dfg[*func_id];
        let Value::Function(func_id) = func_value else {
            return EvaluationResult::NotABrilligCall;
        };

        let Some(func) = brillig_functions.get(func_id) else {
            return EvaluationResult::NotABrilligCall;
        };

        if !arguments.iter().all(|argument| dfg.is_constant(*argument)) {
            return EvaluationResult::CannotEvaluate(*func_id);
        }

        let mut brillig_arguments = Vec::new();
        for argument in arguments {
            let typ = dfg.type_of_value(*argument);
            let Some(parameter) = type_to_brillig_parameter(&typ) else {
                return EvaluationResult::CannotEvaluate(*func_id);
            };
            brillig_arguments.push(parameter);
        }

        // Check that return value types are supported by brillig
        for return_id in func.returns().iter() {
            let typ = func.dfg.type_of_value(*return_id);
            if type_to_brillig_parameter(&typ).is_none() {
                return EvaluationResult::CannotEvaluate(*func_id);
            }
        }

        let Ok(generated_brillig) = gen_brillig_for(func, brillig_arguments, brillig) else {
            return EvaluationResult::CannotEvaluate(*func_id);
        };

        let mut calldata = Vec::new();
        for argument in arguments {
            value_id_to_calldata(*argument, dfg, &mut calldata);
        }

        let bytecode = &generated_brillig.byte_code;
        let foreign_call_results = Vec::new();
        let black_box_solver = Bn254BlackBoxSolver;
        let profiling_active = false;
        let mut vm =
            VM::new(calldata, bytecode, foreign_call_results, &black_box_solver, profiling_active);
        let vm_status: VMStatus<_> = vm.process_opcodes();
        let VMStatus::Finished { return_data_offset, return_data_size } = vm_status else {
            return EvaluationResult::CannotEvaluate(*func_id);
        };

        let memory =
            vm.get_memory()[return_data_offset..(return_data_offset + return_data_size)].to_vec();

        EvaluationResult::Evaluated(memory)
    }

    /// Creates a new value inside this function by reading it from `memory_values` starting at
    /// `memory_index` depending on the given Type: if it's an array multiple values will be read
    /// and a new `make_array` instruction will be created.
    fn new_value_for_type_and_memory_values(
        typ: Type,
        block_id: BasicBlockId,
        memory_values: &[MemoryValue<FieldElement>],
        memory_index: &mut usize,
        dfg: &mut DataFlowGraph,
    ) -> ValueId {
        match typ {
            Type::Numeric(_) => {
                let memory = memory_values[*memory_index];
                *memory_index += 1;

                let field_value = match memory {
                    MemoryValue::Field(field_value) => field_value,
                    MemoryValue::Integer(u128_value, _) => u128_value.into(),
                };
                dfg.make_constant(field_value, typ)
            }
            Type::Array(types, length) => {
                let mut new_array_values = Vector::new();
                for _ in 0..length {
                    for typ in types.iter() {
                        let new_value = Self::new_value_for_type_and_memory_values(
                            typ.clone(),
                            block_id,
                            memory_values,
                            memory_index,
                            dfg,
                        );
                        new_array_values.push_back(new_value);
                    }
                }

                let instruction = Instruction::MakeArray {
                    elements: new_array_values,
                    typ: Type::Array(types, length),
                };
                let instruction_id = dfg.make_instruction(instruction, None);
                dfg[block_id].instructions_mut().push(instruction_id);
                *dfg.instruction_results(instruction_id).first().unwrap()
            }
            Type::Reference(_) => {
                panic!("Unexpected reference type in brillig function result")
            }
            Type::Slice(_) => {
                panic!("Unexpected slice type in brillig function result")
            }
            Type::Function => {
                panic!("Unexpected function type in brillig function result")
            }
        }
    }

    fn remove_possibly_mutated_cached_make_arrays(
        &mut self,
        instruction: &Instruction,
        function: &Function,
    ) {
        use Instruction::{ArraySet, Store};

        // Should we consider calls to slice_push_back and similar to be mutating operations as well?
        if let Store { value: array, .. } | ArraySet { array, .. } = instruction {
            let instruction = match &function.dfg[*array] {
                Value::Instruction { instruction, .. } => &function.dfg[*instruction],
                _ => return,
            };

            if matches!(instruction, Instruction::MakeArray { .. }) {
                self.cached_instruction_results.remove(instruction);
            }
        }
    }
}

impl ResultCache {
    /// Records that an `Instruction` in block `block` produced the result values `results`.
    fn cache(&mut self, block: BasicBlockId, results: Vec<ValueId>) {
        if self.result.is_none() {
            self.result = Some((block, results));
        }
    }

    /// Returns a set of [`ValueId`]s produced from a copy of this [`Instruction`] which sits
    /// within a block which dominates `block`.
    ///
    /// We require that the cached instruction's block dominates `block` in order to avoid
    /// cycles causing issues (e.g. two instructions being replaced with the results of each other
    /// such that neither instruction exists anymore.)
    fn get(
        &self,
        block: BasicBlockId,
        dom: &mut DominatorTree,
        has_side_effects: bool,
    ) -> Option<CacheResult> {
        self.result.as_ref().and_then(|(origin_block, results)| {
            if dom.dominates(*origin_block, block) {
                Some(CacheResult::Cached(results))
            } else if !has_side_effects {
                // Insert a copy of this instruction in the common dominator
                let dominator = dom.common_dominator(*origin_block, block);
                Some(CacheResult::NeedToHoistToCommonBlock(dominator))
            } else {
                None
            }
        })
    }
}

enum CacheResult<'a> {
    Cached(&'a [ValueId]),
    NeedToHoistToCommonBlock(BasicBlockId),
}

/// Result of trying to evaluate an instruction (any instruction) in this pass.
enum EvaluationResult {
    /// Nothing was done because the instruction wasn't a call to a brillig function,
    /// or some arguments to it were not constants.
    NotABrilligCall,
    /// The instruction was a call to a brillig function, but we couldn't evaluate it.
    /// This can occur in the situation where the brillig function reaches a "trap" or a foreign call opcode.
    CannotEvaluate(FunctionId),
    /// The instruction was a call to a brillig function and we were able to evaluate it,
    /// returning evaluation memory values.
    Evaluated(Vec<MemoryValue<FieldElement>>),
}

/// Similar to FunctionContext::ssa_type_to_parameter but never panics and disallows reference types.
pub(crate) fn type_to_brillig_parameter(typ: &Type) -> Option<BrilligParameter> {
    match typ {
        Type::Numeric(_) => Some(BrilligParameter::SingleAddr(get_bit_size_from_ssa_type(typ))),
        Type::Array(item_type, size) => {
            let mut parameters = Vec::with_capacity(item_type.len());
            for item_typ in item_type.iter() {
                parameters.push(type_to_brillig_parameter(item_typ)?);
            }
            Some(BrilligParameter::Array(parameters, *size as usize))
        }
        _ => None,
    }
}

fn value_id_to_calldata(value_id: ValueId, dfg: &DataFlowGraph, calldata: &mut Vec<FieldElement>) {
    if let Some(value) = dfg.get_numeric_constant(value_id) {
        calldata.push(value);
        return;
    }

    if let Some((values, _type)) = dfg.get_array_constant(value_id) {
        for value in values {
            value_id_to_calldata(value, dfg, calldata);
        }
        return;
    }

    panic!("Expected ValueId to be numeric constant or array constant");
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

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{map::Id, types::Type},
        opt::assert_normalized_ssa_equals,
        Ssa,
    };

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

        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 2); // The final return is not counted

        let v0 = main.parameters()[0];
        let two = main.dfg.make_constant(2_u128.into(), Type::field());

        main.dfg.set_value_from_id(v0, two);

        let expected = "
            acir(inline) fn main f0 {
              b0(v0: Field):
                return Field 9
            }
            ";
        let ssa = ssa.fold_constants();
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

        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 2); // The final return is not counted

        let v1 = main.parameters()[1];

        // Note that this constant guarantees that `v0/constant < 2^8`. We then do not need to truncate the result.
        let constant = 2_u128.pow(8);
        let constant = main.dfg.make_constant(constant.into(), Type::unsigned(16));

        main.dfg.set_value_from_id(v1, constant);

        let expected = "
            acir(inline) fn main f0 {
              b0(v0: u16, v1: u16):
                v3 = div v0, u16 256
                return v3
            }
            ";

        let ssa = ssa.fold_constants();
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

        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 2); // The final return is not counted

        let v1 = main.parameters()[1];

        // Note that this constant does not guarantee that `v0/constant < 2^8`. We must then truncate the result.
        let constant = 2_u128.pow(8) - 1;
        let constant = main.dfg.make_constant(constant.into(), Type::unsigned(16));

        main.dfg.set_value_from_id(v1, constant);

        let expected = "
            acir(inline) fn main f0 {
              b0(v0: u16, v1: u16):
                v3 = div v0, u16 255
                v4 = truncate v3 to 8 bits, max_bit_size: 16
                return v4
            }
            ";

        let ssa = ssa.fold_constants();
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
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants();
        assert_normalized_ssa_equals(ssa, src);
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
        let ssa = ssa.fold_constants();
        assert_normalized_ssa_equals(ssa, expected);
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
              b0(v0: [Field; 4], v1: u32, v2: u1, v3: u1):
                enable_side_effects v2
                v5 = array_get v0, index u32 0 -> Field
                v6 = array_get v0, index v1 -> Field
                enable_side_effects v3
                v7 = array_get v0, index v1 -> Field
                return
            }
            ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants();
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn constraint_decomposition() {
        // When constructing this IR, we should automatically decompose the constraint to be in terms of `v0`, `v1` and `v2`.
        //
        // The mul instructions are retained and will be removed in the dead instruction elimination pass.
        let src = "
            acir(inline) fn main f0 {
              b0(v0: u1, v1: u1, v2: u1):
                v3 = mul v0, v1
                v4 = not v2
                v5 = mul v3, v4
                constrain v5 == u1 1
                return
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let expected = "
            acir(inline) fn main f0 {
              b0(v0: u1, v1: u1, v2: u1):
                v3 = mul v0, v1
                v4 = not v2
                v5 = mul v3, v4
                constrain v0 == u1 1
                constrain v1 == u1 1
                constrain v2 == u1 0
                return
            }
            ";
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
          b0(v0: u1, v1: u64):
            enable_side_effects v0
            v4 = make_array [Field 0, Field 1] : [Field; 2]
            v5 = array_get v4, index v1 -> Field
            v6 = not v0
            enable_side_effects v6
            v7 = array_get v4, index v1 -> Field
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        // Expected output is unchanged
        let ssa = ssa.fold_constants();
        assert_normalized_ssa_equals(ssa, src);
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

        let expected = "
            acir(inline) fn main f0 {
              b0(v0: u1, v1: u1, v2: [Field; 2]):
                enable_side_effects v0
                v4 = array_get v2, index u32 0 -> u32
                v7 = array_set v2, index u32 1, value u32 2
                v8 = array_get v7, index u32 0 -> u32
                constrain v4 == v8
                enable_side_effects v1
                v9 = array_set v2, index u32 1, value u32 2
                v10 = array_get v9, index u32 0 -> u32
                constrain v4 == v10
                enable_side_effects v0
                return
            }
            ";

        let ssa = ssa.fold_constants_using_constraints();
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn constant_array_deduplication() {
        // fn main f0 {
        //   b0(v0: u64):
        //     v1 = make_array [v0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0]
        //     v2 = make_array [v0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0]
        //     v5 = call keccakf1600(v1)
        //     v6 = call keccakf1600(v2)
        // }
        // Here we're checking a situation where two identical arrays are being initialized twice and being assigned separate `ValueId`s.
        // This would result in otherwise identical instructions not being deduplicated.
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let v0 = builder.add_parameter(Type::unsigned(64));
        let zero = builder.numeric_constant(0u128, Type::unsigned(64));
        let typ = Type::Array(Arc::new(vec![Type::unsigned(64)]), 25);

        let array_contents = im::vector![
            v0, zero, zero, zero, zero, zero, zero, zero, zero, zero, zero, zero, zero, zero, zero,
            zero, zero, zero, zero, zero, zero, zero, zero, zero, zero,
        ];
        let array1 = builder.insert_make_array(array_contents.clone(), typ.clone());
        let array2 = builder.insert_make_array(array_contents, typ.clone());

        assert_ne!(array1, array2, "arrays were not assigned different value ids");

        let keccakf1600 =
            builder.import_intrinsic("keccakf1600").expect("keccakf1600 intrinsic should exist");
        let _v10 = builder.insert_call(keccakf1600, vec![array1], vec![typ.clone()]);
        let _v11 = builder.insert_call(keccakf1600, vec![array2], vec![typ.clone()]);
        builder.terminate_with_return(Vec::new());

        let mut ssa = builder.finish();
        ssa.normalize_ids();

        println!("{ssa}");

        let main = ssa.main();
        let instructions = main.dfg[main.entry_block()].instructions();
        let starting_instruction_count = instructions.len();
        assert_eq!(starting_instruction_count, 4);

        // fn main f0 {
        //   b0(v0: u64):
        //     v1 = make_array [v0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0, u64 0]
        //     inc_rc v1
        //     v5 = call keccakf1600(v1)
        // }
        let ssa = ssa.fold_constants();

        println!("{ssa}");

        let main = ssa.main();
        let instructions = main.dfg[main.entry_block()].instructions();
        let ending_instruction_count = instructions.len();
        assert_eq!(ending_instruction_count, 3);
    }

    #[test]
    fn deduplicate_across_blocks() {
        // fn main f0 {
        //   b0(v0: u1):
        //     v1 = not v0
        //     jmp b1()
        //   b1():
        //     v2 = not v0
        //     return v2
        // }
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let b1 = builder.insert_block();

        let v0 = builder.add_parameter(Type::bool());
        let _v1 = builder.insert_not(v0);
        builder.terminate_with_jmp(b1, Vec::new());

        builder.switch_to_block(b1);
        let v2 = builder.insert_not(v0);
        builder.terminate_with_return(vec![v2]);

        let ssa = builder.finish();
        let main = ssa.main();
        assert_eq!(main.dfg[main.entry_block()].instructions().len(), 1);
        assert_eq!(main.dfg[b1].instructions().len(), 1);

        // Expected output:
        //
        // fn main f0 {
        //   b0(v0: u1):
        //     v1 = not v0
        //     jmp b1()
        //   b1():
        //     return v1
        // }
        let ssa = ssa.fold_constants_using_constraints();
        let main = ssa.main();
        assert_eq!(main.dfg[main.entry_block()].instructions().len(), 1);
        assert_eq!(main.dfg[b1].instructions().len(), 0);
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

        // v4 has been hoisted, although:
        // - v5 has not yet been removed since it was encountered earlier in the program
        // - v8 hasn't been recognized as a duplicate of v6 yet since they still reference v4 and
        //   v5 respectively
        let expected = "
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
        ";

        let ssa = ssa.fold_constants_using_constraints();
        assert_normalized_ssa_equals(ssa, expected);
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
        let brillig = ssa.to_brillig(false);

        let expected = "
            acir(inline) fn main f0 {
              b0():
                return Field 5
            }
            ";
        let ssa = ssa.fold_constants_with_brillig(&brillig);
        assert_normalized_ssa_equals(ssa, expected);
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
        let brillig = ssa.to_brillig(false);

        let expected = "
            acir(inline) fn main f0 {
              b0():
                return Field 5
            }
            ";
        let ssa = ssa.fold_constants_with_brillig(&brillig);
        assert_normalized_ssa_equals(ssa, expected);
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
                v2 = add v0, v1
                return v2
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let brillig = ssa.to_brillig(false);

        let expected = "
            acir(inline) fn main f0 {
              b0():
                return i32 5
            }
            ";
        let ssa = ssa.fold_constants_with_brillig(&brillig);
        assert_normalized_ssa_equals(ssa, expected);
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
        let brillig = ssa.to_brillig(false);

        let expected = "
            acir(inline) fn main f0 {
              b0():
                v3 = make_array [Field 2, Field 3, Field 4] : [Field; 3]
                return v3
            }
            ";
        let ssa = ssa.fold_constants_with_brillig(&brillig);
        assert_normalized_ssa_equals(ssa, expected);
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
              b0(v0: Field, v1: i32, v2: i32, v3: Field):
                v4 = make_array [v0, v1, v2, v3] : [(Field, i32); 2]
                return v4
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let brillig = ssa.to_brillig(false);

        let expected = "
            acir(inline) fn main f0 {
              b0():
                v4 = make_array [Field 2, i32 3, Field 4, i32 5] : [(Field, i32); 2]
                return v4
            }
            ";
        let ssa = ssa.fold_constants_with_brillig(&brillig);
        assert_normalized_ssa_equals(ssa, expected);
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
        let brillig = ssa.to_brillig(false);

        let expected = "
            acir(inline) fn main f0 {
              b0():
                v2 = make_array [Field 2, Field 3] : [Field; 2]
                return Field 5
            }
            ";
        let ssa = ssa.fold_constants_with_brillig(&brillig);
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn does_not_use_cached_constrain_in_block_that_is_not_dominated() {
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
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants_using_constraints();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn does_not_hoist_constrain_to_common_ancestor() {
        let src = "
            brillig(inline) fn main f0 {
              b0(v0: Field, v1: Field):
                v3 = eq v0, Field 0
                jmpif v3 then: b1, else: b2
              b1():
                constrain v1 == Field 1
                jmp b2()
              b2():
                jmpif v0 then: b3, else: b4
              b3():
                constrain v1 == Field 1 // This was incorrectly hoisted to b0 but this condition is not valid when going b0 -> b2 -> b4
                jmp b4()
              b4():
                return
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants_using_constraints();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn does_not_hoist_sub_to_common_ancestor() {
        let src = "
            acir(inline) fn main f0 {
              b0(v0: u32):
                v2 = eq v0, u32 0
                jmpif v2 then: b4, else: b1
              b1():
                jmpif v0 then: b3, else: b2
              b2():
                jmp b5()
              b3():
                v4 = sub v0, u32 1 // We can't hoist this because v0 is zero here and it will lead to an underflow
                jmp b5()
              b4():
                v5 = sub v0, u32 1
                jmp b5()
              b5():
                return
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants_using_constraints();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn deduplicates_side_effecting_intrinsics() {
        let src = "
        // After EnableSideEffectsIf removal:
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: u1):
            v4 = call is_unconstrained() -> u1
            v7 = call to_be_radix(v0, u32 256) -> [u8; 1]    // `a.to_be_radix(256)`;
            inc_rc v7
            v8 = call to_be_radix(v0, u32 256) -> [u8; 1]    // duplicate load of `a`
            inc_rc v8
            v9 = cast v2 as Field                            // `if c { a.to_be_radix(256) }`
            v10 = mul v0, v9                                 // attaching `c` to `a`
            v11 = call to_be_radix(v10, u32 256) -> [u8; 1]  // calling `to_radix(c * a)`
            inc_rc v11
            enable_side_effects v2                           // side effect var for `c` shifted down by removal
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let expected = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: u1):
            v4 = call is_unconstrained() -> u1
            v7 = call to_be_radix(v0, u32 256) -> [u8; 1]
            inc_rc v7
            inc_rc v7
            v8 = cast v2 as Field
            v9 = mul v0, v8
            v10 = call to_be_radix(v9, u32 256) -> [u8; 1]
            inc_rc v10
            enable_side_effects v2
            return
        }
        ";
        let ssa = ssa.fold_constants_using_constraints();
        assert_normalized_ssa_equals(ssa, expected);
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

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.fold_constants_using_constraints();
        // We expect the code to be unchanged
        assert_normalized_ssa_equals(ssa, src);
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

        let expected = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 3], v1: u32, v2: Field):
            enable_side_effects u1 1
            v4 = array_set v0, index v1, value v2
            return v2
        }
        ";
        let ssa = ssa.fold_constants_using_constraints();
        assert_normalized_ssa_equals(ssa, expected);
    }
}
