//! This module defines the function inlining pass for the SSA IR.
//! The purpose of this pass is to inline the instructions of each function call
//! within the function caller. If all function calls are known, there will only
//! be a single function remaining when the pass finishes.

use std::collections::HashSet;

use crate::{
    errors::RuntimeError,
    ssa::{opt::inlining::inline_info::compute_bottom_up_order, visit_once_deque::VisitOnceDeque},
};
use acvm::acir::AcirField;
use im::HashMap;
use iter_extended::vecmap;
use noirc_errors::{Location, call_stack::CallStackId};

use crate::ssa::{
    function_builder::FunctionBuilder,
    ir::{
        basic_block::BasicBlockId,
        call_graph::CallGraph,
        dfg::InsertInstructionResult,
        function::{Function, FunctionId},
        instruction::{Instruction, InstructionId, TerminatorInstruction},
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

pub(super) mod inline_info;

pub use inline_info::MAX_INSTRUCTIONS;
pub(super) use inline_info::{InlineInfo, InlineInfos, compute_inline_infos};

/// An arbitrary limit to the maximum number of recursive call
/// frames at any point in time.
const RECURSION_LIMIT: u32 = 1000;

impl Ssa {
    /// Inline all functions within the IR.
    ///
    /// In the case of recursive Acir functions, this will attempt
    /// to recursively inline until the RECURSION_LIMIT is reached.
    ///
    /// Functions are recursively inlined into main until either we finish
    /// inlining all functions or we encounter a function whose function id is not known.
    /// When the later happens, the call instruction is kept in addition to the function
    /// it refers to. The function it refers to is kept unmodified without any inlining
    /// changes. This is because if the function's id later becomes known by a later
    /// pass, we would need to re-run all of inlining anyway to inline it, so we might
    /// as well save the work for later instead of performing it twice.
    ///
    /// There are some attributes that allow inlining a function at a different step of codegen.
    /// Currently this is just `InlineType::NoPredicates` for which we have a flag indicating
    /// whether treating that inline functions. The default is to treat these functions as entry points.
    ///
    /// This step should run after runtime separation, since it relies on the runtime of the called functions being final.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn inline_functions(
        self,
        aggressiveness: i64,
        small_function_max_instructions: usize,
    ) -> Result<Ssa, RuntimeError> {
        self.inline_until_fixed_point(aggressiveness, small_function_max_instructions, false)
    }

    /// Run the inlining pass where functions marked with `InlineType::NoPredicates` as not entry points
    pub(crate) fn inline_functions_with_no_predicates(
        self,
        aggressiveness: i64,
        small_function_max_instructions: usize,
    ) -> Result<Ssa, RuntimeError> {
        self.inline_until_fixed_point(aggressiveness, small_function_max_instructions, true)
    }

    /// Inline functions repeatedly until no new functions are inlined.
    pub(crate) fn inline_until_fixed_point(
        mut self,
        aggressiveness: i64,
        small_function_max_instructions: usize,
        inline_no_predicates_functions: bool,
    ) -> Result<Ssa, RuntimeError> {
        loop {
            let num_functions_before = self.functions.len();

            let call_graph = CallGraph::from_ssa_weighted(&self);

            let inline_infos = compute_inline_infos(
                &self,
                &call_graph,
                inline_no_predicates_functions,
                small_function_max_instructions,
                aggressiveness,
            );

            // Bottom-up order, starting with the "leaf" functions.
            let bottom_up = compute_bottom_up_order(&self, &call_graph);
            let bottom_up = vecmap(bottom_up, |(id, _)| id);

            self = Self::inline_functions_inner(self, &inline_infos, &bottom_up)?;

            let num_functions_after = self.functions.len();
            if num_functions_after == num_functions_before {
                break;
            }
        }

        Ok(self)
    }

    /// Inline entry points in the order of appearance in `inline_infos`, assuming it goes in bottom-up order.
    fn inline_functions_inner(
        mut self,
        inline_infos: &InlineInfos,
        bottom_up: &[FunctionId],
    ) -> Result<Ssa, RuntimeError> {
        let inline_targets = bottom_up
            .iter()
            .filter_map(|id| {
                let info = inline_infos.get(id)?;
                let dfg = &self.functions[id].dfg;
                info.is_inline_target(dfg).then_some(*id)
            })
            .collect::<Vec<_>>();

        let should_inline_call = |callee: &Function| -> bool {
            // We defer to the inline info computation to determine whether a function should be inlined
            InlineInfo::should_inline(inline_infos, callee.id())
        };

        // We are going bottom up, so hopefully we can inline leaf functions into their callers and retain less memory.
        let mut new_functions = HashSet::new();
        for entry_point in inline_targets {
            let function = &self.functions[&entry_point];
            let inlined = function.inlined(&self, &should_inline_call)?;
            assert_eq!(inlined.id(), entry_point);
            self.functions.insert(entry_point, inlined);
            new_functions.insert(entry_point);
        }

        // Drop functions that weren't inline targets.
        self.functions.retain(|id, _| new_functions.contains(id));

        Ok(self)
    }
}

impl Function {
    /// Create a new function which has the functions called by this one inlined into its body.
    pub(super) fn inlined(
        &self,
        ssa: &Ssa,
        should_inline_call: &impl Fn(&Function) -> bool,
    ) -> Result<Function, RuntimeError> {
        InlineContext::new(ssa, self.id()).inline_all(ssa, &should_inline_call)
    }
}

/// The context for the function inlining pass.
///
/// This works using an internal FunctionBuilder to build a new main function from scratch.
/// Doing it this way properly handles importing instructions between functions and lets us
/// reuse the existing API at the cost of essentially cloning each of main's instructions.
struct InlineContext {
    recursion_level: u32,
    builder: FunctionBuilder,

    call_stack: CallStackId,

    // The FunctionId of the entry point function we're inlining into in the old, unmodified Ssa.
    entry_point: FunctionId,
}

/// The per-function inlining context contains information that is only valid for one function.
/// For example, each function has its own DataFlowGraph, and thus each function needs a translation
/// layer to translate between BlockId to BlockId for the current function and the function to
/// inline into. The same goes for ValueIds, InstructionIds, and for storing other data like
/// parameter to argument mappings.
struct PerFunctionContext<'function> {
    /// The function that we are inlining calls into.
    entry_function: &'function Function,

    /// The source function is the function we're currently inlining into the function being built.
    source_function: &'function Function,

    /// The shared inlining context for all functions. This notably contains the FunctionBuilder used
    /// to build the function we're inlining into.
    context: &'function mut InlineContext,

    /// Maps ValueIds in the function being inlined to the new ValueIds to use in the function
    /// being inlined into. This mapping also contains the mapping from parameter values to
    /// argument values.
    values: HashMap<ValueId, ValueId>,

    /// Maps blocks in the source function to blocks in the function being inlined into, where
    /// each mapping is from the start of a source block to an inlined block in which the
    /// analogous program point occurs.
    ///
    /// Note that the starts of multiple source blocks can map into a single inlined block.
    /// Conversely the whole of a source block is not guaranteed to map into a single inlined
    /// block.
    blocks: HashMap<BasicBlockId, BasicBlockId>,

    /// True if we're currently working on the entry point function.
    inlining_entry: bool,
}

impl InlineContext {
    /// Create a new context object for the function inlining pass.
    /// This starts off with an empty mapping of instructions for main's parameters.
    /// The function being inlined into will always be the main function, although it is
    /// actually a copy that is created in case the original main is still needed from a function
    /// that could not be inlined calling it.
    fn new(ssa: &Ssa, entry_point: FunctionId) -> Self {
        let source = &ssa.functions[&entry_point];
        let builder = FunctionBuilder::from_existing(source, entry_point);
        Self { builder, recursion_level: 0, entry_point, call_stack: CallStackId::root() }
    }

    /// Start inlining the entry point function and all functions reachable from it.
    fn inline_all(
        mut self,
        ssa: &Ssa,
        should_inline_call: &impl Fn(&Function) -> bool,
    ) -> Result<Function, RuntimeError> {
        let entry_point = &ssa.functions[&self.entry_point];

        let mut context = PerFunctionContext::new(&mut self, entry_point, entry_point);
        context.inlining_entry = true;

        // The entry block is already inserted so we have to add it to context.blocks and add
        // its parameters here. Failing to do so would cause context.translate_block() to add
        // a fresh block for the entry block rather than use the existing one.
        let entry_block = context.context.builder.current_function.entry_block();
        context.translate_block_parameters(context.source_function.entry_block(), entry_block);
        context.blocks.insert(context.source_function.entry_block(), entry_block);
        context.inline_blocks(ssa, should_inline_call)?;
        // translate databus values
        let databus = entry_point.dfg.data_bus.map_values(|t| context.translate_value(t));

        // Finally, we should have 1 function left representing the inlined version of the target function.
        let mut new_ssa = self.builder.finish();
        assert_eq!(new_ssa.functions.len(), 1);
        let mut new_func = new_ssa.functions.pop_first().unwrap().1;
        new_func.dfg.data_bus = databus;
        Ok(new_func)
    }

    /// Inlines a function into the current function and returns the translated return values
    /// of the inlined function.
    fn inline_function(
        &mut self,
        ssa: &Ssa,
        id: FunctionId,
        arguments: &[ValueId],
        should_inline_call: &impl Fn(&Function) -> bool,
    ) -> Result<Vec<ValueId>, RuntimeError> {
        self.recursion_level += 1;

        let source_function = &ssa.functions[&id];
        if self.recursion_level > RECURSION_LIMIT {
            return Err(RuntimeError::RecursionLimit {
                function_name: source_function.name().to_string(),
                limit: RECURSION_LIMIT,
                call_stack: self.builder.current_function.dfg.get_call_stack(self.call_stack),
            });
        }

        let entry_point = &ssa.functions[&self.entry_point];
        let mut context = PerFunctionContext::new(self, entry_point, source_function);

        let parameters = source_function.parameters();
        assert_eq!(parameters.len(), arguments.len());
        context.values = parameters.iter().copied().zip(arguments.iter().copied()).collect();

        let current_block = context.context.builder.current_block();
        context.blocks.insert(source_function.entry_block(), current_block);

        let return_values = context.inline_blocks(ssa, should_inline_call)?;
        self.recursion_level -= 1;
        Ok(return_values)
    }
}

impl<'function> PerFunctionContext<'function> {
    /// Create a new PerFunctionContext from the source function.
    /// The value and block mappings for this context are initially empty except
    /// for containing the mapping between parameters in the source_function and
    /// the arguments of the destination function.
    fn new(
        context: &'function mut InlineContext,
        entry_function: &'function Function,
        source_function: &'function Function,
    ) -> Self {
        Self {
            context,
            entry_function,
            source_function,
            blocks: HashMap::default(),
            values: HashMap::default(),
            inlining_entry: false,
        }
    }

    /// Translates a ValueId from the function being inlined to a ValueId of the function
    /// being inlined into. Note that this expects value ids for all Value::Instruction and
    /// Value::Param values are already handled as a result of previous inlining of instructions
    /// and blocks respectively. If these assertions trigger it means a value is being used before
    /// the instruction or block that defines the value is inserted.
    fn translate_value(&mut self, id: ValueId) -> ValueId {
        if let Some(value) = self.values.get(&id) {
            return *value;
        }

        let new_value = match &self.source_function.dfg[id] {
            value @ Value::Instruction { .. } => {
                if self.source_function.dfg.is_global(id) {
                    return id;
                }
                unreachable!(
                    "All Value::Instructions should already be known during inlining after creating the original inlined instruction. Unknown value {id} = {value:?}"
                )
            }
            value @ Value::Param { .. } => {
                unreachable!(
                    "All Value::Params should already be known from previous calls to translate_block. Unknown value {id} = {value:?}"
                )
            }
            Value::NumericConstant { constant, typ } => {
                // The dfg indexes a global's inner value directly, so we need to check here
                // whether we have a global.
                if self.source_function.dfg.is_global(id) {
                    id
                } else {
                    self.context.builder.numeric_constant(*constant, *typ)
                }
            }
            Value::Function(function) => self.context.builder.import_function(*function),
            Value::Intrinsic(intrinsic) => self.context.builder.import_intrinsic_id(*intrinsic),
            Value::ForeignFunction(function) => {
                self.context.builder.import_foreign_function(function)
            }
            Value::Global(_) => {
                panic!("Expected a global to be resolved to its inner value");
            }
        };

        self.values.insert(id, new_value);
        new_value
    }

    /// Translates the program point representing the start of the given `source_block` to the
    /// inlined block in which the analogous program point occurs. (Once inlined, the source
    /// block's analogous program region may span multiple inlined blocks.)
    ///
    /// If the block isn't already known, this will insert a new block into the target function
    /// with the same parameter types as the source block.
    fn translate_block(
        &mut self,
        source_block: BasicBlockId,
        block_queue: &mut VisitOnceDeque,
    ) -> BasicBlockId {
        if let Some(block) = self.blocks.get(&source_block) {
            return *block;
        }

        // The block is not yet inlined, queue it
        block_queue.push_back(source_block);

        // The block is not already present in the function being inlined into so we must create it.
        // The block's instructions are not copied over as they will be copied later in inlining.
        let new_block = self.context.builder.insert_block();
        self.translate_block_parameters(source_block, new_block);
        self.blocks.insert(source_block, new_block);
        new_block
    }

    /// Copy block parameters from `source_block` into `target_block`.
    fn translate_block_parameters(
        &mut self,
        source_block: BasicBlockId,
        target_block: BasicBlockId,
    ) {
        let original_parameters = self.source_function.dfg.block_parameters(source_block);
        for parameter in original_parameters {
            let typ = self.source_function.dfg.type_of_value(*parameter);
            let new_parameter = self.context.builder.add_block_parameter(target_block, typ);
            self.values.insert(*parameter, new_parameter);
        }
    }

    /// Try to retrieve the function referred to by the given Id.
    /// Expects that the given ValueId belongs to the source_function.
    ///
    /// Returns None if the id is not known to refer to a function.
    fn get_function(&mut self, mut id: ValueId) -> Option<FunctionId> {
        id = self.translate_value(id);
        match self.context.builder[id] {
            Value::Function(id) => Some(id),
            // We don't set failed_to_inline_a_call for intrinsics since those
            // don't correspond to actual functions in the SSA program that would
            // need to be removed afterward.
            Value::Intrinsic(_) => None,
            _ => None,
        }
    }

    /// Inline all reachable blocks within the source_function into the destination function.
    fn inline_blocks(
        &mut self,
        ssa: &Ssa,
        should_inline_call: &impl Fn(&Function) -> bool,
    ) -> Result<Vec<ValueId>, RuntimeError> {
        let mut block_queue = VisitOnceDeque::default();
        block_queue.push_back(self.source_function.entry_block());

        // This Vec will contain each block with a Return instruction along with the
        // returned values of that block.
        let mut function_returns = vec![];

        while let Some(source_block_id) = block_queue.pop_front() {
            let translated_block_id = self.translate_block(source_block_id, &mut block_queue);
            self.context.builder.switch_to_block(translated_block_id);

            self.inline_block_instructions(ssa, source_block_id, should_inline_call)?;

            if let Some((block, values)) =
                self.handle_terminator_instruction(source_block_id, &mut block_queue)
            {
                function_returns.push((block, values));
            }
        }

        Ok(self.handle_function_returns(function_returns))
    }

    /// Handle inlining a function's possibly multiple return instructions.
    /// If there is only 1 return we can just continue inserting into that block.
    /// If there are multiple, we'll need to create a join block to jump to with each value.
    fn handle_function_returns(
        &mut self,
        mut returns: Vec<(BasicBlockId, Vec<ValueId>)>,
    ) -> Vec<ValueId> {
        match returns.len() {
            0 => Vec::new(),
            1 => {
                let (return_block, return_values) = returns.remove(0);
                self.context.builder.switch_to_block(return_block);
                return_values
            }
            _ => {
                panic!(
                    "ICE: found a function with multiple return terminators, but that should not happen"
                )
            }
        }
    }

    /// Inline each instruction in the given block into the function being inlined into.
    /// This may recurse if it finds another function to inline if a call instruction is within this block.
    fn inline_block_instructions(
        &mut self,
        ssa: &Ssa,
        block_id: BasicBlockId,
        should_inline_call: &impl Fn(&Function) -> bool,
    ) -> Result<(), RuntimeError> {
        let mut side_effects_enabled: Option<ValueId> = None;

        let block = &self.source_function.dfg[block_id];
        for id in block.instructions() {
            match &self.source_function.dfg[*id] {
                Instruction::Call { func, arguments } => match self.get_function(*func) {
                    Some(func_id) => {
                        let call_stack = self.source_function.dfg.get_instruction_call_stack(*id);
                        let callee = &ssa.functions[&func_id];

                        // Sanity check to validate runtime compatibility
                        self.validate_callee(callee, call_stack)?;

                        // Do not inline self-recursive functions on the top level.
                        // Inlining a self-recursive function works when there is something to inline into
                        // by importing all the recursive blocks, but for the entry function there is no wrapper.
                        // We must do this check here as inlining can be can triggered on a non-inline target (e.g., non-entry point).
                        let inlining_self_recursion_at_top_level =
                            self.entry_function.id() == func_id;
                        if !inlining_self_recursion_at_top_level && should_inline_call(callee) {
                            self.inline_function(ssa, *id, func_id, arguments, should_inline_call)?;

                            // This is only relevant during handling functions with `InlineType::NoPredicates` as these
                            // can pollute the function they're being inlined into with `Instruction::EnabledSideEffects`,
                            // resulting in predicates not being applied properly.
                            //
                            // Note that this doesn't cover the case in which there exists an `Instruction::EnableSideEffectsIf`
                            // within the function being inlined whilst the source function has not encountered one yet.
                            // In practice this isn't an issue as the last `Instruction::EnableSideEffectsIf` in the
                            // function being inlined will be to turn off predicates rather than to create one.
                            if let Some(condition) = side_effects_enabled {
                                self.context.builder.insert_enable_side_effects_if(condition);
                            }
                        } else {
                            self.push_instruction(*id);
                        }
                    }
                    None => self.push_instruction(*id),
                },
                Instruction::EnableSideEffectsIf { condition } => {
                    side_effects_enabled = Some(self.translate_value(*condition));
                    self.push_instruction(*id);
                }
                _ => self.push_instruction(*id),
            }
        }
        Ok(())
    }

    /// Extra error check where given a caller's runtime its callee runtime is valid.
    /// We determine validity as the following (where we have caller -> callee).
    /// Valid:
    /// - ACIR -> ACIR
    /// - ACIR -> Brillig
    /// - Brillig -> Brillig
    ///
    /// Invalid:
    /// - Brillig -> ACIR
    ///
    /// Whether a valid callee should be inlined is determined separately by the inline info computation.
    fn validate_callee(
        &self,
        callee: &Function,
        call_stack: Vec<Location>,
    ) -> Result<(), RuntimeError> {
        if self.entry_function.runtime().is_brillig() && callee.runtime().is_acir() {
            // If the caller is Brillig and the called function is ACIR,
            // it cannot be inlined because runtimes do not share the same semantics
            return Err(RuntimeError::UnconstrainedCallingConstrained {
                call_stack,
                constrained: callee.name().to_string(),
                unconstrained: self.entry_function.name().to_string(),
            });
        }

        Ok(())
    }

    /// Inline a function call and remember the inlined return values in the values map
    fn inline_function(
        &mut self,
        ssa: &Ssa,
        call_id: InstructionId,
        function: FunctionId,
        arguments: &[ValueId],
        should_inline_call: &impl Fn(&Function) -> bool,
    ) -> Result<(), RuntimeError> {
        let old_results = self.source_function.dfg.instruction_results(call_id);
        let arguments = vecmap(arguments, |arg| self.translate_value(*arg));

        let call_stack = self.source_function.dfg.get_instruction_call_stack(call_id);
        let call_stack_len = call_stack.len();
        let new_call_stack = self
            .context
            .builder
            .current_function
            .dfg
            .call_stack_data
            .extend_call_stack(self.context.call_stack, &call_stack);

        self.context.call_stack = new_call_stack;
        let new_results =
            self.context.inline_function(ssa, function, &arguments, should_inline_call)?;
        self.context.call_stack = self
            .context
            .builder
            .current_function
            .dfg
            .call_stack_data
            .unwind_call_stack(self.context.call_stack, call_stack_len);

        let new_results = InsertInstructionResult::Results(call_id, &new_results);
        Self::insert_new_instruction_results(&mut self.values, old_results, new_results);
        Ok(())
    }

    /// Push the given instruction from the source_function into the current block of the
    /// function being inlined into.
    fn push_instruction(&mut self, id: InstructionId) {
        let instruction = self.source_function.dfg[id].map_values(|id| self.translate_value(id));

        let mut call_stack = self.context.call_stack;
        let source_call_stack = self.source_function.dfg.get_instruction_call_stack(id);
        call_stack = self
            .context
            .builder
            .current_function
            .dfg
            .call_stack_data
            .extend_call_stack(call_stack, &source_call_stack);
        let results = self.source_function.dfg.instruction_results(id).to_vec();

        let ctrl_typevars = instruction
            .requires_ctrl_typevars()
            .then(|| vecmap(&results, |result| self.source_function.dfg.type_of_value(*result)));

        self.context.builder.set_call_stack(call_stack);

        let new_results = self.context.builder.insert_instruction(instruction, ctrl_typevars);
        Self::insert_new_instruction_results(&mut self.values, &results, new_results);
    }

    /// Modify the values HashMap to remember the mapping between an instruction result's previous
    /// ValueId (from the source_function) and its new ValueId in the destination function.
    fn insert_new_instruction_results(
        values: &mut HashMap<ValueId, ValueId>,
        old_results: &[ValueId],
        new_results: InsertInstructionResult,
    ) {
        assert_eq!(old_results.len(), new_results.len());
        for i in 0..old_results.len() {
            values.insert(old_results[i], new_results[i]);
        }
    }

    /// Handle the given terminator instruction from the given source function block.
    /// This will push any new blocks to the destination function as needed, add them
    /// to the block queue, and set the terminator instruction for the current block.
    ///
    /// If the terminator instruction was a Return, this will return the block this instruction
    /// was in as well as the values that were returned.
    fn handle_terminator_instruction(
        &mut self,
        block_id: BasicBlockId,
        block_queue: &mut VisitOnceDeque,
    ) -> Option<(BasicBlockId, Vec<ValueId>)> {
        match &self.source_function.dfg[block_id].unwrap_terminator() {
            TerminatorInstruction::Jmp { destination, arguments, call_stack } => {
                let destination = self.translate_block(*destination, block_queue);
                let arguments = vecmap(arguments, |arg| self.translate_value(*arg));
                self.extend_call_stack(*call_stack);
                self.context.builder.terminate_with_jmp(destination, arguments);
                None
            }
            TerminatorInstruction::JmpIf {
                condition,
                then_destination,
                else_destination,
                call_stack,
            } => {
                let condition = self.translate_value(*condition);

                // See if the value of the condition is known, and if so only inline the reachable
                // branch. This lets us inline some recursive functions without recurring forever.
                let dfg = &mut self.context.builder.current_function.dfg;
                match dfg.get_numeric_constant(condition) {
                    Some(constant) => {
                        let next_block =
                            if constant.is_zero() { *else_destination } else { *then_destination };

                        let next_block = self.translate_block(next_block, block_queue);
                        self.extend_call_stack(*call_stack);
                        self.context.builder.terminate_with_jmp(next_block, vec![]);
                    }
                    None => {
                        let then_block = self.translate_block(*then_destination, block_queue);
                        let else_block = self.translate_block(*else_destination, block_queue);
                        self.extend_call_stack(*call_stack);
                        self.context
                            .builder
                            .terminate_with_jmpif(condition, then_block, else_block);
                    }
                }
                None
            }
            TerminatorInstruction::Return { return_values, call_stack } => {
                let return_values = vecmap(return_values, |value| self.translate_value(*value));

                // Note that `translate_block` would take us back to the point at which the
                // inlining of this source block began. Since additional blocks may have been
                // inlined since, we are interested in the block representing the current program
                // point, obtained via `current_block`.
                let block_id = self.context.builder.current_block();

                if self.inlining_entry {
                    self.extend_call_stack(*call_stack);
                    self.context.builder.terminate_with_return(return_values.clone());
                }

                Some((block_id, return_values))
            }
            TerminatorInstruction::Unreachable { .. } => {
                // Note: `unreachable` terminators are only added during the `remove_unreachable_instructions`,
                // which runs near the end of the optimization pipeline, so never before inlining.
                // If we ever want to run it before inlining we'll have to handle this case.
                panic!("Unreachable terminator instruction should not exist during inlining.")
            }
        }
    }

    fn extend_call_stack(&mut self, call_stack: CallStackId) {
        let call_stack = self.source_function.dfg.get_call_stack(call_stack);
        let call_stack_data = &mut self.context.builder.current_function.dfg.call_stack_data;
        let new_call_stack =
            call_stack_data.extend_call_stack(self.context.call_stack, &call_stack);
        self.context.builder.set_call_stack(new_call_stack);
    }
}

#[cfg(test)]
mod test {
    use crate::{
        assert_ssa_snapshot,
        errors::RuntimeError,
        ssa::{
            Ssa,
            ir::{instruction::TerminatorInstruction, map::Id},
            opt::assert_normalized_ssa_equals,
        },
    };

    // We set zero for `small_function_max_instructions` as to avoid the maximum weight threshold at which we always inline a function.
    const MAX_INSTRUCTIONS: usize = 0;

    #[test]
    fn basic_inlining() {
        let src = "
        acir(inline) fn foo f0 {
          b0():
            v1 = call f1() -> Field
            return v1
        }

        acir(inline) fn bar f1 {
          b0():
            return Field 72
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.inline_functions(i64::MAX, MAX_INSTRUCTIONS).unwrap();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn foo f0 {
          b0():
            return Field 72
        }
        ");
    }

    #[test]
    fn basic_inlining_brillig_not_inlined_into_acir() {
        // This test matches the `basic_inlining` test exactly except that f1 is marked as a Brillig runtime.
        // We expect that Brillig entry points (e.g., Brillig functions called from ACIR) should never be inlined.
        let src = "
        acir(inline) fn foo f0 {
          b0():
            v1 = call f1() -> Field
            return v1
        }
        brillig(inline) fn bar f1 {
          b0():
            return Field 72
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.inline_functions(i64::MAX, MAX_INSTRUCTIONS).unwrap();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn complex_inlining() {
        // This SSA is from issue #1327 which previously failed to inline properly
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v4 = call f2(f1) -> function
            v5 = call f3(v4) -> function
            v6 = call v5(v0) -> Field
            return v6
        }

        acir(inline) fn square f1 {
          b0(v0: Field):
            v1 = mul v0, v0
            return v1
        }

        acir(inline) fn id1 f2 {
          b0(v0: function):
            return v0
        }

        acir(inline) fn id2 f3 {
          b0(v0: function):
            return v0
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.inline_functions(i64::MAX, MAX_INSTRUCTIONS).unwrap();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: Field):
            v1 = mul v0, v0
            return v1
        }
        ");
    }

    #[test]
    fn recursive_functions() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v2 = call f1(u32 5) -> u32
            return v2
        }
        acir(inline) fn factorial f1 {
          b0(v1: u32):
            v2 = lt v1, u32 1
            jmpif v2 then: b1, else: b2
          b1():
            jmp b3(u32 1)
          b2():
            v4 = sub v1, u32 1
            v5 = call f1(v4) -> u32
            v6 = mul v1, v5
            jmp b3(v6)
          b3(v7: u32):
            return v7
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.inline_functions(i64::MAX, MAX_INSTRUCTIONS).unwrap();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            jmp b1()
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
            jmp b7(u32 1)
          b7(v0: u32):
            jmp b8(v0)
          b8(v1: u32):
            v8 = mul u32 2, v1
            jmp b9(v8)
          b9(v2: u32):
            v10 = mul u32 3, v2
            jmp b10(v10)
          b10(v3: u32):
            v12 = mul u32 4, v3
            jmp b11(v12)
          b11(v4: u32):
            v14 = mul u32 5, v4
            jmp b12(v14)
          b12(v5: u32):
            return v5
        }
        ");
    }

    /// This test is the same as [recursive_functions] we just want to test that inlining
    /// does not fail when triggered from the self recursive non-entry point function instead
    /// of the program entry point.
    #[test]
    fn recursive_functions_non_inline_target() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v2 = call f1(u32 5) -> u32
            return v2
        }
        acir(inline) fn factorial f1 {
          b0(v1: u32):
            v2 = lt v1, u32 1
            jmpif v2 then: b1, else: b2
          b1():
            jmp b3(u32 1)
          b2():
            v4 = sub v1, u32 1
            v5 = call f1(v4) -> u32
            v6 = mul v1, v5
            jmp b3(v6)
          b3(v7: u32):
            return v7
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let f1 = &ssa.functions[&Id::test_new(1)];
        let function = f1.inlined(&ssa, &|_| true).unwrap();
        // The expected string must be formatted this way as to account for newlines and whitespace
        assert_eq!(
            function.to_string(),
            "acir(inline) fn factorial f1 {
  b0(v0: u32):
    v3 = eq v0, u32 0
    jmpif v3 then: b1, else: b2
  b1():
    jmp b3(u32 1)
  b2():
    v5 = sub v0, u32 1
    v7 = call f1(v5) -> u32
    v8 = mul v0, v7
    jmp b3(v8)
  b3(v4: u32):
    return v4
}"
        );
    }

    #[test]
    fn displaced_return_mapping() {
        // This test is designed specifically to catch a regression in which the ids of blocks
        // terminated by returns are badly tracked. As a result, the continuation of a source
        // block after a call instruction could but inlined into a block that's already been
        // terminated, producing an incorrect order and orphaning successors.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            jmpif v0 then: b1, else: b2
          b1():
            jmp b3(Field 1)
          b2():
            jmp b3(Field 2)
          b3(v3: Field):
            call assert_constant(v3)
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.inline_functions(i64::MAX, MAX_INSTRUCTIONS).unwrap();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1):
            jmpif v0 then: b1, else: b2
          b1():
            jmp b3(Field 1)
          b2():
            jmp b3(Field 2)
          b3(v1: Field):
            call assert_constant(v1)
            return
        }
        ");
    }

    #[test]
    fn unconditional_recursion() {
        // f1 is calling itself, which results in an infinite recursion
        // it is expected that inlining this program returns an error.
        let src = "
        acir(inline) fn main f0 {
          b0():
            call f1()
            return
        }
        acir(inline) fn foo f1 {
          b0():
            call f1()
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        assert_eq!(ssa.functions.len(), 2);

        let ssa = ssa.inline_functions(i64::MAX, MAX_INSTRUCTIONS);
        let Err(err) = ssa else {
            panic!("inline_functions cannot inline recursive functions");
        };
        insta::assert_snapshot!(err.to_string(), @"Attempted to recurse more than 1000 times during inlining function 'foo'");
    }

    #[test]
    fn inliner_disabled() {
        let src = "
        brillig(inline) fn foo f0 {
          b0():
            v1 = call f1() -> Field
            return v1
        }
        brillig(inline) fn bar f1 {
          b0():
            return Field 72
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.inline_functions(i64::MIN, MAX_INSTRUCTIONS).unwrap();
        // No inlining has happened
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn conditional_inlining() {
        // In this example we call a larger brillig function 3 times so the inliner refuses to inline the function.
        let src = "
        brillig(inline) fn foo f0 {
          b0():
            v1 = call f1() -> Field
            v2 = call f1() -> Field
            v3 = call f1() -> Field
            return v1
        }

        brillig(inline) fn bar f1 {
          b0():
            jmpif u1 1 then: b1, else: b2
          b1():
            jmp b3(Field 1)
          b2():
            jmp b3(Field 2)
          b3(v3: Field):
            return v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.inline_functions(0, MAX_INSTRUCTIONS).unwrap();
        // No inlining has happened in f0
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn foo f0 {
          b0():
            v1 = call f1() -> Field
            v2 = call f1() -> Field
            v3 = call f1() -> Field
            return v1
        }
        brillig(inline) fn bar f1 {
          b0():
            jmp b1()
          b1():
            jmp b2(Field 1)
          b2(v0: Field):
            return v0
        }
        ");
    }

    #[test]
    fn conditional_inlining_const_from_param_and_direct_constant() {
        let src = "
        brillig(inline) fn foo f0 {
          b0():
            v1 = call f1() -> Field
            v2 = call f2(u1 1) -> Field
            v3 = call f2(u1 0) -> Field
            return v1, v2, v3
        }
        brillig(inline) fn bar f1 {
          b0():
            jmpif u1 1 then: b1, else: b2
          b1():
            jmp b3(Field 1)
          b2():
            jmp b3(Field 2)
          b3(v3: Field):
            return v3
        }
        brillig(inline) fn baz f2 {
          b0(v0: u1):
            jmpif v0 then: b1, else: b2
          b1():
            jmp b3(Field 1)
          b2():
            jmp b3(Field 2)
          b3(v3: Field):
            return v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.inline_functions(i64::MAX, MAX_INSTRUCTIONS).unwrap();

        // We expect a block from all calls to f1 and f2 to be pruned and that the constant argument to the f2 call
        // is propagated to the jmpif conditional in b0.
        // Field 1 to be returned from the first call to f2 and Field 2 should be returned from the second call to f2.
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn foo f0 {
          b0():
            jmp b1()
          b1():
            jmp b2(Field 1)
          b2(v0: Field):
            jmp b3()
          b3():
            jmp b4(Field 1)
          b4(v1: Field):
            jmp b5()
          b5():
            jmp b6(Field 2)
          b6(v2: Field):
            return v0, v1, v2
        }
        ");
    }

    #[test]
    fn static_assertions_to_always_be_inlined() {
        let src = "
        brillig(inline) fn main f0 {
            b0():
              call f1(Field 1)
              return
        }
        brillig(inline) fn foo f1 {
            b0(v0: Field):
              call assert_constant(v0)
              return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.inline_functions(i64::MAX, MAX_INSTRUCTIONS).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            return
        }
        ");
    }

    #[test]
    fn no_predicates_flag_inactive() {
        let src = "
        acir(inline) fn main f0 {
            b0():
              call f1()
              return
        }
        acir(no_predicates) fn no_predicates f1 {
            b0():
              return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.inline_functions(i64::MAX, MAX_INSTRUCTIONS).unwrap();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn no_predicates_flag_active() {
        let src = "
        acir(inline) fn main f0 {
            b0():
              call f1()
              return
        }
        acir(no_predicates) fn no_predicates f1 {
            b0():
              return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.inline_functions_with_no_predicates(i64::MAX, MAX_INSTRUCTIONS).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            return
        }
        ");
    }

    #[test]
    fn inline_always_function() {
        let src = "
        brillig(inline) fn main f0 {
            b0():
              call f1()
              return
        }
        brillig(inline_always) fn always_inline f1 {
            b0():
                return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.inline_functions(i64::MIN, MAX_INSTRUCTIONS).unwrap();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            return
        }
        ");

        // Check that with a minimum inliner aggressiveness we do not inline a function
        // not marked with `inline_always`
        let no_inline_always_src = &src.replace("inline_always", "inline");
        let ssa = Ssa::from_str(no_inline_always_src).unwrap();
        let ssa = ssa.inline_functions(i64::MIN, MAX_INSTRUCTIONS).unwrap();
        assert_normalized_ssa_equals(ssa, no_inline_always_src);
    }

    #[test]
    fn acir_global_arrays_keep_same_value_ids() {
        let src = "
        g0 = Field 1
        g1 = Field 2
        g2 = make_array [Field 1, Field 2] : [Field; 2]

        acir(inline) fn main f0 {
          b0():
            v0 = call f1() -> [Field; 2]
            return v0
        }
        acir(inline) fn create_array f1 {
          b0():
            return g2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.inline_functions(i64::MAX, MAX_INSTRUCTIONS).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        g0 = Field 1
        g1 = Field 2
        g2 = make_array [Field 1, Field 2] : [Field; 2]

        acir(inline) fn main f0 {
          b0():
            return g2
        }
        ");
    }

    #[test]
    fn brillig_global_arrays_keep_same_value_ids() {
        let src = "
        g0 = Field 1
        g1 = Field 2
        g2 = make_array [Field 1, Field 2] : [Field; 2]

        brillig(inline) fn main f0 {
          b0():
            v0 = call f1() -> [Field; 2]
            return v0
        }
        brillig(inline) fn create_array f1 {
          b0():
            return g2
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.inline_functions(i64::MAX, MAX_INSTRUCTIONS).unwrap();

        assert_ssa_snapshot!(ssa, @r"
        g0 = Field 1
        g1 = Field 2
        g2 = make_array [Field 1, Field 2] : [Field; 2]

        brillig(inline) fn main f0 {
          b0():
            return g2
        }
        ");
    }

    #[test]
    fn acir_global_constants_keep_same_value_ids() {
        let src = "
        g0 = Field 1

        acir(inline) fn main f0 {
          b0():
            v0 = call f1() -> Field
            return v0
        }
        acir(inline) fn get_constant f1 {
          b0():
            return g0
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.inline_functions(i64::MAX, MAX_INSTRUCTIONS).unwrap();

        // The string output of global constants resolve to their inner values, so we need to check whether they are globals explicitly.
        let main = ssa.main();
        let entry_block = main.entry_block();
        let terminator = main.dfg[entry_block].unwrap_terminator();
        let TerminatorInstruction::Return { return_values, .. } = terminator else {
            panic!("Expected return");
        };
        assert_eq!(return_values.len(), 1);
        assert!(main.dfg.is_global(return_values[0]));

        assert_ssa_snapshot!(ssa, @r"
        g0 = Field 1

        acir(inline) fn main f0 {
          b0():
            return Field 1
        }
        ");
    }

    #[test]
    fn brillig_global_constants_keep_same_value_ids() {
        let src = "
        g0 = Field 1

        brillig(inline) fn main f0 {
          b0():
            v0 = call f1() -> Field
            return v0
        }
        brillig(inline) fn get_constant f1 {
          b0():
            return g0
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.inline_functions(i64::MAX, MAX_INSTRUCTIONS).unwrap();

        // The string output of global constants resolve to their inner values, so we need to check whether they are globals explicitly.
        let main = ssa.main();
        let entry_block = main.entry_block();
        let terminator = main.dfg[entry_block].unwrap_terminator();
        let TerminatorInstruction::Return { return_values, .. } = terminator else {
            panic!("Expected return");
        };
        assert_eq!(return_values.len(), 1);
        assert!(main.dfg.is_global(return_values[0]));

        assert_ssa_snapshot!(ssa, @r"
        g0 = Field 1

        brillig(inline) fn main f0 {
          b0():
            return Field 1
        }
        ");
    }

    #[test]
    #[should_panic(
        expected = "Unreachable terminator instruction should not exist during inlining"
    )]
    fn inlining_unreachable_block() {
        let src = "
        acir(inline) fn foo f0 {
          b0():
            v1 = call f1() -> Field
            return v1
        }
        acir(inline) fn bar f1 {
          b0():
            unreachable
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let _ = ssa.inline_functions(i64::MAX, MAX_INSTRUCTIONS).unwrap();
    }

    #[test]
    // We should not inline an ACIR function called from a Brillig function because ACIR and Brillig semantics are different.
    fn inlining_acir_into_brillig_function() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            call f1(v0)
            return
        }
        acir(inline) fn foo f1 {
          b0(v0: u32):
            v4 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
            v5 = array_get v4, index v0 -> Field
            return
        }
        ";
        let ssa = Ssa::from_str_no_validation(src).unwrap();
        let ssa = ssa.inline_functions(i64::MAX, MAX_INSTRUCTIONS);
        if !matches!(ssa, Err(RuntimeError::UnconstrainedCallingConstrained { .. })) {
            panic!("Expected inlining to fail with RuntimeError::UnconstrainedCallingConstrained");
        }
    }

    #[test]
    fn does_not_inline_acir_fold_functions() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v3 = call f1(v0, v1) -> Field
            v4 = call f1(v0, v1) -> Field
            v5 = call f1(v0, v1) -> Field
            v6 = eq v3, v4
            constrain v3 == v4
            v7 = eq v4, v5
            constrain v4 == v5
            return
        }
        acir(fold) fn foo f1 {
          b0(v0: Field, v1: Field):
            v2 = eq v0, v1
            v3 = not v2
            constrain v2 == u1 0
            return v0
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        assert_normalized_ssa_equals(ssa, src);
    }
}

/// This test module contains tests specifically for inlining small functions which we always expect to be inlined.
#[cfg(test)]
mod simple_functions {
    use crate::{
        assert_ssa_snapshot,
        ssa::{
            Ssa,
            opt::{assert_normalized_ssa_equals, inlining::MAX_INSTRUCTIONS},
        },
    };

    fn assert_does_not_inline(src: &str) {
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.inline_functions(i64::MAX, MAX_INSTRUCTIONS).unwrap();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn inline_functions_with_zero_instructions() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v2 = call f1(v0) -> Field
            v3 = call f1(v0) -> Field
            v4 = add v2, v3
            return v4
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field):
            return v0
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.inline_functions(i64::MIN, MAX_INSTRUCTIONS).unwrap();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = add v0, v0
            return v1
        }
        ");
    }

    #[test]
    fn inline_functions_that_require_multiple_inlines() {
        // f2 has greater than 10 instructions, which should initially prevent it from being inlined into f0.
        // However, once f1 is inlined into f2, we should be able to fully inline into f0.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = call f2(v0) -> Field
            return v1
        }
        brillig(inline) fn foo f1 {
          b0(v0: Field):
            return v0
        }
        brillig(inline) fn bar f2 {
          b0(v0: Field):
            v1 = call f1(v0) -> Field
            v2 = call f1(v0) -> Field
            v3 = call f1(v0) -> Field
            v4 = call f1(v0) -> Field
            v5 = call f1(v0) -> Field
            v6 = call f1(v0) -> Field
            v7 = call f1(v0) -> Field
            v8 = call f1(v0) -> Field
            v9 = call f1(v0) -> Field
            v10 = call f1(v0) -> Field
            v11 = add v1, v2
            return v11
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let mut ssa = ssa.inline_functions(i64::MIN, MAX_INSTRUCTIONS).unwrap();
        assert_ssa_snapshot!(&mut ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = add v0, v0
            return v1
        }
        ");
    }

    #[test]
    fn inline_functions_with_one_instruction() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v2 = call f1(v0) -> Field
            v3 = call f1(v0) -> Field
            v4 = add v2, v3
            return v4
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field):
            v2 = add v0, Field 1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.inline_functions(i64::MIN, MAX_INSTRUCTIONS).unwrap();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v2 = add v0, Field 1
            v3 = add v0, Field 1
            v4 = add v2, v3
            return v4
        }
        ");
    }

    #[test]
    fn does_not_inline_function_with_one_instruction_that_calls_itself() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = call f1(v0) -> Field
            return v1
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field):
            v1 = call f1(v0) -> Field
            return v1
        }
        ";
        assert_does_not_inline(src);
    }

    #[test]
    fn does_not_inline_acir_functions_with_no_predicates() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2 = call f1(v0) -> Field
            v3 = call f1(v0) -> Field
            v4 = add v2, v3
            return v4
        }

        acir(no_predicates) fn foo f1 {
          b0(v0: Field):
            v2 = add v0, Field 1
            return v2
        }
        ";
        assert_does_not_inline(src);
    }

    #[test]
    fn does_inline_brillig_functions_with_no_predicates() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v2 = call f1(v0) -> Field
            v3 = call f1(v0) -> Field
            v4 = add v2, v3
            return v4
        }

        brillig(no_predicates) fn foo f1 {
          b0(v0: Field):
            v2 = add v0, Field 1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.inline_functions(i64::MIN, MAX_INSTRUCTIONS).unwrap();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v2 = add v0, Field 1
            v3 = add v0, Field 1
            v4 = add v2, v3
            return v4
        }
        ");
    }

    #[test]
    fn does_not_inline_brillig_entry_point_functions() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v2 = call f1(v0) -> Field
            v3 = call f1(v0) -> Field
            v4 = add v2, v3
            return v4
        }
        brillig(inline) fn foo f1 {
          b0(v0: Field):
            v2 = add v0, Field 1
            return v2
        }
        ";
        assert_does_not_inline(src);
    }

    #[test]
    fn does_not_inline_mutually_recursive_functions_acir() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            call f1()
            return
        }
        acir(inline) fn starter f1 {
          b0():
            call f2()
            return
        }
        acir(inline) fn main f2 {
          b0():
            call f1()
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.inline_functions(i64::MIN, MAX_INSTRUCTIONS);

        let Err(err) = ssa else {
            panic!("inline_functions cannot inline recursive functions");
        };
        insta::assert_snapshot!(err.to_string(), @"Attempted to recurse more than 1000 times during inlining function 'starter'");
    }

    #[test]
    fn does_not_inline_mutually_recursive_functions_brillig() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            call f1()
            return
        }
        brillig(inline) fn starter f1 {
          b0():
            call f2()
            return
        }
        brillig(inline) fn ping f2 {
          b0():
            call f3()
            return
        }
        brillig(inline) fn pong f3 {
          b0():
            call f2()
            return
        }
        ";
        assert_does_not_inline(src);
    }

    #[test]
    fn does_not_inline_function_with_multiple_instructions() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v1 = call f1(v0) -> Field
            return v1
        }

        brillig(inline) fn foo f1 {
          b0(v0: Field):
            v1 = add v0, Field 1
            v2 = mul v1, Field 2
            v3 = mul v2, Field 2
            v4 = mul v3, Field 2
            v5 = mul v4, Field 2
            v6 = mul v5, Field 2
            v7 = mul v6, Field 2
            v8 = mul v7, Field 2
            v9 = mul v8, Field 2
            v10 = mul v9, Field 2
            v11 = mul v10, Field 2
            return v11
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.inline_functions(i64::MIN, MAX_INSTRUCTIONS).unwrap();
        assert_normalized_ssa_equals(ssa, src);
    }
}
