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
//! of `EnableSideEffect` instructions. Each time the flattening pass enters and exits a branch of
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
//! an equality with c:
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
use fxhash::FxHashMap as HashMap;
use std::collections::{BTreeMap, HashSet};

use acvm::{acir::AcirField, acir::BlackBoxFunc, FieldElement};
use iter_extended::vecmap;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        cfg::ControlFlowGraph,
        dfg::{CallStack, InsertInstructionResult},
        function::Function,
        function_inserter::FunctionInserter,
        instruction::{BinaryOp, Instruction, InstructionId, Intrinsic, TerminatorInstruction},
        types::Type,
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
        for function in self.functions.values_mut() {
            flatten_function_cfg(function);
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

    /// Maps an address to the old and new value of the element at that address
    /// These only hold stores for one block at a time and is cleared
    /// between inlining of branches.
    store_values: HashMap<ValueId, Store>,

    /// Stores all allocations local to the current branch.
    /// Since these branches are local to the current branch (ie. only defined within one branch of
    /// an if expression), they should not be merged with their previous value or stored value in
    /// the other branch since there is no such value. The ValueId here is that which is returned
    /// by the allocate instruction.
    local_allocations: HashSet<ValueId>,

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
}

#[derive(Clone)]
pub(crate) struct Store {
    old_value: ValueId,
    new_value: ValueId,
}

#[derive(Clone)]
struct ConditionalBranch {
    // Contains the last processed block during the processing of the branch.
    last_block: BasicBlockId,
    // The unresolved condition of the branch
    old_condition: ValueId,
    // The condition of the branch
    condition: ValueId,
    // The store values accumulated when processing the branch
    store_values: HashMap<ValueId, Store>,
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
}

fn flatten_function_cfg(function: &mut Function) {
    // This pass may run forever on a brillig function.
    // Analyze will check if the predecessors have been processed and push the block to the back of
    // the queue. This loops forever if there are still any loops present in the program.
    if let crate::ssa::ir::function::RuntimeType::Brillig = function.runtime() {
        return;
    }
    let cfg = ControlFlowGraph::with_function(function);
    let branch_ends = branch_analysis::find_branch_ends(function, &cfg);

    let mut context = Context {
        inserter: FunctionInserter::new(function),
        cfg,
        store_values: HashMap::default(),
        local_allocations: HashSet::new(),
        branch_ends,
        slice_sizes: HashMap::default(),
        condition_stack: Vec::new(),
        arguments_stack: Vec::new(),
    };
    context.flatten();
}

impl<'f> Context<'f> {
    fn flatten(&mut self) {
        // Flatten the CFG by inlining all instructions from the queued blocks
        // until all blocks have been flattened.
        // We follow the terminator of each block to determine which blocks to
        // process next
        let mut queue = vec![self.inserter.function.entry_block()];
        while let Some(block) = queue.pop() {
            self.inline_block(block);
            let to_process = self.handle_terminator(block, &queue);
            for incoming_block in to_process {
                if !queue.contains(&incoming_block) {
                    queue.push(incoming_block);
                }
            }
        }
    }

    /// Returns the updated condition so that
    /// it is 'AND-ed' with the previous condition (if any)
    fn link_condition(&mut self, condition: ValueId) -> ValueId {
        // Retrieve the previous condition
        if let Some(context) = self.condition_stack.last() {
            let previous_branch = context.else_branch.as_ref().unwrap_or(&context.then_branch);
            let and = Instruction::binary(BinaryOp::And, previous_branch.condition, condition);
            self.insert_instruction(and, CallStack::new())
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

    // Inline all instructions from the given block into the entry block, and track slice capacities
    fn inline_block(&mut self, block: BasicBlockId) {
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
            self.push_instruction(*instruction);
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
            TerminatorInstruction::JmpIf { condition, then_destination, else_destination } => {
                self.arguments_stack.push(vec![]);
                self.if_start(condition, then_destination, else_destination, &block)
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
                let call_stack = call_stack.clone();
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
    ) -> Vec<BasicBlockId> {
        // manage conditions
        let old_condition = *condition;
        let then_condition = self.inserter.resolve(old_condition);

        let one = FieldElement::one();
        let old_stores = std::mem::take(&mut self.store_values);
        let old_allocations = std::mem::take(&mut self.local_allocations);
        let branch = ConditionalBranch {
            old_condition,
            condition: self.link_condition(then_condition),
            store_values: old_stores,
            local_allocations: old_allocations,
            last_block: *then_destination,
        };
        let cond_context = ConditionalContext {
            condition: then_condition,
            entry_block: *if_entry,
            then_branch: branch,
            else_branch: None,
        };
        self.condition_stack.push(cond_context);
        self.insert_current_side_effects_enabled();
        // Optimization: within the then branch we know the condition to be true, so replace
        // any references of it within this branch with true. Likewise, do the same with false
        // with the else branch. We must be careful not to replace the condition if it is a
        // known constant, otherwise we can end up setting 1 = 0 or vice-versa.
        if self.inserter.function.dfg.get_numeric_constant(old_condition).is_none() {
            let known_value = self.inserter.function.dfg.make_constant(one, Type::bool());

            self.inserter.map_value(old_condition, known_value);
        }
        vec![self.branch_ends[if_entry], *else_destination, *then_destination]
    }

    /// Switch context to the 'else-branch'
    fn then_stop(&mut self, block: &BasicBlockId) -> Vec<BasicBlockId> {
        let mut cond_context = self.condition_stack.pop().unwrap();
        cond_context.then_branch.last_block = *block;

        let else_condition =
            self.insert_instruction(Instruction::Not(cond_context.condition), CallStack::new());
        let else_condition = self.link_condition(else_condition);

        let zero = FieldElement::zero();
        // Make sure the else branch sees the previous values of each store
        // rather than any values created in the 'then' branch.
        let old_stores = std::mem::take(&mut cond_context.then_branch.store_values);
        cond_context.then_branch.store_values = std::mem::take(&mut self.store_values);
        self.undo_stores_in_then_branch(&cond_context.then_branch.store_values);

        let old_allocations = std::mem::take(&mut self.local_allocations);
        let else_branch = ConditionalBranch {
            old_condition: cond_context.then_branch.old_condition,
            condition: else_condition,
            store_values: old_stores,
            local_allocations: old_allocations,
            last_block: *block,
        };
        let old_condition = else_branch.old_condition;
        cond_context.then_branch.local_allocations.clear();
        cond_context.else_branch = Some(else_branch);
        self.condition_stack.push(cond_context);

        self.insert_current_side_effects_enabled();
        // Optimization: within the then branch we know the condition to be true, so replace
        // any references of it within this branch with true. Likewise, do the same with false
        // with the else branch. We must be careful not to replace the condition if it is a
        // known constant, otherwise we can end up setting 1 = 0 or vice-versa.
        if self.inserter.function.dfg.get_numeric_constant(old_condition).is_none() {
            let known_value = self.inserter.function.dfg.make_constant(zero, Type::bool());

            self.inserter.map_value(old_condition, known_value);
        }
        assert_eq!(self.cfg.successors(*block).len(), 1);
        vec![self.cfg.successors(*block).next().unwrap()]
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
        let stores_in_branch = std::mem::replace(&mut self.store_values, else_branch.store_values);
        self.local_allocations = std::mem::take(&mut else_branch.local_allocations);
        else_branch.last_block = *block;
        else_branch.store_values = stores_in_branch;
        cond_context.else_branch = Some(else_branch);

        // We must remember to reset whether side effects are enabled when both branches
        // end, in addition to resetting the value of old_condition since it is set to
        // known to be true/false within the then/else branch respectively.
        self.insert_current_side_effects_enabled();

        // We must map back to `then_condition` here. Mapping `old_condition` to itself would
        // lose any previous mappings.
        self.inserter
            .map_value(cond_context.then_branch.old_condition, cond_context.then_branch.condition);

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

        let block = self.inserter.function.entry_block();

        // Cannot include this in the previous vecmap since it requires exclusive access to self
        let args = vecmap(args, |(then_arg, else_arg)| {
            let instruction = Instruction::IfElse {
                then_condition: cond_context.then_branch.condition,
                then_value: then_arg,
                else_condition: cond_context.else_branch.as_ref().unwrap().condition,
                else_value: else_arg,
            };
            self.inserter
                .function
                .dfg
                .insert_instruction_and_results(instruction, block, None, CallStack::new())
                .first()
        });

        self.merge_stores(cond_context.then_branch, cond_context.else_branch);
        self.arguments_stack.pop();
        self.arguments_stack.pop();
        self.arguments_stack.push(args);
        destination
    }

    /// Insert a new instruction into the function's entry block.
    /// Unlike push_instruction, this function will not map any ValueIds.
    /// within the given instruction, nor will it modify self.values in any way.
    fn insert_instruction(&mut self, instruction: Instruction, call_stack: CallStack) -> ValueId {
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
    ) -> InsertInstructionResult {
        let block = self.inserter.function.entry_block();
        self.inserter.function.dfg.insert_instruction_and_results(
            instruction,
            block,
            ctrl_typevars,
            CallStack::new(),
        )
    }

    /// Checks the branch condition on the top of the stack and uses it to build and insert an
    /// `EnableSideEffects` instruction into the entry block.
    ///
    /// If the stack is empty, a "true" u1 constant is taken to be the active condition. This is
    /// necessary for re-enabling side-effects when re-emerging to a branch depth of 0.
    fn insert_current_side_effects_enabled(&mut self) {
        let condition = match self.get_last_condition() {
            Some(cond) => cond,
            None => {
                self.inserter.function.dfg.make_constant(FieldElement::one(), Type::unsigned(1))
            }
        };
        let enable_side_effects = Instruction::EnableSideEffects { condition };
        self.insert_instruction_with_typevars(enable_side_effects, None);
    }

    /// Merge any store instructions found in each branch.
    ///
    /// This function relies on the 'then' branch being merged before the 'else' branch of a jmpif
    /// instruction. If this ordering is changed, the ordering that store values are merged within
    /// this function also needs to be changed to reflect that.
    fn merge_stores(
        &mut self,
        then_branch: ConditionalBranch,
        else_branch: Option<ConditionalBranch>,
    ) {
        // Address -> (then_value, else_value, value_before_the_if)
        let mut new_map = BTreeMap::new();

        for (address, store) in then_branch.store_values {
            new_map.insert(address, (store.new_value, store.old_value, store.old_value));
        }

        if else_branch.is_some() {
            for (address, store) in else_branch.clone().unwrap().store_values {
                if let Some(entry) = new_map.get_mut(&address) {
                    entry.1 = store.new_value;
                } else {
                    new_map.insert(address, (store.old_value, store.new_value, store.old_value));
                }
            }
        }

        let then_condition = then_branch.condition;
        let else_condition = if let Some(branch) = else_branch {
            branch.condition
        } else {
            self.inserter.function.dfg.make_constant(FieldElement::zero(), Type::bool())
        };
        let block = self.inserter.function.entry_block();

        // Merging must occur in a separate loop as we cannot borrow `self` as mutable while `value_merger` does
        let mut new_values = HashMap::default();
        for (address, (then_case, else_case, _)) in &new_map {
            let instruction = Instruction::IfElse {
                then_condition,
                then_value: *then_case,
                else_condition,
                else_value: *else_case,
            };
            let value = self
                .inserter
                .function
                .dfg
                .insert_instruction_and_results(instruction, block, None, CallStack::new())
                .first();

            new_values.insert(address, value);
        }

        // Replace stores with new merged values
        for (address, (_, _, old_value)) in &new_map {
            let value = new_values[address];
            let address = *address;
            self.insert_instruction_with_typevars(Instruction::Store { address, value }, None);

            if let Some(store) = self.store_values.get_mut(&address) {
                store.new_value = value;
            } else {
                self.store_values
                    .insert(address, Store { old_value: *old_value, new_value: value });
            }
        }
    }

    fn remember_store(&mut self, address: ValueId, new_value: ValueId) {
        if !self.local_allocations.contains(&address) {
            if let Some(store_value) = self.store_values.get_mut(&address) {
                store_value.new_value = new_value;
            } else {
                let load = Instruction::Load { address };

                let load_type = Some(vec![self.inserter.function.dfg.type_of_value(new_value)]);
                let old_value =
                    self.insert_instruction_with_typevars(load.clone(), load_type).first();

                self.store_values.insert(address, Store { old_value, new_value });
            }
        }
    }

    /// Push the given instruction to the end of the entry block of the current function.
    ///
    /// Note that each ValueId of the instruction will be mapped via self.inserter.resolve.
    /// As a result, the instruction that will be pushed will actually be a new instruction
    /// with a different InstructionId from the original. The results of the given instruction
    /// will also be mapped to the results of the new instruction.
    fn push_instruction(&mut self, id: InstructionId) -> Vec<ValueId> {
        let (instruction, call_stack) = self.inserter.map_instruction(id);
        let instruction = self.handle_instruction_side_effects(instruction, call_stack.clone());
        let is_allocate = matches!(instruction, Instruction::Allocate);

        let entry = self.inserter.function.entry_block();
        let results = self.inserter.push_instruction_value(instruction, id, entry, call_stack);

        // Remember an allocate was created local to this branch so that we do not try to merge store
        // values across branches for it later.
        if is_allocate {
            self.local_allocations.insert(results.first());
        }

        results.results().into_owned()
    }

    /// If we are currently in a branch, we need to modify constrain instructions
    /// to multiply them by the branch's condition (see optimization #1 in the module comment).
    fn handle_instruction_side_effects(
        &mut self,
        instruction: Instruction,
        call_stack: CallStack,
    ) -> Instruction {
        if let Some(condition) = self.get_last_condition() {
            match instruction {
                Instruction::Constrain(lhs, rhs, message) => {
                    // Replace constraint `lhs == rhs` with `condition * lhs == condition * rhs`.

                    // Condition needs to be cast to argument type in order to multiply them together.
                    let argument_type = self.inserter.function.dfg.type_of_value(lhs);
                    // Sanity check that we're not constraining non-primitive types
                    assert!(matches!(argument_type, Type::Numeric(_)));

                    let casted_condition = self.insert_instruction(
                        Instruction::Cast(condition, argument_type),
                        call_stack.clone(),
                    );

                    let lhs = self.insert_instruction(
                        Instruction::binary(BinaryOp::Mul, lhs, casted_condition),
                        call_stack.clone(),
                    );
                    let rhs = self.insert_instruction(
                        Instruction::binary(BinaryOp::Mul, rhs, casted_condition),
                        call_stack,
                    );

                    Instruction::Constrain(lhs, rhs, message)
                }
                Instruction::Store { address, value } => {
                    self.remember_store(address, value);
                    Instruction::Store { address, value }
                }
                Instruction::RangeCheck { value, max_bit_size, assert_message } => {
                    // Replace value with `value * predicate` to zero out value when predicate is inactive.

                    // Condition needs to be cast to argument type in order to multiply them together.
                    let argument_type = self.inserter.function.dfg.type_of_value(value);
                    let casted_condition = self.insert_instruction(
                        Instruction::Cast(condition, argument_type),
                        call_stack.clone(),
                    );

                    let value = self.insert_instruction(
                        Instruction::binary(BinaryOp::Mul, value, casted_condition),
                        call_stack.clone(),
                    );
                    Instruction::RangeCheck { value, max_bit_size, assert_message }
                }
                Instruction::Call { func, mut arguments } => match self.inserter.function.dfg[func]
                {
                    Value::Intrinsic(Intrinsic::ToBits(_) | Intrinsic::ToRadix(_)) => {
                        let field = arguments[0];
                        let argument_type = self.inserter.function.dfg.type_of_value(field);

                        let casted_condition = self.insert_instruction(
                            Instruction::Cast(condition, argument_type),
                            call_stack.clone(),
                        );
                        let field = self.insert_instruction(
                            Instruction::binary(BinaryOp::Mul, field, casted_condition),
                            call_stack.clone(),
                        );

                        arguments[0] = field;

                        Instruction::Call { func, arguments }
                    }
                    //Issue #5045: We set curve points to infinity if condition is false
                    Value::Intrinsic(Intrinsic::BlackBox(BlackBoxFunc::EmbeddedCurveAdd)) => {
                        arguments[2] = self.var_or_one(arguments[2], condition, call_stack.clone());
                        arguments[5] = self.var_or_one(arguments[5], condition, call_stack.clone());

                        Instruction::Call { func, arguments }
                    }
                    Value::Intrinsic(Intrinsic::BlackBox(BlackBoxFunc::MultiScalarMul)) => {
                        let mut array_with_predicate = im::Vector::new();
                        let array_typ;
                        if let Value::Array { array, typ } =
                            &self.inserter.function.dfg[arguments[0]]
                        {
                            array_typ = typ.clone();
                            for (i, value) in array.clone().iter().enumerate() {
                                if i % 3 == 2 {
                                    array_with_predicate.push_back(self.var_or_one(
                                        *value,
                                        condition,
                                        call_stack.clone(),
                                    ));
                                } else {
                                    array_with_predicate.push_back(*value);
                                }
                            }
                        } else {
                            unreachable!();
                        }
                        arguments[0] =
                            self.inserter.function.dfg.make_array(array_with_predicate, array_typ);
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

    // Computes: if condition { var } else { 1 }
    fn var_or_one(&mut self, var: ValueId, condition: ValueId, call_stack: CallStack) -> ValueId {
        let field = self.insert_instruction(
            Instruction::binary(BinaryOp::Mul, var, condition),
            call_stack.clone(),
        );
        let not_condition =
            self.insert_instruction(Instruction::Not(condition), call_stack.clone());
        self.insert_instruction(
            Instruction::binary(BinaryOp::Add, field, not_condition),
            call_stack,
        )
    }

    fn undo_stores_in_then_branch(&mut self, store_values: &HashMap<ValueId, Store>) {
        for (address, store) in store_values {
            let address = *address;
            let value = store.old_value;
            self.insert_instruction_with_typevars(Instruction::Store { address, value }, None);
        }
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use acvm::acir::AcirField;

    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            dfg::DataFlowGraph,
            function::Function,
            instruction::{BinaryOp, Instruction, Intrinsic, TerminatorInstruction},
            map::Id,
            types::Type,
            value::{Value, ValueId},
        },
    };

    #[test]
    fn basic_jmpif() {
        // fn main f0 {
        //   b0(v0: b1):
        //     jmpif v0, then: b1, else: b2
        //   b1():
        //     jmp b3(Field 3)
        //   b2():
        //     jmp b3(Field 4)
        //   b3(v1: Field):
        //     return v1
        // }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let b1 = builder.insert_block();
        let b2 = builder.insert_block();
        let b3 = builder.insert_block();

        let v0 = builder.add_parameter(Type::bool());
        let v1 = builder.add_block_parameter(b3, Type::field());

        let three = builder.field_constant(3u128);
        let four = builder.field_constant(4u128);

        builder.terminate_with_jmpif(v0, b1, b2);

        builder.switch_to_block(b1);
        builder.terminate_with_jmp(b3, vec![three]);

        builder.switch_to_block(b2);
        builder.terminate_with_jmp(b3, vec![four]);

        builder.switch_to_block(b3);
        builder.terminate_with_return(vec![v1]);

        let ssa = builder.finish();
        assert_eq!(ssa.main().reachable_blocks().len(), 4);

        // Expected output:
        // fn main f0 {
        //   b0(v0: u1):
        //     enable_side_effects v0
        //     v5 = not v0
        //     enable_side_effects v5
        //     enable_side_effects u1 1
        //     v7 = mul v0, Field 3
        //     v8 = mul v5, Field 4
        //     v9 = add v7, v8
        //     return v9
        // }
        let ssa = ssa.flatten_cfg();
        assert_eq!(ssa.main().reachable_blocks().len(), 1);
    }

    #[test]
    fn modify_constrain() {
        // fn main f0 {
        //   b0(v0: u1, v1: u1):
        //     jmpif v0, then: b1, else: b2
        //   b1():
        //     constrain v1
        //     jmp b2()
        //   b2():
        //     return
        // }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let b1 = builder.insert_block();
        let b2 = builder.insert_block();

        let v0 = builder.add_parameter(Type::bool());
        let v1 = builder.add_parameter(Type::bool());
        let v_true = builder.numeric_constant(true, Type::bool());

        builder.terminate_with_jmpif(v0, b1, b2);

        builder.switch_to_block(b1);
        builder.insert_constrain(v1, v_true, None);
        builder.terminate_with_jmp(b2, vec![]);

        builder.switch_to_block(b2);
        builder.terminate_with_return(vec![]);

        let ssa = builder.finish();
        assert_eq!(ssa.main().reachable_blocks().len(), 3);

        // Expected output:
        // fn main f0 {
        //   b0(v0: u1, v1: u1):
        //     enable_side_effects v0
        //     v3 = mul v1, v0
        //     v4 = eq v3, v0
        //     constrain v4
        //     v5 = not v0
        //     enable_side_effects v5
        //     enable_side_effects u1 1
        //     return
        // }
        let ssa = ssa.flatten_cfg();
        assert_eq!(ssa.main().reachable_blocks().len(), 1);
    }

    #[test]
    fn merge_stores() {
        // fn main f0 {
        //   b0(v0: u1, v1: &mut Field):
        //     jmpif v0, then: b1, else: b2
        //   b1():
        //     store v1, Field 5
        //     jmp b2()
        //   b2():
        //     return
        // }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let b1 = builder.insert_block();
        let b2 = builder.insert_block();

        let v0 = builder.add_parameter(Type::bool());
        let v1 = builder.add_parameter(Type::Reference(Rc::new(Type::field())));

        builder.terminate_with_jmpif(v0, b1, b2);

        builder.switch_to_block(b1);
        let five = builder.field_constant(5u128);
        builder.insert_store(v1, five);
        builder.terminate_with_jmp(b2, vec![]);

        builder.switch_to_block(b2);
        builder.terminate_with_return(vec![]);

        let ssa = builder.finish();

        // Expected output:
        // fn main f0 {
        //   b0(v0: u1, v1: reference):
        //     enable_side_effects v0
        //     v4 = load v1
        //     store Field 5 at v1
        //     v5 = not v0
        //     store v4 at v1
        //     enable_side_effects u1 1
        //     v6 = cast v0 as Field
        //     v7 = cast v5 as Field
        //     v8 = mul v6, Field 5
        //     v9 = mul v7, v4
        //     v10 = add v8, v9
        //     store v10 at v1
        //     return
        // }
        let ssa = ssa.flatten_cfg();
        let main = ssa.main();

        assert_eq!(main.reachable_blocks().len(), 1);

        let store_count = count_instruction(main, |ins| matches!(ins, Instruction::Store { .. }));
        assert_eq!(store_count, 3);
    }

    #[test]
    fn merge_stores_with_else_block() {
        // fn main f0 {
        //   b0(v0: u1, v1: ref):
        //     jmpif v0, then: b1, else: b2
        //   b1():
        //     store Field 5 in v1
        //     jmp b3()
        //   b2():
        //     store Field 6 in v1
        //     jmp b3()
        //   b3():
        //     return
        // }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let b1 = builder.insert_block();
        let b2 = builder.insert_block();
        let b3 = builder.insert_block();

        let v0 = builder.add_parameter(Type::bool());
        let v1 = builder.add_parameter(Type::Reference(Rc::new(Type::field())));

        builder.terminate_with_jmpif(v0, b1, b2);

        builder.switch_to_block(b1);
        let five = builder.field_constant(5u128);
        builder.insert_store(v1, five);
        builder.terminate_with_jmp(b3, vec![]);

        builder.switch_to_block(b2);
        let six = builder.field_constant(6u128);
        builder.insert_store(v1, six);
        builder.terminate_with_jmp(b3, vec![]);

        builder.switch_to_block(b3);
        builder.terminate_with_return(vec![]);

        let ssa = builder.finish();

        // Expected output:
        // fn main f0 {
        //   b0(v0: u1, v1: reference):
        //     enable_side_effects v0
        //     v5 = load v1
        //     store Field 5 at v1
        //     v6 = not v0
        //     store v5 at v1
        //     enable_side_effects v6
        //     v8 = load v1
        //     store Field 6 at v1
        //     enable_side_effects u1 1
        //     v9 = cast v0 as Field
        //     v10 = cast v6 as Field
        //     v11 = mul v9, Field 5
        //     v12 = mul v10, Field 6
        //     v13 = add v11, v12
        //     store v13 at v1
        //     return
        // }
        let ssa = ssa.flatten_cfg();
        let main = ssa.main();
        assert_eq!(main.reachable_blocks().len(), 1);

        let store_count = count_instruction(main, |ins| matches!(ins, Instruction::Store { .. }));
        assert_eq!(store_count, 4);
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
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let b1 = builder.insert_block();
        let b2 = builder.insert_block();
        let b3 = builder.insert_block();
        let b4 = builder.insert_block();
        let b5 = builder.insert_block();
        let b6 = builder.insert_block();
        let b7 = builder.insert_block();
        let b8 = builder.insert_block();
        let b9 = builder.insert_block();

        let c1 = builder.add_parameter(Type::bool());
        let c4 = builder.add_parameter(Type::bool());

        let r1 = builder.insert_allocate(Type::field());

        let store_value = |builder: &mut FunctionBuilder, value: u128| {
            let value = builder.field_constant(value);
            builder.insert_store(r1, value);
        };

        let test_function = Id::test_new(1);

        let call_test_function = |builder: &mut FunctionBuilder, block: u128| {
            let block = builder.field_constant(block);
            let load = builder.insert_load(r1, Type::field());
            builder.insert_call(test_function, vec![block, load], Vec::new());
        };

        let switch_store_and_test_function =
            |builder: &mut FunctionBuilder, block, block_number: u128| {
                builder.switch_to_block(block);
                store_value(builder, block_number);
                call_test_function(builder, block_number);
            };

        let switch_and_test_function =
            |builder: &mut FunctionBuilder, block, block_number: u128| {
                builder.switch_to_block(block);
                call_test_function(builder, block_number);
            };

        store_value(&mut builder, 0);
        call_test_function(&mut builder, 0);
        builder.terminate_with_jmp(b1, vec![]);

        switch_store_and_test_function(&mut builder, b1, 1);
        builder.terminate_with_jmpif(c1, b2, b3);

        switch_store_and_test_function(&mut builder, b2, 2);
        builder.terminate_with_jmp(b4, vec![]);

        switch_store_and_test_function(&mut builder, b3, 3);
        builder.terminate_with_jmp(b8, vec![]);

        switch_and_test_function(&mut builder, b4, 4);
        builder.terminate_with_jmpif(c4, b5, b6);

        switch_store_and_test_function(&mut builder, b5, 5);
        builder.terminate_with_jmp(b7, vec![]);

        switch_store_and_test_function(&mut builder, b6, 6);
        builder.terminate_with_jmp(b7, vec![]);

        switch_and_test_function(&mut builder, b7, 7);
        builder.terminate_with_jmp(b9, vec![]);

        switch_and_test_function(&mut builder, b8, 8);
        builder.terminate_with_jmp(b9, vec![]);

        switch_and_test_function(&mut builder, b9, 9);
        let load = builder.insert_load(r1, Type::field());
        builder.terminate_with_return(vec![load]);

        let ssa = builder.finish().flatten_cfg().mem2reg();

        // Expected results after mem2reg removes the allocation and each load and store:
        //
        // fn main f0 {
        //   b0(v0: u1, v1: u1):
        //     call test_function(Field 0, Field 0)
        //     call test_function(Field 1, Field 1)
        //     enable_side_effects v0
        //     call test_function(Field 2, Field 2)
        //     call test_function(Field 4, Field 2)
        //     v29 = and v0, v1
        //     enable_side_effects v29
        //     call test_function(Field 5, Field 5)
        //     v32 = not v1
        //     v33 = and v0, v32
        //     enable_side_effects v33
        //     call test_function(Field 6, Field 6)
        //     enable_side_effects v0
        //     v36 = mul v1, Field 5
        //     v37 = mul v32, Field 2
        //     v38 = add v36, v37
        //     v39 = mul v1, Field 5
        //     v40 = mul v32, Field 6
        //     v41 = add v39, v40
        //     call test_function(Field 7, v42)
        //     v43 = not v0
        //     enable_side_effects v43
        //     store Field 3 at v2
        //     call test_function(Field 3, Field 3)
        //     call test_function(Field 8, Field 3)
        //     enable_side_effects Field 1
        //     v47 = mul v0, v41
        //     v48 = mul v43, Field 1
        //     v49 = add v47, v48
        //     v50 = mul v0, v44
        //     v51 = mul v43, Field 3
        //     v52 = add v50, v51
        //     call test_function(Field 9, v53)
        //     return v54
        // }

        let main = ssa.main();
        let ret = match main.dfg[main.entry_block()].terminator() {
            Some(TerminatorInstruction::Return { return_values, .. }) => return_values[0],
            _ => unreachable!("Should have terminator instruction"),
        };

        let merged_values = get_all_constants_reachable_from_instruction(&main.dfg, ret);
        assert_eq!(merged_values, vec![3, 5, 6]);
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
        // // Translates to the following before the flattening pass:
        // fn main f2 {
        //   b0(v0: u1):
        //     jmpif v0 then: b1, else: b2
        //   b1():
        //     v2 = allocate
        //     store Field 0 at v2
        //     v4 = load v2
        //     jmp b2()
        //   b2():
        //     return
        // }
        // The bug is that the flattening pass previously inserted a load
        // before the first store to allocate, which loaded an uninitialized value.
        // In this test we assert the ordering is strictly Allocate then Store then Load.
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let b1 = builder.insert_block();
        let b2 = builder.insert_block();

        let v0 = builder.add_parameter(Type::bool());
        builder.terminate_with_jmpif(v0, b1, b2);

        builder.switch_to_block(b1);
        let v2 = builder.insert_allocate(Type::field());
        let zero = builder.field_constant(0u128);
        builder.insert_store(v2, zero);
        let _v4 = builder.insert_load(v2, Type::field());
        builder.terminate_with_jmp(b2, vec![]);

        builder.switch_to_block(b2);
        builder.terminate_with_return(vec![]);

        let ssa = builder.finish().flatten_cfg();
        let main = ssa.main();

        // Now assert that there is not a load between the allocate and its first store
        // The Expected IR is:
        //
        // fn main f2 {
        //   b0(v0: u1):
        //     enable_side_effects v0
        //     v6 = allocate
        //     store Field 0 at v6
        //     v7 = load v6
        //     v8 = not v0
        //     enable_side_effects u1 1
        //     return
        // }
        let instructions = main.dfg[main.entry_block()].instructions();

        let find_instruction = |predicate: fn(&Instruction) -> bool| {
            instructions.iter().position(|id| predicate(&main.dfg[*id])).unwrap()
        };

        let allocate_index = find_instruction(|i| matches!(i, Instruction::Allocate));
        let store_index = find_instruction(|i| matches!(i, Instruction::Store { .. }));
        let load_index = find_instruction(|i| matches!(i, Instruction::Load { .. }));

        assert!(allocate_index < store_index);
        assert!(store_index < load_index);
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
                let mut values = vec![];
                dfg[instruction].map_values(|value| {
                    values.push(value);
                    value
                });

                let mut values: Vec<_> = values
                    .into_iter()
                    .flat_map(|value| get_all_constants_reachable_from_instruction(dfg, value))
                    .collect();

                values.sort();
                values.dedup();
                values
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
        // fn main f1 {
        //   b0():
        //     jmpif u1 0 then: b1, else: b2
        //   b1():
        //     jmp b2()
        //   b2():
        //     constrain u1 0 // was incorrectly removed
        //     return
        // }
        let main_id = Id::test_new(1);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        builder.insert_block(); // entry

        let b1 = builder.insert_block();
        let b2 = builder.insert_block();
        let v_true = builder.numeric_constant(true, Type::bool());
        let v_false = builder.numeric_constant(false, Type::bool());
        builder.terminate_with_jmpif(v_false, b1, b2);

        builder.switch_to_block(b1);
        builder.terminate_with_jmp(b2, vec![]);

        builder.switch_to_block(b2);
        builder.insert_constrain(v_false, v_true, None); // should not be removed
        builder.terminate_with_return(vec![]);

        let ssa = builder.finish().flatten_cfg();
        let main = ssa.main();

        // Assert we have not incorrectly removed a constraint:
        use Instruction::Constrain;
        let constrain_count = count_instruction(main, |ins| matches!(ins, Constrain(..)));
        assert_eq!(constrain_count, 1);
    }

    #[test]
    fn should_not_merge_incorrectly_to_false() {
        // Regression test for #1792
        // Tests that it does not simplify a true constraint an always-false constraint
        // fn main f1 {
        //   b0():
        //     v4 = call pedersen([Field 0], u32 0)
        //     v5 = array_get v4, index Field 0
        //     v6 = cast v5 as u32
        //     v8 = mod v6, u32 2
        //     v9 = cast v8 as u1
        //     v10 = allocate
        //     store Field 0 at v10
        //     jmpif v9 then: b1, else: b2
        //   b1():
        //     v14 = add v5, Field 1
        //     store v14 at v10
        //     jmp b3()
        //   b3():
        //     v12 = eq v9, u1 1
        //     constrain v12
        //     return
        //   b2():
        //     store Field 0 at v10
        //     jmp b3()
        // }
        let main_id = Id::test_new(1);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        builder.insert_block(); // b0
        let b1 = builder.insert_block();
        let b2 = builder.insert_block();
        let b3 = builder.insert_block();

        let element_type = Rc::new(vec![Type::field()]);
        let array_type = Type::Array(element_type.clone(), 1);

        let zero = builder.field_constant(0_u128);
        let zero_array = builder.array_constant(im::Vector::unit(zero), array_type);
        let i_zero = builder.numeric_constant(0_u128, Type::unsigned(32));
        let pedersen = builder
            .import_intrinsic_id(Intrinsic::BlackBox(acvm::acir::BlackBoxFunc::PedersenCommitment));
        let v4 = builder.insert_call(
            pedersen,
            vec![zero_array, i_zero],
            vec![Type::Array(element_type, 2)],
        )[0];
        let v5 = builder.insert_array_get(v4, zero, Type::field());
        let v6 = builder.insert_cast(v5, Type::unsigned(32));
        let i_two = builder.numeric_constant(2_u128, Type::unsigned(32));
        let v8 = builder.insert_binary(v6, BinaryOp::Mod, i_two);
        let v9 = builder.insert_cast(v8, Type::bool());

        let v10 = builder.insert_allocate(Type::field());
        builder.insert_store(v10, zero);

        builder.terminate_with_jmpif(v9, b1, b2);

        builder.switch_to_block(b1);
        let one = builder.field_constant(1_u128);
        let v14 = builder.insert_binary(v5, BinaryOp::Add, one);
        builder.insert_store(v10, v14);
        builder.terminate_with_jmp(b3, vec![]);

        builder.switch_to_block(b2);
        builder.insert_store(v10, zero);
        builder.terminate_with_jmp(b3, vec![]);

        builder.switch_to_block(b3);
        let v_true = builder.numeric_constant(true, Type::bool());
        let v12 = builder.insert_binary(v9, BinaryOp::Eq, v_true);
        builder.insert_constrain(v12, v_true, None);
        builder.terminate_with_return(vec![]);

        let ssa = builder.finish().flatten_cfg();
        let main = ssa.main();

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
    }

    #[test]
    fn undo_stores() {
        // Regression test for #1826. Ensures the `else` branch does not see the stores of the
        // `then` branch.
        //
        // fn main f1 {
        //   b0():
        //     v0 = allocate
        //     store Field 0 at v0
        //     v2 = allocate
        //     store Field 2 at v2
        //     v4 = load v2
        //     v5 = lt v4, Field 2
        //     jmpif v5 then: b1, else: b2
        //   b1():
        //     v24 = load v0
        //     v25 = load v2
        //     v26 = mul v25, Field 10
        //     v27 = add v24, v26
        //     store v27 at v0
        //     v28 = load v2
        //     v29 = add v28, Field 1
        //     store v29 at v2
        //     jmp b5()
        //   b5():
        //     v14 = load v0
        //     return v14
        //   b2():
        //     v6 = load v2
        //     v8 = lt v6, Field 4
        //     jmpif v8 then: b3, else: b4
        //   b3():
        //     v16 = load v0
        //     v17 = load v2
        //     v19 = mul v17, Field 100
        //     v20 = add v16, v19
        //     store v20 at v0
        //     v21 = load v2
        //     v23 = add v21, Field 1
        //     store v23 at v2
        //     jmp b4()
        //   b4():
        //     jmp b5()
        // }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let b1 = builder.insert_block();
        let b2 = builder.insert_block();
        let b3 = builder.insert_block();
        let b4 = builder.insert_block();
        let b5 = builder.insert_block();

        let zero = builder.field_constant(0u128);
        let one = builder.field_constant(1u128);
        let two = builder.field_constant(2u128);
        let four = builder.field_constant(4u128);
        let ten = builder.field_constant(10u128);
        let one_hundred = builder.field_constant(100u128);

        let v0 = builder.insert_allocate(Type::field());
        builder.insert_store(v0, zero);
        let v2 = builder.insert_allocate(Type::field());
        builder.insert_store(v2, two);
        let v4 = builder.insert_load(v2, Type::field());
        let v5 = builder.insert_binary(v4, BinaryOp::Lt, two);
        builder.terminate_with_jmpif(v5, b1, b2);

        builder.switch_to_block(b1);
        let v24 = builder.insert_load(v0, Type::field());
        let v25 = builder.insert_load(v2, Type::field());
        let v26 = builder.insert_binary(v25, BinaryOp::Mul, ten);
        let v27 = builder.insert_binary(v24, BinaryOp::Add, v26);
        builder.insert_store(v0, v27);
        let v28 = builder.insert_load(v2, Type::field());
        let v29 = builder.insert_binary(v28, BinaryOp::Add, one);
        builder.insert_store(v2, v29);
        builder.terminate_with_jmp(b5, vec![]);

        builder.switch_to_block(b5);
        let v14 = builder.insert_load(v0, Type::field());
        builder.terminate_with_return(vec![v14]);

        builder.switch_to_block(b2);
        let v6 = builder.insert_load(v2, Type::field());
        let v8 = builder.insert_binary(v6, BinaryOp::Lt, four);
        builder.terminate_with_jmpif(v8, b3, b4);

        builder.switch_to_block(b3);
        let v16 = builder.insert_load(v0, Type::field());
        let v17 = builder.insert_load(v2, Type::field());
        let v19 = builder.insert_binary(v17, BinaryOp::Mul, one_hundred);
        let v20 = builder.insert_binary(v16, BinaryOp::Add, v19);
        builder.insert_store(v0, v20);
        let v21 = builder.insert_load(v2, Type::field());
        let v23 = builder.insert_binary(v21, BinaryOp::Add, one);
        builder.insert_store(v2, v23);
        builder.terminate_with_jmp(b4, vec![]);

        builder.switch_to_block(b4);
        builder.terminate_with_jmp(b5, vec![]);

        let ssa = builder.finish().flatten_cfg().mem2reg().fold_constants();

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
    }
}
