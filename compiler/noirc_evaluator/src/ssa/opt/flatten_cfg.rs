//! This file contains the SSA flattening pass - a required pass for ACIR to remove any remaining
//! control-flow in the singular program-function, resulting in a single block containing the
//! program logic.
//!
//! ACIR/Brillig differences within this pass:
//!   - This pass is strictly ACIR-only and never mutates brillig functions.
//!
//! Conditions:
//!   - Precondition: Inlining has been performed which should result in there being no remaining
//!     `call` instructions to acir/constrained functions (unless they are `InlineType::Fold`).
//!     This also means the only acir functions in the program should be `main` (if main is
//!     constrained), or any constrained `InlineType::Fold` functions.
//!   - Precondition: Each constrained function should have no loops (unrolling has been performed).
//!   - Precondition: "Equal" constraints have not been turned into "NotEqual".
//!   - Postcondition: Each constrained function should now consist of only one block where the
//!     terminator instruction is always a return.
//!
//! Relevance to other passes:
//!   - Flattening effectively eliminates control-flow entirely which can make it easier for
//!     subsequent passes to optimize code. Mem2reg for example should be able to remove all
//!     references in constrained (ACIR) code.
//!   - Flattening inserts `Instruction::IfElse` to merge the values from an if-expression's "then"
//!     and "else" branches. These are immediately simplified out for numeric values, but for
//!     arrays and vectors we require the `remove_if_else` SSA pass to later be run to remove the
//!     remaining `Instruction::IfElse` instructions.
//!
//! Implementation details & examples:
//!
//! The flatten cfg optimization pass "flattens" the entire control flow graph into a single block.
//! This includes branches in the CFG with non-constant conditions. Flattening these requires
//! special handling for operations with side-effects and can lead to a loss of information since
//! the jmpif will no longer be in the program. As a result, this pass should usually be towards or
//! at the end of the optimization passes.
//!
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
//! ```text
//! b0(v0: u1):
//!   jmpif v0, then: b1, else: b2
//! b1():
//!   v1 = call f0
//!   jmp b3(v1)
//! ... blocks b2 & b3 ...
//! ```
//!
//! Would brace the call instruction as such:
//! ```text
//!   enable_side_effects v0
//!   v1 = call f0
//!   enable_side_effects u1 1
//! ```
//!
//! (Note: we restore to "true" to indicate that this program point is not nested within any
//! other branches. Each `enable_side_effects` overrides the previous, they do not implicitly stack.)
//!
//! When we are flattening a block that was reached via a jmpif with a non-constant condition `c`,
//! the following transformations of certain instructions within the block are expected:
//!
//! 1. A constraint is multiplied by the condition and changes the constraint to
//!    an equality with `c`:
//! ```text
//! constrain v0
//! ============
//! v1 = mul v0, c
//! v2 = eq v1, c
//! constrain v2
//! ```
//!
//! 2. If we reach the end block of the branch created by the jmpif instruction, its block parameters
//!    will be merged. To merge the jmp arguments of the then and else branches, the formula
//!    `c * then_arg + !c * else_arg` is used for each argument. Note that this is represented by
//!    `Instruction::IfElse` which is often simplified to the above when inserted, but in the case
//!    of complex values (arrays and vectors) this simplification is delayed until the
//!    `remove_if_else` SSA pass.
//!
//! ```text
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
//! ```
//!
//! 3. Each `store v0 in v1` is replaced with a store of a new value
//!    `v4 = if v3 then v0 else v2` where `v3` is the current condition
//!    given by `enable_side_effects v3` and `v2` is the result of
//!    a newly-given `v2 = load v0` inserted before the store.
//!
//! ```text
//! b0(v0: u1):
//!   v1 = allocate -> &mut Field
//!   store Field 3 at v1
//!   jmpif v0, then: b1, else: b2
//! b1():
//!   store Field 5 at v1
//!   ... b1 instructions ...
//!   jmp b3
//! b2():
//!   store Field 7 at v1
//!   ... b2 instructions ...
//!   jmp b3
//! b3():
//!   ... b3 instructions ...
//! =========================
//! b0():
//!   v1 = allocate -> &mut Field
//!   store Field 3 at v1     // no prior value so we do not load & merge
//!   enable_side_effects v0  // former block b1
//!   v2 = load v1 -> Field
//!   v3 = not v0
//!   v4 = if v0 then Field 5 else (if v3) v2
//!   store v4 at v1
//!   ... b1 instructions ...
//!   enable_side_effects v3  // former block b2
//!   v5 = load v1 -> Field
//!   v6 = if v3 then Field 7 else (if v0) v5
//!   store v6 at v1
//!   ... b2 instructions ...
//!   enable_side_effects u1 1
//!   ... b3 instructions ...
//! ```

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use acvm::{FieldElement, acir::AcirField, acir::BlackBoxFunc};
use indexmap::set::IndexSet;
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
        let no_predicates: HashMap<_, _> =
            self.functions.values().map(|f| (f.id(), f.is_no_predicates())).collect();

        for function in self.functions.values_mut() {
            // This pass may run forever on a brillig function - we check if block predecessors have
            // been processed and push the block to the back of the queue. This loops forever if
            // there are still any loops present in the program.
            if matches!(function.runtime(), RuntimeType::Brillig(_)) {
                continue;
            }

            #[cfg(debug_assertions)]
            flatten_cfg_pre_check(function);

            flatten_function_cfg(function, &no_predicates);

            #[cfg(debug_assertions)]
            flatten_cfg_post_check(function);
        }
        self
    }
}

/// Pre-check condition for [Ssa::flatten_cfg].
///
/// Panics if:
///   - Any ACIR function has at least 1 loop
///   - Any ACIR function has a `ConstrainNotEqual` instruction
#[cfg(debug_assertions)]
fn flatten_cfg_pre_check(function: &Function) {
    if !function.runtime().is_acir() {
        return;
    }
    let loops = super::Loops::find_all(function);
    assert_eq!(loops.yet_to_unroll.len(), 0);

    for block in function.reachable_blocks() {
        for instruction in function.dfg[block].instructions() {
            if matches!(function.dfg[*instruction], Instruction::ConstrainNotEqual(_, _, _)) {
                panic!("ConstrainNotEqual should not be introduced before flattening");
            }
        }
    }
}

/// Post-check condition for [Ssa::flatten_cfg].
///
/// Panics if:
///   - Any ACIR function contains > 1 block
#[cfg(debug_assertions)]
pub(super) fn flatten_cfg_post_check(function: &Function) {
    if !function.runtime().is_acir() {
        return;
    }
    let blocks = function.reachable_blocks();
    assert_eq!(blocks.len(), 1, "CFG contains more than 1 block");
}

pub(crate) struct Context<'f> {
    pub(crate) inserter: FunctionInserter<'f>,

    /// This `ControlFlowGraph` is the graph from before the function was modified by this flattening pass.
    cfg: ControlFlowGraph,

    /// Target block of the flattening.
    pub(crate) target_block: BasicBlockId,

    /// Maps start of branch -> end of branch.
    branch_ends: HashMap<BasicBlockId, BasicBlockId>,

    /// A stack of each jmpif condition that was taken to reach a particular point in the program.
    /// When two branches are merged back into one, this constitutes a join point, and is analogous
    /// to the rest of the program after an if statement. When such a join point / end block is
    /// found, the top of this conditions stack is popped since we are no longer under that
    /// condition. If we are under multiple conditions (a nested if), the topmost condition is
    /// the most recent condition combined with all previous conditions via `And` instructions.
    condition_stack: Vec<ConditionalContext>,

    /// Arguments prepared by the last inlined block for the next block we are about to process.
    next_arguments: Option<Vec<ValueId>>,

    /// Stores all allocations local to the current branch.
    ///
    /// Since these are local to the current branch (i.e. only defined within one branch of
    /// an if expression), they should not be merged with their previous value or stored value in
    /// the other branch since there is no such value.
    ///
    /// The `ValueId` here is that which is returned by the allocate instruction.
    local_allocations: HashSet<ValueId>,

    /// A map from `cond` to `Not(cond)`.
    ///
    /// `Not` instructions are inserted constantly by this pass and this map helps keep
    /// us from unnecessarily inserting extra instructions, and keeps IDs unique which
    /// helps simplifications.
    not_instructions: HashMap<ValueId, ValueId>,

    /// Flag to tell the context to not issue 'enable_side_effect' instructions during flattening.
    ///
    /// It is set with an attribute when defining a function that cannot fail whatsoever to avoid
    /// the overhead of handling side effects.
    ///
    /// It can also be set to true when no instruction is known to fail.
    pub(crate) no_predicate: bool,
}

#[derive(Clone)]
struct ConditionalBranch {
    /// Contains the last processed block during the processing of the branch.
    ///
    /// It starts out empty, then gets filled in when we finish the branch.
    last_block: Option<BasicBlockId>,
    /// The resolved condition of the branch, AND-ed with all outer branch conditions.
    condition: ValueId,
}

struct ConditionalContext {
    /// Condition from the conditional statement
    condition: ValueId,
    /// Block containing the conditional statement
    entry_block: BasicBlockId,
    /// First block of the then branch
    then_branch: ConditionalBranch,
    /// First block of the else branch
    else_branch: Option<ConditionalBranch>,
    /// Call stack where the final location is that of the entire `if` expression
    call_stack: CallStackId,
    /// Vector of values which have been replaced with a predicated variant,
    /// mapping them to their original value.
    ///
    /// For example if we have `v1 = v2` predicated upon `v0`, then `v1` becomes `v0 * v2`,
    /// and this mapping will contain `v1 -> v2`.
    ///
    /// We use this information to reset the values to their originals when we exit from branches.
    predicated_values: HashMap<ValueId, ValueId>,
    /// The allocations accumulated before processing the branch.
    local_allocations: HashSet<ValueId>,
}

/// Flattens the control flow graph of the function such that it is left with a
/// single block containing all instructions and no more control-flow.
fn flatten_function_cfg(function: &mut Function, no_predicates: &HashMap<FunctionId, bool>) {
    // Creates a context that will perform the flattening
    // We give it the map of the conditional branches in the CFG
    // and the target block where the flattened instructions should be added.
    let cfg = ControlFlowGraph::with_function(function);
    let branch_ends = branch_analysis::find_branch_ends(function, &cfg);
    let target_block = function.entry_block();

    let mut context = Context::new(function, cfg, branch_ends, target_block);

    context.flatten(no_predicates);
}

/// Blocks enqueued for processing.
///
/// It contains a block at most once.
pub(crate) type WorkVector = IndexSet<BasicBlockId>;

impl<'f> Context<'f> {
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
            next_arguments: None,
            local_allocations: HashSet::default(),
            not_instructions: HashMap::default(),
            target_block,
            no_predicate: false,
        }
    }

    /// Flatten the CFG by inlining all instructions from the queued blocks
    /// until all blocks have been flattened.
    ///
    /// We follow the terminator of each block to determine which blocks to process next:
    /// * If the terminator is a 'JumpIf', we assume we are entering a conditional statement and
    ///   add the start blocks of the 'then_branch', 'else_branch' and the 'exit' block to the queue.
    /// * Other blocks will have only one successor, so we will process them iteratively,
    ///   until we reach one block already in the queue, added when entering a conditional statement,
    ///   i.e. the 'else_branch' or the 'exit'. In that case we switch to the next block in the queue,
    ///   instead of the successor.
    ///
    /// This process ensures that the blocks are always processed in this order:
    /// * if_entry -> then_branch -> else_branch -> exit
    ///
    /// In case of nested if statements, for instance in the 'then_branch', it will be:
    /// * if_entry -> then_branch -> if_entry_2 -> then_branch_2 -> exit_2 -> else_branch -> exit
    ///
    /// Information about the nested if statements is stored in the 'condition_stack' which
    /// is popped/pushed when entering/leaving a conditional statement.
    pub(crate) fn flatten(&mut self, no_predicates: &HashMap<FunctionId, bool>) {
        let mut work_vector = WorkVector::new();
        work_vector.insert(self.target_block);
        while let Some(block) = work_vector.pop() {
            self.inline_block(block, no_predicates);
            let to_process = self.handle_terminator(block, &work_vector);
            work_vector.extend(to_process);
        }
        assert!(self.next_arguments.is_none(), "no leftover arguments");
        self.inserter.map_data_bus_in_place();
    }

    /// Returns the updated condition so that
    /// it is 'AND-ed' with the previous condition (if any)
    fn link_condition(&mut self, condition: ValueId) -> ValueId {
        // Retrieve the previous condition
        if let Some(last_condition) = self.get_last_condition() {
            let and = Instruction::binary(BinaryOp::And, last_condition, condition);
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
    /// process the 'else' branch. At that point, the `ConditionalContext` has the 'else_branch'
    fn get_last_condition(&self) -> Option<ValueId> {
        self.condition_stack
            .last()
            .map(|context| context.else_branch.as_ref().unwrap_or(&context.then_branch))
            .map(|branch| branch.condition)
    }

    /// Use the provided map to say if the instruction is a call to a `no_predicates` function
    fn is_call_to_no_predicate_function(
        &self,
        no_predicates: &HashMap<FunctionId, bool>,
        instruction: &InstructionId,
    ) -> bool {
        if let Instruction::Call { func, .. } = self.inserter.function.dfg[*instruction] {
            if let Value::Function(fid) = self.inserter.function.dfg[func] {
                return no_predicates.get(&fid).copied().unwrap_or_default();
            }
        }
        false
    }

    /// Prepare the arguments for the next block to consume.
    ///
    /// Panics if we already have something prepared.
    fn prepare_args(&mut self, args: Vec<ValueId>) {
        assert!(self.next_arguments.is_none(), "already prepared the arguments");
        assert!(!args.is_empty(), "only prepare args for non-empty parameter vector");
        self.next_arguments = Some(args);
    }

    /// Consume the arguments prepared by the previous block.
    ///
    /// Panics if there was nothing prepared.
    fn consume_args(&mut self) -> Vec<ValueId> {
        self.next_arguments.take().expect("there are no arguments prepared")
    }

    /// Inline all instructions from the given block into the target block, and track vector capacities.
    /// This is done by processing every instruction in the block and using the flattening context
    /// to push them in the target block.
    ///
    /// - `no_predicates` indicates which functions have no predicates and for which we disable the handling of side effects.
    pub(crate) fn inline_block(
        &mut self,
        block: BasicBlockId,
        no_predicates: &HashMap<FunctionId, bool>,
    ) {
        // We do not inline the target block into itself.
        // This is the case in the beginning for the entry block.
        if self.target_block == block {
            return;
        }

        // If the block has parameters, they should have been prepared by the last block.
        if !self.inserter.function.dfg.block_parameters(block).is_empty() {
            let arguments = self.consume_args();
            self.inserter.remember_block_params(block, &arguments);
        }

        // If this is not a separate variable, clippy gets confused and says the to_vec is
        // unnecessary, when removing it actually causes an aliasing/mutability error.
        let instructions = self.inserter.function.dfg[block].instructions().to_vec();
        for instruction in instructions {
            if self.is_call_to_no_predicate_function(no_predicates, &instruction) {
                // disable side effect for no_predicate functions
                let bool_type = NumericType::bool();
                let one = self.inserter.function.dfg.make_constant(FieldElement::one(), bool_type);
                self.insert_instruction_with_typevars(
                    Instruction::EnableSideEffectsIf { condition: one },
                    None,
                    CallStackId::root(),
                );
                self.push_instruction(instruction);
                self.insert_current_side_effects_enabled();
            } else {
                self.push_instruction(instruction);
            }
        }
    }

    /// Returns the vector of blocks that need to be processed after the given block,
    /// and prepare any arguments for the next-to-be-inlined block to consume.
    ///
    /// For a normal block, it would be its successor.
    ///
    /// For blocks related to a conditional statement, we ensure to process
    /// the 'then_branch', then the 'else_branch' (if it exists), and finally the exit block.
    ///
    /// The update of the context is done by the functions `if_start`, `then_stop` and `else_stop`
    /// which perform the business logic when entering a conditional statement, finishing the 'then_branch'
    /// and the 'else_branch', respectively.
    ///
    /// We know if a block is related to the conditional statement if is referenced by the `work_vector`.
    /// Indeed, the start blocks of the 'then_branch' and 'else_branch' are added to the `work_vector` when
    /// starting to process a conditional statement.
    pub(crate) fn handle_terminator(
        &mut self,
        block: BasicBlockId,
        work_vector: &WorkVector,
    ) -> Vec<BasicBlockId> {
        let terminator = self.inserter.function.dfg[block].unwrap_terminator().clone();
        match &terminator {
            TerminatorInstruction::JmpIf {
                condition,
                then_destination,
                else_destination,
                call_stack,
            } => {
                // The 'then' and 'else' blocks have no arguments, so we have nothing to prepare.
                self.if_start(condition, then_destination, else_destination, &block, *call_stack)
            }
            TerminatorInstruction::Jmp { destination, arguments, call_stack: _ } => {
                // If the destination is already on the work vector, it means it's an exit block in an if-then-else,
                // and was put there by `if_start` as the last to be processed out of [then, else, exit].
                if work_vector.contains(destination) {
                    // Since we enqueued [then, else, exit] after each other, if the next block on the work vector
                    // is the exit block, then this must be the else.
                    if work_vector.last().unwrap() == destination {
                        // The arguments for the exit block will be prepared here.
                        self.else_stop(&block);
                    } else {
                        // No need to prepare arguments: the eventual `else_stop` will look them up directly.
                        self.then_stop(&block);
                    }
                    // The destination was in the queue, no need to return anything.
                    vec![]
                } else {
                    // The destination is a normal block, not an exit block, so there is no argument merging involved,
                    // we can prepare any arguments for direct consumption.
                    if !arguments.is_empty() {
                        let arguments = vecmap(arguments, |value| self.inserter.resolve(*value));
                        self.prepare_args(arguments);
                    }
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
            TerminatorInstruction::Unreachable { .. } => {
                // The pass which introduces unreachable terminators must come after flattening, as it destroys the CFG.
                unreachable!("unexpected unreachable terminator in flattening")
            }
        }
    }

    /// Process a conditional statement by creating a `ConditionalContext`
    /// with information about the branch, and storing it in the dedicated stack.
    /// Local allocations are moved to the 'then_branch' of the `ConditionalContext`.
    /// Returns the blocks corresponding to the 'then_branch', 'else_branch',
    /// and exit block of the conditional statement, so that they will be processed in this order.
    fn if_start(
        &mut self,
        condition: &ValueId,
        then_destination: &BasicBlockId,
        else_destination: &BasicBlockId,
        if_entry: &BasicBlockId,
        call_stack: CallStackId,
    ) -> Vec<BasicBlockId> {
        let then_condition = self.inserter.resolve(*condition);

        // Take the current allocations: everything for the new branch is non-local.
        let branch = ConditionalBranch {
            condition: self.link_condition(then_condition),
            // To be filled in by `then_stop`.
            last_block: None,
        };
        let local_allocations = std::mem::take(&mut self.local_allocations);
        let cond_context = ConditionalContext {
            condition: then_condition,
            entry_block: *if_entry,
            then_branch: branch,
            // To be filled in by `then_stop`.
            else_branch: None,
            call_stack,
            predicated_values: HashMap::default(),
            local_allocations,
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

    /// Switch context to the 'else_branch':
    /// - Negates the condition for the 'else_branch' and set it in the `ConditionalContext`
    /// - Move the local allocations to the 'else_branch'
    /// - Reset the predicated values to their old mapping in the inserter
    /// - Issues the 'enable_side_effect' instruction
    fn then_stop(&mut self, block: &BasicBlockId) {
        assert_eq!(self.cfg.successors(*block).len(), 1);

        let mut cond_context = self.condition_stack.pop().unwrap();
        cond_context.then_branch.last_block = Some(*block);

        let condition_call_stack =
            self.inserter.function.dfg.get_value_call_stack_id(cond_context.condition);

        let else_condition = self.not_instruction(cond_context.condition, condition_call_stack);
        let else_condition = self.link_condition(else_condition);

        // Pass on the local allocations that came before the 'then_branch' to the 'else_branch'.
        let else_branch = ConditionalBranch { condition: else_condition, last_block: None };
        // All local allocations on the stopped 'then_branch' go out of scope.
        self.local_allocations.clear();
        cond_context.else_branch = Some(else_branch);
        self.reset_predicated_values(&mut cond_context);
        self.condition_stack.push(cond_context);

        self.insert_current_side_effects_enabled();
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

    /// Switch context the 'exit' block of a conditional statement:
    /// - Retrieves the local allocations from the Conditional Context
    /// - Reset the predicated values to their old mapping in the inserter
    /// - Issues the 'enable_side_effect' instruction
    /// - Joins the arguments from both branches
    fn else_stop(&mut self, block: &BasicBlockId) {
        assert_eq!(self.cfg.successors(*block).len(), 1);

        let mut cond_context = self.condition_stack.pop().unwrap();
        if cond_context.else_branch.is_none() {
            // `then_stop` has not been called, this means that the conditional statement has no else branch
            // so we simply do the `then_stop` now, sandwiched between pushing the context back on the stack,
            // then popping it again after `then_stop` is done popping and pushing.
            self.condition_stack.push(cond_context);
            self.then_stop(block);
            cond_context = self.condition_stack.pop().unwrap();
        }

        let mut else_branch = cond_context.else_branch.unwrap();
        self.local_allocations = std::mem::take(&mut cond_context.local_allocations);
        else_branch.last_block = Some(*block);
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
    }

    /// Inline the ending block of a branch, the point where all blocks from a jmpif instruction
    /// join back together. In particular this function must handle merging block arguments from
    /// all of the join point's predecessors, and it must handle any differing side effects from
    /// each branch.
    ///
    /// The merge of arguments is done by inserting an 'IfElse' instructions which returns
    /// the argument from the 'then_branch' or the 'else_branch' depending the the condition.
    ///
    /// The arguments are prepared for the destination to consume in the next immediate inlining.
    fn inline_branch_end(&mut self, destination: BasicBlockId, cond_context: ConditionalContext) {
        assert_eq!(self.cfg.predecessors(destination).len(), 2);

        // Look up and resolve the 'else' and 'then' arguments directly in their terminators,
        // rather than rely on argument passing in the context.
        let mut else_args = Vec::new();
        if cond_context.else_branch.is_some() {
            let last_else = cond_context.else_branch.clone().unwrap().last_block.unwrap();
            else_args = self.inserter.function.dfg[last_else].terminator_arguments().to_vec();
        }

        let last_then = cond_context.then_branch.last_block.unwrap();
        let then_args = self.inserter.function.dfg[last_then].terminator_arguments().to_vec();

        let params = self.inserter.function.dfg.block_parameters(destination);
        assert_eq!(params.len(), then_args.len());
        assert_eq!(params.len(), else_args.len());

        if params.is_empty() {
            return;
        }

        let args = vecmap(then_args.iter().zip(else_args), |(then_arg, else_arg)| {
            (self.inserter.resolve(*then_arg), self.inserter.resolve(else_arg))
        });
        let Some(else_branch) = cond_context.else_branch else {
            unreachable!("malformed branch");
        };
        let block = self.target_block;

        // Cannot include this in the previous vecmap since it requires exclusive access to self
        let args = vecmap(args, |(then_arg, else_arg)| {
            let instruction = Instruction::IfElse {
                then_condition: cond_context.then_branch.condition,
                then_value: then_arg,
                else_condition: else_branch.condition,
                else_value: else_arg,
            };
            let call_stack = cond_context.call_stack;
            self.inserter
                .function
                .dfg
                .insert_instruction_and_results(instruction, block, None, call_stack)
                .first()
        });

        self.prepare_args(args);
    }

    /// Map the value to its predicated value in the current conditional context, and store the previous mapping
    /// to the 'predicated_values' map if not already stored.
    fn predicate_value(&mut self, value: ValueId, predicated_value: ValueId) {
        let conditional_context = self.condition_stack.last_mut().unwrap();

        conditional_context
            .predicated_values
            .entry(value)
            .or_insert_with(|| self.inserter.resolve(value));

        self.inserter.map_value(value, predicated_value);
    }

    /// Restore the previous mapping of predicated values after a branch is finished.
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
    ) -> InsertInstructionResult<'_> {
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
    /// Note that each ValueId of the instruction will be mapped via `self.inserter.resolve`.
    /// As a result, the instruction that will be pushed will actually be a new instruction
    /// with a different InstructionId from the original. The results of the given instruction
    /// will also be mapped to the results of the new instruction.
    fn push_instruction(&mut self, id: InstructionId) {
        let (instruction, call_stack) = self.inserter.map_instruction(id);
        let instruction = self.handle_instruction_side_effects(instruction, call_stack);

        let instruction_is_allocate = matches!(&instruction, Instruction::Allocate);
        let results = self.inserter.push_instruction_value(
            instruction,
            id,
            self.target_block,
            call_stack,
            true,
        );

        // Remember an allocate was created local to this branch so that we do not try to merge store
        // values across branches for it later.
        if instruction_is_allocate {
            self.local_allocations.insert(results.first());
        }
    }

    /// If we are currently in a branch, we need to modify instructions that have side effects
    /// (e.g. constraints, stores, range checks) to ensure that the side effect is only applied
    /// if their branch is taken.
    ///
    /// For instance we multiply constrain instructions by the branch's condition (see optimization #1 in the module comment).
    fn handle_instruction_side_effects(
        &mut self,
        instruction: Instruction,
        call_stack: CallStackId,
    ) -> Instruction {
        let Some(condition) = self.get_last_condition() else { return instruction };

        match instruction {
            Instruction::Constrain(lhs, rhs, message) => {
                // Replace constraint `lhs == rhs` with `condition * lhs == condition * rhs`.
                let lhs = self.mul_by_condition(lhs, condition, call_stack);
                let rhs = self.mul_by_condition(rhs, condition, call_stack);
                Instruction::Constrain(lhs, rhs, message)
            }
            Instruction::ConstrainNotEqual(_, _, _) => {
                unreachable!("flattening cannot handle ConstrainNotEqual");
            }
            Instruction::Store { address, value } => {
                // If this store is to a reference that was allocated on this branch,
                // then we don't have to merge with anything else, we can ignore the condition.
                if self.local_allocations.contains(&address) {
                    Instruction::Store { address, value }
                } else {
                    // If the reference was allocated before this condition took effect, then we must only
                    // overwrite it if the condition is true.
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
                let predicate_value = self.mul_by_condition(value, casted_condition, call_stack);
                // Issue #8617: update the value to be the predicated value.
                // This ensures that the value has the correct bit size in all cases.
                self.predicate_value(value, predicate_value);
                Instruction::RangeCheck { value: predicate_value, max_bit_size, assert_message }
            }
            Instruction::Call { func, arguments } => {
                let arguments =
                    self.handle_call_side_effects(condition, func, arguments, call_stack);
                Instruction::Call { func, arguments }
            }
            // The following instructions don't need their arguments nullified:
            Instruction::Binary(_)
            | Instruction::Cast(_, _)
            | Instruction::Not(_)
            | Instruction::Truncate { .. }
            | Instruction::Allocate
            | Instruction::Load { .. }
            | Instruction::EnableSideEffectsIf { .. }
            | Instruction::ArrayGet { .. }
            | Instruction::ArraySet { .. }
            | Instruction::IncrementRc { .. }
            | Instruction::DecrementRc { .. }
            | Instruction::IfElse { .. }
            | Instruction::MakeArray { .. }
            | Instruction::Noop => instruction,
        }
    }

    /// Weave the `condition` into call arguments, returning the modified arguments.
    fn handle_call_side_effects(
        &mut self,
        condition: ValueId,
        func: ValueId,
        arguments: Vec<ValueId>,
        call_stack: CallStackId,
    ) -> Vec<ValueId> {
        match self.inserter.function.dfg[func] {
            Value::Intrinsic(intrinsic) => {
                self.handle_intrinsic_side_effects(condition, intrinsic, arguments, call_stack)
            }
            Value::Function(_) | Value::ForeignFunction(_) => arguments,
            Value::Instruction { .. }
            | Value::Param { .. }
            | Value::NumericConstant { .. }
            | Value::Global(_) => unreachable!("unexpected function value"),
        }
    }

    /// Weave the `condition` into intrinsic call arguments, returning the modified arguments.
    fn handle_intrinsic_side_effects(
        &mut self,
        condition: ValueId,
        intrinsic: Intrinsic,
        mut arguments: Vec<ValueId>,
        call_stack: CallStackId,
    ) -> Vec<ValueId> {
        match intrinsic {
            Intrinsic::ToBits(_) | Intrinsic::ToRadix(_) => {
                let field = arguments[0];
                let casted_condition =
                    self.cast_condition_to_value_type(condition, field, call_stack);
                let field = self.mul_by_condition(field, casted_condition, call_stack);

                arguments[0] = field;

                arguments
            }
            Intrinsic::BlackBox(blackbox) => {
                self.handle_blackbox_side_effects(condition, blackbox, arguments, call_stack)
            }
            // The following intrinsics may have side effects, but we don't deal with them by
            // multiplying their arguments with the condition.
            Intrinsic::ArrayLen
            | Intrinsic::ArrayAsStrUnchecked
            | Intrinsic::AsVector
            | Intrinsic::AssertConstant
            | Intrinsic::StaticAssert
            | Intrinsic::VectorPushBack
            | Intrinsic::VectorPushFront
            | Intrinsic::VectorPopBack
            | Intrinsic::VectorPopFront
            | Intrinsic::VectorInsert
            | Intrinsic::VectorRemove
            | Intrinsic::ApplyRangeConstraint
            | Intrinsic::StrAsBytes
            | Intrinsic::Hint(_)
            | Intrinsic::AsWitness
            | Intrinsic::IsUnconstrained
            | Intrinsic::DerivePedersenGenerators
            | Intrinsic::FieldLessThan
            | Intrinsic::ArrayRefCount
            | Intrinsic::VectorRefCount => arguments,
        }
    }

    /// Weave the `condition` into blackbox call arguments, returning the modified arguments.
    fn handle_blackbox_side_effects(
        &mut self,
        condition: ValueId,
        blackbox: BlackBoxFunc,
        mut arguments: Vec<ValueId>,
        call_stack: CallStackId,
    ) -> Vec<ValueId> {
        match blackbox {
            BlackBoxFunc::EmbeddedCurveAdd => {
                arguments[6] = self.mul_by_condition(arguments[6], condition, call_stack);
                arguments
            }

            BlackBoxFunc::MultiScalarMul => {
                arguments[2] = self.mul_by_condition(arguments[2], condition, call_stack);
                arguments
            }

            BlackBoxFunc::EcdsaSecp256k1 | BlackBoxFunc::EcdsaSecp256r1 => {
                arguments[4] = self.mul_by_condition(arguments[4], condition, call_stack);
                arguments
            }

            // The predicate is injected in ACIRgen so no modification is needed here.
            BlackBoxFunc::RecursiveAggregation => arguments,

            // These functions will always be satisfiable no matter the input so no modification is needed.
            BlackBoxFunc::AND
            | BlackBoxFunc::XOR
            | BlackBoxFunc::AES128Encrypt
            | BlackBoxFunc::Blake2s
            | BlackBoxFunc::Blake3
            | BlackBoxFunc::Keccakf1600
            | BlackBoxFunc::Poseidon2Permutation
            | BlackBoxFunc::Sha256Compression => arguments,

            BlackBoxFunc::RANGE => {
                unreachable!("RANGE should have been converted into `Instruction::RangeCheck`")
            }
        }
    }

    /// 'Cast' the 'condition' to 'value' type
    ///
    /// This is needed because we need to multiply the condition with several values
    /// in order to 'nullify' side-effects when the 'condition' is false (in 'handle_instruction_side_effects' function).
    ///
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
    fn not_merge_with_previous_store_if_local_before_branch() {
        // The SSA is for the following graph:
        // * We allocate a reference under the v0 condition in b1
        // * We branch off from b1 under the v1 condition
        // * We store to the reference in the exit block b5:
        //   it should not involve merging store value because it's local to the b1-b6 branch.
        //         b0
        //       ↙   ↘
        //     b1     b2  (allocate and store)
        //   ↙  ↘     |
        // b3    b4   |
        //   ↘  ↙     |
        //    b5      |  (store)
        //    |       |
        //    b6      |  (load)
        //      ↘   ↙
        //       b7
        let src = "
            acir(inline) fn main f0 {
              b0(v0: u1, v1: u1):
                jmpif v0 then: b1, else: b2
              b1():
                v2 = allocate -> &mut Field
                store Field 1 at v2
                jmpif v1 then: b3, else: b4
              b2():
                jmp b7(Field 2)
              b3():
                jmp b5()
              b4():
                jmp b5()
              b5():
                store Field 5 at v2
                jmp b6()
              b6():
                v3 = load v2 -> Field
                jmp b7(v3)
              b7(v4: Field):
                return v4
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.flatten_cfg();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u1):
            enable_side_effects v0
            v2 = allocate -> &mut Field
            store Field 1 at v2
            v4 = unchecked_mul v0, v1
            enable_side_effects v4
            v5 = not v1
            v6 = unchecked_mul v0, v5
            enable_side_effects v0
            store Field 5 at v2
            v8 = load v2 -> Field
            v9 = not v0
            enable_side_effects u1 1
            v11 = cast v0 as Field
            v12 = cast v9 as Field
            v13 = mul v11, v8
            v15 = mul v12, Field 2
            v16 = add v13, v15
            return v16
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
    fn nested_branch_args() {
        // Here we build some SSA with control flow given by the following graph.
        //
        //
        //         b0
        //         ↓
        //         b1
        //       ↙   ↘
        //     b2     b3
        //     ↓      |
        //     b4     |
        //   ↙  ↘     |
        // b5    b6   |
        //   ↘  ↙     ↓
        //    b7      b8
        //      ↘   ↙
        //       b9

        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u1):
            jmp b1(u32 0)
          b1(v2: u32):
            jmpif v0 then: b2, else: b3
          b2():
            jmp b4(u32 2)
          b4(v3: u32):
            jmpif v1 then: b5, else: b6
          b5():
            jmp b7(u32 5)
          b7(v4: u32):
            jmp b9(v4)
          b9(v5: u32):
            return v5
          b6():
            jmp b7(u32 6)
          b3():
            jmp b8(u32 3)
          b8(v6: u32):
            jmp b9(v6)
        }";

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.flatten_cfg();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u1):
            enable_side_effects v0
            v2 = unchecked_mul v0, v1
            enable_side_effects v2
            v3 = not v1
            v4 = unchecked_mul v0, v3
            enable_side_effects v0
            v5 = cast v2 as u32
            v6 = cast v4 as u32
            v8 = unchecked_mul v5, u32 5
            v10 = unchecked_mul v6, u32 6
            v11 = unchecked_add v8, v10
            v12 = not v0
            enable_side_effects u1 1
            v14 = cast v0 as u32
            v15 = cast v12 as u32
            v16 = unchecked_mul v14, v11
            v18 = unchecked_mul v15, u32 3
            v19 = unchecked_add v16, v18
            return v19
        }
        ");
        // v19 = v16 + v18
        //     = v14 * v11 + v15 * 3 =
        //     = v0 * (v8 + v10) + !v0 * 3
        //     = v0 * (v5 * 5 + v6 * 6) + !v0 * 3
        //     = v0 * (v0 * v1 * 5 + v0 * !v1 * 6) + !v0 * 3
        //     = v0 * v1 * 5 + v0 * !v1 * 6 + !v0 * 3
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
            v6 = allocate -> &mut u8
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
            v6 = allocate -> &mut u8
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

        let ssa = ssa.flatten_cfg().mem2reg().fold_constants(1);

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
            v1 = allocate -> &mut Field
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

        let ssa = ssa.flatten_cfg().mem2reg().fold_constants(1);

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1):
            v1 = allocate -> &mut Field
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
            .fold_constants(1)
            .dead_instruction_elimination();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1, v1: u1):
            enable_side_effects v0
            enable_side_effects u1 1
            v3 = cast v0 as Field
            enable_side_effects v0
            enable_side_effects u1 1
            v5 = mul v3, Field 2
            v6 = make_array [v5] : [Field; 1]
            enable_side_effects u1 1
            return v6
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
            v5 = unchecked_mul v3, v4
            range_check v5 to 16 bits
            v6 = not v0
            enable_side_effects v6
            v8 = add u32 3, v3
            enable_side_effects u1 1
            v10 = cast v0 as u32
            v11 = cast v6 as u32
            v12 = unchecked_mul v10, v3
            v13 = unchecked_mul v11, v8
            v14 = unchecked_add v12, v13
            return v14
        }
        ");
    }

    #[test]
    fn simplifies_during_insertion() {
        // `if v0 { false } else { true }`
        let src = "
            acir(inline) fn main f0 {
              b0(v0: u1):
                jmpif v0 then: b1, else: b2
              b1():
                jmp b3(u1 0)
              b2():
                jmp b3(u1 1)
              b3(v1: u1):
                return v1
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        assert_eq!(ssa.main().reachable_blocks().len(), 4);

        let ssa = ssa.flatten_cfg();
        // All the casting an merging should be simplified out and reduced to: `not v0`
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u1):
            enable_side_effects v0
            v1 = not v0
            enable_side_effects u1 1
            return v1
        }
        ");
    }
}
