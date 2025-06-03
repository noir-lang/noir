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
use std::collections::{BTreeMap, HashSet, VecDeque};

use acvm::{
    FieldElement,
    acir::AcirField,
    brillig_vm::{MemoryValue, VM, VMStatus},
};
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use im::Vector;
use iter_extended::vecmap;

use crate::{
    brillig::{
        Brillig, BrilligOptions,
        brillig_gen::gen_brillig_for,
        brillig_ir::{artifact::BrilligParameter, brillig_variable::get_bit_size_from_ssa_type},
    },
    ssa::{
        ir::{
            basic_block::BasicBlockId,
            dfg::{DataFlowGraph, InsertInstructionResult},
            dom::DominatorTree,
            function::{Function, FunctionId, RuntimeType},
            instruction::{ArrayOffset, Instruction, InstructionId},
            types::{NumericType, Type},
            value::{Value, ValueId, ValueMapping},
        },
        opt::pure::Purity,
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
    pub fn fold_constants_with_brillig(mut self, brillig: &Brillig) -> Ssa {
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
            // We have already performed our final Brillig generation, so constant folding
            // Brillig functions is unnecessary work.
            if function.dfg.runtime().is_brillig() {
                continue;
            }
            function.constant_fold(false, brillig_info);
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

    values_to_replace: ValueMapping,
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
            values_to_replace: Default::default(),
        }
    }

    fn fold_constants_in_block(
        &mut self,
        function: &mut Function,
        dom: &mut DominatorTree,
        block_id: BasicBlockId,
    ) {
        let instructions = function.dfg[block_id].take_instructions();

        // Default side effect condition variable with an enabled state.
        let mut side_effects_enabled_var =
            function.dfg.make_constant(FieldElement::one(), NumericType::bool());

        for instruction_id in instructions {
            let instruction = &mut function.dfg[instruction_id];
            instruction.replace_values(&self.values_to_replace);

            self.fold_constants_into_instruction(
                function,
                dom,
                block_id,
                instruction_id,
                &mut side_effects_enabled_var,
            );
        }

        // Map the block terminator, resolving any values in the terminator with the
        // internal value mapping generated by this pass.
        function.dfg.replace_values_in_block_terminator(block_id, &self.values_to_replace);
        function.dfg.data_bus.replace_values(&self.values_to_replace);

        // Map a terminator in place, replacing any ValueId in the terminator with the
        // resolved version of that value id from the simplification cache's internal value mapping.
        // We need this in addition to the value replacement above in order to take advantage
        // of constraints that may have advised simplifications.
        // The value mapping (`self.values_to_replace`) only maps old instruction results to new instruction results.
        // However, constraints do not have "results" like other instructions, thus are not included in `self.values_to_replace`.
        // To take advantage of constraint simplification we need to still resolve its cache.
        let mut terminator = function.dfg[block_id].take_terminator();
        terminator.map_values_mut(|value| {
            Self::resolve_cache(
                block_id,
                dom,
                self.get_constraint_map(side_effects_enabled_var),
                value,
            )
        });
        function.dfg[block_id].set_terminator(terminator);
        function.dfg.data_bus.map_values_mut(|value| {
            Self::resolve_cache(
                block_id,
                dom,
                self.get_constraint_map(side_effects_enabled_var),
                value,
            )
        });

        self.block_queue.extend(function.dfg[block_id].successors());
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
        let runtime_is_brillig = dfg.runtime().is_brillig();
        if let Some(cache_result) =
            self.get_cached(dfg, dom, &instruction, *side_effects_enabled_var, block)
        {
            match cache_result {
                CacheResult::Cached(cached) => {
                    // We track whether we may mutate MakeArray instructions before we deduplicate
                    // them but we still need to issue an extra inc_rc in case they're mutated afterward.
                    if runtime_is_brillig && matches!(instruction, Instruction::MakeArray { .. }) {
                        let value = *cached.last().unwrap();
                        let inc_rc = Instruction::IncrementRc { value };
                        let call_stack = dfg.get_instruction_call_stack_id(id);
                        dfg.insert_instruction_and_results(inc_rc, block, None, call_stack);
                    }

                    let cached = cached.to_vec();
                    self.replace_result_ids(&old_results, &cached);
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
        let new_results = if runtime_is_brillig {
            Self::push_instruction(id, instruction.clone(), &old_results, block, dfg)
        } else {
            // We only want to try to inline Brillig calls for Brillig entry points (functions called from an ACIR runtime).
            Self::try_inline_brillig_call_with_all_constants(
                &instruction,
                &old_results,
                block,
                dfg,
                self.brillig_info,
            )
            // Otherwise, try inserting the instruction again to apply any optimizations using the newly resolved inputs.
            .unwrap_or_else(|| {
                Self::push_instruction(id, instruction.clone(), &old_results, block, dfg)
            })
        };

        self.replace_result_ids(&old_results, &new_results);

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

    // Alternate between resolving `value_id` in the `dfg` and checking to see if the resolved value
    // has been constrained to be equal to some simpler value in the current block.
    //
    // This allows us to reach a stable final `ValueId` for each instruction input as we add more
    // constraints to the cache.
    fn resolve_cache(
        block: BasicBlockId,
        dom: &mut DominatorTree,
        cache: &HashMap<ValueId, SimplificationCache>,
        value_id: ValueId,
    ) -> ValueId {
        match cache.get(&value_id) {
            Some(simplification_cache) => {
                if let Some(simplified) = simplification_cache.get(block, dom) {
                    Self::resolve_cache(block, dom, cache, simplified)
                } else {
                    value_id
                }
            }
            None => value_id,
        }
    }

    /// Fetches an [`Instruction`] by its [`InstructionId`] and fully resolves its inputs.
    fn resolve_instruction(
        instruction_id: InstructionId,
        block: BasicBlockId,
        dfg: &DataFlowGraph,
        dom: &mut DominatorTree,
        constraint_simplification_mapping: &HashMap<ValueId, SimplificationCache>,
    ) -> Instruction {
        let mut instruction = dfg[instruction_id].clone();

        // Resolve any inputs to ensure that we're comparing like-for-like instructions.
        instruction.map_values_mut(|value_id| {
            Self::resolve_cache(block, dom, constraint_simplification_mapping, value_id)
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
        let new_results = match dfg.insert_instruction_and_results_if_simplified(
            instruction,
            block,
            ctrl_typevars,
            call_stack,
            Some(id),
        ) {
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
            let predicate = self.use_constraint_info.then_some(side_effects_enabled_var);

            let offset = ArrayOffset::None;
            let array_get =
                Instruction::ArrayGet { array: instruction_results[0], index: *index, offset };

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
            can_be_deduplicated(&instruction, function, self.use_constraint_info);

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
    fn replace_result_ids(&mut self, old_results: &[ValueId], new_results: &[ValueId]) {
        for (old_result, new_result) in old_results.iter().zip(new_results) {
            self.values_to_replace.insert(*old_result, *new_result);
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
            EvaluationResult::NotABrilligCall | EvaluationResult::CannotEvaluate => None,
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
            return EvaluationResult::CannotEvaluate;
        }

        let mut brillig_arguments = Vec::new();
        for argument in arguments {
            let typ = dfg.type_of_value(*argument);
            let Some(parameter) = type_to_brillig_parameter(&typ) else {
                return EvaluationResult::CannotEvaluate;
            };
            brillig_arguments.push(parameter);
        }

        // Check that return value types are supported by brillig
        for return_id in func.returns().iter() {
            let typ = func.dfg.type_of_value(*return_id);
            if type_to_brillig_parameter(&typ).is_none() {
                return EvaluationResult::CannotEvaluate;
            }
        }

        let Ok(generated_brillig) =
            gen_brillig_for(func, brillig_arguments, brillig, &BrilligOptions::default())
        else {
            return EvaluationResult::CannotEvaluate;
        };

        let mut calldata = Vec::new();
        for argument in arguments {
            value_id_to_calldata(*argument, dfg, &mut calldata);
        }

        let bytecode = &generated_brillig.byte_code;
        let pedantic_solving = true;
        let black_box_solver = Bn254BlackBoxSolver(pedantic_solving);
        let profiling_active = false;
        let mut vm = VM::new(calldata, bytecode, &black_box_solver, profiling_active, None);
        let vm_status: VMStatus<_> = vm.process_opcodes();
        let VMStatus::Finished { return_data_offset, return_data_size } = vm_status else {
            return EvaluationResult::CannotEvaluate;
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
            Type::Numeric(typ) => {
                let memory = memory_values[*memory_index];
                *memory_index += 1;

                let field_value = memory.to_field();
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
            if function.dfg.is_global(*array) {
                // Early return as we expect globals to be immutable.
                return;
            };

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

#[derive(Debug)]
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
    CannotEvaluate,
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

/// Indicates if the instruction can be safely replaced with the results of another instruction with the same inputs.
/// If `deduplicate_with_predicate` is set, we assume we're deduplicating with the instruction
/// and its predicate, rather than just the instruction. Setting this means instructions that
/// rely on predicates can be deduplicated as well.
///
/// Some instructions get the predicate attached to their inputs by `handle_instruction_side_effects` in `flatten_cfg`.
/// These can be deduplicated because they implicitly depend on the predicate, not only when the caller uses the
/// predicate variable as a key to cache results. However, to avoid tight coupling between passes, we make the deduplication
/// conditional on whether the caller wants the predicate to be taken into account or not.
pub(crate) fn can_be_deduplicated(
    instruction: &Instruction,
    function: &Function,
    deduplicate_with_predicate: bool,
) -> bool {
    use Instruction::*;

    match instruction {
        // These either have side-effects or interact with memory
        EnableSideEffectsIf { .. }
        | Allocate
        | Load { .. }
        | Store { .. }
        | IncrementRc { .. }
        | DecrementRc { .. } => false,

        Call { func, .. } => {
            let purity = match function.dfg[*func] {
                Value::Intrinsic(intrinsic) => Some(intrinsic.purity()),
                Value::Function(id) => function.dfg.purity_of(id),
                _ => None,
            };
            match purity {
                Some(Purity::Pure) => true,
                Some(Purity::PureWithPredicate) => deduplicate_with_predicate,
                Some(Purity::Impure) => false,
                None => false,
            }
        }

        // We can deduplicate these instructions if we know the predicate is also the same.
        Constrain(..) | ConstrainNotEqual(..) | RangeCheck { .. } => deduplicate_with_predicate,

        // Noop instructions can always be deduplicated, although they're more likely to be
        // removed entirely.
        Noop => true,

        // These instructions can always be deduplicated
        Cast(_, _) | Not(_) | Truncate { .. } | IfElse { .. } => true,

        // Arrays can be mutated in unconstrained code so code that handles this case must
        // take care to track whether the array was possibly mutated or not before
        // deduplicating. Since we don't know if the containing pass checks for this, we
        // can only assume these are safe to deduplicate in constrained code.
        MakeArray { .. } => function.runtime().is_acir(),

        // These can have different behavior depending on the EnableSideEffectsIf context.
        // Replacing them with a similar instruction potentially enables replacing an instruction
        // with one that was disabled. See
        // https://github.com/noir-lang/noir/pull/4716#issuecomment-2047846328.
        Binary(_) | ArrayGet { .. } | ArraySet { .. } => {
            deduplicate_with_predicate || !instruction.requires_acir_gen_predicate(&function.dfg)
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use noirc_frontend::monomorphization::ast::InlineType;

    use crate::{
        assert_ssa_snapshot,
        brillig::BrilligOptions,
        ssa::{
            Ssa,
            function_builder::FunctionBuilder,
            ir::{
                function::RuntimeType,
                map::Id,
                types::{NumericType, Type},
                value::ValueMapping,
            },
            opt::assert_normalized_ssa_equals,
        },
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
        let ssa = ssa.fold_constants();
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
        builder.set_runtime(RuntimeType::Brillig(InlineType::default()));
        let v0 = builder.add_parameter(Type::unsigned(64));
        let zero = builder.numeric_constant(0u128, NumericType::unsigned(64));
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
                v4 = shl v0, u8 1
                v5 = lt v0, v4
                constrain v5 == u1 1
                jmp b2()
              b2():
                v7 = lt u32 1000, v0
                jmpif v7 then: b3, else: b4
              b3():
                v8 = shl v0, u8 1
                v9 = lt v0, v8
                constrain v9 == u1 1
                jmp b4()
              b4():
                return
            }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants_using_constraints();

        // v4 has been hoisted, although:
        // - v5 has not yet been removed since it was encountered earlier in the program
        // - v8 hasn't been recognized as a duplicate of v6 yet since they still reference v4 and
        //   v5 respectively
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v2 = lt u32 1000, v0
            v4 = shl v0, u8 1
            jmpif v2 then: b1, else: b2
          b1():
            v5 = shl v0, u8 1
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
        let brillig = ssa.to_brillig(&BrilligOptions::default());

        let ssa = ssa.fold_constants_with_brillig(&brillig);
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
        let brillig = ssa.to_brillig(&BrilligOptions::default());

        let ssa = ssa.fold_constants_with_brillig(&brillig);
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
                v2 = add v0, v1
                v3 = truncate v2 to 32 bits, max_bit_size: 33
                return v3
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let brillig = ssa.to_brillig(&BrilligOptions::default());

        let ssa = ssa.fold_constants_with_brillig(&brillig);
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
        let brillig = ssa.to_brillig(&BrilligOptions::default());

        let ssa = ssa.fold_constants_with_brillig(&brillig);
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
              b0(v0: Field, v1: i32, v2: i32, v3: Field):
                v4 = make_array [v0, v1, v2, v3] : [(Field, i32); 2]
                return v4
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let brillig = ssa.to_brillig(&BrilligOptions::default());

        let ssa = ssa.fold_constants_with_brillig(&brillig);
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
        // Need to run SSA pass that sets up Brillig array gets
        let ssa = ssa.brillig_array_get_and_set();
        let brillig = ssa.to_brillig(&BrilligOptions::default());

        let ssa = ssa.fold_constants_with_brillig(&brillig);
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
        let mut ssa = ssa.dead_instruction_elimination();
        let used_globals_map = std::mem::take(&mut ssa.used_globals);
        let brillig = ssa.to_brillig_with_globals(&BrilligOptions::default(), used_globals_map);

        let ssa = ssa.fold_constants_with_brillig(&brillig);
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
        let mut ssa = ssa.dead_instruction_elimination();
        let used_globals_map = std::mem::take(&mut ssa.used_globals);
        let brillig = ssa.to_brillig_with_globals(&BrilligOptions::default(), used_globals_map);

        let ssa = ssa.fold_constants_with_brillig(&brillig);
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
                v5 = sub v0, u32 1 // We can't hoist this because v0 is zero here and it will lead to an underflow
                jmp b5()
              b4():
                v4 = sub v0, u32 1
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

        let ssa = ssa.fold_constants_using_constraints();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: Field, v1: Field, v2: u1):
            v5 = call to_be_radix(v0, u32 256) -> [u8; 1]
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

        let ssa = ssa.fold_constants_using_constraints();
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
        let ssa = ssa.purity_analysis().fold_constants_using_constraints();
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

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants();
        assert_normalized_ssa_equals(ssa, src);
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

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants();
        assert_normalized_ssa_equals(ssa, src);
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

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants();
        assert_normalized_ssa_equals(ssa, src);
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

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants();
        assert_normalized_ssa_equals(ssa, src);
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

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u32, v2: u1):
            enable_side_effects v2
            v4 = div v1, u32 2
            v5 = not v2
            enable_side_effects v5
            v6 = div v1, u32 2
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
        let ssa = ssa.fold_constants_using_constraints();

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
    fn unknown_issue_regression_test() {
        // Regression test for an issue discovered in https://github.com/AztecProtocol/aztec-packages/pull/14492
        // Marking `f1` as `predicate_pure` instead of `impure` results in an invalid storage write.
        let src = r#"
        g0 = u32 44
        g1 = u32 13
        g2 = u32 6
        g3 = u32 3
        
        brillig(inline) impure fn constructor f0 {
          b0(v4: Field, v5: Field, v6: Field):
            v12 = allocate -> &mut u1
            store u1 0 at v12
            v14 = allocate -> &mut Field
            store Field 0 at v14
            v16 = allocate -> &mut Field
            store Field 2 at v16
            v19 = call f5() -> Field
            v21 = call f6(v19) -> [(u1, Field); 1]
            v23 = array_get v21, index u32 0 -> u1
            v25 = array_get v21, index u32 1 -> Field
            v26 = not v23
            v27 = cast v23 as Field
            v28 = cast v26 as Field
            v29 = mul v27, v25
            constrain v23 == u1 1
            v32 = call f7(v19) -> [(u1, Field); 1]
            v33 = array_get v32, index u32 0 -> u1
            v34 = array_get v32, index u32 1 -> Field
            v35 = not v33
            v36 = cast v33 as Field
            v37 = cast v35 as Field
            v38 = mul v36, v34
            constrain v33 == u1 1
            v40 = call f8(u32 0, u32 1) -> [Field; 1]
            v41 = array_get v40, index u32 0 -> Field
            v42 = truncate v41 to 32 bits, max_bit_size: 254
            v43 = allocate -> &mut u1
            v44 = allocate -> &mut Field
            v45 = allocate -> &mut Field
            store Field 2 at v45
            v46 = call f9(u32 1, u32 3) -> [Field; 3]
            inc_rc v46
            v48 = make_array [Field 44] : [Field; 1]
            v49 = make_array [Field 44, Field 44, Field 44, Field 44] : [Field; 4]
            v50 = allocate -> &mut [Field; 4]
            store v49 at v50
            jmp b1(u32 0)
          b1(v7: u32):
            v51 = lt v7, u32 3
            jmpif v51 then: b2, else: b3
          b2():
            v143 = unchecked_add v7, u32 1
            v144 = array_get v46, index v7 -> Field
            v145 = load v50 -> [Field; 4]
            v146 = lt v143, u32 4
            constrain v146 == u1 1, "Index out of bounds"
            v147 = array_set v145, index v143, value v144
            store v147 at v50
            jmp b1(v143)
          b3():
            v52 = load v50 -> [Field; 4]
            v55, v56, v57, v58 = call f1(Field 73786976294838206464) -> ([Field; 3], [Field; 4], u32, u1)
            v59 = allocate -> &mut [Field; 3]
            store v55 at v59
            v60 = allocate -> &mut [Field; 4]
            store v56 at v60
            v61 = allocate -> &mut u32
            store v57 at v61
            v62 = allocate -> &mut u1
            store v58 at v62
            inc_rc v52
            jmp b4(u32 0)
          b4(v8: u32):
            v64 = lt v8, u32 4
            jmpif v64 then: b5, else: b6
          b5():
            v141 = array_get v52, index v8 -> Field
            call f2(v59, v60, v61, v62, v141)
            v142 = unchecked_add v8, u32 1
            jmp b4(v142)
          b6():
            v66 = call f3(v59, v60, v61, v62) -> Field
            v67 = load v45 -> Field
            store v67 at v45
            v68 = make_array [v42, v66] : [Field; 2]
            v70 = make_array [Field 13] : [Field; 1]
            v71 = make_array [Field 13, Field 13, Field 13] : [Field; 3]
            v72 = allocate -> &mut [Field; 3]
            v73 = make_array [Field 13, v42, Field 13] : [Field; 3]
            v74 = make_array [Field 13, v42, v66] : [Field; 3]
            v76, v77, v78, v79 = call f1(Field 55340232221128654848) -> ([Field; 3], [Field; 4], u32, u1)
            v80 = allocate -> &mut [Field; 3]
            store v76 at v80
            v81 = allocate -> &mut [Field; 4]
            store v77 at v81
            v82 = allocate -> &mut u32
            store v78 at v82
            v83 = allocate -> &mut u1
            store v79 at v83
            inc_rc v74
            jmp b7(u32 0)
          b7(v9: u32):
            v84 = lt v9, u32 3
            jmpif v84 then: b8, else: b9
          b8():
            v139 = array_get v74, index v9 -> Field
            call f2(v80, v81, v82, v83, v139)
            v140 = unchecked_add v9, u32 1
            jmp b7(v140)
          b9():
            v85 = call f3(v80, v81, v82, v83) -> Field
            constrain v38 == v85, "Initialization hash does not match"
            v86 = eq v29, Field 0
            v88 = call f11() -> Field
            v89 = eq v29, v88
            v90 = or v86, v89
            constrain v90 == u1 1, "Initializer address is not the contract deployer"
            v92 = make_array [Field 1] : [Field; 1]
            v94 = make_array [Field 6] : [Field; 1]
            v95 = make_array [Field 6, Field 6] : [Field; 2]
            v96 = allocate -> &mut [Field; 2]
            v97 = make_array [Field 6, Field 1] : [Field; 2]
            v99, v100, v101, v102 = call f1(Field 36893488147419103232) -> ([Field; 3], [Field; 4], u32, u1)
            v103 = allocate -> &mut [Field; 3]
            store v99 at v103
            v104 = allocate -> &mut [Field; 4]
            store v100 at v104
            v105 = allocate -> &mut u32
            store v101 at v105
            v106 = allocate -> &mut u1
            store v102 at v106
            inc_rc v97
            call f2(v103, v104, v105, v106, Field 6)
            call f2(v103, v104, v105, v106, Field 1)
            v108 = call f3(v103, v104, v105, v106) -> Field
            call f10(v108)
            v110 = load v12 -> u1
            v111 = load v14 -> Field
            v112 = load v16 -> Field
            v113 = make_array [v4, v5, v6] : [Field; 3]
            inc_rc v113
            v114, v115, v116, v117 = call f1(Field 55340232221128654848) -> ([Field; 3], [Field; 4], u32, u1)
            v118 = allocate -> &mut [Field; 3]
            store v114 at v118
            v119 = allocate -> &mut [Field; 4]
            store v115 at v119
            v120 = allocate -> &mut u32
            store v116 at v120
            v121 = allocate -> &mut u1
            store v117 at v121
            inc_rc v113
            jmp b10(u32 0)
          b10(v10: u32):
            v122 = lt v10, u32 3
            jmpif v122 then: b11, else: b12
          b11():
            v137 = array_get v113, index v10 -> Field
            call f2(v118, v119, v120, v121, v137)
            v138 = unchecked_add v10, u32 1
            jmp b10(v138)
          b12():
            v123 = call f3(v118, v119, v120, v121) -> Field
            v124 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v125 = allocate -> &mut [Field; 4]
            v126 = make_array [v4, Field 0, Field 0, Field 0] : [Field; 4]
            v127 = make_array [v4, v5, Field 0, Field 0] : [Field; 4]
            v128 = make_array [v4, v5, v6, Field 0] : [Field; 4]
            v129 = make_array [v4, v5, v6, v123] : [Field; 4]
            jmp b13(u32 0)
          b13(v11: u32):
            v130 = lt v11, u32 4
            jmpif v130 then: b14, else: b15
          b14():
            v132 = cast v11 as Field
            v133 = add Field 1, v132
            v134 = array_get v129, index v11 -> Field
            call f12(v133, v134)
            v136 = unchecked_add v11, u32 1
            jmp b13(v136)
          b15():
            v131 = call f5() -> Field
            call f10(v131)
            return
        }
        brillig(inline) predicate_pure fn new f1 {
          b0(v4: Field):
            v6 = make_array [Field 0, Field 0, Field 0] : [Field; 3]
            v7 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v8 = allocate -> &mut [Field; 3]
            v9 = allocate -> &mut [Field; 4]
            v10 = allocate -> &mut u32
            v11 = allocate -> &mut u1
            v12 = make_array [Field 0, Field 0, Field 0, v4] : [Field; 4]
            return v6, v12, u32 0, u1 0
        }
        brillig(inline) impure fn absorb f2 {
          b0(v4: &mut [Field; 3], v5: &mut [Field; 4], v6: &mut u32, v7: &mut u1, v8: Field):
            v9 = load v7 -> u1
            constrain v9 == u1 0
            v11 = load v6 -> u32
            v12 = eq v11, u32 3
            jmpif v12 then: b1, else: b2
          b1():
            call f4(v4, v5, v6, v7)
            v23 = load v4 -> [Field; 3]
            v24 = load v5 -> [Field; 4]
            v25 = load v6 -> u32
            v26 = load v7 -> u1
            v28 = array_set v23, index u32 0, value v8
            store v28 at v4
            store v24 at v5
            store u32 1 at v6
            store v26 at v7
            jmp b3()
          b2():
            v13 = load v6 -> u32
            v14 = load v4 -> [Field; 3]
            v15 = load v5 -> [Field; 4]
            v16 = load v7 -> u1
            v17 = lt v13, u32 3
            constrain v17 == u1 1, "Index out of bounds"
            v19 = array_set v14, index v13, value v8
            v21 = add v13, u32 1
            store v19 at v4
            store v15 at v5
            store v21 at v6
            store v16 at v7
            jmp b3()
          b3():
            return
        }
        brillig(inline) impure fn squeeze f3 {
          b0(v4: &mut [Field; 3], v5: &mut [Field; 4], v6: &mut u32, v7: &mut u1):
            v8 = load v7 -> u1
            constrain v8 == u1 0
            call f4(v4, v5, v6, v7)
            v11 = load v4 -> [Field; 3]
            v12 = load v5 -> [Field; 4]
            v13 = load v6 -> u32
            store v11 at v4
            store v12 at v5
            store v13 at v6
            store u1 1 at v7
            v16 = array_get v12, index u32 0 -> Field
            return v16
        }
        brillig(inline) impure fn perform_duplex f4 {
          b0(v4: &mut [Field; 3], v5: &mut [Field; 4], v6: &mut u32, v7: &mut u1):
            jmp b1(u32 0)
          b1(v8: u32):
            v10 = lt v8, u32 3
            jmpif v10 then: b2, else: b3
          b2():
            v18 = load v6 -> u32
            v19 = lt v8, v18
            jmpif v19 then: b4, else: b5
          b3():
            v11 = load v5 -> [Field; 4]
            inc_rc v11
            v14 = call poseidon2_permutation(v11, u32 4) -> [Field; 4]
            v15 = load v4 -> [Field; 3]
            v16 = load v6 -> u32
            v17 = load v7 -> u1
            store v15 at v4
            store v14 at v5
            store v16 at v6
            store v17 at v7
            return
          b4():
            v20 = load v5 -> [Field; 4]
            v21 = array_get v20, index v8 -> Field
            v22 = load v4 -> [Field; 3]
            v23 = array_get v22, index v8 -> Field
            v24 = add v21, v23
            v25 = load v6 -> u32
            v26 = load v7 -> u1
            v27 = array_set v20, index v8, value v24
            store v22 at v4
            store v27 at v5
            store v25 at v6
            store v26 at v7
            jmp b5()
          b5():
            v29 = unchecked_add v8, u32 1
            jmp b1(v29)
        }

        brillig(inline) impure fn avmOpcodeAddress f5 {
          b0():
            return Field 0
        }

        brillig(inline) impure fn avmOpcodeGetContractInstanceDeployer f6 {
          b0(v0: Field):
            v1 = make_array [u1 0, Field 0] : [(u1, Field); 1]
            return v1
        }

        brillig(inline) impure fn avmOpcodeGetContractInstanceInitializationHash f7 {
          b0(v0: Field):
            v1 = make_array [u1 0, Field 0] : [(u1, Field); 1]
            return v1
        }

        brillig(inline) impure fn avmOpcodeCalldataCopy f8 {
          b0(v0: u32, v1: u32):
            v2 = make_array [Field 0] : [Field; 1]
            return v2
        }
        
        brillig(inline) impure fn avmOpcodeCalldataCopy f9 {
          b0(v0: u32, v1: u32):
            v2 = make_array [Field 0] : [Field; 3]
            return v2
        }

        brillig(inline) impure fn avmOpcodeEmitNullifier f10 {
          b0(v0: Field):
            return
        }

        brillig(inline) impure fn avmOpcodeSender f11 {
          b0():
            return
        }

        brillig(inline) impure fn avmOpcodeStorageWrite f12 {
          b0(v0: Field, v1: Field):
            return
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.fold_constants_using_constraints();

        assert_ssa_snapshot!(ssa, @r#"
        g0 = u32 44
        g1 = u32 13
        g2 = u32 6
        g3 = u32 3

        brillig(inline) impure fn constructor f0 {
          b0(v4: Field, v5: Field, v6: Field):
            v12 = allocate -> &mut u1
            store u1 0 at v12
            v14 = allocate -> &mut Field
            store Field 0 at v14
            v16 = allocate -> &mut Field
            store Field 2 at v16
            v19 = call f5() -> Field
            v21 = call f6(v19) -> [(u1, Field); 1]
            v23 = array_get v21, index u32 0 -> u1
            v25 = array_get v21, index u32 1 -> Field
            v26 = not v23
            v27 = cast v23 as Field
            v28 = cast v26 as Field
            v29 = mul v27, v25
            constrain v23 == u1 1
            v32 = call f7(v19) -> [(u1, Field); 1]
            v33 = array_get v32, index u32 0 -> u1
            v34 = array_get v32, index u32 1 -> Field
            v35 = not v33
            v36 = cast v33 as Field
            v37 = cast v35 as Field
            v38 = mul v36, v34
            constrain v33 == u1 1
            v40 = call f8(u32 0, u32 1) -> [Field; 1]
            v41 = array_get v40, index u32 0 -> Field
            v42 = truncate v41 to 32 bits, max_bit_size: 254
            v43 = allocate -> &mut u1
            v44 = allocate -> &mut Field
            v45 = allocate -> &mut Field
            store Field 2 at v45
            v47 = call f9(u32 1, u32 3) -> [Field; 3]
            inc_rc v47
            v49 = make_array [Field 44] : [Field; 1]
            v50 = make_array [Field 44, Field 44, Field 44, Field 44] : [Field; 4]
            v51 = allocate -> &mut [Field; 4]
            store v50 at v51
            jmp b1(u32 0)
          b1(v7: u32):
            v52 = lt v7, u32 3
            jmpif v52 then: b2, else: b3
          b2():
            v144 = unchecked_add v7, u32 1
            v145 = array_get v47, index v7 -> Field
            v146 = load v51 -> [Field; 4]
            v147 = lt v144, u32 4
            constrain v147 == u1 1, "Index out of bounds"
            v148 = array_set v146, index v144, value v145
            store v148 at v51
            jmp b1(v144)
          b3():
            v53 = load v51 -> [Field; 4]
            v56, v57, v58, v59 = call f1(Field 73786976294838206464) -> ([Field; 3], [Field; 4], u32, u1)
            v60 = allocate -> &mut [Field; 3]
            store v56 at v60
            v61 = allocate -> &mut [Field; 4]
            store v57 at v61
            v62 = allocate -> &mut u32
            store v58 at v62
            v63 = allocate -> &mut u1
            store v59 at v63
            inc_rc v53
            jmp b4(u32 0)
          b4(v8: u32):
            v65 = lt v8, u32 4
            jmpif v65 then: b5, else: b6
          b5():
            v142 = array_get v53, index v8 -> Field
            call f2(v60, v61, v62, v63, v142)
            v143 = unchecked_add v8, u32 1
            jmp b4(v143)
          b6():
            v67 = call f3(v60, v61, v62, v63) -> Field
            v68 = load v45 -> Field
            store v68 at v45
            v69 = make_array [v42, v67] : [Field; 2]
            v71 = make_array [Field 13] : [Field; 1]
            v72 = make_array [Field 13, Field 13, Field 13] : [Field; 3]
            v73 = allocate -> &mut [Field; 3]
            v74 = make_array [Field 13, v42, Field 13] : [Field; 3]
            v75 = make_array [Field 13, v42, v67] : [Field; 3]
            v77, v78, v79, v80 = call f1(Field 55340232221128654848) -> ([Field; 3], [Field; 4], u32, u1)
            v81 = allocate -> &mut [Field; 3]
            store v77 at v81
            v82 = allocate -> &mut [Field; 4]
            store v78 at v82
            v83 = allocate -> &mut u32
            store v79 at v83
            v84 = allocate -> &mut u1
            store v80 at v84
            inc_rc v75
            jmp b7(u32 0)
          b7(v9: u32):
            v85 = lt v9, u32 3
            jmpif v85 then: b8, else: b9
          b8():
            v140 = array_get v75, index v9 -> Field
            call f2(v81, v82, v83, v84, v140)
            v141 = unchecked_add v9, u32 1
            jmp b7(v141)
          b9():
            v86 = call f3(v81, v82, v83, v84) -> Field
            constrain v38 == v86, "Initialization hash does not match"
            v87 = eq v29, Field 0
            v89 = call f11() -> Field
            v90 = eq v29, v89
            v91 = or v87, v90
            constrain v91 == u1 1, "Initializer address is not the contract deployer"
            v93 = make_array [Field 1] : [Field; 1]
            v95 = make_array [Field 6] : [Field; 1]
            v96 = make_array [Field 6, Field 6] : [Field; 2]
            v97 = allocate -> &mut [Field; 2]
            v98 = make_array [Field 6, Field 1] : [Field; 2]
            v100, v101, v102, v103 = call f1(Field 36893488147419103232) -> ([Field; 3], [Field; 4], u32, u1)
            v104 = allocate -> &mut [Field; 3]
            store v100 at v104
            v105 = allocate -> &mut [Field; 4]
            store v101 at v105
            v106 = allocate -> &mut u32
            store v102 at v106
            v107 = allocate -> &mut u1
            store v103 at v107
            inc_rc v98
            call f2(v104, v105, v106, v107, Field 6)
            call f2(v104, v105, v106, v107, Field 1)
            v109 = call f3(v104, v105, v106, v107) -> Field
            call f10(v109)
            v111 = load v12 -> u1
            v112 = load v14 -> Field
            v113 = load v16 -> Field
            v114 = make_array [v4, v5, v6] : [Field; 3]
            inc_rc v114
            v115, v116, v117, v118 = call f1(Field 55340232221128654848) -> ([Field; 3], [Field; 4], u32, u1)
            v119 = allocate -> &mut [Field; 3]
            store v115 at v119
            v120 = allocate -> &mut [Field; 4]
            store v116 at v120
            v121 = allocate -> &mut u32
            store v117 at v121
            v122 = allocate -> &mut u1
            store v118 at v122
            inc_rc v114
            jmp b10(u32 0)
          b10(v10: u32):
            v123 = lt v10, u32 3
            jmpif v123 then: b11, else: b12
          b11():
            v138 = array_get v114, index v10 -> Field
            call f2(v119, v120, v121, v122, v138)
            v139 = unchecked_add v10, u32 1
            jmp b10(v139)
          b12():
            v124 = call f3(v119, v120, v121, v122) -> Field
            v125 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v126 = allocate -> &mut [Field; 4]
            v127 = make_array [v4, Field 0, Field 0, Field 0] : [Field; 4]
            v128 = make_array [v4, v5, Field 0, Field 0] : [Field; 4]
            v129 = make_array [v4, v5, v6, Field 0] : [Field; 4]
            v130 = make_array [v4, v5, v6, v124] : [Field; 4]
            jmp b13(u32 0)
          b13(v11: u32):
            v131 = lt v11, u32 4
            jmpif v131 then: b14, else: b15
          b14():
            v133 = cast v11 as Field
            v134 = add Field 1, v133
            v135 = array_get v130, index v11 -> Field
            call f12(v134, v135)
            v137 = unchecked_add v11, u32 1
            jmp b13(v137)
          b15():
            v132 = call f5() -> Field
            call f10(v132)
            return
        }
        brillig(inline) impure fn new f1 {
          b0(v4: Field):
            v6 = make_array [Field 0, Field 0, Field 0] : [Field; 3]
            v7 = make_array [Field 0, Field 0, Field 0, Field 0] : [Field; 4]
            v8 = allocate -> &mut [Field; 3]
            v9 = allocate -> &mut [Field; 4]
            v10 = allocate -> &mut u32
            v11 = allocate -> &mut u1
            v12 = make_array [Field 0, Field 0, Field 0, v4] : [Field; 4]
            return v6, v12, u32 0, u1 0
        }
        brillig(inline) impure fn absorb f2 {
          b0(v4: &mut [Field; 3], v5: &mut [Field; 4], v6: &mut u32, v7: &mut u1, v8: Field):
            v9 = load v7 -> u1
            constrain v9 == u1 0
            v11 = load v6 -> u32
            v12 = eq v11, u32 3
            jmpif v12 then: b1, else: b2
          b1():
            call f4(v4, v5, v6, v7)
            v23 = load v4 -> [Field; 3]
            v24 = load v5 -> [Field; 4]
            v25 = load v6 -> u32
            v26 = load v7 -> u1
            v28 = array_set v23, index u32 0, value v8
            store v28 at v4
            store v24 at v5
            store u32 1 at v6
            store v26 at v7
            jmp b3()
          b2():
            v13 = load v6 -> u32
            v14 = load v4 -> [Field; 3]
            v15 = load v5 -> [Field; 4]
            v16 = load v7 -> u1
            v17 = lt v13, u32 3
            constrain v17 == u1 1, "Index out of bounds"
            v19 = array_set v14, index v13, value v8
            v21 = add v13, u32 1
            store v19 at v4
            store v15 at v5
            store v21 at v6
            store v16 at v7
            jmp b3()
          b3():
            return
        }
        brillig(inline) impure fn squeeze f3 {
          b0(v4: &mut [Field; 3], v5: &mut [Field; 4], v6: &mut u32, v7: &mut u1):
            v8 = load v7 -> u1
            constrain v8 == u1 0
            call f4(v4, v5, v6, v7)
            v11 = load v4 -> [Field; 3]
            v12 = load v5 -> [Field; 4]
            v13 = load v6 -> u32
            store v11 at v4
            store v12 at v5
            store v13 at v6
            store u1 1 at v7
            v16 = array_get v12, index u32 0 -> Field
            return v16
        }
        brillig(inline) impure fn perform_duplex f4 {
          b0(v4: &mut [Field; 3], v5: &mut [Field; 4], v6: &mut u32, v7: &mut u1):
            jmp b1(u32 0)
          b1(v8: u32):
            v10 = lt v8, u32 3
            jmpif v10 then: b2, else: b3
          b2():
            v18 = load v6 -> u32
            v19 = lt v8, v18
            jmpif v19 then: b4, else: b5
          b3():
            v11 = load v5 -> [Field; 4]
            inc_rc v11
            v14 = call poseidon2_permutation(v11, u32 4) -> [Field; 4]
            v15 = load v4 -> [Field; 3]
            v16 = load v6 -> u32
            v17 = load v7 -> u1
            store v15 at v4
            store v14 at v5
            store v16 at v6
            store v17 at v7
            return
          b4():
            v20 = load v5 -> [Field; 4]
            v21 = array_get v20, index v8 -> Field
            v22 = load v4 -> [Field; 3]
            v23 = array_get v22, index v8 -> Field
            v24 = add v21, v23
            v25 = load v6 -> u32
            v26 = load v7 -> u1
            v27 = array_set v20, index v8, value v24
            store v22 at v4
            store v27 at v5
            store v25 at v6
            store v26 at v7
            jmp b5()
          b5():
            v29 = unchecked_add v8, u32 1
            jmp b1(v29)
        }
        brillig(inline) impure fn avmOpcodeAddress f5 {
          b0():
            return Field 0
        }
        brillig(inline) impure fn avmOpcodeGetContractInstanceDeployer f6 {
          b0(v4: Field):
            v7 = make_array [u1 0, Field 0] : [(u1, Field); 1]
            return v7
        }
        brillig(inline) impure fn avmOpcodeGetContractInstanceInitializationHash f7 {
          b0(v4: Field):
            v7 = make_array [u1 0, Field 0] : [(u1, Field); 1]
            return v7
        }
        brillig(inline) impure fn avmOpcodeCalldataCopy f8 {
          b0(v4: u32, v5: u32):
            v7 = make_array [Field 0] : [Field; 1]
            return v7
        }
        brillig(inline) impure fn avmOpcodeCalldataCopy f9 {
          b0(v4: u32, v5: u32):
            v7 = make_array [Field 0] : [Field; 3]
            return v7
        }
        brillig(inline) impure fn avmOpcodeEmitNullifier f10 {
          b0(v4: Field):
            return
        }
        brillig(inline) impure fn avmOpcodeSender f11 {
          b0():
            return
        }
        brillig(inline) impure fn avmOpcodeStorageWrite f12 {
          b0(v4: Field, v5: Field):
            return
        }
        "#);
    }
}
