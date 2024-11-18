//! This module defines the function inlining pass for the SSA IR.
//! The purpose of this pass is to inline the instructions of each function call
//! within the function caller. If all function calls are known, there will only
//! be a single function remaining when the pass finishes.
use std::collections::{BTreeSet, HashSet, VecDeque};

use acvm::acir::AcirField;
use iter_extended::{btree_map, vecmap};

use crate::ssa::{
    function_builder::FunctionBuilder,
    ir::{
        basic_block::BasicBlockId,
        dfg::{CallStack, InsertInstructionResult},
        function::{Function, FunctionId, RuntimeType},
        instruction::{Instruction, InstructionId, TerminatorInstruction},
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};
use fxhash::FxHashMap as HashMap;

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
    pub(crate) fn inline_functions(self, aggressiveness: i64) -> Ssa {
        Self::inline_functions_inner(self, aggressiveness, false)
    }

    // Run the inlining pass where functions marked with `InlineType::NoPredicates` as not entry points
    pub(crate) fn inline_functions_with_no_predicates(self, aggressiveness: i64) -> Ssa {
        Self::inline_functions_inner(self, aggressiveness, true)
    }

    fn inline_functions_inner(
        mut self,
        aggressiveness: i64,
        inline_no_predicates_functions: bool,
    ) -> Ssa {
        let inline_sources =
            get_functions_to_inline_into(&self, inline_no_predicates_functions, aggressiveness);
        self.functions = btree_map(&inline_sources, |entry_point| {
            let new_function = InlineContext::new(
                &self,
                *entry_point,
                inline_no_predicates_functions,
                inline_sources.clone(),
            )
            .inline_all(&self);
            (*entry_point, new_function)
        });
        self
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

    call_stack: CallStack,

    // The FunctionId of the entry point function we're inlining into in the old, unmodified Ssa.
    entry_point: FunctionId,

    /// Whether the inlining pass should inline any functions marked with [`InlineType::NoPredicates`]
    /// or whether these should be preserved as entrypoint functions.
    ///
    /// This is done as we delay inlining of functions with the attribute `#[no_predicates]` until after
    /// the control flow graph has been flattened.
    inline_no_predicates_functions: bool,

    // These are the functions of the program that we shouldn't inline.
    functions_not_to_inline: BTreeSet<FunctionId>,
}

/// The per-function inlining context contains information that is only valid for one function.
/// For example, each function has its own DataFlowGraph, and thus each function needs a translation
/// layer to translate between BlockId to BlockId for the current function and the function to
/// inline into. The same goes for ValueIds, InstructionIds, and for storing other data like
/// parameter to argument mappings.
struct PerFunctionContext<'function> {
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

/// Utility function to find out the direct calls of a function.
fn called_functions_vec(func: &Function) -> Vec<FunctionId> {
    let mut called_function_ids = Vec::new();
    for block_id in func.reachable_blocks() {
        for instruction_id in func.dfg[block_id].instructions() {
            let Instruction::Call { func: called_value_id, .. } = &func.dfg[*instruction_id] else {
                continue;
            };

            if let Value::Function(function_id) = func.dfg[*called_value_id] {
                called_function_ids.push(function_id);
            }
        }
    }

    called_function_ids
}

/// Utility function to find out the deduplicated direct calls of a function.
fn called_functions(func: &Function) -> BTreeSet<FunctionId> {
    called_functions_vec(func).into_iter().collect()
}

/// The functions we should inline into (and that should be left in the final program) are:
///  - main
///  - Any Brillig function called from Acir
///  - Some Brillig functions depending on aggressiveness and some metrics
///  - Any Acir functions with a [fold inline type][InlineType::Fold],
fn get_functions_to_inline_into(
    ssa: &Ssa,
    inline_no_predicates_functions: bool,
    aggressiveness: i64,
) -> BTreeSet<FunctionId> {
    let mut brillig_entry_points = BTreeSet::default();
    let mut acir_entry_points = BTreeSet::default();

    if matches!(ssa.main().runtime(), RuntimeType::Brillig(_)) {
        brillig_entry_points.insert(ssa.main_id);
    } else {
        acir_entry_points.insert(ssa.main_id);
    }

    for (func_id, function) in ssa.functions.iter() {
        if matches!(function.runtime(), RuntimeType::Brillig(_)) {
            continue;
        }

        // If we have not already finished the flattening pass, functions marked
        // to not have predicates should be preserved.
        let preserve_function = !inline_no_predicates_functions && function.is_no_predicates();
        if function.runtime().is_entry_point() || preserve_function {
            acir_entry_points.insert(*func_id);
        }

        for called_function_id in called_functions(function) {
            if matches!(ssa.functions[&called_function_id].runtime(), RuntimeType::Brillig(_)) {
                brillig_entry_points.insert(called_function_id);
            }
        }
    }

    let times_called = compute_times_called(ssa);

    let brillig_functions_to_retain: BTreeSet<_> = compute_functions_to_retain(
        ssa,
        &brillig_entry_points,
        &times_called,
        inline_no_predicates_functions,
        aggressiveness,
    );

    acir_entry_points
        .into_iter()
        .chain(brillig_entry_points)
        .chain(brillig_functions_to_retain)
        .collect()
}

fn compute_times_called(ssa: &Ssa) -> HashMap<FunctionId, usize> {
    ssa.functions
        .iter()
        .flat_map(|(_caller_id, function)| {
            let called_functions_vec = called_functions_vec(function);
            called_functions_vec.into_iter()
        })
        .chain(std::iter::once(ssa.main_id))
        .fold(HashMap::default(), |mut map, func_id| {
            *map.entry(func_id).or_insert(0) += 1;
            map
        })
}

fn should_retain_recursive(
    ssa: &Ssa,
    func: FunctionId,
    times_called: &HashMap<FunctionId, usize>,
    should_retain_function: &mut HashMap<FunctionId, (bool, i64)>,
    mut explored_functions: im::HashSet<FunctionId>,
    inline_no_predicates_functions: bool,
    aggressiveness: i64,
) {
    // We have already decided on this function
    if should_retain_function.get(&func).is_some() {
        return;
    }
    // Recursive, this function won't be inlined
    if explored_functions.contains(&func) {
        should_retain_function.insert(func, (true, 0));
        return;
    }
    explored_functions.insert(func);

    // Decide on dependencies first
    let called_functions = called_functions(&ssa.functions[&func]);
    for function in called_functions.iter() {
        should_retain_recursive(
            ssa,
            *function,
            times_called,
            should_retain_function,
            explored_functions.clone(),
            inline_no_predicates_functions,
            aggressiveness,
        );
    }
    // We could have decided on this function while deciding on dependencies
    // If the function is recursive
    if should_retain_function.get(&func).is_some() {
        return;
    }

    // We'll use some heuristics to decide whether to inline or not.
    // We compute the weight (roughly the number of instructions) of the function after inlining
    // And the interface cost of the function (the inherent cost at the callsite, roughly the number of args and returns)
    // We then can compute an approximation of the cost of inlining vs the cost of retaining the function
    // We do this computation using saturating i64s to avoid overflows
    let inlined_function_weights: i64 = called_functions.iter().fold(0, |acc, called_function| {
        let (should_retain, weight) = should_retain_function[called_function];
        if should_retain {
            acc
        } else {
            acc.saturating_add(weight)
        }
    });

    let this_function_weight = inlined_function_weights
        .saturating_add(compute_function_own_weight(&ssa.functions[&func]) as i64);

    let interface_cost = compute_function_interface_cost(&ssa.functions[&func]) as i64;

    let times_called = times_called[&func] as i64;

    let inline_cost = times_called.saturating_mul(this_function_weight);
    let retain_cost = times_called.saturating_mul(interface_cost) + this_function_weight;

    let runtime = ssa.functions[&func].runtime();
    // We inline if the aggressiveness is higher than inline cost minus the retain cost
    // If aggressiveness is infinite, we'll always inline
    // If aggressiveness is 0, we'll inline when the inline cost is lower than the retain cost
    // If aggressiveness is minus infinity, we'll never inline (other than in the mandatory cases)
    let should_inline = ((inline_cost.saturating_sub(retain_cost)) < aggressiveness)
        || runtime.is_inline_always()
        || (runtime.is_no_predicates() && inline_no_predicates_functions);

    should_retain_function.insert(func, (!should_inline, this_function_weight));
}

fn compute_functions_to_retain(
    ssa: &Ssa,
    entry_points: &BTreeSet<FunctionId>,
    times_called: &HashMap<FunctionId, usize>,
    inline_no_predicates_functions: bool,
    aggressiveness: i64,
) -> BTreeSet<FunctionId> {
    let mut should_retain_function = HashMap::default();

    for entry_point in entry_points.iter() {
        should_retain_recursive(
            ssa,
            *entry_point,
            times_called,
            &mut should_retain_function,
            im::HashSet::default(),
            inline_no_predicates_functions,
            aggressiveness,
        );
    }

    should_retain_function
        .into_iter()
        .filter_map(
            |(func_id, (should_retain, _))| {
                if should_retain {
                    Some(func_id)
                } else {
                    None
                }
            },
        )
        .collect()
}

fn compute_function_own_weight(func: &Function) -> usize {
    let mut weight = 0;
    for block_id in func.reachable_blocks() {
        weight += func.dfg[block_id].instructions().len() + 1; // We add one for the terminator
    }
    // We use an approximation of the average increase in instruction ratio from SSA to Brillig
    // In order to get the actual weight we'd need to codegen this function to brillig.
    weight
}

fn compute_function_interface_cost(func: &Function) -> usize {
    func.parameters().len() + func.returns().len()
}

impl InlineContext {
    /// Create a new context object for the function inlining pass.
    /// This starts off with an empty mapping of instructions for main's parameters.
    /// The function being inlined into will always be the main function, although it is
    /// actually a copy that is created in case the original main is still needed from a function
    /// that could not be inlined calling it.
    fn new(
        ssa: &Ssa,
        entry_point: FunctionId,
        inline_no_predicates_functions: bool,
        functions_not_to_inline: BTreeSet<FunctionId>,
    ) -> InlineContext {
        let source = &ssa.functions[&entry_point];
        let mut builder = FunctionBuilder::new(source.name().to_owned(), entry_point);
        builder.set_runtime(source.runtime());
        Self {
            builder,
            recursion_level: 0,
            entry_point,
            call_stack: CallStack::new(),
            inline_no_predicates_functions,
            functions_not_to_inline,
        }
    }

    /// Start inlining the entry point function and all functions reachable from it.
    fn inline_all(mut self, ssa: &Ssa) -> Function {
        let entry_point = &ssa.functions[&self.entry_point];

        let mut context = PerFunctionContext::new(&mut self, entry_point);
        context.inlining_entry = true;

        // The entry block is already inserted so we have to add it to context.blocks and add
        // its parameters here. Failing to do so would cause context.translate_block() to add
        // a fresh block for the entry block rather than use the existing one.
        let entry_block = context.context.builder.current_function.entry_block();
        let original_parameters = context.source_function.parameters();

        for parameter in original_parameters {
            let typ = context.source_function.dfg.type_of_value(*parameter);
            let new_parameter = context.context.builder.add_block_parameter(entry_block, typ);
            context.values.insert(*parameter, new_parameter);
        }

        context.blocks.insert(context.source_function.entry_block(), entry_block);
        context.inline_blocks(ssa);
        // translate databus values
        let databus = entry_point.dfg.data_bus.map_values(|t| context.translate_value(t));

        // Finally, we should have 1 function left representing the inlined version of the target function.
        let mut new_ssa = self.builder.finish();
        assert_eq!(new_ssa.functions.len(), 1);
        let mut new_func = new_ssa.functions.pop_first().unwrap().1;
        new_func.dfg.data_bus = databus;
        new_func
    }

    /// Inlines a function into the current function and returns the translated return values
    /// of the inlined function.
    fn inline_function(
        &mut self,
        ssa: &Ssa,
        id: FunctionId,
        arguments: &[ValueId],
    ) -> Vec<ValueId> {
        self.recursion_level += 1;

        let source_function = &ssa.functions[&id];

        if self.recursion_level > RECURSION_LIMIT {
            panic!(
                "Attempted to recur more than {RECURSION_LIMIT} times during inlining function '{}': {}", source_function.name(), source_function
            );
        }

        let mut context = PerFunctionContext::new(self, source_function);

        let parameters = source_function.parameters();
        assert_eq!(parameters.len(), arguments.len());
        context.values = parameters.iter().copied().zip(arguments.iter().copied()).collect();

        let current_block = context.context.builder.current_block();
        context.blocks.insert(source_function.entry_block(), current_block);

        let return_values = context.inline_blocks(ssa);
        self.recursion_level -= 1;
        return_values
    }
}

impl<'function> PerFunctionContext<'function> {
    /// Create a new PerFunctionContext from the source function.
    /// The value and block mappings for this context are initially empty except
    /// for containing the mapping between parameters in the source_function and
    /// the arguments of the destination function.
    fn new(context: &'function mut InlineContext, source_function: &'function Function) -> Self {
        Self {
            context,
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
                unreachable!("All Value::Instructions should already be known during inlining after creating the original inlined instruction. Unknown value {id} = {value:?}")
            }
            value @ Value::Param { .. } => {
                unreachable!("All Value::Params should already be known from previous calls to translate_block. Unknown value {id} = {value:?}")
            }
            Value::NumericConstant { constant, typ } => {
                self.context.builder.numeric_constant(*constant, typ.clone())
            }
            Value::Function(function) => self.context.builder.import_function(*function),
            Value::Intrinsic(intrinsic) => self.context.builder.import_intrinsic_id(*intrinsic),
            Value::ForeignFunction(function) => {
                self.context.builder.import_foreign_function(function)
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
        block_queue: &mut VecDeque<BasicBlockId>,
    ) -> BasicBlockId {
        if let Some(block) = self.blocks.get(&source_block) {
            return *block;
        }

        // The block is not yet inlined, queue it
        block_queue.push_back(source_block);

        // The block is not already present in the function being inlined into so we must create it.
        // The block's instructions are not copied over as they will be copied later in inlining.
        let new_block = self.context.builder.insert_block();
        let original_parameters = self.source_function.dfg.block_parameters(source_block);

        for parameter in original_parameters {
            let typ = self.source_function.dfg.type_of_value(*parameter);
            let new_parameter = self.context.builder.add_block_parameter(new_block, typ);
            self.values.insert(*parameter, new_parameter);
        }

        self.blocks.insert(source_block, new_block);
        new_block
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
    fn inline_blocks(&mut self, ssa: &Ssa) -> Vec<ValueId> {
        let mut seen_blocks = HashSet::new();
        let mut block_queue = VecDeque::new();
        block_queue.push_back(self.source_function.entry_block());

        // This Vec will contain each block with a Return instruction along with the
        // returned values of that block.
        let mut function_returns = vec![];

        while let Some(source_block_id) = block_queue.pop_front() {
            if seen_blocks.contains(&source_block_id) {
                continue;
            }
            let translated_block_id = self.translate_block(source_block_id, &mut block_queue);
            self.context.builder.switch_to_block(translated_block_id);

            seen_blocks.insert(source_block_id);
            self.inline_block_instructions(ssa, source_block_id);

            if let Some((block, values)) =
                self.handle_terminator_instruction(source_block_id, &mut block_queue)
            {
                function_returns.push((block, values));
            }
        }

        self.handle_function_returns(function_returns)
    }

    /// Handle inlining a function's possibly multiple return instructions.
    /// If there is only 1 return we can just continue inserting into that block.
    /// If there are multiple, we'll need to create a join block to jump to with each value.
    fn handle_function_returns(
        &mut self,
        mut returns: Vec<(BasicBlockId, Vec<ValueId>)>,
    ) -> Vec<ValueId> {
        // Clippy complains if this were written as an if statement
        match returns.len() {
            1 => {
                let (return_block, return_values) = returns.remove(0);
                self.context.builder.switch_to_block(return_block);
                return_values
            }
            n if n > 1 => {
                // If there is more than 1 return instruction we'll need to create a single block we
                // can return to and continue inserting in afterwards.
                let return_block = self.context.builder.insert_block();

                for (block, return_values) in returns {
                    self.context.builder.switch_to_block(block);
                    self.context.builder.terminate_with_jmp(return_block, return_values);
                }

                self.context.builder.switch_to_block(return_block);
                self.context.builder.block_parameters(return_block).to_vec()
            }
            _ => unreachable!("Inlined function had no return values"),
        }
    }

    /// Inline each instruction in the given block into the function being inlined into.
    /// This may recurse if it finds another function to inline if a call instruction is within this block.
    fn inline_block_instructions(&mut self, ssa: &Ssa, block_id: BasicBlockId) {
        let mut side_effects_enabled: Option<ValueId> = None;

        let block = &self.source_function.dfg[block_id];
        for id in block.instructions() {
            match &self.source_function.dfg[*id] {
                Instruction::Call { func, arguments } => match self.get_function(*func) {
                    Some(func_id) => {
                        if self.should_inline_call(ssa, func_id) {
                            self.inline_function(ssa, *id, func_id, arguments);

                            // This is only relevant during handling functions with `InlineType::NoPredicates` as these
                            // can pollute the function they're being inlined into with `Instruction::EnabledSideEffects`,
                            // resulting in predicates not being applied properly.
                            //
                            // Note that this doesn't cover the case in which there exists an `Instruction::EnabledSideEffects`
                            // within the function being inlined whilst the source function has not encountered one yet.
                            // In practice this isn't an issue as the last `Instruction::EnabledSideEffects` in the
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
    }

    fn should_inline_call(&self, ssa: &Ssa, called_func_id: FunctionId) -> bool {
        let function = &ssa.functions[&called_func_id];

        if let RuntimeType::Acir(inline_type) = function.runtime() {
            // If the called function is acir, we inline if it's not an entry point

            // If we have not already finished the flattening pass, functions marked
            // to not have predicates should be preserved.
            let preserve_function =
                !self.context.inline_no_predicates_functions && function.is_no_predicates();
            !inline_type.is_entry_point() && !preserve_function
        } else {
            // If the called function is brillig, we inline only if it's into brillig and the function is not recursive
            matches!(ssa.functions[&self.context.entry_point].runtime(), RuntimeType::Brillig(_))
                && !self.context.functions_not_to_inline.contains(&called_func_id)
        }
    }

    /// Inline a function call and remember the inlined return values in the values map
    fn inline_function(
        &mut self,
        ssa: &Ssa,
        call_id: InstructionId,
        function: FunctionId,
        arguments: &[ValueId],
    ) {
        let old_results = self.source_function.dfg.instruction_results(call_id);
        let arguments = vecmap(arguments, |arg| self.translate_value(*arg));

        let call_stack = self.source_function.dfg.get_call_stack(call_id);
        let call_stack_len = call_stack.len();
        self.context.call_stack.append(call_stack);

        let new_results = self.context.inline_function(ssa, function, &arguments);

        self.context.call_stack.truncate(self.context.call_stack.len() - call_stack_len);

        let new_results = InsertInstructionResult::Results(call_id, &new_results);
        Self::insert_new_instruction_results(&mut self.values, old_results, new_results);
    }

    /// Push the given instruction from the source_function into the current block of the
    /// function being inlined into.
    fn push_instruction(&mut self, id: InstructionId) {
        let instruction = self.source_function.dfg[id].map_values(|id| self.translate_value(id));

        let mut call_stack = self.context.call_stack.clone();
        call_stack.append(self.source_function.dfg.get_call_stack(id));

        let results = self.source_function.dfg.instruction_results(id);
        let results = vecmap(results, |id| self.source_function.dfg.resolve(*id));

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

        match new_results {
            InsertInstructionResult::SimplifiedTo(new_result) => {
                values.insert(old_results[0], new_result);
            }
            InsertInstructionResult::SimplifiedToMultiple(new_results) => {
                for (old_result, new_result) in old_results.iter().zip(new_results) {
                    values.insert(*old_result, new_result);
                }
            }
            InsertInstructionResult::Results(_, new_results) => {
                for (old_result, new_result) in old_results.iter().zip(new_results) {
                    values.insert(*old_result, *new_result);
                }
            }
            InsertInstructionResult::InstructionRemoved => (),
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
        block_queue: &mut VecDeque<BasicBlockId>,
    ) -> Option<(BasicBlockId, Vec<ValueId>)> {
        match self.source_function.dfg[block_id].unwrap_terminator() {
            TerminatorInstruction::Jmp { destination, arguments, call_stack } => {
                let destination = self.translate_block(*destination, block_queue);
                let arguments = vecmap(arguments, |arg| self.translate_value(*arg));

                let mut new_call_stack = self.context.call_stack.clone();
                new_call_stack.append(call_stack.clone());

                self.context
                    .builder
                    .set_call_stack(new_call_stack)
                    .terminate_with_jmp(destination, arguments);
                None
            }
            TerminatorInstruction::JmpIf {
                condition,
                then_destination,
                else_destination,
                call_stack,
            } => {
                let condition = self.translate_value(*condition);

                let mut new_call_stack = self.context.call_stack.clone();
                new_call_stack.append(call_stack.clone());

                // See if the value of the condition is known, and if so only inline the reachable
                // branch. This lets us inline some recursive functions without recurring forever.
                let dfg = &mut self.context.builder.current_function.dfg;
                match dfg.get_numeric_constant(condition) {
                    Some(constant) => {
                        let next_block =
                            if constant.is_zero() { *else_destination } else { *then_destination };

                        let next_block = self.translate_block(next_block, block_queue);
                        self.context
                            .builder
                            .set_call_stack(new_call_stack)
                            .terminate_with_jmp(next_block, vec![]);
                    }
                    None => {
                        let then_block = self.translate_block(*then_destination, block_queue);
                        let else_block = self.translate_block(*else_destination, block_queue);
                        self.context
                            .builder
                            .set_call_stack(new_call_stack)
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
                    let mut new_call_stack = self.context.call_stack.clone();
                    new_call_stack.append(call_stack.clone());
                    self.context
                        .builder
                        .set_call_stack(new_call_stack)
                        .terminate_with_return(return_values.clone());
                }

                Some((block_id, return_values))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use acvm::{acir::AcirField, FieldElement};
    use noirc_frontend::monomorphization::ast::InlineType;

    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            basic_block::BasicBlockId,
            function::RuntimeType,
            instruction::{BinaryOp, Intrinsic, TerminatorInstruction},
            map::Id,
            types::{NumericType, Type},
        },
    };

    #[test]
    fn basic_inlining() {
        // fn foo {
        //   b0():
        //     v0 = call bar()
        //     return v0
        // }
        // fn bar {
        //   b0():
        //     return 72
        // }
        let foo_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("foo".into(), foo_id);

        let bar_id = Id::test_new(1);
        let bar = builder.import_function(bar_id);
        let results = builder.insert_call(bar, Vec::new(), vec![Type::field()]).to_vec();
        builder.terminate_with_return(results);

        builder.new_function("bar".into(), bar_id, InlineType::default());
        let expected_return = 72u128;
        let seventy_two = builder.field_constant(expected_return);
        builder.terminate_with_return(vec![seventy_two]);

        let ssa = builder.finish();
        assert_eq!(ssa.functions.len(), 2);

        let inlined = ssa.inline_functions(i64::MAX);
        assert_eq!(inlined.functions.len(), 1);
    }

    #[test]
    fn complex_inlining() {
        // This SSA is from issue #1327 which previously failed to inline properly
        //
        // fn main f0 {
        //   b0(v0: Field):
        //     v7 = call f2(f1)
        //     v13 = call f3(v7)
        //     v16 = call v13(v0)
        //     return v16
        // }
        // fn square f1 {
        //   b0(v0: Field):
        //     v2 = mul v0, v0
        //     return v2
        // }
        // fn id1 f2 {
        //   b0(v0: function):
        //     return v0
        // }
        // fn id2 f3 {
        //   b0(v0: function):
        //     return v0
        // }
        let main_id = Id::test_new(0);
        let square_id = Id::test_new(1);
        let id1_id = Id::test_new(2);
        let id2_id = Id::test_new(3);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id);
        let main_v0 = builder.add_parameter(Type::field());

        let main_f1 = builder.import_function(square_id);
        let main_f2 = builder.import_function(id1_id);
        let main_f3 = builder.import_function(id2_id);

        let main_v7 = builder.insert_call(main_f2, vec![main_f1], vec![Type::Function])[0];
        let main_v13 = builder.insert_call(main_f3, vec![main_v7], vec![Type::Function])[0];
        let main_v16 = builder.insert_call(main_v13, vec![main_v0], vec![Type::field()])[0];
        builder.terminate_with_return(vec![main_v16]);

        // Compiling square f1
        builder.new_function("square".into(), square_id, InlineType::default());
        let square_v0 = builder.add_parameter(Type::field());
        let square_v2 = builder.insert_binary(square_v0, BinaryOp::Mul, square_v0);
        builder.terminate_with_return(vec![square_v2]);

        // Compiling id1 f2
        builder.new_function("id1".into(), id1_id, InlineType::default());
        let id1_v0 = builder.add_parameter(Type::Function);
        builder.terminate_with_return(vec![id1_v0]);

        // Compiling id2 f3
        builder.new_function("id2".into(), id2_id, InlineType::default());
        let id2_v0 = builder.add_parameter(Type::Function);
        builder.terminate_with_return(vec![id2_v0]);

        // Done, now we test that we can successfully inline all functions.
        let ssa = builder.finish();
        assert_eq!(ssa.functions.len(), 4);

        let inlined = ssa.inline_functions(i64::MAX);
        assert_eq!(inlined.functions.len(), 1);
    }

    #[test]
    fn recursive_functions() {
        // fn main f0 {
        //   b0():
        //     v0 = call factorial(Field 5)
        //     return v0
        // }
        // fn factorial f1 {
        //   b0(v0: Field):
        //     v1 = lt v0, Field 1
        //     jmpif v1, then: b1, else: b2
        //   b1():
        //     return Field 1
        //   b2():
        //     v2 = sub v0, Field 1
        //     v3 = call factorial(v2)
        //     v4 = mul v0, v3
        //     return v4
        // }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let factorial_id = Id::test_new(1);
        let factorial = builder.import_function(factorial_id);

        let five = builder.field_constant(5u128);
        let results = builder.insert_call(factorial, vec![five], vec![Type::field()]).to_vec();
        builder.terminate_with_return(results);

        builder.new_function("factorial".into(), factorial_id, InlineType::default());
        let b1 = builder.insert_block();
        let b2 = builder.insert_block();

        let one = builder.field_constant(1u128);

        let v0 = builder.add_parameter(Type::field());
        let v1 = builder.insert_binary(v0, BinaryOp::Lt, one);
        builder.terminate_with_jmpif(v1, b1, b2);

        builder.switch_to_block(b1);
        builder.terminate_with_return(vec![one]);

        builder.switch_to_block(b2);
        let factorial_id = builder.import_function(factorial_id);
        let v2 = builder.insert_binary(v0, BinaryOp::Sub, one);
        let v3 = builder.insert_call(factorial_id, vec![v2], vec![Type::field()])[0];
        let v4 = builder.insert_binary(v0, BinaryOp::Mul, v3);
        builder.terminate_with_return(vec![v4]);

        let ssa = builder.finish();
        assert_eq!(ssa.functions.len(), 2);

        // Expected SSA:
        //
        // fn main f2 {
        //   b0():
        //     jmp b1()
        //   b1():
        //     jmp b2()
        //   b2():
        //     jmp b3()
        //   b3():
        //     jmp b4()
        //   b4():
        //     jmp b5()
        //   b5():
        //     jmp b6()
        //   b6():
        //     return Field 120
        // }
        let inlined = ssa.inline_functions(i64::MAX);
        assert_eq!(inlined.functions.len(), 1);

        let main = inlined.main();
        let b6_id: BasicBlockId = Id::test_new(6);
        let b6 = &main.dfg[b6_id];

        match b6.terminator() {
            Some(TerminatorInstruction::Return { return_values, .. }) => {
                assert_eq!(return_values.len(), 1);
                let value = main
                    .dfg
                    .get_numeric_constant(return_values[0])
                    .expect("Expected a constant for the return value")
                    .to_u128();
                assert_eq!(value, 120);
            }
            other => unreachable!("Unexpected terminator {other:?}"),
        }
    }

    #[test]
    fn displaced_return_mapping() {
        // This test is designed specifically to catch a regression in which the ids of blocks
        // terminated by returns are badly tracked. As a result, the continuation of a source
        // block after a call instruction could but inlined into a block that's already been
        // terminated, producing an incorrect order and orphaning successors.

        // fn main f0 {
        //   b0(v0: u1):
        //     v2 = call f1(v0)
        //     call assert_constant(v2)
        //     return
        // }
        // fn inner1 f1 {
        //   b0(v0: u1):
        //     v2 = call f2(v0)
        //     return v2
        // }
        // fn inner2 f2 {
        //   b0(v0: u1):
        //     jmpif v0 then: b1, else: b2
        //   b1():
        //     jmp b3(Field 1)
        //   b3(v3: Field):
        //     return v3
        //   b2():
        //     jmp b3(Field 2)
        // }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let main_cond = builder.add_parameter(Type::bool());
        let inner1_id = Id::test_new(1);
        let inner1 = builder.import_function(inner1_id);
        let main_v2 = builder.insert_call(inner1, vec![main_cond], vec![Type::field()])[0];
        let assert_constant = builder.import_intrinsic_id(Intrinsic::AssertConstant);
        builder.insert_call(assert_constant, vec![main_v2], vec![]);
        builder.terminate_with_return(vec![]);

        builder.new_function("inner1".into(), inner1_id, InlineType::default());
        let inner1_cond = builder.add_parameter(Type::bool());
        let inner2_id = Id::test_new(2);
        let inner2 = builder.import_function(inner2_id);
        let inner1_v2 = builder.insert_call(inner2, vec![inner1_cond], vec![Type::field()])[0];
        builder.terminate_with_return(vec![inner1_v2]);

        builder.new_function("inner2".into(), inner2_id, InlineType::default());
        let inner2_cond = builder.add_parameter(Type::bool());
        let then_block = builder.insert_block();
        let else_block = builder.insert_block();
        let join_block = builder.insert_block();
        builder.terminate_with_jmpif(inner2_cond, then_block, else_block);
        builder.switch_to_block(then_block);
        let one = builder.numeric_constant(FieldElement::one(), Type::field());
        builder.terminate_with_jmp(join_block, vec![one]);
        builder.switch_to_block(else_block);
        let two = builder.numeric_constant(FieldElement::from(2_u128), Type::field());
        builder.terminate_with_jmp(join_block, vec![two]);
        let join_param = builder.add_block_parameter(join_block, Type::field());
        builder.switch_to_block(join_block);
        builder.terminate_with_return(vec![join_param]);

        let ssa = builder.finish().inline_functions(i64::MAX);
        // Expected result:
        // fn main f3 {
        //   b0(v0: u1):
        //     jmpif v0 then: b1, else: b2
        //   b1():
        //     jmp b3(Field 1)
        //   b3(v3: Field):
        //     call assert_constant(v3)
        //     return
        //   b2():
        //     jmp b3(Field 2)
        // }
        let main = ssa.main();
        assert_eq!(main.reachable_blocks().len(), 4);
    }

    #[test]
    #[should_panic(
        expected = "Attempted to recur more than 1000 times during inlining function 'main': acir(inline) fn main f0 {"
    )]
    fn unconditional_recursion() {
        // fn main f1 {
        //   b0():
        //     call f1()
        //     return
        // }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let main = builder.import_function(main_id);
        let results = builder.insert_call(main, Vec::new(), vec![]).to_vec();
        builder.terminate_with_return(results);

        let ssa = builder.finish();
        assert_eq!(ssa.functions.len(), 1);

        let inlined = ssa.inline_functions(i64::MAX);
        assert_eq!(inlined.functions.len(), 0);
    }

    #[test]
    fn inliner_disabled() {
        // brillig fn foo {
        //   b0():
        //     v0 = call bar()
        //     return v0
        // }
        // brillig fn bar {
        //   b0():
        //     return 72
        // }
        let foo_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("foo".into(), foo_id);
        builder.set_runtime(RuntimeType::Brillig(InlineType::default()));

        let bar_id = Id::test_new(1);
        let bar = builder.import_function(bar_id);
        let results = builder.insert_call(bar, Vec::new(), vec![Type::field()]).to_vec();
        builder.terminate_with_return(results);

        builder.new_brillig_function("bar".into(), bar_id, InlineType::default());
        let expected_return = 72u128;
        let seventy_two = builder.field_constant(expected_return);
        builder.terminate_with_return(vec![seventy_two]);

        let ssa = builder.finish();
        assert_eq!(ssa.functions.len(), 2);

        let inlined = ssa.inline_functions(i64::MIN);
        // No inlining has happened
        assert_eq!(inlined.functions.len(), 2);
    }

    #[test]
    fn conditional_inlining() {
        // In this example we call a larger brillig function 3 times so the inliner refuses to inline the function.
        // brillig fn foo {
        //   b0():
        //     v0 = call bar()
        //     v1 = call bar()
        //     v2 = call bar()
        //     return v0
        // }
        // brillig fn bar {
        //   b0():
        //     jmpif 1 then: b1, else: b2
        //   b1():
        //     jmp b3(Field 1)
        //   b3(v3: Field):
        //     return v3
        //   b2():
        //     jmp b3(Field 2)
        // }
        let foo_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("foo".into(), foo_id);
        builder.set_runtime(RuntimeType::Brillig(InlineType::default()));

        let bar_id = Id::test_new(1);
        let bar = builder.import_function(bar_id);
        let v0 = builder.insert_call(bar, Vec::new(), vec![Type::field()]).to_vec();
        let _v1 = builder.insert_call(bar, Vec::new(), vec![Type::field()]).to_vec();
        let _v2 = builder.insert_call(bar, Vec::new(), vec![Type::field()]).to_vec();
        builder.terminate_with_return(v0);

        builder.new_brillig_function("bar".into(), bar_id, InlineType::default());
        let bar_v0 =
            builder.numeric_constant(1_usize, Type::Numeric(NumericType::Unsigned { bit_size: 1 }));
        let then_block = builder.insert_block();
        let else_block = builder.insert_block();
        let join_block = builder.insert_block();
        builder.terminate_with_jmpif(bar_v0, then_block, else_block);
        builder.switch_to_block(then_block);
        let one = builder.numeric_constant(FieldElement::one(), Type::field());
        builder.terminate_with_jmp(join_block, vec![one]);
        builder.switch_to_block(else_block);
        let two = builder.numeric_constant(FieldElement::from(2_u128), Type::field());
        builder.terminate_with_jmp(join_block, vec![two]);
        let join_param = builder.add_block_parameter(join_block, Type::field());
        builder.switch_to_block(join_block);
        builder.terminate_with_return(vec![join_param]);

        let ssa = builder.finish();
        assert_eq!(ssa.functions.len(), 2);

        let inlined = ssa.inline_functions(0);
        // No inlining has happened
        assert_eq!(inlined.functions.len(), 2);
    }
}
