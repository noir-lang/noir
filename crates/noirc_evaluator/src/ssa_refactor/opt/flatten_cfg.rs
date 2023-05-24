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
//!    will be merged. To merge the jmp arguments of the then and else branches, the forumula
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

    conditions: Vec<(BasicBlockId, ValueId)>,
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
                // before continuing from the end of the loop here.
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

    fn translate_value(&self, value: ValueId) -> ValueId {
        self.values.get(&value).copied().unwrap_or(value)
    }

    fn push_condition(&mut self, block: BasicBlockId, condition: ValueId) {
        let block = self.branch_ends[&block];

        if let Some((_, previous_conditon)) = self.conditions.last() {
            let and = Instruction::binary(BinaryOp::And, *previous_conditon, condition);
            let new_condition = self.insert_instruction(and);
            self.conditions.push((block, new_condition));
        } else {
            self.conditions.push((block, condition));
        }
    }

    fn insert_instruction(&mut self, instruction: Instruction) -> ValueId {
        let block = self.function.entry_block();
        self.function.dfg.insert_instruction_and_results(instruction, block, None).first()
    }

    fn analyze_function(&mut self) {
        let post_order = PostOrder::with_function(self.function);
        let dom_tree = DominatorTree::with_cfg_and_post_order(&self.cfg, &post_order);
        let mut branch_beginnings = Vec::new();

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_front(self.function.entry_block());
        visited.insert(self.function.entry_block());

        while let Some(block_id) = queue.pop_front() {
            // If there is more than one predecessor, this must be an end block
            let mut predecessors = self.cfg.predecessors(block_id);
            if predecessors.len() > 1 {
                // If we haven't already visited all of this block's predecessors, delay analyzing
                // the block until we have. This ensures we analyze the function in evaluation order.
                if !predecessors.all(|block| visited.contains(&block)) {
                    queue.push_back(block_id);
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
            visited.extend(block.successors());
        }
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

        let args = vecmap(then_args.into_iter().zip(else_args), |(then_arg, else_arg)| {
            (self.translate_value(*then_arg), self.translate_value(*else_arg))
        });

        // Cannot include this in the previous vecmap since it requires exclusive access to self
        let args =
            vecmap(args, |(then_arg, else_arg)| self.merge_values(condition, then_arg, else_arg));

        // insert merge instruction
        self.inline_block(destination, &args)
    }

    fn inline_block(&mut self, destination: BasicBlockId, arguments: &[ValueId]) -> BasicBlockId {
        let parameters = self.function.dfg.block_parameters(destination);
        Self::insert_new_instruction_results(
            &mut self.values,
            parameters,
            InsertInstructionResult::Results(arguments),
        );

        for instruction in self.function.dfg[destination].instructions().to_vec() {
            self.push_instruction(instruction);
        }

        self.handle_terminator(destination)
    }

    fn push_instruction(&mut self, id: InstructionId) {
        let instruction = self.function.dfg[id].map_values(|id| self.translate_value(id));
        let instruction = self.handle_instruction_side_effects(instruction);
        let results = self.function.dfg.instruction_results(id).to_vec();

        let ctrl_typevars = instruction
            .requires_ctrl_typevars()
            .then(|| vecmap(&results, |result| self.function.dfg.type_of_value(*result)));

        let block = self.function.entry_block();
        let new_results = self.function.dfg.insert_instruction_and_results(instruction, block, ctrl_typevars);
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
    use crate::ssa_refactor::{
        ir::{map::Id, types::Type},
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
}
