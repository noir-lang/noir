//! This file contains the loop unrolling pass for the new SSA IR.
//!
//! This pass is divided into a few steps:
//! 1. Find all loops in the program (`find_all_loops`)
//! 2. For each loop:
//!    a. If the loop is in our list of loops that previously failed to unroll, skip it.
//!    b. If we have previously modified any of the blocks in the loop,
//!       restart from step 1 to refresh the context.
//!    c. If not, try to unroll the loop. If successful, remember the modified
//!       blocks. If unsuccessfully either error if the abort_on_error flag is set,
//!       or otherwise remember that the loop failed to unroll and leave it unmodified.
//!
//! Note that this pass also often creates superfluous jmp instructions in the
//! program that will need to be removed by a later simplify cfg pass.
//! Note also that unrolling is skipped for Brillig runtime and as a result
//! we remove reference count instructions because they are only used by Brillig bytecode
use std::collections::HashSet;

use acvm::acir::AcirField;

use crate::{
    errors::RuntimeError,
    ssa::{
        ir::{
            basic_block::BasicBlockId,
            cfg::ControlFlowGraph,
            dfg::{CallStack, DataFlowGraph},
            dom::DominatorTree,
            function::{Function, RuntimeType},
            function_inserter::FunctionInserter,
            instruction::{Instruction, TerminatorInstruction},
            post_order::PostOrder,
            value::ValueId,
        },
        ssa_gen::Ssa,
    },
};
use fxhash::FxHashMap as HashMap;

impl Ssa {
    /// Loop unrolling can return errors, since ACIR functions need to be fully unrolled.
    /// This meta-pass will keep trying to unroll loops and simplifying the SSA until no more errors are found.
    pub(crate) fn unroll_loops_iteratively(mut ssa: Ssa) -> Result<Ssa, RuntimeError> {
        // Try to unroll loops first:
        let mut unroll_errors;
        (ssa, unroll_errors) = ssa.try_to_unroll_loops();

        // Keep unrolling until no more errors are found
        while !unroll_errors.is_empty() {
            let prev_unroll_err_count = unroll_errors.len();

            // Simplify the SSA before retrying

            // Do a mem2reg after the last unroll to aid simplify_cfg
            ssa = ssa.mem2reg();
            ssa = ssa.simplify_cfg();
            // Do another mem2reg after simplify_cfg to aid the next unroll
            ssa = ssa.mem2reg();

            // Unroll again
            (ssa, unroll_errors) = ssa.try_to_unroll_loops();
            // If we didn't manage to unroll any more loops, exit
            if unroll_errors.len() >= prev_unroll_err_count {
                return Err(unroll_errors.swap_remove(0));
            }
        }
        Ok(ssa)
    }

    /// Tries to unroll all loops in each SSA function.
    /// If any loop cannot be unrolled, it is left as-is or in a partially unrolled state.
    /// Returns the ssa along with all unrolling errors encountered
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn try_to_unroll_loops(mut self) -> (Ssa, Vec<RuntimeError>) {
        let mut errors = vec![];
        for function in self.functions.values_mut() {
            // Loop unrolling in brillig can lead to a code explosion currently. This can
            // also be true for ACIR, but we have no alternative to unrolling in ACIR.
            // Brillig also generally prefers smaller code rather than faster code.
            if function.runtime() == RuntimeType::Brillig {
                continue;
            }

            errors.extend(find_all_loops(function).unroll_each_loop(function));
        }
        (self, errors)
    }
}

struct Loop {
    /// The header block of a loop is the block which dominates all the
    /// other blocks in the loop.
    header: BasicBlockId,

    /// The start of the back_edge n -> d is the block n at the end of
    /// the loop that jumps back to the header block d which restarts the loop.
    back_edge_start: BasicBlockId,

    /// All the blocks contained within the loop, including `header` and `back_edge_start`.
    pub(crate) blocks: HashSet<BasicBlockId>,
}

struct Loops {
    /// The loops that failed to be unrolled so that we do not try to unroll them again.
    /// Each loop is identified by its header block id.
    failed_to_unroll: HashSet<BasicBlockId>,

    yet_to_unroll: Vec<Loop>,
    modified_blocks: HashSet<BasicBlockId>,
    cfg: ControlFlowGraph,
}

/// Find a loop in the program by finding a node that dominates any predecessor node.
/// The edge where this happens will be the back-edge of the loop.
fn find_all_loops(function: &Function) -> Loops {
    let cfg = ControlFlowGraph::with_function(function);
    let post_order = PostOrder::with_function(function);
    let mut dom_tree = DominatorTree::with_cfg_and_post_order(&cfg, &post_order);

    let mut loops = vec![];

    for (block, _) in function.dfg.basic_blocks_iter() {
        // These reachable checks wouldn't be needed if we only iterated over reachable blocks
        if dom_tree.is_reachable(block) {
            for predecessor in cfg.predecessors(block) {
                if dom_tree.is_reachable(predecessor) && dom_tree.dominates(block, predecessor) {
                    // predecessor -> block is the back-edge of a loop
                    loops.push(find_blocks_in_loop(block, predecessor, &cfg));
                }
            }
        }
    }

    // Sort loops by block size so that we unroll the larger, outer loops of nested loops first.
    // This is needed because inner loops may use the induction variable from their outer loops in
    // their loop range.
    loops.sort_by_key(|loop_| loop_.blocks.len());

    Loops {
        failed_to_unroll: HashSet::new(),
        yet_to_unroll: loops,
        modified_blocks: HashSet::new(),
        cfg,
    }
}

impl Loops {
    /// Unroll all loops within a given function.
    /// Any loops which fail to be unrolled (due to using non-constant indices) will be unmodified.
    fn unroll_each_loop(mut self, function: &mut Function) -> Vec<RuntimeError> {
        let mut unroll_errors = vec![];
        while let Some(next_loop) = self.yet_to_unroll.pop() {
            // If we've previously modified a block in this loop we need to refresh the context.
            // This happens any time we have nested loops.
            if next_loop.blocks.iter().any(|block| self.modified_blocks.contains(block)) {
                let mut new_context = find_all_loops(function);
                new_context.failed_to_unroll = self.failed_to_unroll;
                return unroll_errors
                    .into_iter()
                    .chain(new_context.unroll_each_loop(function))
                    .collect();
            }

            // Don't try to unroll the loop again if it is known to fail
            if !self.failed_to_unroll.contains(&next_loop.header) {
                match unroll_loop(function, &self.cfg, &next_loop) {
                    Ok(_) => self.modified_blocks.extend(next_loop.blocks),
                    Err(call_stack) => {
                        self.failed_to_unroll.insert(next_loop.header);
                        unroll_errors.push(RuntimeError::UnknownLoopBound { call_stack });
                    }
                }
            }
        }
        unroll_errors
    }
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

/// Unroll a single loop in the function.
/// Returns Err(()) if it failed to unroll and Ok(()) otherwise.
fn unroll_loop(
    function: &mut Function,
    cfg: &ControlFlowGraph,
    loop_: &Loop,
) -> Result<(), CallStack> {
    let mut unroll_into = get_pre_header(cfg, loop_);
    let mut jump_value = get_induction_variable(function, unroll_into)?;

    while let Some(context) = unroll_loop_header(function, loop_, unroll_into, jump_value)? {
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
fn get_induction_variable(function: &Function, block: BasicBlockId) -> Result<ValueId, CallStack> {
    match function.dfg[block].terminator() {
        Some(TerminatorInstruction::Jmp { arguments, call_stack: location, .. }) => {
            // This assumption will no longer be valid if e.g. mutable variables are represented as
            // block parameters. If that becomes the case we'll need to figure out which variable
            // is generally constant and increasing to guess which parameter is the induction
            // variable.
            assert_eq!(arguments.len(), 1, "It is expected that a loop's induction variable is the only block parameter of the loop header");
            let value = arguments[0];
            if function.dfg.get_numeric_constant(value).is_some() {
                Ok(value)
            } else {
                Err(location.clone())
            }
        }
        _ => Err(CallStack::new()),
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
) -> Result<Option<LoopIteration<'a>>, CallStack> {
    // We insert into a fresh block first and move instructions into the unroll_into block later
    // only once we verify the jmpif instruction has a constant condition. If it does not, we can
    // just discard this fresh block and leave the loop unmodified.
    let fresh_block = function.dfg.make_block();

    let mut context = LoopIteration::new(function, loop_, fresh_block, loop_.header);
    let source_block = &context.dfg()[context.source_block];
    assert_eq!(source_block.parameters().len(), 1, "Expected only 1 argument in loop header");

    // Insert the current value of the loop induction variable into our context.
    let first_param = source_block.parameters()[0];
    context.inserter.try_map_value(first_param, induction_value);
    context.inline_instructions_from_block();

    match context.dfg()[fresh_block].unwrap_terminator() {
        TerminatorInstruction::JmpIf { condition, then_destination, else_destination } => {
            let condition = *condition;
            let next_blocks = context.handle_jmpif(condition, *then_destination, *else_destination);

            // If there is only 1 next block the jmpif evaluated to a single known block.
            // This is the expected case and lets us know if we should loop again or not.
            if next_blocks.len() == 1 {
                context.dfg_mut().inline_block(fresh_block, unroll_into);

                // The fresh block is gone now so we're committing to insert into the original
                // unroll_into block from now on.
                context.insert_block = unroll_into;

                Ok(loop_.blocks.contains(&context.source_block).then_some(context))
            } else {
                // If this case is reached the loop either uses non-constant indices or we need
                // another pass, such as mem2reg to resolve them to constants.
                Err(context.inserter.function.dfg.get_value_call_stack(condition))
            }
        }
        other => unreachable!("Expected loop header to terminate in a JmpIf to the loop body, but found {other:?} instead"),
    }
}

/// The context object for each loop iteration.
/// Notably each loop iteration maps each loop block to a fresh, unrolled block.
struct LoopIteration<'f> {
    inserter: FunctionInserter<'f>,
    loop_: &'f Loop,

    /// Maps pre-unrolled block ids from within the loop to new block ids of each loop
    /// block for each loop iteration.
    blocks: HashMap<BasicBlockId, BasicBlockId>,

    /// Maps unrolled block ids back to the original source block ids
    original_blocks: HashMap<BasicBlockId, BasicBlockId>,
    visited_blocks: HashSet<BasicBlockId>,

    insert_block: BasicBlockId,
    source_block: BasicBlockId,

    /// The induction value (and the block it was found in) is the new value for
    /// the variable traditionally called `i` on each iteration of the loop.
    /// This is None until we visit the block which jumps back to the start of the
    /// loop, at which point we record its value and the block it was found in.
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
            inserter: FunctionInserter::new(function),
            loop_,
            insert_block,
            source_block,
            blocks: HashMap::default(),
            original_blocks: HashMap::default(),
            visited_blocks: HashSet::default(),
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

        match self.inserter.function.dfg[self.insert_block].unwrap_terminator() {
            TerminatorInstruction::JmpIf { condition, then_destination, else_destination } => {
                self.handle_jmpif(*condition, *then_destination, *else_destination)
            }
            TerminatorInstruction::Jmp { destination, arguments, call_stack: _ } => {
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
        let condition = self.inserter.resolve(condition);

        match self.dfg().get_numeric_constant(condition) {
            Some(constant) => {
                let destination =
                    if constant.is_zero() { else_destination } else { then_destination };

                self.source_block = self.get_original_block(destination);

                let arguments = Vec::new();
                let jmp = TerminatorInstruction::Jmp {
                    destination,
                    arguments,
                    call_stack: CallStack::new(),
                };
                self.inserter.function.dfg.set_block_terminator(self.insert_block, jmp);
                vec![destination]
            }
            None => vec![then_destination, else_destination],
        }
    }

    /// Translate a block id to a block id in the unrolled loop. If the given
    /// block id is not within the loop, it is returned as-is.
    fn get_or_insert_block(&mut self, block: BasicBlockId) -> BasicBlockId {
        if let Some(new_block) = self.blocks.get(&block) {
            return *new_block;
        }

        // If the block is in the loop we create a fresh block for each iteration
        if self.loop_.blocks.contains(&block) {
            let new_block = self.dfg_mut().make_block_with_parameters_from_block(block);
            self.inserter.remember_block_params_from_block(block, new_block);

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
        let source_block = &self.dfg()[self.source_block];
        let instructions = source_block.instructions().to_vec();

        // We cannot directly append each instruction since we need to substitute any
        // instances of the induction variable or any values that were changed as a result
        // of the new induction variable value.
        for instruction in instructions {
            // Skip reference count instructions since they are only used for brillig, and brillig code is not unrolled
            if !matches!(
                self.dfg()[instruction],
                Instruction::IncrementRc { .. } | Instruction::DecrementRc { .. }
            ) {
                self.inserter.push_instruction(instruction, self.insert_block);
            }
        }
        let mut terminator = self.dfg()[self.source_block]
            .unwrap_terminator()
            .clone()
            .map_values(|value| self.inserter.resolve(value));

        terminator.mutate_blocks(|block| self.get_or_insert_block(block));
        self.inserter.function.dfg.set_block_terminator(self.insert_block, terminator);
    }

    fn dfg(&self) -> &DataFlowGraph {
        &self.inserter.function.dfg
    }

    fn dfg_mut(&mut self) -> &mut DataFlowGraph {
        &mut self.inserter.function.dfg
    }
}

#[cfg(test)]
mod tests {
    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{instruction::BinaryOp, map::Id, types::Type},
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
        builder.insert_constrain(v5, one, None);
        let v6 = builder.insert_binary(v2, BinaryOp::Add, one);
        builder.terminate_with_jmp(b4, vec![v6]);

        // b6
        builder.switch_to_block(b6);
        let v7 = builder.insert_binary(v0, BinaryOp::Add, one);
        builder.terminate_with_jmp(b1, vec![v7]);

        let ssa = builder.finish();
        assert_eq!(ssa.main().reachable_blocks().len(), 7);

        // Expected output:
        //
        // fn main f0 {
        //   b0():
        //     constrain Field 0
        //     constrain Field 0
        //     constrain Field 0
        //     constrain Field 0
        //     jmp b23()
        //   b23():
        //     constrain Field 0
        //     constrain Field 0
        //     constrain Field 0
        //     constrain Field 0
        //     jmp b27()
        //   b27():
        //     constrain Field 0
        //     constrain Field 0
        //     constrain Field 0
        //     constrain Field 0
        //     jmp b31()
        //   b31():
        //     jmp b3()
        //   b3():
        //     return Field 0
        // }
        // The final block count is not 1 because unrolling creates some unnecessary jmps.
        // If a simplify cfg pass is ran afterward, the expected block count will be 1.
        let (ssa, errors) = ssa.try_to_unroll_loops();
        assert_eq!(errors.len(), 0, "All loops should be unrolled");
        assert_eq!(ssa.main().reachable_blocks().len(), 5);
    }

    // Test that the pass can still be run on loops which fail to unroll properly
    #[test]
    fn fail_to_unroll_loop() {
        // fn main f0 {
        //   b0(v0: Field):
        //     jmp b1(v0)
        //   b1(v1: Field):
        //     v2 = lt v1, 5
        //     jmpif v2, then: b2, else: b3
        //   b2():
        //     v3 = add v1, Field 1
        //     jmp b1(v3)
        //   b3():
        //     return Field 0
        // }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let b1 = builder.insert_block();
        let b2 = builder.insert_block();
        let b3 = builder.insert_block();

        let v0 = builder.add_parameter(Type::field());
        let v1 = builder.add_block_parameter(b1, Type::field());

        builder.terminate_with_jmp(b1, vec![v0]);

        builder.switch_to_block(b1);
        let five = builder.field_constant(5u128);
        let v2 = builder.insert_binary(v1, BinaryOp::Lt, five);
        builder.terminate_with_jmpif(v2, b2, b3);

        builder.switch_to_block(b2);
        let one = builder.field_constant(1u128);
        let v3 = builder.insert_binary(v1, BinaryOp::Add, one);
        builder.terminate_with_jmp(b1, vec![v3]);

        builder.switch_to_block(b3);
        let zero = builder.field_constant(0u128);
        builder.terminate_with_return(vec![zero]);

        let ssa = builder.finish();
        assert_eq!(ssa.main().reachable_blocks().len(), 4);

        // Expected that we failed to unroll the loop
        let (_, errors) = ssa.try_to_unroll_loops();
        assert_eq!(errors.len(), 1, "Expected to fail to unroll loop");
    }
}
