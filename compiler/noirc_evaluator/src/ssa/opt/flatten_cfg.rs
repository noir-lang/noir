//! The flatten cfg optimization pass "flattens" the entire control flow graph into a single block.
//! This includes branches in the CFG with non-constant conditions. Flattening these requires
//! special handling for operations with side-effects and can lead to a loss of information since
//! the jmpif will no longer be in the program. As a result, this pass should usually be towards or
//! at the end of the optimization passes.
//! Furthermore, this pass assumes that no loops are present in the program and will assume
//! that a jmpif is a branch point and will attempt to merge both blocks. No actual looping will occur.
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
use std::sync::Arc;

use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

use acvm::{FieldElement, acir::AcirField, acir::BlackBoxFunc};
use iter_extended::vecmap;
use noirc_errors::call_stack::CallStackId;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
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

pub(crate) struct Context<'f> {
    pub(crate) inserter: FunctionInserter<'f>,

    /// This ControlFlowGraph is the graph from before the function was modified by this flattening pass.
    cfg: ControlFlowGraph,

    /// Target block of the flattening
    pub(crate) target_block: BasicBlockId,

    /// Maps start of branch -> end of branch
    branch_ends: HashMap<BasicBlockId, BasicBlockId>,

    /// A stack of each jmpif condition that was taken to reach a particular point in the program.
    /// When two branches are merged back into one, this constitutes a join point, and is analogous
    /// to the rest of the program after an if statement. When such a join point / end block is
    /// found, the top of this conditions stack is popped since we are no longer under that
    /// condition. If we are under multiple conditions (a nested if), the topmost condition is
    /// the most recent condition combined with all previous conditions via `And` instructions.
    condition_stack: Vec<ConditionalContext>,

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

    /// Flag to tell the context to not issue 'enable_side_effect' instructions during flattening.
    ///
    /// It is set with an attribute when defining a function that cannot fail whatsoever to avoid
    /// the overhead of handling side effects.
    /// It can also be set to true by flatten_single(), when no instruction is known to fail.
    pub(crate) no_predicate: bool,
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
    // List of predicated values, and their previous mapping
    predicated_values: HashMap<ValueId, ValueId>,
}

/// Flattens the control flow graph of the function such that it is left with a
/// single block containing all instructions and no more control-flow.
fn flatten_function_cfg(function: &mut Function, no_predicates: &HashMap<FunctionId, bool>) {
    // This pass may run forever on a brillig function.
    // Analyze will check if the predecessors have been processed and push the block to the back of
    // the queue. This loops forever if there are still any loops present in the program.
    if matches!(function.runtime(), RuntimeType::Brillig(_)) {
        return;
    }

    // Creates a context that will perform the flattening
    // We give it the map of the conditional branches in the CFG
    // and the target block where the flattened instructions should be added.
    let cfg = ControlFlowGraph::with_function(function);
    let branch_ends = branch_analysis::find_branch_ends(function, &cfg);
    let target_block = function.entry_block();

    let mut context = Context {
        inserter: FunctionInserter::new(function),
        cfg,
        branch_ends,
        condition_stack: Vec::new(),
        arguments_stack: Vec::new(),
        local_allocations: HashSet::default(),
        not_instructions: HashMap::default(),
        target_block,
        no_predicate: false,
    };
    context.flatten(no_predicates);
}

impl<'f> Context<'f> {
    //impl Context<'_> {
    pub(crate) fn new(
        function: &'f mut Function,
        cfg: ControlFlowGraph,
        branch_ends: HashMap<BasicBlockId, BasicBlockId>,
        target_block: BasicBlockId,
    ) -> Self {
        Context {
            inserter: FunctionInserter::new(function),
            cfg,
            branch_ends,
            condition_stack: Vec::new(),
            arguments_stack: Vec::new(),
            local_allocations: HashSet::default(),
            not_instructions: HashMap::default(),
            target_block,
            no_predicate: false,
        }
    }

    /// Flatten the CFG by inlining all instructions from the queued blocks
    /// until all blocks have been flattened.
    /// We follow the terminator of each block to determine which blocks to
    /// process next:
    /// If the terminator is a 'JumpIf', we assume we are entering a conditional statement and
    /// add the start blocks of the 'then_branch', 'else_branch' and the 'exit' block to the queue.
    /// Other blocks will have only one successor, so we will process them iteratively,
    /// until we reach one block already in the queue, i.e added when entering a conditional statement,
    /// i.e the 'else_branch' or the 'exit'. In that case we switch to the next block in the queue, instead
    /// of the successor.
    /// This process ensure that the blocks are always processed in this order:
    /// if_entry -> then_branch -> else_branch -> exit
    /// In case of nested if statements, for instance in the 'then_branch', it will be:
    /// if_entry -> then_branch -> if_entry_2 -> then_branch_2 -> exit_2 -> else_branch -> exit
    /// Information about the nested if statements is stored in the 'condition_stack' which
    /// is pop-ed/push-ed when entering/leaving a conditional statement.
    pub(crate) fn flatten(&mut self, no_predicates: &HashMap<FunctionId, bool>) {
        let mut queue = vec![self.target_block];
        while let Some(block) = queue.pop() {
            self.inline_block(block, no_predicates);
            let to_process = self.handle_terminator(block, &queue);
            for incoming_block in to_process {
                // Do not add blocks already in the queue
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
    ///
    /// The conditions are in a stack, they are added as conditional branches are encountered
    /// so the last one is the current condition.
    /// When processing a conditional branch, we first follow the 'then' branch and only after we
    /// process the 'else' branch. At that point, the ConditionalContext has the 'else_branch'
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

    /// Inline all instructions from the given block into the target block, and track slice capacities
    /// This is done by processing every instructions in the block and using the flattening context
    /// to push them in the target block
    ///
    /// - `no_predicates` indicates which functions have no predicates and for which we disable the handling side effects
    pub(crate) fn inline_block(
        &mut self,
        block: BasicBlockId,
        no_predicates: &HashMap<FunctionId, bool>,
    ) {
        if self.target_block == block {
            // we do not inline the target block into itself
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
    /// The update of the context is done by the functions 'if_start', 'then_stop' and 'else_stop'
    /// which perform the business logic when  entering a conditional statement, finishing the 'then-branch'
    /// and the 'else-branch, respectively.
    /// We know if a block is related to the conditional statement if is referenced by the 'work_list'
    /// Indeed, the start blocks of the 'then_branch' and 'else_branch' are added to the 'work_list' when
    /// starting to process a conditional statement.
    pub(crate) fn handle_terminator(
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
                let target = self.target_block;

                self.inserter.function.dfg.set_block_terminator(target, new_return);
                vec![]
            }
        }
    }

    /// Process a conditional statement by creating a 'ConditionalContext'
    /// with information about the branch, and storing it in the dedicated stack.
    /// Local allocations are moved to the 'then_branch' of the ConditionalContext.
    /// Returns the blocks corresponding to the 'then_branch', 'else_branch', and exit block of the conditional statement,
    /// so that they will be processed in this order.
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
            predicated_values: HashMap::default(),
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

    /// Switch context to the 'else-branch':
    /// - Negates the condition for the 'else_branch' and set it in the ConditionalContext
    /// - Move the local allocations to the 'else_branch'
    /// - Reset the predicated values to their old mapping in the inserter
    /// - Issues the 'enable_side_effect' instruction
    /// - Returns the exit block of the conditional statement
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
        self.reset_predicated_values(&mut cond_context);
        self.condition_stack.push(cond_context);

        self.insert_current_side_effects_enabled();

        assert_eq!(self.cfg.successors(*block).len(), 1);
        vec![self.cfg.successors(*block).next().unwrap()]
    }

    /// Negates a boolean value by inserting a Not instruction
    fn not_instruction(&mut self, condition: ValueId, call_stack: CallStackId) -> ValueId {
        if let Some(existing) = self.not_instructions.get(&condition) {
            return *existing;
        }

        let not = self.insert_instruction(Instruction::Not(condition), call_stack);
        self.not_instructions.insert(condition, not);
        not
    }

    /// Process the 'exit' block of a conditional statement:
    /// - Retrieves the local allocations from the Conditional Context
    /// - Reset the predicated values to their old mapping in the inserter
    /// - Issues the 'enable_side_effect' instruction
    /// - Joins the arguments from both branches
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

        self.reset_predicated_values(&mut cond_context);

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
    /// The merge of arguments is done by inserting an 'IfElse' instructions which returns
    /// the argument from the then_branch or the else_branch depending the the condition.
    /// They are added to the 'arguments_stack' instead of the arguments of the 2 branches.
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
        let block = self.target_block;

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

    /// Map the value to its predicated value, and store the previous mapping
    /// to the 'predicated_values' map if not already stored.
    fn predicate_value(&mut self, value: ValueId, predicated_value: ValueId) {
        let conditional_context = self.condition_stack.last_mut().unwrap();

        conditional_context
            .predicated_values
            .entry(value)
            .or_insert_with(|| self.inserter.resolve(value));

        self.inserter.map_value(value, predicated_value);
    }

    /// Restore the previous mapping of predicated values.
    fn reset_predicated_values(&mut self, conditional_context: &mut ConditionalContext) {
        for (value, old_mapping) in conditional_context.predicated_values.drain() {
            self.inserter.map_value(value, old_mapping);
        }
    }

    /// Insert a new instruction into the target block.
    /// Unlike push_instruction, this function will not map any ValueIds.
    /// within the given instruction, nor will it modify self.values in any way.
    fn insert_instruction(&mut self, instruction: Instruction, call_stack: CallStackId) -> ValueId {
        let block = self.target_block;
        self.inserter
            .function
            .dfg
            .insert_instruction_and_results(instruction, block, None, call_stack)
            .first()
    }

    /// Inserts a new instruction into the target block, using the given
    /// control type variables to specify result types if needed.
    /// Unlike push_instruction, this function will not map any ValueIds.
    /// within the given instruction, nor will it modify self.values in any way.
    fn insert_instruction_with_typevars(
        &mut self,
        instruction: Instruction,
        ctrl_typevars: Option<Vec<Type>>,
        call_stack: CallStackId,
    ) -> InsertInstructionResult {
        let block = self.target_block;
        self.inserter.function.dfg.insert_instruction_and_results(
            instruction,
            block,
            ctrl_typevars,
            call_stack,
        )
    }

    /// Checks the branch condition on the top of the stack and uses it to build and insert an
    /// `EnableSideEffectsIf` instruction into the target block.
    ///
    /// If the stack is empty, a "true" u1 constant is taken to be the active condition. This is
    /// necessary for re-enabling side-effects when re-emerging to a branch depth of 0.
    fn insert_current_side_effects_enabled(&mut self) {
        if self.no_predicate {
            return;
        }
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

    /// Push the given instruction to the end of the target block of the current function.
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
        let results =
            self.inserter.push_instruction_value(instruction, id, self.target_block, call_stack);

        // Remember an allocate was created local to this branch so that we do not try to merge store
        // values across branches for it later.
        if instruction_is_allocate {
            self.local_allocations.insert(results.first());
        }
    }

    /// If we are currently in a branch, we need to modify instructions that have side effects
    /// (e.g. constraints, stores, range checks) to ensure that the side effect is only applied
    /// if their branch is taken.
    /// For instance we multiply constrain instructions by the branch's condition (see optimization #1 in the module comment).
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
                    let casted_condition =
                        self.cast_condition_to_value_type(condition, lhs, call_stack);
                    let lhs = self.mul_by_condition(lhs, casted_condition, call_stack);
                    let rhs = self.mul_by_condition(rhs, casted_condition, call_stack);
                    Instruction::Constrain(lhs, rhs, message)
                }
                Instruction::Store { address, value } => {
                    // If this instruction immediately follows an allocate, and stores to that
                    // address there is no previous value to load and we don't need a merge anyway.
                    if self.local_allocations.contains(&address) {
                        Instruction::Store { address, value }
                    } else {
                        // Instead of storing `value`, we store: `if condition { value } else { previous_value }`
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
                    let casted_condition =
                        self.cast_condition_to_value_type(condition, value, call_stack);
                    let predicate_value =
                        self.mul_by_condition(value, casted_condition, call_stack);
                    // Issue #8617: update the value to be the predicated value.
                    // This ensures that the value has the correct bit size in all cases.
                    self.predicate_value(value, predicate_value);
                    Instruction::RangeCheck { value: predicate_value, max_bit_size, assert_message }
                }
                Instruction::Call { func, mut arguments } => match self.inserter.function.dfg[func]
                {
                    // A ToBits (or ToRadix in general) can fail if the input has more bits than the target.
                    // We ensure it does not fail by multiplying the input by the condition.
                    Value::Intrinsic(Intrinsic::ToBits(_) | Intrinsic::ToRadix(_)) => {
                        let field = arguments[0];
                        let casted_condition =
                            self.cast_condition_to_value_type(condition, field, call_stack);
                        let field = self.mul_by_condition(field, casted_condition, call_stack);

                        arguments[0] = field;

                        Instruction::Call { func, arguments }
                    }

                    Value::Intrinsic(Intrinsic::BlackBox(blackbox)) => match blackbox {
                        //Issue #5045: We set curve points to g1, g2=2g1 if condition is false, to ensure that they are on the curve, if not the addition may fail.
                        // If inputs are distinct curve points, then so is their predicate version.
                        // If inputs are identical (point doubling), then so is their predicate version
                        // Hence the assumptions for calling EmbeddedCurveAdd are kept by this transformation.
                        BlackBoxFunc::EmbeddedCurveAdd => {
                            #[cfg(feature = "bn254")]
                            {
                                let generators = Self::grumpkin_generators();
                                // Convert the generators to ValueId
                                let generators = generators
                                    .iter()
                                    .map(|v| {
                                        self.inserter
                                            .function
                                            .dfg
                                            .make_constant(*v, NumericType::NativeField)
                                    })
                                    .collect::<Vec<ValueId>>();
                                let (point1_x, point2_x) = self.predicate_argument(
                                    &arguments,
                                    &generators,
                                    true,
                                    condition,
                                    call_stack,
                                );
                                let (point1_y, point2_y) = self.predicate_argument(
                                    &arguments,
                                    &generators,
                                    false,
                                    condition,
                                    call_stack,
                                );
                                arguments[0] = point1_x;
                                arguments[1] = point1_y;
                                arguments[3] = point2_x;
                                arguments[4] = point2_y;
                            }

                            Instruction::Call { func, arguments }
                        }

                        // For MSM, we also ensure the inputs are on the curve if the predicate is false.
                        BlackBoxFunc::MultiScalarMul => {
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

                        // The ECDSA blackbox functions will fail to prove inside barretenberg in the situation where
                        // the public key doesn't not sit on the relevant curve.
                        //
                        // We then replace the public key with the generator point if the constraint is inactive to avoid
                        // invalid public keys from causing constraints to fail.
                        BlackBoxFunc::EcdsaSecp256k1 => {
                            // See: https://github.com/RustCrypto/elliptic-curves/blob/3381a99b6412ef9fa556e32a834e401d569007e3/k256/src/arithmetic/affine.rs#L57-L76
                            const GENERATOR_X: [u8; 32] = [
                                0x79, 0xbe, 0x66, 0x7e, 0xf9, 0xdc, 0xbb, 0xac, 0x55, 0xa0, 0x62,
                                0x95, 0xce, 0x87, 0x0b, 0x07, 0x02, 0x9b, 0xfc, 0xdb, 0x2d, 0xce,
                                0x28, 0xd9, 0x59, 0xf2, 0x81, 0x5b, 0x16, 0xf8, 0x17, 0x98,
                            ];
                            const GENERATOR_Y: [u8; 32] = [
                                0x48, 0x3a, 0xda, 0x77, 0x26, 0xa3, 0xc4, 0x65, 0x5d, 0xa4, 0xfb,
                                0xfc, 0x0e, 0x11, 0x08, 0xa8, 0xfd, 0x17, 0xb4, 0x48, 0xa6, 0x85,
                                0x54, 0x19, 0x9c, 0x47, 0xd0, 0x8f, 0xfb, 0x10, 0xd4, 0xb8,
                            ];

                            arguments[0] = self.merge_with_array_constant(
                                arguments[0],
                                GENERATOR_X,
                                condition,
                                call_stack,
                            );
                            arguments[1] = self.merge_with_array_constant(
                                arguments[1],
                                GENERATOR_Y,
                                condition,
                                call_stack,
                            );

                            Instruction::Call { func, arguments }
                        }
                        BlackBoxFunc::EcdsaSecp256r1 => {
                            // See: https://github.com/RustCrypto/elliptic-curves/blob/3381a99b6412ef9fa556e32a834e401d569007e3/p256/src/arithmetic.rs#L46-L57
                            const GENERATOR_X: [u8; 32] = [
                                0x6b, 0x17, 0xd1, 0xf2, 0xe1, 0x2c, 0x42, 0x47, 0xf8, 0xbc, 0xe6,
                                0xe5, 0x63, 0xa4, 0x40, 0xf2, 0x77, 0x03, 0x7d, 0x81, 0x2d, 0xeb,
                                0x33, 0xa0, 0xf4, 0xa1, 0x39, 0x45, 0xd8, 0x98, 0xc2, 0x96,
                            ];
                            const GENERATOR_Y: [u8; 32] = [
                                0x4f, 0xe3, 0x42, 0xe2, 0xfe, 0x1a, 0x7f, 0x9b, 0x8e, 0xe7, 0xeb,
                                0x4a, 0x7c, 0x0f, 0x9e, 0x16, 0x2b, 0xce, 0x33, 0x57, 0x6b, 0x31,
                                0x5e, 0xce, 0xcb, 0xb6, 0x40, 0x68, 0x37, 0xbf, 0x51, 0xf5,
                            ];

                            arguments[0] = self.merge_with_array_constant(
                                arguments[0],
                                GENERATOR_X,
                                condition,
                                call_stack,
                            );
                            arguments[1] = self.merge_with_array_constant(
                                arguments[1],
                                GENERATOR_Y,
                                condition,
                                call_stack,
                            );

                            Instruction::Call { func, arguments }
                        }

                        // TODO: https://github.com/noir-lang/noir/issues/8998
                        BlackBoxFunc::RecursiveAggregation => Instruction::Call { func, arguments },

                        // These functions will always be satisfiable no matter the input so no modification is needed.
                        BlackBoxFunc::AND
                        | BlackBoxFunc::XOR
                        | BlackBoxFunc::AES128Encrypt
                        | BlackBoxFunc::Blake2s
                        | BlackBoxFunc::Blake3
                        | BlackBoxFunc::Keccakf1600
                        | BlackBoxFunc::Poseidon2Permutation
                        | BlackBoxFunc::Sha256Compression => Instruction::Call { func, arguments },

                        BlackBoxFunc::RANGE => unreachable!(
                            "RANGE should have been converted into `Instruction::RangeCheck`"
                        ),

                        BlackBoxFunc::BigIntAdd
                        | BlackBoxFunc::BigIntSub
                        | BlackBoxFunc::BigIntMul
                        | BlackBoxFunc::BigIntDiv
                        | BlackBoxFunc::BigIntFromLeBytes
                        | BlackBoxFunc::BigIntToLeBytes => {
                            todo!("BigInt opcodes are not supported yet")
                        }
                    },

                    _ => Instruction::Call { func, arguments },
                },
                other => other,
            }
        } else {
            instruction
        }
    }

    #[cfg(feature = "bn254")]
    fn grumpkin_generators() -> Vec<FieldElement> {
        let g1_x = FieldElement::from_hex("0x01").unwrap();
        let g1_y =
            FieldElement::from_hex("0x02cf135e7506a45d632d270d45f1181294833fc48d823f272c").unwrap();
        let g2_x = FieldElement::from_hex(
            "0x06ce1b0827aafa85ddeb49cdaa36306d19a74caa311e13d46d8bc688cdbffffe",
        )
        .unwrap();
        let g2_y = FieldElement::from_hex(
            "0x1c122f81a3a14964909ede0ba2a6855fc93faf6fa1a788bf467be7e7a43f80ac",
        )
        .unwrap();
        vec![g1_x, g1_y, g2_x, g2_y]
    }

    /// Merges the given array with a constant array of 32 elements of type `u8`.
    ///
    /// This is expected to be used for the ECDSA secp256k1 and secp256r1 generators,
    /// where the x and y coordinates of the generators are constant values.
    fn merge_with_array_constant(
        &mut self,
        array: ValueId,
        constant: [u8; 32],
        condition: ValueId,
        call_stack: CallStackId,
    ) -> ValueId {
        let expected_array_type = Type::Array(Arc::new(vec![Type::unsigned(8)]), 32);
        let array_type = self.inserter.function.dfg.type_of_value(array);
        assert_eq!(array_type, expected_array_type);

        let elements = constant
            .iter()
            .map(|elem| {
                self.inserter
                    .function
                    .dfg
                    .make_constant(FieldElement::from(*elem as u32), NumericType::unsigned(8))
            })
            .collect();
        let constant_array = Instruction::MakeArray { elements, typ: expected_array_type };
        let constant_array_value = self.insert_instruction(constant_array, call_stack);
        let not_condition = self.not_instruction(condition, call_stack);

        self.insert_instruction(
            Instruction::IfElse {
                then_condition: condition,
                then_value: array,
                else_condition: not_condition,
                else_value: constant_array_value,
            },
            call_stack,
        )
    }

    /// Returns the values corresponding to the given inputs by doing
    /// 'if condition {inputs} else {generators}'
    /// It is done for the abscissas or the ordinates, depending on 'abscissa'.
    /// Inputs are supposed to be of the form:
    /// - inputs: (point1_x, point1_y, point1_infinite, point2_x, point2_y, point2_infinite)
    /// - generators: [g1_x, g1_y, g2_x, g2_y]
    /// - index: true for abscissa, false for ordinate
    #[cfg(feature = "bn254")]
    fn predicate_argument(
        &mut self,
        inputs: &[ValueId],
        generators: &[ValueId],
        abscissa: bool,
        condition: ValueId,
        call_stack: CallStackId,
    ) -> (ValueId, ValueId) {
        let index = !abscissa as usize;
        if inputs[3 + index] == inputs[index] {
            let predicated_value =
                self.var_or(inputs[index], condition, generators[index], call_stack);
            (predicated_value, predicated_value)
        } else {
            (
                self.var_or(inputs[index], condition, generators[index], call_stack),
                self.var_or(inputs[3 + index], condition, generators[2 + index], call_stack),
            )
        }
    }

    /// 'Cast' the 'condition' to 'value' type
    ///
    /// This needed because we need to multiply the condition with several values
    /// in order to 'nullify' side-effects when the 'condition' is false (in 'handle_instruction_side_effects()' function).
    /// Since the condition is a boolean, it can be safely casted to any other type.
    fn cast_condition_to_value_type(
        &mut self,
        condition: ValueId,
        value: ValueId,
        call_stack: CallStackId,
    ) -> ValueId {
        let argument_type = self.inserter.function.dfg.type_of_value(value);
        let cast = Instruction::Cast(condition, argument_type.unwrap_numeric());
        self.insert_instruction(cast, call_stack)
    }

    /// Insert a multiplication between 'condition' and 'value'
    fn mul_by_condition(
        &mut self,
        value: ValueId,
        condition: ValueId,
        call_stack: CallStackId,
    ) -> ValueId {
        // Unchecked mul because the condition is always 0 or 1
        let cast_condition = self.cast_condition_to_value_type(condition, value, call_stack);
        self.insert_instruction(
            Instruction::binary(BinaryOp::Mul { unchecked: true }, value, cast_condition),
            call_stack,
        )
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
        let field = self.mul_by_condition(var, condition, call_stack);
        let not_condition = self.not_instruction(condition, call_stack);
        // Unchecked add because of the values is guaranteed to be 0
        self.insert_instruction(
            Instruction::binary(BinaryOp::Add { unchecked: true }, field, not_condition),
            call_stack,
        )
    }
    // Computes: if condition { var } else { other }
    #[cfg(feature = "bn254")]
    fn var_or(
        &mut self,
        var: ValueId,
        condition: ValueId,
        other: ValueId,
        call_stack: CallStackId,
    ) -> ValueId {
        let field = self.mul_by_condition(var, condition, call_stack);
        let not_condition = self.not_instruction(condition, call_stack);
        let else_field = self.mul_by_condition(other, not_condition, call_stack);
        // Unchecked add because one of the values is guaranteed to be 0
        self.insert_instruction(
            Instruction::binary(BinaryOp::Add { unchecked: true }, field, else_field),
            call_stack,
        )
    }
}

#[cfg(test)]
mod test {
    use acvm::acir::AcirField;

    use crate::{
        assert_ssa_snapshot,
        ssa::{
            Ssa,
            ir::{
                dfg::DataFlowGraph,
                instruction::{Instruction, TerminatorInstruction},
                value::{Value, ValueId},
            },
        },
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

        let ssa = ssa.flatten_cfg();
        assert_ssa_snapshot!(ssa, @r"
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
        ");
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

        let ssa = ssa.flatten_cfg();
        assert_eq!(ssa.main().reachable_blocks().len(), 1);
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u1):
            enable_side_effects v0
            v2 = unchecked_mul v1, v0
            constrain v2 == v0
            v3 = not v0
            enable_side_effects u1 1
            return
        }
        ");
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

        let ssa = ssa.flatten_cfg();
        assert_ssa_snapshot!(ssa, @r"
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
        ");
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

        let ssa = ssa.flatten_cfg();
        assert_ssa_snapshot!(ssa, @r"
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
        ");
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

        let main = ssa.main();
        let ret = match main.dfg[main.entry_block()].terminator() {
            Some(TerminatorInstruction::Return { return_values, .. }) => return_values[0],
            _ => unreachable!("Should have terminator instruction"),
        };

        let merged_values = get_all_constants_reachable_from_instruction(&main.dfg, ret);
        assert_eq!(merged_values, vec![2, 3, 5, 6]);

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u1):
            v2 = allocate -> &mut Field
            enable_side_effects v0
            v3 = not v0
            v4 = cast v0 as Field
            v5 = cast v3 as Field
            v7 = mul v4, Field 2
            v8 = add v7, v5
            v9 = unchecked_mul v0, v1
            enable_side_effects v9
            v10 = not v9
            v11 = cast v9 as Field
            v12 = cast v10 as Field
            v14 = mul v11, Field 5
            v15 = mul v12, v8
            v16 = add v14, v15
            v17 = not v1
            v18 = unchecked_mul v0, v17
            enable_side_effects v18
            v19 = not v18
            v20 = cast v18 as Field
            v21 = cast v19 as Field
            v23 = mul v20, Field 6
            v24 = mul v21, v16
            v25 = add v23, v24
            enable_side_effects v0
            enable_side_effects v3
            v26 = cast v3 as Field
            v27 = cast v0 as Field
            v29 = mul v26, Field 3
            v30 = mul v27, v25
            v31 = add v29, v30
            enable_side_effects u1 1
            return v31
        }
        ");
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

        assert_ssa_snapshot!(flattened_ssa, @r"
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
        ");
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

        let ssa = ssa.flatten_cfg();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            enable_side_effects u1 1
            constrain u1 0 == u1 1
            return
        }
        ");
    }

    #[test]
    fn should_not_merge_incorrectly_to_false() {
        // Regression test for #1792
        // Tests that it does not simplify a true constraint an always-false constraint

        let src = "
        acir(inline) fn main f0 {
          b0(v0: [u8; 2]):
            v2 = array_get v0, index u32 0 -> u8
            v3 = cast v2 as u32
            v4 = truncate v3 to 1 bits, max_bit_size: 32
            v5 = cast v4 as u1
            v6 = allocate -> &mut Field
            store u8 0 at v6
            jmpif v5 then: b2, else: b1
          b2():
            v7 = cast v2 as Field
            v9 = add v7, Field 1
            v10 = truncate v9 to 8 bits, max_bit_size: 254
            v11 = cast v10 as u8
            store v11 at v6
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

        assert_ssa_snapshot!(flattened_ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: [u8; 2]):
            v2 = array_get v0, index u32 0 -> u8
            v3 = cast v2 as u32
            v4 = truncate v3 to 1 bits, max_bit_size: 32
            v5 = cast v4 as u1
            v6 = allocate -> &mut Field
            store u8 0 at v6
            enable_side_effects v5
            v8 = cast v2 as Field
            v10 = add v8, Field 1
            v11 = truncate v10 to 8 bits, max_bit_size: 254
            v12 = cast v11 as u8
            v13 = load v6 -> u8
            v14 = not v5
            v15 = cast v4 as u8
            v16 = cast v14 as u8
            v17 = unchecked_mul v15, v12
            v18 = unchecked_mul v16, v13
            v19 = unchecked_add v17, v18
            store v19 at v6
            enable_side_effects v14
            v20 = load v6 -> u8
            v21 = cast v14 as u8
            v22 = cast v4 as u8
            v23 = unchecked_mul v22, v20
            store v23 at v6
            enable_side_effects u1 1
            constrain v5 == u1 1
            return
        }
        ");
    }

    #[test]
    fn undo_stores() {
        // Regression test for #1826. Ensures the `else` branch does not see the stores of the
        // `then` branch.
        //
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut u32
            store u32 0 at v0
            v2 = allocate -> &mut u32
            store u32 2 at v2
            v4 = load v2 -> u32
            v5 = lt v4, u32 2
            jmpif v5 then: b4, else: b1
          b1():
            v6 = load v2 -> u32
            v8 = lt v6, u32 4
            jmpif v8 then: b2, else: b3
          b2():
            v9 = load v0 -> u32
            v10 = load v2 -> u32
            v12 = mul v10, u32 100
            v13 = add v9, v12
            store v13 at v0
            v14 = load v2 -> u32
            v16 = add v14, u32 1
            store v16 at v2
            jmp b3()
          b3():
            jmp b5()
          b4():
            v17 = load v0 -> u32
            v18 = load v2 -> u32
            v20 = mul v18, u32 10
            v21 = add v17, v20
            store v21 at v0
            v22 = load v2 -> u32
            v23 = add v22, u32 1
            store v23 at v2
            jmp b5()
          b5():
            v24 = load v0 -> u32
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

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut u32
            v1 = allocate -> &mut u32
            enable_side_effects u1 1
            return u32 200
        }
        ");
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

        assert_ssa_snapshot!(ssa, @r"
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
        ");
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
            .unwrap()
            .fold_constants()
            .dead_instruction_elimination();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u1):
            enable_side_effects v0
            v2 = cast v0 as Field
            v4 = mul v2, Field 2
            v5 = make_array [v4] : [Field; 1]
            enable_side_effects u1 1
            return v5
        }
        ");
    }

    #[test]
    fn do_not_replace_else_condition_with_nested_if_same_then_cond() {
        // When inserting an `IfElse` instruction we will attempt to simplify when the then condition
        // of the inner if-else matches the parent's if-else then condition.
        // e.g. such as the following pseudocode:
        // ```
        // if cond {
        //   if cond { ... } else { ... }
        // } else {
        //   ...
        // }
        // ```
        // In the SSA below we can see how the jmpif condition in b0 matches the jmpif condition in b1.
        let src = "
        acir(inline) pure fn main f0 {
          b0(v0: u1, v1: [[u1; 2]; 3]):
            v4 = not v0
            jmpif v0 then: b1, else: b2
          b1():
            v7 = not v0
            jmpif v0 then: b3, else: b4
          b2():
            v6 = array_get v1, index u32 0 -> [u1; 2]
            jmp b5(v6)
          b3():
            v9 = array_get v1, index u32 0 -> [u1; 2]
            jmp b6(v9)
          b4():
            v8 = array_get v1, index u32 0 -> [u1; 2]
            jmp b6(v8)
          b5(v2: [u1; 2]):
            return v2
          b6(v3: [u1; 2]):
            jmp b5(v3)
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.flatten_cfg();

        // You will notice in the expected SSA that there is no nested if statement. This is because the
        // final instruction `v12 = if v0 then v5 else (if v6) v10` used to have `v9` as its then block value.
        // As they share the same then condition we can simplify the then value in the outer if-else statement to the inner if-else
        // statement's then value. This is why the then value is `v5` in both if-else instructions below.
        // We want to make sure that the else condition in the final instruction `v12 = if v0 then v5 else (if v6) v10`
        // remains v6 and is not altered when performing this optimization.
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) pure fn main f0 {
          b0(v0: u1, v1: [[u1; 2]; 3]):
            v2 = not v0
            enable_side_effects v0
            v3 = not v0
            enable_side_effects v0
            v5 = array_get v1, index u32 0 -> [u1; 2]
            v6 = not v0
            v7 = unchecked_mul v0, v6
            enable_side_effects v7
            v8 = array_get v1, index u32 0 -> [u1; 2]
            enable_side_effects v0
            v9 = if v0 then v5 else (if v7) v8
            enable_side_effects v6
            v10 = array_get v1, index u32 0 -> [u1; 2]
            enable_side_effects u1 1
            v12 = if v0 then v5 else (if v6) v10
            return v12
        }
        ");
    }

    #[test]
    #[cfg(feature = "bn254")]
    fn test_grumpkin_points() {
        use crate::ssa::opt::flatten_cfg::Context;
        use acvm::acir::FieldElement;

        let generators = Context::grumpkin_generators();
        let len = generators.len();
        for i in (0..len).step_by(2) {
            let gen_x = generators[i];
            let gen_y = generators[i + 1];
            assert!(
                gen_y * gen_y - gen_x * gen_x * gen_x + FieldElement::from(17_u128)
                    == FieldElement::zero()
            );
        }
    }

    #[test]
    fn use_predicated_value() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: bool, v1: u32):
            v3 = add u32 42, v1
            jmpif v0 then: b1, else: b2
          b1():
            range_check v3 to 16 bits
            jmp b3(v3)
          b2():
            v4 = add u32 3, v3
            jmp b3(v4)
          b3(v5: u32):
            return v5
        }";

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.flatten_cfg();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u32):
            v3 = add u32 42, v1
            enable_side_effects v0
            v4 = cast v0 as u32
            v5 = cast v0 as u32
            v6 = unchecked_mul v3, v5
            range_check v6 to 16 bits
            v7 = not v0
            enable_side_effects v7
            v9 = add u32 3, v3
            enable_side_effects u1 1
            v11 = cast v0 as u32
            v12 = cast v7 as u32
            v13 = unchecked_mul v11, v3
            v14 = unchecked_mul v12, v9
            v15 = unchecked_add v13, v14
            return v15
        }
        ");
    }
}
