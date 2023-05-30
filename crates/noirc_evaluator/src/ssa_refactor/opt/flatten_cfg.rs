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
//! 3. UNIMPLEMENTED: After being stored to in at least one predecessor of a block with multiple predecessors, the
//!    value of a memory address is the value it had in both branches combined via c * a + !c * b
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
use std::collections::{HashMap, HashSet, VecDeque};

use acvm::FieldElement;
use iter_extended::vecmap;

use crate::ssa_refactor::{
    ir::{
        basic_block::BasicBlockId,
        cfg::ControlFlowGraph,
        dfg::InsertInstructionResult,
        dom::DominatorTree,
        function::Function,
        instruction::{BinaryOp, Instruction, InstructionId, TerminatorInstruction},
        post_order::PostOrder,
        types::Type,
        value::ValueId,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Flattens the control flow graph of each function such that the function is left with a
    /// single block containing all instructions and no more control-flow.
    ///
    /// This pass will modify any instructions with side effects in particular, often multiplying
    /// them by jump conditions to maintain correctness even when all branches of a jmpif are inlined.
    /// For more information, see the module-level comment at the top of this file.
    pub(crate) fn flatten_cfg(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            flatten_function_cfg(function);
        }
        self
    }
}

struct Context<'f> {
    function: &'f mut Function,

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
    conditions: Vec<(BasicBlockId, ValueId)>,

    /// A map of values from the unmodified function to their values given from this pass.
    /// In particular, this pass will remove all block arguments except for function parameters.
    /// Each value in the function's entry block is also left unchanged.
    values: HashMap<ValueId, ValueId>,
}

fn flatten_function_cfg(function: &mut Function) {
    let mut context = Context {
        cfg: ControlFlowGraph::with_function(function),
        function,
        branch_ends: HashMap::new(),
        conditions: Vec::new(),
        values: HashMap::new(),
    };
    context.flatten();
}

impl<'f> Context<'f> {
    fn flatten(&mut self) {
        self.analyze_function();

        // Start with following the terminator of the entry block since we don't
        // need to flatten the entry block into itself.
        self.handle_terminator(self.function.entry_block());
    }

    /// Visits every block in the current function to find all blocks with a jmpif instruction and
    /// all blocks which terminate the jmpif by having each of its branches as a predecessor.
    fn analyze_function(&mut self) {
        let post_order = PostOrder::with_function(self.function);
        let dom_tree = DominatorTree::with_cfg_and_post_order(&self.cfg, &post_order);
        let mut branch_beginnings = Vec::new();

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_front(self.function.entry_block());

        while let Some(block_id) = queue.pop_front() {
            // If multiple blocks branch to the same successor before we visit it we can end up in
            // situations where the same block occurs multiple times in our queue. This check
            // prevents visiting the same block twice.
            if visited.contains(&block_id) {
                continue;
            } else {
                visited.insert(block_id);
            }

            // If there is more than one predecessor, this must be an end block
            let mut predecessors = self.cfg.predecessors(block_id);
            if predecessors.len() > 1 {
                // If we haven't already visited all of this block's predecessors, delay analyzing
                // the block until we have. This ensures we analyze the function in evaluation order.
                if !predecessors.all(|block| visited.contains(&block)) {
                    queue.push_back(block_id);
                    visited.remove(&block_id);
                    continue;
                }

                // We expect the merging of two branches to be ordered such that only the most
                // recent jmpif is a candidate for being the start of the two branches merged by
                // a block with 2 predecessors.
                let branch_beginning =
                    branch_beginnings.pop().expect("Expected the beginning of a branch");

                for predecessor in self.cfg.predecessors(block_id) {
                    assert!(dom_tree.dominates(branch_beginning, predecessor));
                }

                self.branch_ends.insert(branch_beginning, block_id);
            }

            let block = &self.function.dfg[block_id];
            if let Some(TerminatorInstruction::JmpIf { .. }) = block.terminator() {
                branch_beginnings.push(block_id);
            }

            queue.extend(block.successors().filter(|block| !visited.contains(block)));
        }

        assert!(branch_beginnings.is_empty());
    }

    /// Check the terminator of the given block and recursively inline any blocks reachable from
    /// it. Since each block from a jmpif terminator is inlined successively, we must handle
    /// instructions with side effects like constrain and store specially to preserve correctness.
    /// For these instructions we must keep track of what the current condition is and modify
    /// the instructions according to the module-level comment at the top of this file. Note that
    /// the current condition is all the jmpif conditions required to reach the current block,
    /// combined via `And` instructions.
    ///
    /// Returns the last block to be inlined. This is either the return block of the function or,
    /// if self.conditions is not empty, the end block of the most recent condition.
    fn handle_terminator(&mut self, block: BasicBlockId) -> BasicBlockId {
        match self.function.dfg[block].unwrap_terminator() {
            TerminatorInstruction::JmpIf { condition, then_destination, else_destination } => {
                let old_condition = *condition;
                let new_condition = self.translate_value(old_condition);
                let then_destination = *then_destination;
                let else_destination = *else_destination;

                let one = FieldElement::one();
                let last_then_block =
                    self.inline_branch(block, then_destination, old_condition, new_condition, one);

                let else_condition = self.insert_instruction(Instruction::Not(new_condition));
                let zero = FieldElement::zero();

                let last_else_block = self.inline_branch(
                    block,
                    else_destination,
                    old_condition,
                    else_condition,
                    zero,
                );

                // While there is a condition on the stack we don't compile outside the condition
                // until it is popped. This ensures we inline the full then and else branches
                // before continuing from the end of the loop here where they can be merged properly.
                let end = self.branch_ends[&block];
                self.inline_branch_end(end, new_condition, last_then_block, last_else_block)
            }
            TerminatorInstruction::Jmp { destination, arguments } => {
                if let Some((end_block, _)) = self.conditions.last() {
                    if destination == end_block {
                        return block;
                    }
                }
                let arguments = vecmap(arguments, |value| self.translate_value(*value));
                self.inline_block(*destination, &arguments)
            }
            TerminatorInstruction::Return { return_values } => {
                let return_values = vecmap(return_values, |value| self.translate_value(*value));
                let entry = self.function.entry_block();
                let new_return = TerminatorInstruction::Return { return_values };
                self.function.dfg.set_block_terminator(entry, new_return);
                block
            }
        }
    }

    /// Translate a value id from before the function was modified to one from after it has been
    /// flattened. In particular, all block parameters should be removed, having been mapped to
    /// their (merged) arguments, and all values from the entry block are unchanged.
    fn translate_value(&self, value: ValueId) -> ValueId {
        self.values.get(&value).copied().unwrap_or(value)
    }

    /// Push a condition to the stack of conditions.
    ///
    /// This condition should be present while we're inlining each block reachable from the 'then'
    /// branch of a jmpif instruction, until the branches eventually join back together. Likewise,
    /// !condition should be present while we're inlining each block reachable from the 'else'
    /// branch of a jmpif instruction until the join block.
    fn push_condition(&mut self, start_block: BasicBlockId, condition: ValueId) {
        let end_block = self.branch_ends[&start_block];

        if let Some((_, previous_condition)) = self.conditions.last() {
            let and = Instruction::binary(BinaryOp::And, *previous_condition, condition);
            let new_condition = self.insert_instruction(and);
            self.conditions.push((end_block, new_condition));
        } else {
            self.conditions.push((end_block, condition));
        }
    }

    /// Insert a new instruction into the function's entry block.
    ///
    /// Note that this does not modify self.values.
    fn insert_instruction(&mut self, instruction: Instruction) -> ValueId {
        let block = self.function.entry_block();
        self.function.dfg.insert_instruction_and_results(instruction, block, None).first()
    }

    /// Merge two values a and b from separate basic blocks to a single value. This
    /// function would return the result of `if c { a } else { b }` as  `c*a + (!c)*b`.
    fn merge_values(
        &mut self,
        condition: ValueId,
        then_value: ValueId,
        else_value: ValueId,
    ) -> ValueId {
        let block = self.function.entry_block();
        let mul = Instruction::binary(BinaryOp::Mul, condition, then_value);
        let then_value = self.function.dfg.insert_instruction_and_results(mul, block, None).first();

        let not = Instruction::Not(condition);
        let not = self.function.dfg.insert_instruction_and_results(not, block, None).first();

        let mul = Instruction::binary(BinaryOp::Mul, not, else_value);
        let else_value = self.function.dfg.insert_instruction_and_results(mul, block, None).first();

        let add = Instruction::binary(BinaryOp::Add, then_value, else_value);
        self.function.dfg.insert_instruction_and_results(add, block, None).first()
    }

    /// Inline one branch of a jmpif instruction.
    ///
    /// This will continue inlining recursively until the next end block is reached where each branch
    /// of the jmpif instruction is joined back into a single block.
    ///
    /// Within a branch of a jmpif instruction, we can assume the condition of the jmpif to be
    /// always true or false, depending on which branch we're in.
    ///
    /// Returns the ending block / join block of this branch.
    fn inline_branch(
        &mut self,
        jmpif_block: BasicBlockId,
        destination: BasicBlockId,
        old_condition: ValueId,
        new_condition: ValueId,
        condition_value: FieldElement,
    ) -> BasicBlockId {
        self.push_condition(jmpif_block, new_condition);

        // Remember the old condition value is now known to be true/false within this branch
        let known_value = self.function.dfg.make_constant(condition_value, Type::bool());
        self.values.insert(old_condition, known_value);

        // TODO: Keep track of stores in branch
        let final_block = self.inline_block(destination, &[]);

        self.conditions.pop();
        final_block
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
        condition: ValueId,
        last_then_block: BasicBlockId,
        last_else_block: BasicBlockId,
    ) -> BasicBlockId {
        assert_eq!(self.cfg.predecessors(destination).len(), 2);

        let then_args = self.function.dfg[last_then_block].terminator_arguments();
        let else_args = self.function.dfg[last_else_block].terminator_arguments();

        let params = self.function.dfg.block_parameters(destination);
        assert_eq!(params.len(), then_args.len());
        assert_eq!(params.len(), else_args.len());

        let args = vecmap(then_args.iter().zip(else_args), |(then_arg, else_arg)| {
            (self.translate_value(*then_arg), self.translate_value(*else_arg))
        });

        // Cannot include this in the previous vecmap since it requires exclusive access to self
        let args =
            vecmap(args, |(then_arg, else_arg)| self.merge_values(condition, then_arg, else_arg));

        // insert merge instruction
        self.inline_block(destination, &args)
    }

    /// Inline all instructions from the given destination block into the entry block.
    /// Afterwards, check the block's terminator and continue inlining recursively.
    ///
    /// Returns the final block that was inlined.
    ///
    /// Expects that the `arguments` given are already translated via self.translate_value.
    /// If they are not, it is possible some values which no longer exist, such as block
    /// parameters, will be kept in the program.
    fn inline_block(&mut self, destination: BasicBlockId, arguments: &[ValueId]) -> BasicBlockId {
        let parameters = self.function.dfg.block_parameters(destination);
        Self::insert_new_instruction_results(
            &mut self.values,
            parameters,
            InsertInstructionResult::Results(arguments),
        );

        // If this is not a separate variable, clippy gets confused and says the to_vec is
        // unnecessary, when removing it actually causes an aliasing/mutability error.
        let instructions = self.function.dfg[destination].instructions().to_vec();
        for instruction in instructions {
            self.push_instruction(instruction);
        }

        self.handle_terminator(destination)
    }

    /// Push the given instruction to the end of the entry block of the current function.
    ///
    /// Note that each ValueId of the instruction will be mapped via self.translate_value.
    /// As a result, the instruction that will be pushed will actually be a new instruction
    /// with a different InstructionId from the original. The results of the given instruction
    /// will also be mapped to the results of the new instruction.
    fn push_instruction(&mut self, id: InstructionId) {
        let instruction = self.function.dfg[id].map_values(|id| self.translate_value(id));
        let instruction = self.handle_instruction_side_effects(instruction);
        let results = self.function.dfg.instruction_results(id).to_vec();

        let ctrl_typevars = instruction
            .requires_ctrl_typevars()
            .then(|| vecmap(&results, |result| self.function.dfg.type_of_value(*result)));

        let block = self.function.entry_block();
        let new_results =
            self.function.dfg.insert_instruction_and_results(instruction, block, ctrl_typevars);
        Self::insert_new_instruction_results(&mut self.values, &results, new_results);
    }

    /// If we are currently in a branch, we need to modify constrain instructions
    /// to multiply them by the branch's condition (see optimization #1 in the module comment).
    fn handle_instruction_side_effects(&mut self, instruction: Instruction) -> Instruction {
        if let Some((_, condition)) = self.conditions.last().copied() {
            match instruction {
                Instruction::Constrain(value) => {
                    let mul = self.insert_instruction(Instruction::binary(
                        BinaryOp::Mul,
                        value,
                        condition,
                    ));
                    let eq =
                        self.insert_instruction(Instruction::binary(BinaryOp::Eq, mul, condition));
                    Instruction::Constrain(eq)
                }
                // TODO: Need to log any stores found
                other => other,
            }
        } else {
            instruction
        }
    }

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
            InsertInstructionResult::Results(new_results) => {
                for (old_result, new_result) in old_results.iter().zip(new_results) {
                    values.insert(*old_result, *new_result);
                }
            }
            InsertInstructionResult::InstructionRemoved => (),
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::ssa_refactor::{
        ir::{cfg::ControlFlowGraph, map::Id, types::Type},
        ssa_builder::FunctionBuilder,
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
        //     v4 = not v0
        //     v5 = mul v0, Field 3
        //     v7 = not v0
        //     v8 = mul v7, Field 4
        //     v9 = add v5, v8
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

        builder.terminate_with_jmpif(v0, b1, b2);

        builder.switch_to_block(b1);
        builder.insert_constrain(v1);
        builder.terminate_with_jmp(b2, vec![]);

        builder.switch_to_block(b2);
        builder.terminate_with_return(vec![]);

        let ssa = builder.finish();
        assert_eq!(ssa.main().reachable_blocks().len(), 3);

        // Expected output (sans useless extra 'not' instruction):
        // fn main f0 {
        //   b0(v0: u1, v1: u1):
        //     v2 = mul v1, v0
        //     v3 = eq v2, v0
        //     constrain v3
        //     return v1
        // }
        let ssa = ssa.flatten_cfg();
        assert_eq!(ssa.main().reachable_blocks().len(), 1);
    }

    #[test]
    fn nested_branch_analysis() {
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

        builder.terminate_with_jmp(b1, vec![]);
        builder.switch_to_block(b1);
        builder.terminate_with_jmpif(c1, b2, b3);
        builder.switch_to_block(b2);
        builder.terminate_with_jmp(b4, vec![]);
        builder.switch_to_block(b3);
        builder.terminate_with_jmp(b8, vec![]);
        builder.switch_to_block(b4);
        builder.terminate_with_jmpif(c4, b5, b6);
        builder.switch_to_block(b5);
        builder.terminate_with_jmp(b7, vec![]);
        builder.switch_to_block(b6);
        builder.terminate_with_jmp(b7, vec![]);
        builder.switch_to_block(b7);
        builder.terminate_with_jmp(b9, vec![]);
        builder.switch_to_block(b8);
        builder.terminate_with_jmp(b9, vec![]);
        builder.switch_to_block(b9);
        builder.terminate_with_return(vec![]);

        let mut ssa = builder.finish();
        let function = ssa.main_mut();
        let mut context = super::Context {
            cfg: ControlFlowGraph::with_function(function),
            function,
            branch_ends: HashMap::new(),
            conditions: Vec::new(),
            values: HashMap::new(),
        };
        context.analyze_function();
        assert_eq!(context.branch_ends.len(), 2);
        assert_eq!(context.branch_ends.get(&b1), Some(&b9));
        assert_eq!(context.branch_ends.get(&b4), Some(&b7));
    }
}
