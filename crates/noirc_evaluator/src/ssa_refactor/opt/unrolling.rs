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

impl Ssa {
    pub(crate) fn unroll_loops(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            unroll_loops_in_function(function);
        }
        self
    }
}

fn unroll_loops_in_function(function: &mut Function) {
    // Arbitrary maximum of 10k loops unrolled in a program to prevent looping forever
    // if a bug causes us to continually unroll the same loop.
    let max_loops_unrolled = 10_000;

    for _ in 0..max_loops_unrolled {
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

    panic!("Did not finish unrolling all loops after the maximum of {max_loops_unrolled} loops unrolled")
}

/// Find a loop in the program by finding a node that dominates any predecessor node.
/// The edge where this happens will be the back-edge of the loop.
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

    // Sort loops by block size so that we unroll the smaller, nested loops first.
    loops.sort_by_key(|loop_| loop_.blocks.len());
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

    let mut stack = vec![];
    insert(back_edge_start, &mut stack);

    while let Some(block) = stack.pop() {
        for predecessor in cfg.predecessors(block) {
            insert(predecessor, &mut stack);
        }
    }

    Loop { header, back_edge_start, blocks }
}

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

/// Unroll a single iteration of the loop. Returns true if we should perform another iteration.
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
    println!(
        "Remembering {} <- {} ({:?})",
        first_param,
        induction_value,
        context.function.dfg.get_numeric_constant(induction_value)
    );
    context.values.insert(first_param, induction_value);

    context.inline_instructions_from_block();

    match context.function.dfg[unroll_into].unwrap_terminator() {
        TerminatorInstruction::JmpIf { condition, then_destination, else_destination } => {
            let condition = context.get_value(*condition);

            match context.function.dfg.get_numeric_constant(condition) {
                Some(constant) => {
                    let next_block = if constant.is_zero() { *else_destination } else { *then_destination };

                    // context.insert_block = next_block;
                    context.source_block = context.get_original_block(next_block);

                    context.function.dfg.set_block_terminator(context.insert_block, TerminatorInstruction::Jmp {
                        destination: next_block,
                        arguments: Vec::new(),
                    });

                    // If the next block to jump to is outside of the loop, return None
                    loop_.blocks.contains(&context.source_block).then_some(context)
                },
                None => {
                    // Non-constant loop. We have to reset the then and else destination back to
                    // the original blocks here since we won't be unrolling into the new blocks.
                    context.function.dfg.set_block_terminator(context.insert_block, TerminatorInstruction::JmpIf {
                        condition,
                        then_destination: context.get_original_block(*then_destination),
                        else_destination: context.get_original_block(*else_destination),
                    });

                    None
                },
            }

        }
        other => panic!("Expected loop header to terminate in a JmpIf to the loop body, but found {other:?} instead"),
    }
}

struct LoopIteration<'f> {
    function: &'f mut Function,
    loop_: &'f Loop,
    values: HashMap<ValueId, ValueId>,
    blocks: HashMap<BasicBlockId, BasicBlockId>,
    original_blocks: HashMap<BasicBlockId, BasicBlockId>,
    visited_blocks: HashSet<BasicBlockId>,

    insert_block: BasicBlockId,
    source_block: BasicBlockId,
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
    fn unroll_loop_iteration(mut self) -> (BasicBlockId, ValueId) {
        let mut next_blocks = self.unroll_loop_block();
        next_blocks.retain(|block| self.loop_.blocks.contains(&self.get_original_block(*block)));

        while let Some(block) = next_blocks.pop() {
            self.insert_block = block;
            self.source_block = self.get_original_block(block);

            if !self.visited_blocks.contains(&self.source_block) {
                let mut blocks = self.unroll_loop_block();
                blocks.retain(|block| self.loop_.blocks.contains(&self.get_original_block(*block)));
                next_blocks.append(&mut blocks);
            }
        }

        self.induction_value
            .expect("Expected to find the induction variable by end of loop iteration")
    }

    /// Unroll a single block in the current iteration of the loop
    fn unroll_loop_block(&mut self) -> Vec<BasicBlockId> {
        self.inline_instructions_from_block();
        self.visited_blocks.insert(self.source_block);

        match self.function.dfg[self.insert_block].unwrap_terminator() {
            TerminatorInstruction::JmpIf { condition, then_destination, else_destination } => {
                let condition = self.get_value(*condition);

                match self.function.dfg.get_numeric_constant(condition) {
                    Some(constant) => {
                        let destination =
                            if constant.is_zero() { *else_destination } else { *then_destination };

                        let jmp = TerminatorInstruction::Jmp { destination, arguments: Vec::new() };
                        self.function.dfg.set_block_terminator(self.insert_block, jmp);

                        vec![destination]
                    }
                    None => {
                        vec![*then_destination, *else_destination]
                    }
                }
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

    fn get_value(&self, value: ValueId) -> ValueId {
        self.values.get(&value).copied().unwrap_or(value)
    }

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
                self.values.insert(*param, *new_param);
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
        // assert_eq!(
        //     source_block.parameters().len(),
        //     jmp_args.len(),
        //     "Parameter len != arg len when inlining block {} into {}",
        //     self.source_block,
        //     self.insert_block,
        // );

        // Map each parameter to its new value
        // for (param, arg) in source_block.parameters().iter().zip(jmp_args) {
        //     self.values.insert(*param, *arg);
        // }

        let instructions = source_block.instructions().to_vec();

        // We cannot directly append each instruction since we need to substitute the
        // block parameter values.
        for instruction in instructions {
            self.push_instruction(instruction);
        }

        let mut terminator = self.function.dfg[self.source_block]
            .terminator()
            .expect(
                "Expected each block during the loop unrolling to have a terminator instruction",
            )
            .map_values(|id| self.get_value(id));

        terminator.mutate_blocks(|block| self.get_or_insert_block(block));
        self.function.dfg.set_block_terminator(self.insert_block, terminator);

        println!("Unrolled block: \n{}", self.function);
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
