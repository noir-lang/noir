//! This file contains the loop unrolling pass for the new SSA IR.
//!
//! This pass is divided into three steps:
//! 1. Find a loop in the program (`find_next_loop`)
//! 2. Unroll that loop into its "pre-header" block (`unroll_loop`)
//! 3. Repeat until no more loops are found
//!
//! Note that unrolling loops will fail if there are loops with non-constant
//! indices. This pass also often creates superfluous jmp instructions in the
//! program that will need to be removed by a later simplify cfg pass.
use std::collections::{HashMap, HashSet};

use iter_extended::vecmap;

use crate::ssa_refactor::{
    ir::{
        basic_block::BasicBlockId,
        cfg::ControlFlowGraph,
        dfg::InsertInstructionResult,
        dom::DominatorTree,
        function::Function,
        instruction::{InstructionId, TerminatorInstruction},
        post_order::PostOrder,
        value::ValueId,
    },
    ssa_gen::Ssa,
};

/// Arbitrary maximum of 10k loops unrolled in a program to prevent looping forever
/// if a bug causes us to continually unroll the same loop.
const MAX_LOOPS_UNROLLED: u32 = 10_000;

impl Ssa {
    /// Unroll all loops in each SSA function.
    /// Panics if any loop cannot be unrolled.
    pub(crate) fn unroll_loops(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            unroll_loops_in_function(function);
        }
        self
    }
}

/// Unroll all loops within a given function.
/// This will panic if the function has more than MAX_LOOPS_UNROLLED loops to unroll
/// or if the function has loops that cannot be unrolled because it has non-constant indices.
fn unroll_loops_in_function(function: &mut Function) {
    for _ in 0..MAX_LOOPS_UNROLLED {
        // Recompute the cfg & dom_tree after each loop in case we unrolled into another loop.
        // TODO: Optimize: lazily recompute this only if the next loops' blocks have already been visited.
        let cfg = ControlFlowGraph::with_function(function);
        let post_order = PostOrder::with_function(function);
        let dom_tree = DominatorTree::with_cfg_and_post_order(&cfg, &post_order);

        if let Some(loop_) = find_next_loop(function, &cfg, &dom_tree) {
            unroll_loop(function, &cfg, loop_).ok();
        } else {
            return;
        }
    }

    panic!("Did not finish unrolling all loops after the maximum of {MAX_LOOPS_UNROLLED} loops unrolled")
}

struct Loop {
    /// The header block of a loop is the block which dominates all the
    /// other blocks in the loop.
    header: BasicBlockId,

    /// The start of the back_edge n -> d is the block n at the end of
    /// the loop that jumps back to the header block d which restarts the loop.
    back_edge_start: BasicBlockId,

    /// All the blocks contained within the loop, including `header` and `back_edge_start`.
    blocks: HashSet<BasicBlockId>,
}

/// Find a loop in the program by finding a node that dominates any predecessor node.
/// The edge where this happens will be the back-edge of the loop.
///
/// We could change this to return all loops in the function instead, but we'd have to
/// make sure to automatically refresh the list if any blocks within one loop were modified
/// as a result of inlining another.
fn find_next_loop(
    function: &Function,
    cfg: &ControlFlowGraph,
    dom_tree: &DominatorTree,
) -> Option<Loop> {
    let mut loops = vec![];

    for (block, _) in function.dfg.basic_blocks_iter() {
        // These reachable checks wouldn't be needed if we only iterated over reachable blocks
        if dom_tree.is_reachable(block) {
            for predecessor in cfg.predecessors(block) {
                if dom_tree.is_reachable(predecessor) && dom_tree.dominates(block, predecessor) {
                    // predecessor -> block is the back-edge of a loop
                    loops.push(find_blocks_in_loop(block, predecessor, cfg));
                }
            }
        }
    }

    // Sort loops by block size so that we unroll the smaller, nested loops first as an
    // optimization.
    loops.sort_by(|loop_a, loop_b| loop_b.blocks.len().cmp(&loop_a.blocks.len()));
    loops.pop()
}

/// Return each block that is in a loop starting in the given header block.
/// Expects back_edge_start -> header to be the back edge of the loop.
fn find_blocks_in_loop(
    header: BasicBlockId,
    back_edge_start: BasicBlockId,
    cfg: &ControlFlowGraph,
) -> Loop {
    let mut blocks = HashSet::new();
    blocks.insert(header);

    let mut insert = |block, stack: &mut Vec<BasicBlockId>| {
        if !blocks.contains(&block) {
            blocks.insert(block);
            stack.push(block);
        }
    };

    // Starting from the back edge of the loop, each predecessor of this block until
    // the header is within the loop.
    let mut stack = vec![];
    insert(back_edge_start, &mut stack);

    while let Some(block) = stack.pop() {
        for predecessor in cfg.predecessors(block) {
            insert(predecessor, &mut stack);
        }
    }

    Loop { header, back_edge_start, blocks }
}

/// Unroll a single loop in the function
fn unroll_loop(function: &mut Function, cfg: &ControlFlowGraph, loop_: Loop) -> Result<(), ()> {
    let mut unroll_into = get_pre_header(cfg, &loop_);
    let mut jump_value = get_induction_variable(function, unroll_into)?;

    while let Some(context) = unroll_loop_header(function, &loop_, unroll_into, jump_value) {
        let (last_block, last_value) = context.unroll_loop_iteration();
        unroll_into = last_block;
        jump_value = last_value;
    }

    Ok(())
}

/// The loop pre-header is the block that comes before the loop begins. Generally a header block
/// is expected to have 2 predecessors: the pre-header and the final block of the loop which jumps
/// back to the beginning.
fn get_pre_header(cfg: &ControlFlowGraph, loop_: &Loop) -> BasicBlockId {
    let mut pre_header = cfg
        .predecessors(loop_.header)
        .filter(|predecessor| *predecessor != loop_.back_edge_start)
        .collect::<Vec<_>>();

    assert_eq!(pre_header.len(), 1);
    pre_header.remove(0)
}

/// Return the induction value of the current iteration of the loop, from the given block's jmp arguments.
///
/// Expects the current block to terminate in `jmp h(N)` where h is the loop header and N is
/// a Field value.
fn get_induction_variable(function: &Function, block: BasicBlockId) -> Result<ValueId, ()> {
    match function.dfg[block].terminator() {
        Some(TerminatorInstruction::Jmp { arguments, .. }) => {
            assert_eq!(arguments.len(), 1);
            let value = arguments[0];
            if function.dfg.get_numeric_constant(value).is_some() {
                Ok(value)
            } else {
                Err(())
            }
        }
        _ => Err(()),
    }
}

/// Unrolls the header block of the loop. This is the block that dominates all other blocks in the
/// loop and contains the jmpif instruction that lets us know if we should continue looping.
/// Returns Some(iteration context) if we should perform another iteration.
fn unroll_loop_header<'a>(
    function: &'a mut Function,
    loop_: &'a Loop,
    unroll_into: BasicBlockId,
    induction_value: ValueId,
) -> Option<LoopIteration<'a>> {
    let mut context = LoopIteration::new(function, loop_, unroll_into, loop_.header);
    let source_block = &context.function.dfg[context.source_block];
    assert_eq!(source_block.parameters().len(), 1, "Expected only 1 argument in loop header",);

    let first_param = source_block.parameters()[0];
    context.values.insert(first_param, induction_value);
    context.inline_instructions_from_block();

    match context.function.dfg[unroll_into].unwrap_terminator() {
        TerminatorInstruction::JmpIf { condition, then_destination, else_destination } => {
            let next_blocks = context.handle_jmpif(*condition, *then_destination, *else_destination);

            // If there is only 1 next block the jmpif evaluated to a single known block.
            // This is the expected case and lets us know if we should loop again or not.
            if next_blocks.len() == 1 {
                loop_.blocks.contains(&context.source_block).then_some(context)
            } else {
                // Non-constant loop. We have to reset the then and else destination back to
                // the original blocks here since we won't be unrolling into the new blocks.
                context.function.dfg.get_block_terminator_mut(context.insert_block)
                    .mutate_blocks(|block| context.original_blocks[&block]);

                None
            }
        }
        other => panic!("Expected loop header to terminate in a JmpIf to the loop body, but found {other:?} instead"),
    }
}

/// The context object for each loop iteration.
/// Notably each loop iteration maps each loop block to a fresh, unrolled block.
struct LoopIteration<'f> {
    function: &'f mut Function,
    loop_: &'f Loop,
    values: HashMap<ValueId, ValueId>,
    blocks: HashMap<BasicBlockId, BasicBlockId>,

    /// Maps unrolled block ids back to the original source block ids
    original_blocks: HashMap<BasicBlockId, BasicBlockId>,
    visited_blocks: HashSet<BasicBlockId>,

    insert_block: BasicBlockId,
    source_block: BasicBlockId,

    /// The induction value (and the block it was found in) is the new value for
    /// the variable traditionally called `i` on each iteration of the loop.
    induction_value: Option<(BasicBlockId, ValueId)>,
}

impl<'f> LoopIteration<'f> {
    fn new(
        function: &'f mut Function,
        loop_: &'f Loop,
        insert_block: BasicBlockId,
        source_block: BasicBlockId,
    ) -> Self {
        Self {
            function,
            loop_,
            insert_block,
            source_block,
            values: HashMap::new(),
            blocks: HashMap::new(),
            original_blocks: HashMap::new(),
            visited_blocks: HashSet::new(),
            induction_value: None,
        }
    }

    /// Unroll a single iteration of the loop.
    ///
    /// Note that after unrolling a single iteration, the loop is _not_ in a valid state.
    /// It is expected the terminator instructions are set up to branch into an empty block
    /// for further unrolling. When the loop is finished this will need to be mutated to
    /// jump to the end of the loop instead.
    fn unroll_loop_iteration(mut self) -> (BasicBlockId, ValueId) {
        let mut next_blocks = self.unroll_loop_block();

        while let Some(block) = next_blocks.pop() {
            self.insert_block = block;
            self.source_block = self.get_original_block(block);

            if !self.visited_blocks.contains(&self.source_block) {
                let mut blocks = self.unroll_loop_block();
                next_blocks.append(&mut blocks);
            }
        }

        self.induction_value
            .expect("Expected to find the induction variable by end of loop iteration")
    }

    /// Unroll a single block in the current iteration of the loop
    fn unroll_loop_block(&mut self) -> Vec<BasicBlockId> {
        let mut next_blocks = self.unroll_loop_block_helper();
        next_blocks.retain(|block| {
            let b = self.get_original_block(*block);
            self.loop_.blocks.contains(&b)
        });
        next_blocks
    }

    /// Unroll a single block in the current iteration of the loop
    fn unroll_loop_block_helper(&mut self) -> Vec<BasicBlockId> {
        self.inline_instructions_from_block();
        self.visited_blocks.insert(self.source_block);

        match self.function.dfg[self.insert_block].unwrap_terminator() {
            TerminatorInstruction::JmpIf { condition, then_destination, else_destination } => {
                self.handle_jmpif(*condition, *then_destination, *else_destination)
            }
            TerminatorInstruction::Jmp { destination, arguments } => {
                if self.get_original_block(*destination) == self.loop_.header {
                    assert_eq!(arguments.len(), 1);
                    self.induction_value = Some((self.insert_block, arguments[0]));
                }
                vec![*destination]
            }
            TerminatorInstruction::Return { .. } => vec![],
        }
    }

    /// Find the next branch(es) to take from a jmpif terminator and return them.
    /// If only one block is returned, it means the jmpif condition evaluated to a known
    /// constant and we can safely take only the given branch.
    fn handle_jmpif(
        &mut self,
        condition: ValueId,
        then_destination: BasicBlockId,
        else_destination: BasicBlockId,
    ) -> Vec<BasicBlockId> {
        let condition = self.get_value(condition);

        match self.function.dfg.get_numeric_constant(condition) {
            Some(constant) => {
                let destination =
                    if constant.is_zero() { else_destination } else { then_destination };

                self.source_block = self.get_original_block(destination);
                let jmp = TerminatorInstruction::Jmp { destination, arguments: Vec::new() };
                self.function.dfg.set_block_terminator(self.insert_block, jmp);
                vec![destination]
            }
            None => vec![then_destination, else_destination],
        }
    }

    fn get_value(&self, value: ValueId) -> ValueId {
        self.values.get(&value).copied().unwrap_or(value)
    }

    /// Translate a block id to a block id in the unrolled loop. If the given
    /// block id is not within the loop, it is returned as-is.
    fn get_or_insert_block(&mut self, block: BasicBlockId) -> BasicBlockId {
        if let Some(new_block) = self.blocks.get(&block) {
            return *new_block;
        }

        // If the block is in the loop we create a fresh block for each iteration
        if self.loop_.blocks.contains(&block) {
            let new_block = self.function.dfg.make_block_with_parameters_from_block(block);

            let old_parameters = self.function.dfg.block_parameters(block);
            let new_parameters = self.function.dfg.block_parameters(new_block);

            for (param, new_param) in old_parameters.iter().zip(new_parameters) {
                // Don't overwrite any existing entries to avoid overwriting the induction variable
                self.values.entry(*param).or_insert(*new_param);
            }

            self.blocks.insert(block, new_block);
            self.original_blocks.insert(new_block, block);
            new_block
        } else {
            block
        }
    }

    fn get_original_block(&self, block: BasicBlockId) -> BasicBlockId {
        self.original_blocks.get(&block).copied().unwrap_or(block)
    }

    fn inline_instructions_from_block(&mut self) {
        let source_block = &self.function.dfg[self.source_block];
        let instructions = source_block.instructions().to_vec();

        // We cannot directly append each instruction since we need to substitute any
        // instances of the induction variable or any values that were changed as a result
        // of the new induction variable value.
        for instruction in instructions {
            self.push_instruction(instruction);
        }

        let mut terminator = self.function.dfg[self.source_block]
            .unwrap_terminator()
            .map_values(|value| self.get_value(value));

        terminator.mutate_blocks(|block| self.get_or_insert_block(block));
        self.function.dfg.set_block_terminator(self.insert_block, terminator);
    }

    fn push_instruction(&mut self, id: InstructionId) {
        let instruction = self.function.dfg[id].map_values(|id| self.get_value(id));
        let results = self.function.dfg.instruction_results(id).to_vec();

        let ctrl_typevars = instruction
            .requires_ctrl_typevars()
            .then(|| vecmap(&results, |result| self.function.dfg.type_of_value(*result)));

        let new_results = self.function.dfg.insert_instruction_and_results(
            instruction,
            self.insert_block,
            ctrl_typevars,
        );

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
mod tests {
    use crate::ssa_refactor::{
        ir::{dom::DominatorTree, instruction::BinaryOp, map::Id, types::Type},
        ssa_builder::FunctionBuilder,
        ssa_gen::Ssa,
    };

    #[test]
    fn unroll_nested_loops() {
        // fn main() {
        //     for i in 0..3 {
        //         for j in 0..4 {
        //             assert(i + j > 10);
        //         }
        //     }
        // }
        //
        // fn main f0 {
        //   b0():
        //     jmp b1(Field 0)
        //   b1(v0: Field):  // header of outer loop
        //     v1 = lt v0, Field 3
        //     jmpif v1, then: b2, else: b3
        //   b2():
        //     jmp b4(Field 0)
        //   b4(v2: Field):  // header of inner loop
        //     v3 = lt v2, Field 4
        //     jmpif v3, then: b5, else: b6
        //   b5():
        //     v4 = add v0, v2
        //     v5 = lt Field 10, v4
        //     constrain v5
        //     v6 = add v2, Field 1
        //     jmp b4(v6)
        //   b6(): // end of inner loop
        //     v7 = add v0, Field 1
        //     jmp b1(v7)
        //   b3(): // end of outer loop
        //     return Field 0
        // }
        let main_id = Id::test_new(0);

        // Compiling main
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let b1 = builder.insert_block();
        let b2 = builder.insert_block();
        let b3 = builder.insert_block();
        let b4 = builder.insert_block();
        let b5 = builder.insert_block();
        let b6 = builder.insert_block();

        let v0 = builder.add_block_parameter(b1, Type::field());
        let v2 = builder.add_block_parameter(b4, Type::field());

        let zero = builder.field_constant(0u128);
        let one = builder.field_constant(1u128);
        let three = builder.field_constant(3u128);
        let four = builder.field_constant(4u128);
        let ten = builder.field_constant(10u128);

        builder.terminate_with_jmp(b1, vec![zero]);

        // b1
        builder.switch_to_block(b1);
        let v1 = builder.insert_binary(v0, BinaryOp::Lt, three);
        builder.terminate_with_jmpif(v1, b2, b3);

        // b2
        builder.switch_to_block(b2);
        builder.terminate_with_jmp(b4, vec![zero]);

        // b3
        builder.switch_to_block(b3);
        builder.terminate_with_return(vec![zero]);

        // b4
        builder.switch_to_block(b4);
        let v3 = builder.insert_binary(v2, BinaryOp::Lt, four);
        builder.terminate_with_jmpif(v3, b5, b6);

        // b5
        builder.switch_to_block(b5);
        let v4 = builder.insert_binary(v0, BinaryOp::Add, v2);
        let v5 = builder.insert_binary(ten, BinaryOp::Lt, v4);
        builder.insert_constrain(v5);
        let v6 = builder.insert_binary(v2, BinaryOp::Add, one);
        builder.terminate_with_jmp(b4, vec![v6]);

        // b6
        builder.switch_to_block(b6);
        let v7 = builder.insert_binary(v0, BinaryOp::Add, one);
        builder.terminate_with_jmp(b1, vec![v7]);

        // basic_blocks_iter iterates over unreachable blocks as well, so we must filter those out.
        let count_reachable_blocks = |ssa: &Ssa| {
            let function = ssa.main();
            let dom_tree = DominatorTree::with_function(function);
            function
                .dfg
                .basic_blocks_iter()
                .filter(|(block, _)| dom_tree.is_reachable(*block))
                .count()
        };

        let ssa = builder.finish();
        assert_eq!(count_reachable_blocks(&ssa), 7);

        // The final block count is not 1 because the block creates some unnecessary jmps.
        // If a simplify cfg pass is ran afterward, the expected block count will be 1.
        let ssa = ssa.unroll_loops();
        assert_eq!(count_reachable_blocks(&ssa), 5);
    }
}
