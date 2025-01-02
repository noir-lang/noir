//! The flatten cfg optimization pass "flattens" the entire control flow graph into a single block.
//! This includes branches in the CFG with non-constant conditions. Flattening these requires
//! special handling for operations with side-effects and can lead to a loss of information since
//! the jmpif will no longer be in the program. As a result, this pass should usually be towards or
//! at the end of the optimization passes. Note that this pass will also perform unexpectedly if
//! loops are still present in the program. Since the pass sees a normal jmpif, it will attempt to
//! merge both blocks, but no actual looping will occur.
//!
//! This pass is also known to produce some extra instructions which may go unused (usually 'Not')
//! while merging branches. These extra instructions can be cleaned up by a later dead instruction
//! elimination (DIE) pass.
//!
//! Though CFG information is lost during this pass, some key information is retained in the form
//! of `EnableSideEffectsIf` instructions. Each time the flattening pass enters and exits a branch of
//! a jmpif, an instruction is inserted to capture a condition that is analogous to the activeness
//! of the program point. For example:
//!
//! b0(v0: u1):
//!   jmpif v0, then: b1, else: b2
//! b1():
//!   v1 = call f0
//!   jmp b3(v1)
//! ... blocks b2 & b3 ...
//!
//! Would brace the call instruction as such:
//!   enable_side_effects v0
//!   v1 = call f0
//!   enable_side_effects u1 1
//!
//! (Note: we restore to "true" to indicate that this program point is not nested within any
//! other branches.)
//!
//! When we are flattening a block that was reached via a jmpif with a non-constant condition c,
//! the following transformations of certain instructions within the block are expected:
//!
//! 1. A constraint is multiplied by the condition and changes the constraint to
//!    an equality with c:
//!
//! constrain v0
//! ============
//! v1 = mul v0, c
//! v2 = eq v1, c
//! constrain v2
//!
//! 2. If we reach the end block of the branch created by the jmpif instruction, its block parameters
//!    will be merged. To merge the jmp arguments of the then and else branches, the formula
//!    `c * then_arg + !c * else_arg` is used for each argument.
//!
//! b0(v0: u1, v1: Field, v2: Field):
//!   jmpif v0, then: b1, else: b2
//! b1():
//!   jmp b3(v1)
//! b2():
//!   jmp b3(v2)
//! b3(v3: Field):
//!   ... b3 instructions ...
//! =========================
//! b0(v0: u1, v1: Field, v2: Field):
//!   v3 = mul v0, v1
//!   v4 = not v0
//!   v5 = mul v4, v2
//!   v6 = add v3, v5
//!   ... b3 instructions ...
//!
//! 3. After being stored to in at least one predecessor of a block with multiple predecessors, the
//!    value of a memory address is the value it had in both branches combined via c * a + !c * b.
//!    Note that the following example is simplified to remove extra load instructions and combine
//!    the separate merged stores for each branch into one store. See the next example for a
//!    non-simplified version with address offsets.
//!
//! b0(v0: u1):
//!   v1 = allocate 1 Field
//!   jmpif v0, then: b1, else: b2
//! b1():
//!   store v1, Field 5
//!   ... b1 instructions ...
//!   jmp b3
//! b2():
//!   store v1, Field 7
//!   ... b2 instructions ...
//!   jmp b3
//! b3():
//!   ... b3 instructions ...
//! =========================
//! b0():
//!   v1 = allocate 1 Field
//!   store v1, Field 5
//!   ... b1 instructions ...
//!   store v1, Field 7
//!   ... b2 instructions ...
//!   v2 = mul v0, Field 5
//!   v3 = not v0
//!   v4 = mul v3, Field 7
//!   v5 = add v2, v4
//!   store v1, v5
//!   ... b3 instructions ...
//!
//! Note that if the ValueId of the address stored to is not the same, two merging store
//! instructions will be made - one to each address. This is the case even if both addresses refer
//! to the same address internally. This can happen when they are equivalent offsets:
//!
//! b0(v0: u1, v1: ref)
//!   jmpif v0, then: b1, else: b2
//! b1():
//!   v2 = add v1, Field 1
//!   store Field 11 in v2
//!   ... b1 instructions ...
//! b2():
//!   v3 = add v1, Field 1
//!   store Field 12 in v3
//!   ... b2 instructions ...
//!
//! In this example, both store instructions store to an offset of 1 from v1, but because the
//! ValueIds differ (v2 and v3), two store instructions will be created:
//!
//! b0(v0: u1, v1: ref)
//!   v2 = add v1, Field 1
//!   v3 = load v2            (new load)
//!   store Field 11 in v2
//!   ... b1 instructions ...
//!   v4 = not v0             (new not)
//!   v5 = add v1, Field 1
//!   v6 = load v5            (new load)
//!   store Field 12 in v5
//!   ... b2 instructions ...
//!   v7 = mul v0, Field 11
//!   v8 = mul v4, v3
//!   v9 = add v7, v8
//!   store v9 at v2          (new store)
//!   v10 = mul v0, v6
//!   v11 = mul v4, Field 12
//!   v12 = add v10, v11
//!   store v12 at v5         (new store)
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

use acvm::{acir::AcirField, acir::BlackBoxFunc, FieldElement};
use iter_extended::vecmap;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        call_stack::CallStackId,
        cfg::ControlFlowGraph,
        dfg::InsertInstructionResult,
        function::{Function, FunctionId, RuntimeType},
        function_inserter::FunctionInserter,
        instruction::{BinaryOp, Instruction, InstructionId, Intrinsic, TerminatorInstruction},
        types::{NumericType, Type},
        value::{Value, ValueId},
    },
    ssa_gen::Ssa,
};

mod branch_analysis;
mod capacity_tracker;
pub(crate) mod value_merger;

impl Ssa {
    /// Flattens the control flow graph of main such that the function is left with a
    /// single block containing all instructions and no more control-flow.
    ///
    /// This pass will modify any instructions with side effects in particular, often multiplying
    /// them by jump conditions to maintain correctness even when all branches of a jmpif are inlined.
    /// For more information, see the module-level comment at the top of this file.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn flatten_cfg(mut self) -> Ssa {
        // Retrieve the 'no_predicates' attribute of the functions in a map, to avoid problems with borrowing
        let mut no_predicates = HashMap::default();
        for function in self.functions.values() {
            no_predicates.insert(function.id(), function.is_no_predicates());
        }

        for function in self.functions.values_mut() {
            flatten_function_cfg(function, &no_predicates);
        }
        self
    }
}

struct Context<'f> {
    inserter: FunctionInserter<'f>,

    /// This ControlFlowGraph is the graph from before the function was modified by this flattening pass.
    cfg: ControlFlowGraph,

    /// Maps start of branch -> end of branch
    branch_ends: HashMap<BasicBlockId, BasicBlockId>,

    /// A stack of each jmpif condition that was taken to reach a particular point in the program.
    /// When two branches are merged back into one, this constitutes a join point, and is analogous
    /// to the rest of the program after an if statement. When such a join point / end block is
    /// found, the top of this conditions stack is popped since we are no longer under that
    /// condition. If we are under multiple conditions (a nested if), the topmost condition is
    /// the most recent condition combined with all previous conditions via `And` instructions.
    condition_stack: Vec<ConditionalContext>,

    /// Maps SSA array values with a slice type to their size.
    /// This is maintained by appropriate calls to the `SliceCapacityTracker` and is used by the `ValueMerger`.
    slice_sizes: HashMap<ValueId, usize>,

    /// Stack of block arguments
    /// When processing a block, we pop this stack to get its arguments
    /// and at the end we push the arguments for his successor
    arguments_stack: Vec<Vec<ValueId>>,

    /// Stores all allocations local to the current branch.
    ///
    /// Since these branches are local to the current branch (i.e. only defined within one branch of
    /// an if expression), they should not be merged with their previous value or stored value in
    /// the other branch since there is no such value.
    ///
    /// The `ValueId` here is that which is returned by the allocate instruction.
    local_allocations: HashSet<ValueId>,

    /// A map from `cond` to `Not(cond)`
    ///
    /// `Not` instructions are inserted constantly by this pass and this map helps keep
    /// us from unnecessarily inserting extra instructions, and keeps ids unique which
    /// helps simplifications.
    not_instructions: HashMap<ValueId, ValueId>,
}

#[derive(Clone)]
struct ConditionalBranch {
    // Contains the last processed block during the processing of the branch.
    last_block: BasicBlockId,
    // The unresolved condition of the branch
    old_condition: ValueId,
    // The condition of the branch
    condition: ValueId,
    // The allocations accumulated when processing the branch
    local_allocations: HashSet<ValueId>,
}

struct ConditionalContext {
    // Condition from the conditional statement
    condition: ValueId,
    // Block containing the conditional statement
    entry_block: BasicBlockId,
    // First block of the then branch
    then_branch: ConditionalBranch,
    // First block of the else branch
    else_branch: Option<ConditionalBranch>,
    // Call stack where the final location is that of the entire `if` expression
    call_stack: CallStackId,
}

fn flatten_function_cfg(function: &mut Function, no_predicates: &HashMap<FunctionId, bool>) {
    // This pass may run forever on a brillig function.
    // Analyze will check if the predecessors have been processed and push the block to the back of
    // the queue. This loops forever if there are still any loops present in the program.
    if matches!(function.runtime(), RuntimeType::Brillig(_)) {
        return;
    }
    let cfg = ControlFlowGraph::with_function(function);
    let branch_ends = branch_analysis::find_branch_ends(function, &cfg);

    let mut context = Context {
        inserter: FunctionInserter::new(function),
        cfg,
        branch_ends,
        slice_sizes: HashMap::default(),
        condition_stack: Vec::new(),
        arguments_stack: Vec::new(),
        local_allocations: HashSet::default(),
        not_instructions: HashMap::default(),
    };
    context.flatten(no_predicates);
}

impl<'f> Context<'f> {
    fn flatten(&mut self, no_predicates: &HashMap<FunctionId, bool>) {
        // Flatten the CFG by inlining all instructions from the queued blocks
        // until all blocks have been flattened.
        // We follow the terminator of each block to determine which blocks to
        // process next
        let mut queue = vec![self.inserter.function.entry_block()];
        while let Some(block) = queue.pop() {
            self.inline_block(block, no_predicates);
            let to_process = self.handle_terminator(block, &queue);
            for incoming_block in to_process {
                if !queue.contains(&incoming_block) {
                    queue.push(incoming_block);
                }
            }
        }
        self.inserter.map_data_bus_in_place();
    }

    /// Returns the updated condition so that
    /// it is 'AND-ed' with the previous condition (if any)
    fn link_condition(&mut self, condition: ValueId) -> ValueId {
        // Retrieve the previous condition
        if let Some(context) = self.condition_stack.last() {
            let previous_branch = context.else_branch.as_ref().unwrap_or(&context.then_branch);
            let and = Instruction::binary(BinaryOp::And, previous_branch.condition, condition);
            let call_stack = self.inserter.function.dfg.get_value_call_stack_id(condition);
            self.insert_instruction(and, call_stack)
        } else {
            condition
        }
    }

    /// Returns the current condition
    fn get_last_condition(&self) -> Option<ValueId> {
        self.condition_stack.last().map(|context| match &context.else_branch {
            Some(else_branch) => else_branch.condition,
            None => context.then_branch.condition,
        })
    }

    /// Use the provided map to say if the instruction is a call to a no_predicates function
    fn is_no_predicate(
        &self,
        no_predicates: &HashMap<FunctionId, bool>,
        instruction: &InstructionId,
    ) -> bool {
        let mut result = false;
        if let Instruction::Call { func, .. } = self.inserter.function.dfg[*instruction] {
            if let Value::Function(fid) = self.inserter.function.dfg[func] {
                result = *no_predicates.get(&fid).unwrap_or(&false);
            }
        }
        result
    }

    // Inline all instructions from the given block into the entry block, and track slice capacities
    fn inline_block(&mut self, block: BasicBlockId, no_predicates: &HashMap<FunctionId, bool>) {
        if self.inserter.function.entry_block() == block {
            // we do not inline the entry block into itself
            // for the outer block before we start inlining
            return;
        }

        let arguments = self.arguments_stack.pop().unwrap();
        self.inserter.remember_block_params(block, &arguments);

        // If this is not a separate variable, clippy gets confused and says the to_vec is
        // unnecessary, when removing it actually causes an aliasing/mutability error.
        let instructions = self.inserter.function.dfg[block].instructions().to_vec();
        for instruction in instructions.iter() {
            if self.is_no_predicate(no_predicates, instruction) {
                // disable side effect for no_predicate functions
                let bool_type = NumericType::bool();
                let one = self.inserter.function.dfg.make_constant(FieldElement::one(), bool_type);
                self.insert_instruction_with_typevars(
                    Instruction::EnableSideEffectsIf { condition: one },
                    None,
                    CallStackId::root(),
                );
                self.push_instruction(*instruction);
                self.insert_current_side_effects_enabled();
            } else {
                self.push_instruction(*instruction);
            }
        }
    }

    /// Returns the list of blocks that need to be processed after the given block
    /// For a normal block, it would be its successor
    /// For blocks related to a conditional statement, we ensure to process
    /// the 'then-branch', then the 'else-branch' (if it exists), and finally the end block
    fn handle_terminator(
        &mut self,
        block: BasicBlockId,
        work_list: &[BasicBlockId],
    ) -> Vec<BasicBlockId> {
        let terminator = self.inserter.function.dfg[block].unwrap_terminator().clone();
        match &terminator {
            TerminatorInstruction::JmpIf {
                condition,
                then_destination,
                else_destination,
                call_stack,
            } => {
                self.arguments_stack.push(vec![]);
                self.if_start(condition, then_destination, else_destination, &block, *call_stack)
            }
            TerminatorInstruction::Jmp { destination, arguments, call_stack: _ } => {
                let arguments = vecmap(arguments.clone(), |value| self.inserter.resolve(value));
                self.arguments_stack.push(arguments);
                if work_list.contains(destination) {
                    if work_list.last() == Some(destination) {
                        self.else_stop(&block)
                    } else {
                        self.then_stop(&block)
                    }
                } else {
                    vec![*destination]
                }
            }
            TerminatorInstruction::Return { return_values, call_stack } => {
                let call_stack = *call_stack;
                let return_values =
                    vecmap(return_values.clone(), |value| self.inserter.resolve(value));
                let new_return = TerminatorInstruction::Return { return_values, call_stack };
                let entry = self.inserter.function.entry_block();

                self.inserter.function.dfg.set_block_terminator(entry, new_return);
                vec![]
            }
        }
    }

    /// Process a conditional statement
    fn if_start(
        &mut self,
        condition: &ValueId,
        then_destination: &BasicBlockId,
        else_destination: &BasicBlockId,
        if_entry: &BasicBlockId,
        call_stack: CallStackId,
    ) -> Vec<BasicBlockId> {
        // manage conditions
        let old_condition = *condition;
        let then_condition = self.inserter.resolve(old_condition);

        let old_allocations = std::mem::take(&mut self.local_allocations);
        let branch = ConditionalBranch {
            old_condition,
            condition: self.link_condition(then_condition),
            last_block: *then_destination,
            local_allocations: old_allocations,
        };
        let cond_context = ConditionalContext {
            condition: then_condition,
            entry_block: *if_entry,
            then_branch: branch,
            else_branch: None,
            call_stack,
        };
        self.condition_stack.push(cond_context);
        self.insert_current_side_effects_enabled();

        // We disallow this case as it results in the `else_destination` block
        // being inlined before the `then_destination` block due to block deduplication in the work queue.
        //
        // The `else_destination` block then gets treated as if it were the `then_destination` block
        // and has the incorrect condition applied to it.
        assert_ne!(
            self.branch_ends[if_entry], *then_destination,
            "ICE: branches merge inside of `then` branch"
        );
        vec![self.branch_ends[if_entry], *else_destination, *then_destination]
    }

    /// Switch context to the 'else-branch'
    fn then_stop(&mut self, block: &BasicBlockId) -> Vec<BasicBlockId> {
        let mut cond_context = self.condition_stack.pop().unwrap();
        cond_context.then_branch.last_block = *block;

        let condition_call_stack =
            self.inserter.function.dfg.get_value_call_stack_id(cond_context.condition);

        let else_condition = self.not_instruction(cond_context.condition, condition_call_stack);
        let else_condition = self.link_condition(else_condition);

        let old_allocations = std::mem::take(&mut self.local_allocations);
        let else_branch = ConditionalBranch {
            old_condition: cond_context.then_branch.old_condition,
            condition: else_condition,
            last_block: *block,
            local_allocations: old_allocations,
        };
        cond_context.then_branch.local_allocations.clear();
        cond_context.else_branch = Some(else_branch);
        self.condition_stack.push(cond_context);

        self.insert_current_side_effects_enabled();

        assert_eq!(self.cfg.successors(*block).len(), 1);
        vec![self.cfg.successors(*block).next().unwrap()]
    }

    fn not_instruction(&mut self, condition: ValueId, call_stack: CallStackId) -> ValueId {
        if let Some(existing) = self.not_instructions.get(&condition) {
            return *existing;
        }

        let not = self.insert_instruction(Instruction::Not(condition), call_stack);
        self.not_instructions.insert(condition, not);
        not
    }

    /// Process the 'exit' block of a conditional statement
    fn else_stop(&mut self, block: &BasicBlockId) -> Vec<BasicBlockId> {
        let mut cond_context = self.condition_stack.pop().unwrap();
        if cond_context.else_branch.is_none() {
            // then_stop() has not been called, this means that the conditional statement has no else branch
            // so we simply do the then_stop() now
            self.condition_stack.push(cond_context);
            self.then_stop(block);
            cond_context = self.condition_stack.pop().unwrap();
        }

        let mut else_branch = cond_context.else_branch.unwrap();
        self.local_allocations = std::mem::take(&mut else_branch.local_allocations);
        else_branch.last_block = *block;
        cond_context.else_branch = Some(else_branch);

        // We must remember to reset whether side effects are enabled when both branches
        // end, in addition to resetting the value of old_condition since it is set to
        // known to be true/false within the then/else branch respectively.
        self.insert_current_side_effects_enabled();

        // While there is a condition on the stack we don't compile outside the condition
        // until it is popped. This ensures we inline the full then and else branches
        // before continuing from the end of the conditional here where they can be merged properly.
        let end = self.branch_ends[&cond_context.entry_block];

        // Merge arguments and stores from the else/end branches
        self.inline_branch_end(end, cond_context);

        vec![self.cfg.successors(*block).next().unwrap()]
    }

    /// Inline the ending block of a branch, the point where all blocks from a jmpif instruction
    /// join back together. In particular this function must handle merging block arguments from
    /// all of the join point's predecessors, and it must handle any differing side effects from
    /// each branch.
    ///
    /// Afterwards, continues inlining recursively until it finds the next end block or finds the
    /// end of the function.
    ///
    /// Returns the final block that was inlined.
    fn inline_branch_end(
        &mut self,
        destination: BasicBlockId,
        cond_context: ConditionalContext,
    ) -> BasicBlockId {
        assert_eq!(self.cfg.predecessors(destination).len(), 2);
        let last_then = cond_context.then_branch.last_block;
        let mut else_args = Vec::new();
        if cond_context.else_branch.is_some() {
            let last_else = cond_context.else_branch.clone().unwrap().last_block;
            else_args = self.inserter.function.dfg[last_else].terminator_arguments().to_vec();
        }

        let then_args = self.inserter.function.dfg[last_then].terminator_arguments().to_vec();

        let params = self.inserter.function.dfg.block_parameters(destination);
        assert_eq!(params.len(), then_args.len());
        assert_eq!(params.len(), else_args.len());

        let args = vecmap(then_args.iter().zip(else_args), |(then_arg, else_arg)| {
            (self.inserter.resolve(*then_arg), self.inserter.resolve(else_arg))
        });
        let else_condition = if let Some(branch) = cond_context.else_branch {
            branch.condition
        } else {
            self.inserter.function.dfg.make_constant(FieldElement::zero(), NumericType::bool())
        };
        let block = self.inserter.function.entry_block();

        // Cannot include this in the previous vecmap since it requires exclusive access to self
        let args = vecmap(args, |(then_arg, else_arg)| {
            let instruction = Instruction::IfElse {
                then_condition: cond_context.then_branch.condition,
                then_value: then_arg,
                else_condition,
                else_value: else_arg,
            };
            let call_stack = cond_context.call_stack;
            self.inserter
                .function
                .dfg
                .insert_instruction_and_results(instruction, block, None, call_stack)
                .first()
        });

        self.arguments_stack.pop();
        self.arguments_stack.pop();
        self.arguments_stack.push(args);
        destination
    }

    /// Insert a new instruction into the function's entry block.
    /// Unlike push_instruction, this function will not map any ValueIds.
    /// within the given instruction, nor will it modify self.values in any way.
    fn insert_instruction(&mut self, instruction: Instruction, call_stack: CallStackId) -> ValueId {
        let block = self.inserter.function.entry_block();
        self.inserter
            .function
            .dfg
            .insert_instruction_and_results(instruction, block, None, call_stack)
            .first()
    }

    /// Inserts a new instruction into the function's entry block, using the given
    /// control type variables to specify result types if needed.
    /// Unlike push_instruction, this function will not map any ValueIds.
    /// within the given instruction, nor will it modify self.values in any way.
    fn insert_instruction_with_typevars(
        &mut self,
        instruction: Instruction,
        ctrl_typevars: Option<Vec<Type>>,
        call_stack: CallStackId,
    ) -> InsertInstructionResult {
        let block = self.inserter.function.entry_block();
        self.inserter.function.dfg.insert_instruction_and_results(
            instruction,
            block,
            ctrl_typevars,
            call_stack,
        )
    }

    /// Checks the branch condition on the top of the stack and uses it to build and insert an
    /// `EnableSideEffectsIf` instruction into the entry block.
    ///
    /// If the stack is empty, a "true" u1 constant is taken to be the active condition. This is
    /// necessary for re-enabling side-effects when re-emerging to a branch depth of 0.
    fn insert_current_side_effects_enabled(&mut self) {
        let condition = match self.get_last_condition() {
            Some(cond) => cond,
            None => {
                self.inserter.function.dfg.make_constant(FieldElement::one(), NumericType::bool())
            }
        };
        let enable_side_effects = Instruction::EnableSideEffectsIf { condition };
        let call_stack = self.inserter.function.dfg.get_value_call_stack_id(condition);
        self.insert_instruction_with_typevars(enable_side_effects, None, call_stack);
    }

    /// Push the given instruction to the end of the entry block of the current function.
    ///
    /// Note that each ValueId of the instruction will be mapped via self.inserter.resolve.
    /// As a result, the instruction that will be pushed will actually be a new instruction
    /// with a different InstructionId from the original. The results of the given instruction
    /// will also be mapped to the results of the new instruction.
    ///
    /// `previous_allocate_result` should only be set to the result of an allocate instruction
    /// if that instruction was the instruction immediately previous to this one - if there are
    /// any instructions in between it should be None.
    fn push_instruction(&mut self, id: InstructionId) {
        let (instruction, call_stack) = self.inserter.map_instruction(id);
        let instruction = self.handle_instruction_side_effects(instruction, call_stack);

        let instruction_is_allocate = matches!(&instruction, Instruction::Allocate);
        let entry = self.inserter.function.entry_block();
        let results = self.inserter.push_instruction_value(instruction, id, entry, call_stack);

        // Remember an allocate was created local to this branch so that we do not try to merge store
        // values across branches for it later.
        if instruction_is_allocate {
            self.local_allocations.insert(results.first());
        }
    }

    /// If we are currently in a branch, we need to modify constrain instructions
    /// to multiply them by the branch's condition (see optimization #1 in the module comment).
    fn handle_instruction_side_effects(
        &mut self,
        instruction: Instruction,
        call_stack: CallStackId,
    ) -> Instruction {
        if let Some(condition) = self.get_last_condition() {
            match instruction {
                Instruction::Constrain(lhs, rhs, message) => {
                    // Replace constraint `lhs == rhs` with `condition * lhs == condition * rhs`.

                    // Condition needs to be cast to argument type in order to multiply them together.
                    let argument_type = self.inserter.function.dfg.type_of_value(lhs);

                    let cast = Instruction::Cast(condition, argument_type.unwrap_numeric());
                    let casted_condition = self.insert_instruction(cast, call_stack);

                    let lhs = self.insert_instruction(
                        Instruction::binary(BinaryOp::Mul, lhs, casted_condition),
                        call_stack,
                    );
                    let rhs = self.insert_instruction(
                        Instruction::binary(BinaryOp::Mul, rhs, casted_condition),
                        call_stack,
                    );

                    Instruction::Constrain(lhs, rhs, message)
                }
                Instruction::Store { address, value } => {
                    // If this instruction immediately follows an allocate, and stores to that
                    // address there is no previous value to load and we don't need a merge anyway.
                    if self.local_allocations.contains(&address) {
                        Instruction::Store { address, value }
                    } else {
                        // Instead of storing `value`, store `if condition { value } else { previous_value }`
                        let typ = self.inserter.function.dfg.type_of_value(value);
                        let load = Instruction::Load { address };
                        let previous_value = self
                            .insert_instruction_with_typevars(load, Some(vec![typ]), call_stack)
                            .first();

                        let else_condition = self.not_instruction(condition, call_stack);

                        let instruction = Instruction::IfElse {
                            then_condition: condition,
                            then_value: value,
                            else_condition,
                            else_value: previous_value,
                        };

                        let updated_value = self.insert_instruction(instruction, call_stack);
                        Instruction::Store { address, value: updated_value }
                    }
                }
                Instruction::RangeCheck { value, max_bit_size, assert_message } => {
                    // Replace value with `value * predicate` to zero out value when predicate is inactive.

                    // Condition needs to be cast to argument type in order to multiply them together.
                    let argument_type = self.inserter.function.dfg.type_of_value(value);
                    let cast = Instruction::Cast(condition, argument_type.unwrap_numeric());
                    let casted_condition = self.insert_instruction(cast, call_stack);

                    let value = self.insert_instruction(
                        Instruction::binary(BinaryOp::Mul, value, casted_condition),
                        call_stack,
                    );
                    Instruction::RangeCheck { value, max_bit_size, assert_message }
                }
                Instruction::Call { func, mut arguments } => match self.inserter.function.dfg[func]
                {
                    Value::Intrinsic(Intrinsic::ToBits(_) | Intrinsic::ToRadix(_)) => {
                        let field = arguments[0];
                        let argument_type = self.inserter.function.dfg.type_of_value(field);

                        let cast = Instruction::Cast(condition, argument_type.unwrap_numeric());
                        let casted_condition = self.insert_instruction(cast, call_stack);
                        let field = self.insert_instruction(
                            Instruction::binary(BinaryOp::Mul, field, casted_condition),
                            call_stack,
                        );

                        arguments[0] = field;

                        Instruction::Call { func, arguments }
                    }
                    //Issue #5045: We set curve points to infinity if condition is false
                    Value::Intrinsic(Intrinsic::BlackBox(BlackBoxFunc::EmbeddedCurveAdd)) => {
                        arguments[2] = self.var_or_one(arguments[2], condition, call_stack);
                        arguments[5] = self.var_or_one(arguments[5], condition, call_stack);

                        Instruction::Call { func, arguments }
                    }
                    Value::Intrinsic(Intrinsic::BlackBox(BlackBoxFunc::MultiScalarMul)) => {
                        let points_array_idx = if matches!(
                            self.inserter.function.dfg.type_of_value(arguments[0]),
                            Type::Array { .. }
                        ) {
                            0
                        } else {
                            // if the first argument is not an array, we assume it is a slice
                            // which means the array is the second argument
                            1
                        };
                        let (elements, typ) = self.apply_predicate_to_msm_argument(
                            arguments[points_array_idx],
                            condition,
                            call_stack,
                        );

                        let instruction = Instruction::MakeArray { elements, typ };
                        let array = self.insert_instruction(instruction, call_stack);
                        arguments[points_array_idx] = array;
                        Instruction::Call { func, arguments }
                    }
                    _ => Instruction::Call { func, arguments },
                },
                other => other,
            }
        } else {
            instruction
        }
    }

    /// When a MSM is done under a predicate, we need to apply the predicate
    /// to the is_infinity property of the input points in order to ensure
    /// that the points will be on the curve no matter what.
    fn apply_predicate_to_msm_argument(
        &mut self,
        argument: ValueId,
        predicate: ValueId,
        call_stack: CallStackId,
    ) -> (im::Vector<ValueId>, Type) {
        let array_typ;
        let mut array_with_predicate = im::Vector::new();
        if let Some((array, typ)) = &self.inserter.function.dfg.get_array_constant(argument) {
            array_typ = typ.clone();
            for (i, value) in array.clone().iter().enumerate() {
                if i % 3 == 2 {
                    array_with_predicate.push_back(self.var_or_one(*value, predicate, call_stack));
                } else {
                    array_with_predicate.push_back(*value);
                }
            }
        } else {
            unreachable!(
                "Expected an array, got {}",
                &self.inserter.function.dfg.type_of_value(argument)
            );
        };

        (array_with_predicate, array_typ)
    }

    // Computes: if condition { var } else { 1 }
    fn var_or_one(&mut self, var: ValueId, condition: ValueId, call_stack: CallStackId) -> ValueId {
        let field =
            self.insert_instruction(Instruction::binary(BinaryOp::Mul, var, condition), call_stack);
        let not_condition = self.not_instruction(condition, call_stack);
        self.insert_instruction(
            Instruction::binary(BinaryOp::Add, field, not_condition),
            call_stack,
        )
    }
}

#[cfg(test)]
mod test {
    use acvm::acir::AcirField;

    use crate::ssa::{
        ir::{
            dfg::DataFlowGraph,
            function::Function,
            instruction::{Instruction, TerminatorInstruction},
            value::{Value, ValueId},
        },
        opt::assert_normalized_ssa_equals,
        Ssa,
    };

    #[test]
    fn basic_jmpif() {
        let src = "
            acir(inline) fn main f0 {
              b0(v0: u1):
                jmpif v0 then: b1, else: b2
              b1():
                jmp b3(Field 3)
              b3(v1: Field):
                return v1
              b2():
                jmp b3(Field 4)
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        assert_eq!(ssa.main().reachable_blocks().len(), 4);

        let expected = "
            acir(inline) fn main f0 {
              b0(v0: u1):
                enable_side_effects v0
                v1 = not v0
                enable_side_effects u1 1
                v3 = cast v0 as Field
                v4 = cast v1 as Field
                v6 = mul v3, Field 3
                v8 = mul v4, Field 4
                v9 = add v6, v8
                return v9
            }
            ";

        let ssa = ssa.flatten_cfg();
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn modify_constrain() {
        let src = "
            acir(inline) fn main f0 {
              b0(v0: u1, v1: u1):
                jmpif v0 then: b1, else: b2
              b1():
                constrain v1 == u1 1
                jmp b2()
              b2():
                return
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        assert_eq!(ssa.main().reachable_blocks().len(), 3);

        let expected = "
            acir(inline) fn main f0 {
              b0(v0: u1, v1: u1):
                enable_side_effects v0
                v2 = mul v1, v0
                constrain v2 == v0
                v3 = not v0
                enable_side_effects u1 1
                return
            }
            ";
        let ssa = ssa.flatten_cfg();
        assert_eq!(ssa.main().reachable_blocks().len(), 1);
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn merge_stores() {
        let src = "
            acir(inline) fn main f0 {
              b0(v0: u1, v1: &mut Field):
                jmpif v0 then: b1, else: b2
              b1():
                store Field 5 at v1
                jmp b2()
              b2():
                return
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let expected = "
            acir(inline) fn main f0 {
              b0(v0: u1, v1: &mut Field):
                enable_side_effects v0
                v2 = load v1 -> Field
                v3 = not v0
                v4 = cast v0 as Field
                v5 = cast v3 as Field
                v7 = mul v4, Field 5
                v8 = mul v5, v2
                v9 = add v7, v8
                store v9 at v1
                enable_side_effects u1 1
                return
            }
            ";
        let ssa = ssa.flatten_cfg();
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn merge_stores_with_else_block() {
        let src = "
            acir(inline) fn main f0 {
              b0(v0: u1, v1: &mut Field):
                jmpif v0 then: b1, else: b2
              b1():
                store Field 5 at v1
                jmp b3()
              b2():
                store Field 6 at v1
                jmp b3()
              b3():
                return
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let expected = "
            acir(inline) fn main f0 {
              b0(v0: u1, v1: &mut Field):
                enable_side_effects v0
                v2 = load v1 -> Field
                v3 = not v0
                v4 = cast v0 as Field
                v5 = cast v3 as Field
                v7 = mul v4, Field 5
                v8 = mul v5, v2
                v9 = add v7, v8
                store v9 at v1
                enable_side_effects v3
                v10 = load v1 -> Field
                v11 = cast v3 as Field
                v12 = cast v0 as Field
                v14 = mul v11, Field 6
                v15 = mul v12, v10
                v16 = add v14, v15
                store v16 at v1
                enable_side_effects u1 1
                return
            }
            ";
        let ssa = ssa.flatten_cfg();
        assert_normalized_ssa_equals(ssa, expected);
    }

    fn count_instruction(function: &Function, f: impl Fn(&Instruction) -> bool) -> usize {
        function.dfg[function.entry_block()]
            .instructions()
            .iter()
            .filter(|id| f(&function.dfg[**id]))
            .count()
    }

    #[test]
    fn nested_branch_stores() {
        // Here we build some SSA with control flow given by the following graph.
        // To test stores in nested if statements are handled correctly this graph is
        // also nested. To keep things simple, each block stores to the same address
        // an integer that matches its block number. So block 2 stores the value 2,
        // block 3 stores 3 and so on. Note that only blocks { 0, 1, 2, 3, 5, 6 }
        // will store values. Other blocks do not store values so that we can test
        // how these existing values are merged at each join point.
        //
        // For debugging purposes, each block also has a call to test_function with two
        // arguments. The first is the block the test_function was originally in, and the
        // second is the current value stored in the reference.
        //
        //         b0   (0 stored)
        //         ↓
        //         b1   (1 stored)
        //       ↙   ↘
        //     b2     b3  (2 stored in b2) (3 stored in b3)
        //     ↓      |
        //     b4     |
        //   ↙  ↘     |
        // b5    b6   |   (5 stored in b5) (6 stored in b6)
        //   ↘  ↙     ↓
        //    b7      b8
        //      ↘   ↙
        //       b9

        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u1):
            v2 = allocate -> &mut Field
            store Field 0 at v2
            v4 = load v2 -> Field
            // call v1(Field 0, v4)
            jmp b1()
          b1():
            store Field 1 at v2
            v6 = load v2 -> Field
            // call v1(Field 1, v6)
            jmpif v0 then: b2, else: b3
          b2():
            store Field 2 at v2
            v8 = load v2 -> Field
            // call v1(Field 2, v8)
            jmp b4()
          b4():
            v12 = load v2 -> Field
            // call v1(Field 4, v12)
            jmpif v1 then: b5, else: b6
          b5():
            store Field 5 at v2
            v14 = load v2 -> Field
            // call v1(Field 5, v14)
            jmp b7()
          b7():
            v18 = load v2 -> Field
            // call v1(Field 7, v18)
            jmp b9()
          b9():
            v22 = load v2 -> Field
            // call v1(Field 9, v22)
            v23 = load v2 -> Field
            return v23
          b6():
            store Field 6 at v2
            v16 = load v2 -> Field
            // call v1(Field 6, v16)
            jmp b7()
          b3():
            store Field 3 at v2
            v10 = load v2 -> Field
            // call v1(Field 3, v10)
            jmp b8()
          b8():
            v20 = load v2 -> Field
            // call v1(Field 8, v20)
            jmp b9()
        }";

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.flatten_cfg().mem2reg();

        let expected = "
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u1):
            v2 = allocate -> &mut Field
            enable_side_effects v0
            v3 = not v0
            v4 = cast v0 as Field
            v5 = cast v3 as Field
            v7 = mul v4, Field 2
            v8 = add v7, v5
            v9 = mul v0, v1
            enable_side_effects v9
            v10 = not v9
            v11 = cast v9 as Field
            v12 = cast v10 as Field
            v14 = mul v11, Field 5
            v15 = mul v12, v8
            v16 = add v14, v15
            v17 = not v1
            v18 = mul v0, v17
            enable_side_effects v18
            v19 = not v18
            v20 = cast v18 as Field
            v21 = cast v19 as Field
            v23 = mul v20, Field 6
            v24 = mul v21, v16
            v25 = add v23, v24
            enable_side_effects v3
            v26 = cast v3 as Field
            v27 = cast v0 as Field
            v29 = mul v26, Field 3
            v30 = mul v27, v25
            v31 = add v29, v30
            enable_side_effects u1 1
            return v31
        }";

        let main = ssa.main();
        let ret = match main.dfg[main.entry_block()].terminator() {
            Some(TerminatorInstruction::Return { return_values, .. }) => return_values[0],
            _ => unreachable!("Should have terminator instruction"),
        };

        let merged_values = get_all_constants_reachable_from_instruction(&main.dfg, ret);
        assert_eq!(merged_values, vec![2, 3, 5, 6]);

        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn allocate_in_single_branch() {
        // Regression test for #1756
        // fn foo() -> Field {
        //     let mut x = 0;
        //     x
        // }
        //
        // fn main(cond:bool) {
        //     if cond {
        //         foo();
        //     };
        // }
        //
        // Translates to the following before the flattening pass:
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            jmpif v0 then: b1, else: b2
          b1():
            v1 = allocate -> &mut Field
            store Field 0 at v1
            v3 = load v1 -> Field
            jmp b2()
          b2():
            return
        }";
        // The bug is that the flattening pass previously inserted a load
        // before the first store to allocate, which loaded an uninitialized value.
        // In this test we assert the ordering is strictly Allocate then Store then Load.
        let ssa = Ssa::from_str(src).unwrap();
        let flattened_ssa = ssa.flatten_cfg();

        // Now assert that there is not a load between the allocate and its first store
        // The Expected IR is:
        let expected = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            enable_side_effects v0
            v1 = allocate -> &mut Field
            store Field 0 at v1
            v3 = load v1 -> Field
            v4 = not v0
            enable_side_effects u1 1
            return
        }
        ";

        let main = flattened_ssa.main();
        let instructions = main.dfg[main.entry_block()].instructions();

        let find_instruction = |predicate: fn(&Instruction) -> bool| {
            instructions.iter().position(|id| predicate(&main.dfg[*id])).unwrap()
        };

        let allocate_index = find_instruction(|i| matches!(i, Instruction::Allocate));
        let store_index = find_instruction(|i| matches!(i, Instruction::Store { .. }));
        let load_index = find_instruction(|i| matches!(i, Instruction::Load { .. }));

        assert!(allocate_index < store_index);
        assert!(store_index < load_index);

        assert_normalized_ssa_equals(flattened_ssa, expected);
    }

    /// Work backwards from an instruction to find all the constant values
    /// that were used to construct it. E.g for:
    ///
    /// b0(v0: Field):
    ///   v1 = add v0, Field 6
    ///   v2 = mul v1, Field 2
    ///   v3 = sub v2, v0
    ///   return v3
    ///
    /// Calling this function on v3 will return [2, 6].
    fn get_all_constants_reachable_from_instruction(
        dfg: &DataFlowGraph,
        value: ValueId,
    ) -> Vec<u128> {
        match dfg[value] {
            Value::Instruction { instruction, .. } => {
                let mut constants = vec![];

                dfg[instruction].for_each_value(|value| {
                    constants.extend(get_all_constants_reachable_from_instruction(dfg, value));
                });

                constants.sort();
                constants.dedup();
                constants
            }
            Value::NumericConstant { constant, .. } => vec![constant.to_u128()],
            _ => Vec::new(),
        }
    }

    #[test]
    fn should_not_merge_away_constraints() {
        // Very simplified derived regression test for #1792
        // Tests that it does not simplify to a true constraint an always-false constraint
        // The original function is replaced by the following:
        let src = "
            acir(inline) fn main f1 {
              b0():
                jmpif u1 0 then: b1, else: b2
              b1():
                jmp b2()
              b2():
                constrain u1 0 == u1 1 // was incorrectly removed
                return
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let expected = "
            acir(inline) fn main f0 {
              b0():
                enable_side_effects u1 1
                constrain u1 0 == u1 1
                return
            }
            ";
        let ssa = ssa.flatten_cfg();
        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn should_not_merge_incorrectly_to_false() {
        // Regression test for #1792
        // Tests that it does not simplify a true constraint an always-false constraint

        let src = "
        acir(inline) fn main f0 {
          b0(v0: [u8; 2]):
            v2 = array_get v0, index u8 0 -> u8
            v3 = cast v2 as u32
            v4 = truncate v3 to 1 bits, max_bit_size: 32
            v5 = cast v4 as u1
            v6 = allocate -> &mut Field
            store u8 0 at v6
            jmpif v5 then: b2, else: b1
          b2():
            v7 = cast v2 as Field
            v9 = add v7, Field 1
            v10 = cast v9 as u8
            store v10 at v6
            jmp b3()
          b3():
            constrain v5 == u1 1
            return
          b1():
            store u8 0 at v6
            jmp b3()
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        let expected = "
        acir(inline) fn main f0 {
          b0(v0: [u8; 2]):
            v2 = array_get v0, index u8 0 -> u8
            v3 = cast v2 as u32
            v4 = truncate v3 to 1 bits, max_bit_size: 32
            v5 = cast v4 as u1
            v6 = allocate -> &mut Field
            store u8 0 at v6
            enable_side_effects v5
            v7 = cast v2 as Field
            v9 = add v7, Field 1
            v10 = cast v9 as u8
            v11 = load v6 -> u8
            v12 = not v5
            v13 = cast v4 as u8
            v14 = cast v12 as u8
            v15 = mul v13, v10
            v16 = mul v14, v11
            v17 = add v15, v16
            store v17 at v6
            enable_side_effects v12
            v18 = load v6 -> u8
            v19 = cast v12 as u8
            v20 = cast v4 as u8
            v21 = mul v20, v18
            store v21 at v6
            enable_side_effects u1 1
            constrain v5 == u1 1
            return
        }
        ";

        let flattened_ssa = ssa.flatten_cfg();
        let main = flattened_ssa.main();

        // Now assert that there is not an always-false constraint after flattening:
        let mut constrain_count = 0;
        for instruction in main.dfg[main.entry_block()].instructions() {
            if let Instruction::Constrain(lhs, rhs, ..) = main.dfg[*instruction] {
                if let (Some(lhs), Some(rhs)) =
                    (main.dfg.get_numeric_constant(lhs), main.dfg.get_numeric_constant(rhs))
                {
                    assert_eq!(lhs, rhs);
                }
                constrain_count += 1;
            }
        }
        assert_eq!(constrain_count, 1);

        assert_normalized_ssa_equals(flattened_ssa, expected);
    }

    #[test]
    fn undo_stores() {
        // Regression test for #1826. Ensures the `else` branch does not see the stores of the
        // `then` branch.
        //
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 0 at v0
            v2 = allocate -> &mut Field
            store Field 2 at v2
            v4 = load v2 -> Field
            v5 = lt v4, Field 2
            jmpif v5 then: b4, else: b1
          b1():
            v6 = load v2 -> Field
            v8 = lt v6, Field 4
            jmpif v8 then: b2, else: b3
          b2():
            v9 = load v0 -> Field
            v10 = load v2 -> Field
            v12 = mul v10, Field 100
            v13 = add v9, v12
            store v13 at v0
            v14 = load v2 -> Field
            v16 = add v14, Field 1
            store v16 at v2
            jmp b3()
          b3():
            jmp b5()
          b4():
            v17 = load v0 -> Field
            v18 = load v2 -> Field
            v20 = mul v18, Field 10
            v21 = add v17, v20
            store v21 at v0
            v22 = load v2 -> Field
            v23 = add v22, Field 1
            store v23 at v2
            jmp b5()
          b5():
            v24 = load v0 -> Field
            return v24
        }";

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.flatten_cfg().mem2reg().fold_constants();

        let main = ssa.main();

        // The return value should be 200, not 310
        match main.dfg[main.entry_block()].terminator() {
            Some(TerminatorInstruction::Return { return_values, .. }) => {
                match main.dfg.get_numeric_constant(return_values[0]) {
                    Some(constant) => {
                        let value = constant.to_u128();
                        assert_eq!(value, 200);
                    }
                    None => unreachable!("Expected constant 200 for return value"),
                }
            }
            _ => unreachable!("Should have terminator instruction"),
        }

        let expected = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = allocate -> &mut Field
            enable_side_effects u1 1
            return Field 200
        }
        ";

        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    #[should_panic = "ICE: branches merge inside of `then` branch"]
    fn panics_if_branches_merge_within_then_branch() {
        //! This is a regression test for https://github.com/noir-lang/noir/issues/6620

        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            jmpif v0 then: b2, else: b1
          b2():
            return
          b1():
            jmp b2()
        }
        ";
        let merged_ssa = Ssa::from_str(src).unwrap();
        let _ = merged_ssa.flatten_cfg();
    }

    #[test]
    fn eliminates_unnecessary_if_else_instructions_on_numeric_types() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: bool):
            v1 = allocate -> &mut [Field; 1]
            store Field 0 at v1
            jmpif v0 then: b1, else: b2
          b1():
            store Field 1 at v1 
            store Field 2 at v1 
            jmp b2()
          b2():
            v3 = load v1 -> Field
            return v3
        }";

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.flatten_cfg().mem2reg().fold_constants();

        let expected = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            v1 = allocate -> &mut [Field; 1]
            enable_side_effects v0
            v2 = not v0
            v3 = cast v0 as Field
            v4 = cast v2 as Field
            v6 = mul v3, Field 2
            v7 = mul v4, v3
            v8 = add v6, v7
            enable_side_effects u1 1
            return v8
        }
        ";

        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn eliminates_unnecessary_if_else_instructions_on_array_types() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: bool, v1: bool):
            v2 = make_array [Field 0] : [Field; 1]
            v3 = allocate -> &mut [Field; 1]
            store v2 at v3
            jmpif v0 then: b1, else: b2
          b1():
            v4 = make_array [Field 1] : [Field; 1]
            store v4 at v3 
            v5 = make_array [Field 2] : [Field; 1]
            store v5 at v3 
            jmp b2()
          b2():
            v24 = load v3 -> Field
            return v24
        }";

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa
            .flatten_cfg()
            .mem2reg()
            .remove_if_else()
            .fold_constants()
            .dead_instruction_elimination();

        let expected = "
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u1):
            enable_side_effects v0
            v2 = cast v0 as Field
            v4 = mul v2, Field 2
            v5 = make_array [v4] : [Field; 1]
            enable_side_effects u1 1
            return v5
        }
        ";

        assert_normalized_ssa_equals(ssa, expected);
    }
}
