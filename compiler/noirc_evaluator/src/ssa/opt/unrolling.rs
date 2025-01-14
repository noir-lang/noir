//! This file contains the loop unrolling pass for the new SSA IR.
//!
//! This pass is divided into a few steps:
//! 1. Find all loops in the program (`find_all_loops`)
//! 2. For each loop:
//!    a. If the loop is in our list of loops that previously failed to unroll, skip it.
//!    b. If we have previously modified any of the blocks in the loop,
//!       restart from step 1 to refresh the context.
//!    c. If not, try to unroll the loop. If successful, remember the modified
//!       blocks. If unsuccessful either error if the abort_on_error flag is set,
//!       or otherwise remember that the loop failed to unroll and leave it unmodified.
//!
//! Note that this pass also often creates superfluous jmp instructions in the
//! program that will need to be removed by a later simplify CFG pass.
//!
//! Note also that unrolling is skipped for Brillig runtime, unless the loops are deemed
//! sufficiently small that inlining can be done without increasing the bytecode.
//!
//! When unrolling ACIR code, we remove reference count instructions because they are
//! only used by Brillig bytecode.
use std::collections::BTreeSet;

use acvm::{acir::AcirField, FieldElement};
use im::HashSet;

use crate::{
    brillig::brillig_gen::convert_ssa_function,
    errors::RuntimeError,
    ssa::{
        ir::{
            basic_block::BasicBlockId,
            call_stack::{CallStack, CallStackId},
            cfg::ControlFlowGraph,
            dfg::DataFlowGraph,
            dom::DominatorTree,
            function::Function,
            function_inserter::{ArrayCache, FunctionInserter},
            instruction::{Binary, BinaryOp, Instruction, InstructionId, TerminatorInstruction},
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
    ///
    /// The `max_bytecode_incr_pct`, when given, is used to limit the growth of the Brillig bytecode size
    /// after unrolling small loops to some percentage of the original loop. For example a value of 150 would
    /// mean the new loop can be 150% (ie. 2.5 times) larger than the original loop. It will still contain
    /// fewer SSA instructions, but that can still result in more Brillig opcodes.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn unroll_loops_iteratively(
        mut self: Ssa,
        max_bytecode_increase_percent: Option<i32>,
    ) -> Result<Ssa, RuntimeError> {
        for (_, function) in self.functions.iter_mut() {
            // Take a snapshot of the function to compare byte size increase,
            // but only if the setting indicates we have to, otherwise skip it.
            let orig_func_and_max_incr_pct = max_bytecode_increase_percent
                .filter(|_| function.runtime().is_brillig())
                .map(|max_incr_pct| (function.clone(), max_incr_pct));

            // Try to unroll loops first:
            let (mut has_unrolled, mut unroll_errors) = function.try_unroll_loops();

            // Keep unrolling until no more errors are found
            while !unroll_errors.is_empty() {
                let prev_unroll_err_count = unroll_errors.len();

                // Simplify the SSA before retrying
                simplify_between_unrolls(function);

                // Unroll again
                let (new_unrolled, new_errors) = function.try_unroll_loops();
                unroll_errors = new_errors;
                has_unrolled |= new_unrolled;

                // If we didn't manage to unroll any more loops, exit
                if unroll_errors.len() >= prev_unroll_err_count {
                    return Err(unroll_errors.swap_remove(0));
                }
            }

            if has_unrolled {
                if let Some((orig_function, max_incr_pct)) = orig_func_and_max_incr_pct {
                    let new_size = brillig_bytecode_size(function);
                    let orig_size = brillig_bytecode_size(&orig_function);
                    if !is_new_size_ok(orig_size, new_size, max_incr_pct) {
                        *function = orig_function;
                    }
                }
            }
        }
        Ok(self)
    }
}

impl Function {
    // Loop unrolling in brillig can lead to a code explosion currently.
    // This can also be true for ACIR, but we have no alternative to unrolling in ACIR.
    // Brillig also generally prefers smaller code rather than faster code,
    // so we only attempt to unroll small loops, which we decide on a case-by-case basis.
    fn try_unroll_loops(&mut self) -> (bool, Vec<RuntimeError>) {
        Loops::find_all(self).unroll_each(self)
    }
}

pub(super) struct Loop {
    /// The header block of a loop is the block which dominates all the
    /// other blocks in the loop.
    pub(super) header: BasicBlockId,

    /// The start of the back_edge n -> d is the block n at the end of
    /// the loop that jumps back to the header block d which restarts the loop.
    back_edge_start: BasicBlockId,

    /// All the blocks contained within the loop, including `header` and `back_edge_start`.
    pub(super) blocks: BTreeSet<BasicBlockId>,
}

pub(super) struct Loops {
    /// The loops that failed to be unrolled so that we do not try to unroll them again.
    /// Each loop is identified by its header block id.
    failed_to_unroll: HashSet<BasicBlockId>,

    pub(super) yet_to_unroll: Vec<Loop>,
    modified_blocks: HashSet<BasicBlockId>,
    pub(super) cfg: ControlFlowGraph,
}

impl Loops {
    /// Find a loop in the program by finding a node that dominates any predecessor node.
    /// The edge where this happens will be the back-edge of the loop.
    ///
    /// For example consider the following SSA of a basic loop:
    /// ```text
    /// main():
    ///   v0 = ... start ...
    ///   v1 = ... end ...
    ///   jmp loop_entry(v0)
    /// loop_entry(i: Field):
    ///   v2 = lt i v1
    ///   jmpif v2, then: loop_body, else: loop_end
    /// loop_body():
    ///   v3 = ... body ...
    ///   v4 = add 1, i
    ///   jmp loop_entry(v4)
    /// loop_end():
    /// ```
    ///
    /// The CFG will look something like this:
    /// ```text
    /// main
    ///   ↓
    /// loop_entry ←---↰
    ///   ↓        ↘   |
    /// loop_end    loop_body
    /// ```
    /// `loop_entry` has two predecessors: `main` and `loop_body`, and it dominates `loop_body`.
    pub(super) fn find_all(function: &Function) -> Self {
        let cfg = ControlFlowGraph::with_function(function);
        let post_order = PostOrder::with_function(function);
        let mut dom_tree = DominatorTree::with_cfg_and_post_order(&cfg, &post_order);

        let mut loops = vec![];

        for (block, _) in function.dfg.basic_blocks_iter() {
            // These reachable checks wouldn't be needed if we only iterated over reachable blocks
            if dom_tree.is_reachable(block) {
                for predecessor in cfg.predecessors(block) {
                    // In the above example, we're looking for when `block` is `loop_entry` and `predecessor` is `loop_body`.
                    if dom_tree.is_reachable(predecessor) && dom_tree.dominates(block, predecessor)
                    {
                        // predecessor -> block is the back-edge of a loop
                        loops.push(Loop::find_blocks_in_loop(block, predecessor, &cfg));
                    }
                }
            }
        }

        // Sort loops by block size so that we unroll the larger, outer loops of nested loops first.
        // This is needed because inner loops may use the induction variable from their outer loops in
        // their loop range. We will start popping loops from the back.
        loops.sort_by_key(|loop_| loop_.blocks.len());

        Self {
            failed_to_unroll: HashSet::default(),
            yet_to_unroll: loops,
            modified_blocks: HashSet::default(),
            cfg,
        }
    }

    /// Unroll all loops within a given function.
    /// Any loops which fail to be unrolled (due to using non-constant indices) will be unmodified.
    /// Returns whether any blocks have been modified
    fn unroll_each(mut self, function: &mut Function) -> (bool, Vec<RuntimeError>) {
        let mut unroll_errors = vec![];
        let mut has_unrolled = false;
        while let Some(next_loop) = self.yet_to_unroll.pop() {
            if function.runtime().is_brillig() && !next_loop.is_small_loop(function, &self.cfg) {
                continue;
            }
            // If we've previously modified a block in this loop we need to refresh the context.
            // This happens any time we have nested loops.
            if next_loop.blocks.iter().any(|block| self.modified_blocks.contains(block)) {
                let mut new_loops = Self::find_all(function);
                new_loops.failed_to_unroll = self.failed_to_unroll;
                let (new_unrolled, new_errors) = new_loops.unroll_each(function);
                return (has_unrolled || new_unrolled, [unroll_errors, new_errors].concat());
            }

            // Don't try to unroll the loop again if it is known to fail
            if !self.failed_to_unroll.contains(&next_loop.header) {
                match next_loop.unroll(function, &self.cfg) {
                    Ok(_) => {
                        has_unrolled = true;
                        self.modified_blocks.extend(next_loop.blocks);
                    }
                    Err(call_stack) => {
                        self.failed_to_unroll.insert(next_loop.header);
                        unroll_errors.push(RuntimeError::UnknownLoopBound { call_stack });
                    }
                }
            }
        }
        (has_unrolled, unroll_errors)
    }
}

impl Loop {
    /// Return each block that is in a loop starting in the given header block.
    /// Expects back_edge_start -> header to be the back edge of the loop.
    fn find_blocks_in_loop(
        header: BasicBlockId,
        back_edge_start: BasicBlockId,
        cfg: &ControlFlowGraph,
    ) -> Self {
        let mut blocks = BTreeSet::default();
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

        Self { header, back_edge_start, blocks }
    }

    /// Find the lower bound of the loop in the pre-header and return it
    /// if it's a numeric constant, which it will be if the previous SSA
    /// steps managed to inline it.
    ///
    /// Consider the following example of a `for i in 0..4` loop:
    /// ```text
    /// brillig(inline) fn main f0 {
    ///   b0(v0: u32):                  // Pre-header
    ///     ...
    ///     jmp b1(u32 0)               // Lower-bound
    ///   b1(v1: u32):                  // Induction variable
    ///     v5 = lt v1, u32 4
    ///     jmpif v5 then: b3, else: b2
    /// ```
    fn get_const_lower_bound(
        &self,
        function: &Function,
        cfg: &ControlFlowGraph,
    ) -> Option<FieldElement> {
        let pre_header = self.get_pre_header(function, cfg).ok()?;
        let jump_value = get_induction_variable(function, pre_header).ok()?;
        function.dfg.get_numeric_constant(jump_value)
    }

    /// Find the upper bound of the loop in the loop header and return it
    /// if it's a numeric constant, which it will be if the previous SSA
    /// steps managed to inline it.
    ///
    /// Consider the following example of a `for i in 0..4` loop:
    /// ```text
    /// brillig(inline) fn main f0 {
    ///   b0(v0: u32):
    ///     ...
    ///     jmp b1(u32 0)
    ///   b1(v1: u32):                  // Loop header
    ///     v5 = lt v1, u32 4           // Upper bound
    ///     jmpif v5 then: b3, else: b2
    /// ```
    pub(super) fn get_const_upper_bound(&self, function: &Function) -> Option<FieldElement> {
        let block = &function.dfg[self.header];
        let instructions = block.instructions();
        if instructions.is_empty() {
            // If the loop condition is constant time, the loop header will be
            // simplified to a simple jump.
            return None;
        }
        assert_eq!(
            instructions.len(),
            1,
            "The header should just compare the induction variable and jump"
        );
        match &function.dfg[instructions[0]] {
            Instruction::Binary(Binary { lhs: _, operator: BinaryOp::Lt, rhs }) => {
                function.dfg.get_numeric_constant(*rhs)
            }
            Instruction::Binary(Binary { lhs: _, operator: BinaryOp::Eq, rhs }) => {
                // `for i in 0..1` is turned into:
                // b1(v0: u32):
                //   v12 = eq v0, u32 0
                //   jmpif v12 then: b3, else: b2
                function.dfg.get_numeric_constant(*rhs).map(|c| c + FieldElement::one())
            }
            other => panic!("Unexpected instruction in header: {other:?}"),
        }
    }

    /// Get the lower and upper bounds of the loop if both are constant numeric values.
    fn get_const_bounds(
        &self,
        function: &Function,
        cfg: &ControlFlowGraph,
    ) -> Option<(FieldElement, FieldElement)> {
        let lower = self.get_const_lower_bound(function, cfg)?;
        let upper = self.get_const_upper_bound(function)?;
        Some((lower, upper))
    }

    /// Unroll a single loop in the function.
    /// Returns Ok(()) if it succeeded, Err(callstack) if it failed,
    /// where the callstack indicates the location of the instruction
    /// that could not be processed, or empty if such information was
    /// not available.
    ///
    /// Consider this example:
    /// ```text
    /// main():
    ///   v0 = 0
    ///   v1 = 2
    ///   jmp loop_entry(v0)
    /// loop_entry(i: Field):
    ///   v2 = lt i v1
    ///   jmpif v2, then: loop_body, else: loop_end
    /// ```
    ///
    /// The first step is to unroll the header by recognizing that jump condition
    /// is a constant, which means it will go to `loop_body`:
    /// ```text
    /// main():
    ///   v0 = 0
    ///   v1 = 2
    ///   v2 = lt v0 v1
    ///   // jmpif v2, then: loop_body, else: loop_end
    ///   jmp dest: loop_body
    /// ```
    ///
    /// Following that we unroll the loop body, which is the next source, replace
    /// the induction variable with the new value created in the body, and have
    /// another go at the header.
    /// ```text
    /// main():
    ///   v0 = 0
    ///   v1 = 2
    ///   v2 = lt v0 v1
    ///   v3 = ... body ...
    ///   v4 = add 1, 0
    ///   jmp loop_entry(v4)
    /// ```
    ///
    /// At the end we reach a point where the condition evaluates to 0 and we jump to the end.
    /// ```text
    /// main():
    ///   v0 = 0
    ///   v1 = 2
    ///   v2 = lt 0
    ///   v3 = ... body ...
    ///   v4 = add 1, v0
    ///   v5 = lt v4 v1
    ///   v6 = ... body ...
    ///   v7 = add v4, 1
    ///   v8 = lt v5 v1
    ///   jmp loop_end
    /// ```
    ///
    /// When e.g. `v8 = lt v5 v1` cannot be evaluated to a constant, the loop signals by returning `Err`
    /// that a few SSA passes are required to evaluate and simplify these values.
    fn unroll(&self, function: &mut Function, cfg: &ControlFlowGraph) -> Result<(), CallStack> {
        let mut unroll_into = self.get_pre_header(function, cfg)?;
        let mut jump_value = get_induction_variable(function, unroll_into)?;
        let mut array_cache = Some(ArrayCache::default());

        while let Some(mut context) = self.unroll_header(function, unroll_into, jump_value)? {
            // The inserter's array cache must be explicitly enabled. This is to
            // confirm that we're inserting in insertion order. This is true here since:
            // 1. We have a fresh inserter for each loop
            // 2. Each loop is unrolled in iteration order
            //
            // Within a loop we do not insert in insertion order. This is fine however since the
            // array cache is buffered with a separate fresh_array_cache which collects arrays
            // but does not deduplicate. When we later call `into_array_cache`, that will merge
            // the fresh cache in with the old one so that each iteration of the loop can cache
            // from previous iterations but not the current iteration.
            context.inserter.set_array_cache(array_cache, unroll_into);
            (unroll_into, jump_value, array_cache) = context.unroll_loop_iteration();
        }

        Ok(())
    }

    /// The loop pre-header is the block that comes before the loop begins. Generally a header block
    /// is expected to have 2 predecessors: the pre-header and the final block of the loop which jumps
    /// back to the beginning. Other predecessors can come from `break` or `continue`.
    pub(super) fn get_pre_header(
        &self,
        function: &Function,
        cfg: &ControlFlowGraph,
    ) -> Result<BasicBlockId, CallStack> {
        let mut pre_header = cfg
            .predecessors(self.header)
            .filter(|predecessor| *predecessor != self.back_edge_start)
            .collect::<Vec<_>>();

        if function.runtime().is_acir() {
            assert_eq!(pre_header.len(), 1);
            Ok(pre_header.remove(0))
        } else if pre_header.len() == 1 {
            Ok(pre_header.remove(0))
        } else {
            // We can come back into the header from multiple blocks, so we can't unroll this.
            Err(CallStack::new())
        }
    }

    /// Unrolls the header block of the loop. This is the block that dominates all other blocks in the
    /// loop and contains the jmpif instruction that lets us know if we should continue looping.
    /// Returns Some(iteration context) if we should perform another iteration.
    fn unroll_header<'a>(
        &'a self,
        function: &'a mut Function,
        unroll_into: BasicBlockId,
        induction_value: ValueId,
    ) -> Result<Option<LoopIteration<'a>>, CallStack> {
        // We insert into a fresh block first and move instructions into the unroll_into block later
        // only once we verify the jmpif instruction has a constant condition. If it does not, we can
        // just discard this fresh block and leave the loop unmodified.
        let fresh_block = function.dfg.make_block();

        let mut context = LoopIteration::new(function, self, fresh_block, self.header);
        let source_block = &context.dfg()[context.source_block];
        assert_eq!(source_block.parameters().len(), 1, "Expected only 1 argument in loop header");

        // Insert the current value of the loop induction variable into our context.
        let first_param = source_block.parameters()[0];
        context.inserter.try_map_value(first_param, induction_value);
        // Copy over all instructions and a fresh terminator.
        context.inline_instructions_from_block();
        // Mutate the terminator if possible so that it points at the iteration block.
        match context.dfg()[fresh_block].unwrap_terminator() {
            TerminatorInstruction::JmpIf { condition, then_destination, else_destination, call_stack } => {
                let condition = *condition;
                let next_blocks = context.handle_jmpif(condition, *then_destination, *else_destination, *call_stack);

                // If there is only 1 next block the jmpif evaluated to a single known block.
                // This is the expected case and lets us know if we should loop again or not.
                if next_blocks.len() == 1 {
                    context.dfg_mut().inline_block(fresh_block, unroll_into);

                    // The fresh block is gone now so we're committing to insert into the original
                    // unroll_into block from now on.
                    context.insert_block = unroll_into;

                    // In the last iteration, `handle_jmpif` will have replaced `context.source_block`
                    // with the `else_destination`, that is, the `loop_end`, which signals that we
                    // have no more loops to unroll, because that block was not part of the loop itself,
                    // ie. it wasn't between `loop_header` and `loop_body`. Otherwise we have the `loop_body`
                    // in `source_block` and can unroll that into the destination.
                    Ok(self.blocks.contains(&context.source_block).then_some(context))
                } else {
                    // If this case is reached the loop either uses non-constant indices or we need
                    // another pass, such as mem2reg to resolve them to constants.
                    Err(context.inserter.function.dfg.get_value_call_stack(condition))
                }
            }
            other => unreachable!("Expected loop header to terminate in a JmpIf to the loop body, but found {other:?} instead"),
        }
    }

    /// Find all reference values which were allocated before the pre-header.
    ///
    /// These are accessible inside the loop body, and they can be involved
    /// in load/store operations that could be eliminated if we unrolled the
    /// body into the pre-header.
    ///
    /// Consider this loop:
    /// ```text
    /// let mut sum = 0;
    /// let mut arr = &[];
    /// for i in 0..3 {
    ///     sum = sum + i;
    ///     arr.push_back(sum)
    /// }
    /// sum
    /// ```
    ///
    /// The SSA has a load+store for the `sum` and a load+push for the `arr`:
    /// ```text
    /// b0(v0: u32):
    ///   v2 = allocate -> &mut u32     // reference allocated for `sum`
    ///   store u32 0 at v2             // initial value for `sum`
    ///   v4 = allocate -> &mut u32     // reference allocated for the length of `arr`
    ///   store u32 0 at v4             // initial length of `arr`
    ///   inc_rc [] of u32              // storage for `arr`
    ///   v6 = allocate -> &mut [u32]   // reference allocated to point at the storage of `arr`
    ///   store [] of u32 at v6         // initial value for the storage of `arr`
    ///   jmp b1(u32 0)                 // start looping from 0
    /// b1(v1: u32):                    // `i` induction variable
    ///   v8 = lt v1, u32 3             // loop until 3
    ///   jmpif v8 then: b3, else: b2
    /// b3():
    ///   v11 = load v2 -> u32          // load `sum`
    ///   v12 = add v11, v1             // add `i` to `sum`
    ///   store v12 at v2               // store updated `sum`
    ///   v13 = load v4 -> u32          // load length of `arr`
    ///   v14 = load v6 -> [u32]        // load storage of `arr`
    ///   v16, v17 = call slice_push_back(v13, v14, v12) -> (u32, [u32]) // builtin to push, will store to storage and length references
    ///   v19 = add v1, u32 1           // increase `arr`
    ///   jmp b1(v19)                   // back-edge of the loop
    /// b2():                           // after the loop
    ///   v9 = load v2 -> u32           // read final value of `sum`
    /// ```
    ///
    /// We won't always find load _and_ store ops (e.g. the push above doesn't come with a store),
    /// but it's likely that mem2reg could eliminate a lot of the loads we can find, so we can
    /// use this as an approximation of the gains we would see.
    fn find_pre_header_reference_values(
        &self,
        function: &Function,
        cfg: &ControlFlowGraph,
    ) -> Option<HashSet<ValueId>> {
        // We need to traverse blocks from the pre-header up to the block entry point.
        let pre_header = self.get_pre_header(function, cfg).ok()?;
        let function_entry = function.entry_block();

        // The algorithm in `find_blocks_in_loop` expects to collect the blocks between the header and the back-edge of the loop,
        // but technically works the same if we go from the pre-header up to the function entry as well.
        let blocks = Self::find_blocks_in_loop(function_entry, pre_header, cfg).blocks;

        // Collect allocations in all blocks above the header.
        let allocations = blocks.iter().flat_map(|block| {
            let instructions = function.dfg[*block].instructions().iter();
            instructions
                .filter(|i| matches!(&function.dfg[**i], Instruction::Allocate))
                // Get the value into which the allocation was stored.
                .map(|i| function.dfg.instruction_results(*i)[0])
        });

        // Collect reference parameters of the function itself.
        let params =
            function.parameters().iter().filter(|p| function.dfg.value_is_reference(**p)).copied();

        Some(params.chain(allocations).collect())
    }

    /// Count the number of load and store instructions of specific variables in the loop.
    ///
    /// Returns `(loads, stores)` in case we want to differentiate in the estimates.
    fn count_loads_and_stores(
        &self,
        function: &Function,
        refs: &HashSet<ValueId>,
    ) -> (usize, usize) {
        let mut loads = 0;
        let mut stores = 0;
        for block in &self.blocks {
            for instruction in function.dfg[*block].instructions() {
                match &function.dfg[*instruction] {
                    Instruction::Load { address } if refs.contains(address) => {
                        loads += 1;
                    }
                    Instruction::Store { address, .. } if refs.contains(address) => {
                        stores += 1;
                    }
                    _ => {}
                }
            }
        }
        (loads, stores)
    }

    /// Count the number of instructions in the loop, including the terminating jumps.
    fn count_all_instructions(&self, function: &Function) -> usize {
        let iter = self.blocks.iter().map(|block| {
            let block = &function.dfg[*block];
            block.instructions().len() + block.terminator().is_some() as usize
        });
        iter.sum()
    }

    /// Count the number of increments to the induction variable.
    /// It should be one, but it can be duplicated.
    /// The increment should be in the block where the back-edge was found.
    fn count_induction_increments(&self, function: &Function) -> usize {
        let back = &function.dfg[self.back_edge_start];
        let header = &function.dfg[self.header];
        let induction_var = header.parameters()[0];

        back.instructions()
            .iter()
            .filter(|instruction| {
                let instruction = &function.dfg[**instruction];
                matches!(instruction,
                    Instruction::Binary(Binary { lhs, operator: BinaryOp::Add { .. }, rhs: _ })
                        if *lhs == induction_var
                )
            })
            .count()
    }

    /// Decide if this loop is small enough that it can be inlined in a way that the number
    /// of unrolled instructions times the number of iterations would result in smaller bytecode
    /// than if we keep the loops with their overheads.
    fn is_small_loop(&self, function: &Function, cfg: &ControlFlowGraph) -> bool {
        self.boilerplate_stats(function, cfg).map(|s| s.is_small()).unwrap_or_default()
    }

    /// Collect boilerplate stats if we can figure out the upper and lower bounds of the loop,
    /// and the loop doesn't have multiple back-edges from breaks and continues.
    fn boilerplate_stats(
        &self,
        function: &Function,
        cfg: &ControlFlowGraph,
    ) -> Option<BoilerplateStats> {
        let (lower, upper) = self.get_const_bounds(function, cfg)?;
        let lower = lower.try_to_u64()?;
        let upper = upper.try_to_u64()?;
        let refs = self.find_pre_header_reference_values(function, cfg)?;

        let (loads, stores) = self.count_loads_and_stores(function, &refs);
        let increments = self.count_induction_increments(function);
        let all_instructions = self.count_all_instructions(function);

        Some(BoilerplateStats {
            iterations: (upper - lower) as usize,
            loads,
            stores,
            increments,
            all_instructions,
        })
    }
}

/// All the instructions in the following example are boilerplate:
/// ```text
/// brillig(inline) fn main f0 {
///   b0(v0: u32):
///     ...
///     jmp b1(u32 0)
///   b1(v1: u32):
///     v5 = lt v1, u32 4
///     jmpif v5 then: b3, else: b2
///   b3():
///     ...
///     v11 = add v1, u32 1
///     jmp b1(v11)
///   b2():
///     ...
/// }
/// ```
#[derive(Debug)]
struct BoilerplateStats {
    /// Number of iterations in the loop.
    iterations: usize,
    /// Number of loads  pre-header references in the loop.
    loads: usize,
    /// Number of stores into pre-header references in the loop.
    stores: usize,
    /// Number of increments to the induction variable (might be duplicated).
    increments: usize,
    /// Number of instructions in the loop, including boilerplate,
    /// but excluding the boilerplate which is outside the loop.
    all_instructions: usize,
}

impl BoilerplateStats {
    /// Instruction count if we leave the loop as-is.
    /// It's the instructions in the loop, plus the one to kick it off in the pre-header.
    fn baseline_instructions(&self) -> usize {
        self.all_instructions + 1
    }

    /// Estimated number of _useful_ instructions, which is the ones in the loop
    /// minus all in-loop boilerplate.
    fn useful_instructions(&self) -> usize {
        // Two jumps + plus the comparison with the upper bound
        let boilerplate = 3;
        // Be conservative and only assume that mem2reg gets rid of load followed by store.
        // NB we have not checked that these are actual pairs.
        let load_and_store = self.loads.min(self.stores) * 2;
        self.all_instructions - self.increments - load_and_store - boilerplate
    }

    /// Estimated number of instructions if we unroll the loop.
    fn unrolled_instructions(&self) -> usize {
        self.useful_instructions() * self.iterations
    }

    /// A small loop is where if we unroll it into the pre-header then considering the
    /// number of iterations we still end up with a smaller bytecode than if we leave
    /// the blocks in tact with all the boilerplate involved in jumping, and the extra
    /// reference access instructions.
    fn is_small(&self) -> bool {
        self.unrolled_instructions() < self.baseline_instructions()
    }
}

/// Return the induction value of the current iteration of the loop, from the given block's jmp arguments.
///
/// Expects the current block to terminate in `jmp h(N)` where h is the loop header and N is
/// a Field value. Returns an `Err` if this isn't the case.
///
/// Consider the following example:
/// ```text
/// main():
///   v0 = ... start ...
///   v1 = ... end ...
///   jmp loop_entry(v0)
/// loop_entry(i: Field):
///   ...
/// ```
/// We're looking for the terminating jump of the `main` predecessor of `loop_entry`.
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
                let call_stack = function.dfg.get_call_stack(*location);
                Err(call_stack)
            }
        }
        Some(terminator) => Err(function.dfg.get_call_stack(terminator.call_stack())),
        None => Err(CallStack::new()),
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
    fn unroll_loop_iteration(mut self) -> (BasicBlockId, ValueId, Option<ArrayCache>) {
        let mut next_blocks = self.unroll_loop_block();

        while let Some(block) = next_blocks.pop() {
            self.insert_block = block;
            self.source_block = self.get_original_block(block);

            if !self.visited_blocks.contains(&self.source_block) {
                let mut blocks = self.unroll_loop_block();
                next_blocks.append(&mut blocks);
            }
        }
        // After having unrolled all blocks in the loop body, we must know how to get back to the header;
        // this is also the block into which we have to unroll into next.
        let (end_block, induction_value) = self
            .induction_value
            .expect("Expected to find the induction variable by end of loop iteration");

        (end_block, induction_value, self.inserter.into_array_cache())
    }

    /// Unroll a single block in the current iteration of the loop
    fn unroll_loop_block(&mut self) -> Vec<BasicBlockId> {
        let mut next_blocks = self.unroll_loop_block_helper();
        // Guarantee that the next blocks we set up to be unrolled, are actually part of the loop,
        // which we recorded while inlining the instructions of the blocks already processed.
        next_blocks.retain(|block| {
            let b = self.get_original_block(*block);
            self.loop_.blocks.contains(&b)
        });
        next_blocks
    }

    /// Unroll a single block in the current iteration of the loop
    fn unroll_loop_block_helper(&mut self) -> Vec<BasicBlockId> {
        // Copy instructions from the loop body to the unroll destination, replacing the terminator.
        self.inline_instructions_from_block();
        self.visited_blocks.insert(self.source_block);

        match self.inserter.function.dfg[self.insert_block].unwrap_terminator() {
            TerminatorInstruction::JmpIf {
                condition,
                then_destination,
                else_destination,
                call_stack,
            } => self.handle_jmpif(*condition, *then_destination, *else_destination, *call_stack),
            TerminatorInstruction::Jmp { destination, arguments, call_stack: _ } => {
                if self.get_original_block(*destination) == self.loop_.header {
                    // We found the back-edge of the loop.
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
    /// constant and we can safely take only the given branch. In this case the method
    /// also replaces the terminator of the insert block (a.k.a fresh block) to be a `Jmp`,
    /// and changes the source block in the context for the next iteration to be the
    /// destination indicated by the constant condition (ie. the `then` or the `else`).
    fn handle_jmpif(
        &mut self,
        condition: ValueId,
        then_destination: BasicBlockId,
        else_destination: BasicBlockId,
        call_stack: CallStackId,
    ) -> Vec<BasicBlockId> {
        let condition = self.inserter.resolve(condition);

        match self.dfg().get_numeric_constant(condition) {
            Some(constant) => {
                let destination =
                    if constant.is_zero() { else_destination } else { then_destination };

                self.source_block = self.get_original_block(destination);

                let arguments = Vec::new();
                let jmp = TerminatorInstruction::Jmp { destination, arguments, call_stack };
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

    /// Find the original ID of a block that replaced it.
    fn get_original_block(&self, block: BasicBlockId) -> BasicBlockId {
        self.original_blocks.get(&block).copied().unwrap_or(block)
    }

    /// Copy over instructions from the source into the insert block,
    /// while simplifying instructions and keeping track of original block IDs.
    fn inline_instructions_from_block(&mut self) {
        let source_block = &self.dfg()[self.source_block];
        let instructions = source_block.instructions().to_vec();

        // We cannot directly append each instruction since we need to substitute any
        // instances of the induction variable or any values that were changed as a result
        // of the new induction variable value.
        for instruction in instructions {
            // Reference counting is only used by Brillig, ACIR doesn't need them.
            if self.inserter.function.runtime().is_acir() && self.is_refcount(instruction) {
                continue;
            }
            self.inserter.push_instruction(instruction, self.insert_block);
        }
        let mut terminator = self.dfg()[self.source_block].unwrap_terminator().clone();

        terminator.map_values_mut(|value| self.inserter.resolve(value));

        // Replace the blocks in the terminator with fresh one with the same parameters,
        // while remembering which were the original block IDs.
        terminator.mutate_blocks(|block| self.get_or_insert_block(block));
        self.inserter.function.dfg.set_block_terminator(self.insert_block, terminator);
    }

    /// Is the instruction an `Rc`?
    fn is_refcount(&self, instruction: InstructionId) -> bool {
        matches!(
            self.dfg()[instruction],
            Instruction::IncrementRc { .. } | Instruction::DecrementRc { .. }
        )
    }

    fn dfg(&self) -> &DataFlowGraph {
        &self.inserter.function.dfg
    }

    fn dfg_mut(&mut self) -> &mut DataFlowGraph {
        &mut self.inserter.function.dfg
    }
}

/// Unrolling leaves some duplicate instructions which can potentially be removed.
fn simplify_between_unrolls(function: &mut Function) {
    // Do a mem2reg after the last unroll to aid simplify_cfg
    function.mem2reg();
    function.simplify_function();
    // Do another mem2reg after simplify_cfg to aid the next unroll
    function.mem2reg();
}

/// Convert the function to Brillig bytecode and return the resulting size.
fn brillig_bytecode_size(function: &Function) -> usize {
    // We need to do some SSA passes in order for the conversion to be able to go ahead,
    // otherwise we can hit `unreachable!()` instructions in `convert_ssa_instruction`.
    // Creating a clone so as not to modify the originals.
    let mut temp = function.clone();

    // Might as well give it the best chance.
    simplify_between_unrolls(&mut temp);

    // This is to try to prevent hitting ICE.
    temp.dead_instruction_elimination(false);

    convert_ssa_function(&temp, false).byte_code.len()
}

/// Decide if the new bytecode size is acceptable, compared to the original.
///
/// The maximum increase can be expressed as a negative value if we demand a decrease.
/// (Values -100 and under mean the new size should be 0).
fn is_new_size_ok(orig_size: usize, new_size: usize, max_incr_pct: i32) -> bool {
    let max_size_pct = 100i32.saturating_add(max_incr_pct).max(0) as usize;
    let max_size = orig_size.saturating_mul(max_size_pct);
    new_size.saturating_mul(100) <= max_size
}

#[cfg(test)]
mod tests {
    use acvm::FieldElement;
    use test_case::test_case;

    use crate::errors::RuntimeError;
    use crate::ssa::{ir::value::ValueId, opt::assert_normalized_ssa_equals, Ssa};

    use super::{is_new_size_ok, BoilerplateStats, Loops};

    /// Tries to unroll all loops in each SSA function once, calling the `Function` directly,
    /// bypassing the iterative loop done by the SSA which does further optimisations.
    ///
    /// If any loop cannot be unrolled, it is left as-is or in a partially unrolled state.
    fn try_unroll_loops(mut ssa: Ssa) -> (Ssa, Vec<RuntimeError>) {
        let mut errors = vec![];
        for function in ssa.functions.values_mut() {
            errors.extend(function.try_unroll_loops().1);
        }
        (ssa, errors)
    }

    #[test]
    fn unroll_nested_loops() {
        // fn main() {
        //     for i in 0..3 {
        //         for j in 0..4 {
        //             assert(i + j > 10);
        //         }
        //     }
        // }
        let src = "
            acir(inline) fn main f0 {
                b0():
                    jmp b1(Field 0)
                b1(v0: Field):  // header of outer loop
                    v1 = lt v0, Field 3
                    jmpif v1 then: b2, else: b3
                b2():
                    jmp b4(Field 0)
                b4(v2: Field):  // header of inner loop
                    v3 = lt v2, Field 4
                    jmpif v3 then: b5, else: b6
                b5():
                    v4 = add v0, v2
                    v5 = lt Field 10, v4
                    constrain v5 == Field 1
                    v6 = add v2, Field 1
                    jmp b4(v6)
                b6(): // end of inner loop
                    v7 = add v0, Field 1
                    jmp b1(v7)
                b3(): // end of outer loop
                    return Field 0
            }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let expected = "
            acir(inline) fn main f0 {
              b0():
                constrain u1 0 == Field 1
                constrain u1 0 == Field 1
                constrain u1 0 == Field 1
                constrain u1 0 == Field 1
                jmp b1()
              b1():
                constrain u1 0 == Field 1
                constrain u1 0 == Field 1
                constrain u1 0 == Field 1
                constrain u1 0 == Field 1
                jmp b2()
              b2():
                constrain u1 0 == Field 1
                constrain u1 0 == Field 1
                constrain u1 0 == Field 1
                constrain u1 0 == Field 1
                jmp b3()
              b3():
                jmp b4()
              b4():
                return Field 0
            }
        ";

        // The final block count is not 1 because unrolling creates some unnecessary jmps.
        // If a simplify cfg pass is ran afterward, the expected block count will be 1.
        let (ssa, errors) = try_unroll_loops(ssa);
        assert_eq!(errors.len(), 0, "All loops should be unrolled");
        assert_eq!(ssa.main().reachable_blocks().len(), 5);

        assert_normalized_ssa_equals(ssa, expected);
    }

    // Test that the pass can still be run on loops which fail to unroll properly
    #[test]
    fn fail_to_unroll_loop() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            jmp b1(v0)
          b1(v1: Field):
            v2 = lt v1, Field 5
            jmpif v2 then: b2, else: b3
          b2():
            v3 = add v1, Field 1
            jmp b1(v3)
          b3():
            return Field 0
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        // Sanity check
        assert_eq!(ssa.main().reachable_blocks().len(), 4);

        // Expected that we failed to unroll the loop
        let (_, errors) = try_unroll_loops(ssa);
        assert_eq!(errors.len(), 1, "Expected to fail to unroll loop");
    }

    #[test]
    fn test_get_const_bounds() {
        let ssa = brillig_unroll_test_case();
        let function = ssa.main();
        let loops = Loops::find_all(function);
        assert_eq!(loops.yet_to_unroll.len(), 1);

        let (lower, upper) = loops.yet_to_unroll[0]
            .get_const_bounds(function, &loops.cfg)
            .expect("bounds are numeric const");

        assert_eq!(lower, FieldElement::from(0u32));
        assert_eq!(upper, FieldElement::from(4u32));
    }

    #[test]
    fn test_find_pre_header_reference_values() {
        let ssa = brillig_unroll_test_case();
        let function = ssa.main();
        let mut loops = Loops::find_all(function);
        let loop0 = loops.yet_to_unroll.pop().unwrap();

        let refs = loop0.find_pre_header_reference_values(function, &loops.cfg).unwrap();
        assert_eq!(refs.len(), 1);
        assert!(refs.contains(&ValueId::test_new(2)));

        let (loads, stores) = loop0.count_loads_and_stores(function, &refs);
        assert_eq!(loads, 1);
        assert_eq!(stores, 1);

        let all = loop0.count_all_instructions(function);
        assert_eq!(all, 7);
    }

    #[test]
    fn test_boilerplate_stats() {
        let ssa = brillig_unroll_test_case();
        let stats = loop0_stats(&ssa);
        assert_eq!(stats.iterations, 4);
        assert_eq!(stats.all_instructions, 2 + 5); // Instructions in b1 and b3
        assert_eq!(stats.increments, 1);
        assert_eq!(stats.loads, 1);
        assert_eq!(stats.stores, 1);
        assert_eq!(stats.useful_instructions(), 1); // Adding to sum
        assert_eq!(stats.baseline_instructions(), 8);
        assert!(stats.is_small());
    }

    #[test]
    fn test_boilerplate_stats_6470() {
        let ssa = brillig_unroll_test_case_6470(3);
        let stats = loop0_stats(&ssa);
        assert_eq!(stats.iterations, 3);
        assert_eq!(stats.all_instructions, 2 + 8); // Instructions in b1 and b3
        assert_eq!(stats.increments, 2);
        assert_eq!(stats.loads, 1);
        assert_eq!(stats.stores, 1);
        assert_eq!(stats.useful_instructions(), 3); // array get, add, array set
        assert_eq!(stats.baseline_instructions(), 11);
        assert!(stats.is_small());
    }

    /// Test that we can unroll a small loop.
    #[test]
    fn test_brillig_unroll_small_loop() {
        let ssa = brillig_unroll_test_case();

        // Expectation taken by compiling the Noir program as ACIR,
        // ie. by removing the `unconstrained` from `main`.
        let expected = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v1 = allocate -> &mut u32
            store u32 0 at v1
            v3 = load v1 -> u32
            store v3 at v1
            v4 = load v1 -> u32
            v6 = add v4, u32 1
            store v6 at v1
            v7 = load v1 -> u32
            v9 = add v7, u32 2
            store v9 at v1
            v10 = load v1 -> u32
            v12 = add v10, u32 3
            store v12 at v1
            jmp b1()
          b1():
            v13 = load v1 -> u32
            v14 = eq v13, v0
            constrain v13 == v0
            return
        }
        ";

        let (ssa, errors) = try_unroll_loops(ssa);
        assert_eq!(errors.len(), 0, "Unroll should have no errors");
        assert_eq!(ssa.main().reachable_blocks().len(), 2, "The loop should be unrolled");

        assert_normalized_ssa_equals(ssa, expected);
    }

    /// Test that we can unroll the loop in the ticket if we don't have too many iterations.
    #[test]
    fn test_brillig_unroll_6470_small() {
        // Few enough iterations so that we can perform the unroll.
        let ssa = brillig_unroll_test_case_6470(3);
        let (ssa, errors) = try_unroll_loops(ssa);
        assert_eq!(errors.len(), 0, "Unroll should have no errors");
        assert_eq!(ssa.main().reachable_blocks().len(), 2, "The loop should be unrolled");

        // The IDs are shifted by one compared to what the ACIR version printed.
        let expected = "
        brillig(inline) fn main f0 {
          b0(v0: [u64; 6]):
            inc_rc v0
            v2 = make_array [u64 0, u64 0, u64 0, u64 0, u64 0, u64 0] : [u64; 6]
            inc_rc v2
            v3 = allocate -> &mut [u64; 6]
            store v2 at v3
            v4 = load v3 -> [u64; 6]
            v6 = array_get v0, index u32 0 -> u64
            v8 = add v6, u64 1
            v9 = array_set v4, index u32 0, value v8
            store v9 at v3
            v10 = load v3 -> [u64; 6]
            v12 = array_get v0, index u32 1 -> u64
            v13 = add v12, u64 1
            v14 = array_set v10, index u32 1, value v13
            store v14 at v3
            v15 = load v3 -> [u64; 6]
            v17 = array_get v0, index u32 2 -> u64
            v18 = add v17, u64 1
            v19 = array_set v15, index u32 2, value v18
            store v19 at v3
            jmp b1()
          b1():
            v20 = load v3 -> [u64; 6]
            dec_rc v0
            return v20
        }
        ";
        assert_normalized_ssa_equals(ssa, expected);
    }

    /// Test that with more iterations it's not unrolled.
    #[test]
    fn test_brillig_unroll_6470_large() {
        // More iterations than it can unroll
        let parse_ssa = || brillig_unroll_test_case_6470(6);
        let ssa = parse_ssa();
        let stats = loop0_stats(&ssa);
        assert!(!stats.is_small(), "the loop should be considered large");

        let (ssa, errors) = try_unroll_loops(ssa);
        assert_eq!(errors.len(), 0, "Unroll should have no errors");
        // Check that it's still the original
        assert_normalized_ssa_equals(ssa, parse_ssa().to_string().as_str());
    }

    #[test]
    fn test_brillig_unroll_iteratively_respects_max_increase() {
        let ssa = brillig_unroll_test_case();
        let ssa = ssa.unroll_loops_iteratively(Some(-90)).unwrap();
        // Check that it's still the original
        assert_normalized_ssa_equals(ssa, brillig_unroll_test_case().to_string().as_str());
    }

    #[test]
    fn test_brillig_unroll_iteratively_with_large_max_increase() {
        let ssa = brillig_unroll_test_case();
        let ssa = ssa.unroll_loops_iteratively(Some(50)).unwrap();
        // Check that it did the unroll
        assert_eq!(ssa.main().reachable_blocks().len(), 2, "The loop should be unrolled");
    }

    /// Test that `break` and `continue` stop unrolling without any panic.
    #[test]
    fn test_brillig_unroll_break_and_continue() {
        // unconstrained fn main() {
        //     let mut count = 0;
        //     for i in 0..10 {
        //         if i == 2 {
        //             continue;
        //         }
        //         if i == 5 {
        //             break;
        //         }
        //         count += 1;
        //     }
        //     assert(count == 4);
        // }
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v1 = allocate -> &mut Field
            store Field 0 at v1
            jmp b1(u32 0)
          b1(v0: u32):
            v5 = lt v0, u32 10
            jmpif v5 then: b2, else: b6
          b2():
            v7 = eq v0, u32 2
            jmpif v7 then: b7, else: b3
          b3():
            v9 = eq v0, u32 5
            jmpif v9 then: b5, else: b4
          b4():
            v10 = load v1 -> Field
            v12 = add v10, Field 1
            store v12 at v1
            v14 = add v0, u32 1
            jmp b1(v14)
          b5():
            jmp b6()
          b6():
            v15 = load v1 -> Field
            v17 = eq v15, Field 4
            constrain v15 == Field 4
            return
          b7():
            v18 = add v0, u32 1
            jmp b1(v18)
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let (ssa, errors) = try_unroll_loops(ssa);
        assert_eq!(errors.len(), 0, "Unroll should have no errors");
        assert_normalized_ssa_equals(ssa, src);
    }

    /// Simple test loop:
    /// ```text
    /// unconstrained fn main(sum: u32) {
    ///     assert(loop(0, 4) == sum);
    /// }
    ///
    /// fn loop(from: u32, to: u32) -> u32 {
    ///      let mut sum = 0;
    ///      for i in from..to {
    ///          sum = sum + i;
    ///      }
    ///      sum
    ///  }
    /// ```
    /// We can check what the ACIR unrolling behavior would be by
    /// removing the `unconstrained` from the `main` function and
    /// compiling the program with `nargo --test-program . compile --show-ssa`.
    fn brillig_unroll_test_case() -> Ssa {
        let src = "
        // After `static_assert` and `assert_constant`:
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v2 = allocate -> &mut u32
            store u32 0 at v2
            jmp b1(u32 0)
          b1(v1: u32):
            v5 = lt v1, u32 4
            jmpif v5 then: b3, else: b2
          b3():
            v8 = load v2 -> u32
            v9 = add v8, v1
            store v9 at v2
            v11 = add v1, u32 1
            jmp b1(v11)
          b2():
            v6 = load v2 -> u32
            v7 = eq v6, v0
            constrain v6 == v0
            return
        }
        ";
        Ssa::from_str(src).unwrap()
    }

    /// Test case from #6470:
    /// ```text
    /// unconstrained fn __validate_gt_remainder(a_u60: [u64; 6]) -> [u64; 6] {
    ///     let mut result_u60: [u64; 6] = [0; 6];
    ///
    ///     for i in 0..6 {
    ///         result_u60[i] = a_u60[i] + 1;
    ///     }
    ///
    ///     result_u60
    /// }
    /// ```
    /// The `num_iterations` parameter can be used to make it more costly to inline.
    fn brillig_unroll_test_case_6470(num_iterations: usize) -> Ssa {
        let src = format!(
            "
        // After `static_assert` and `assert_constant`:
        brillig(inline) fn main f0 {{
          b0(v0: [u64; 6]):
            inc_rc v0
            v3 = make_array [u64 0, u64 0, u64 0, u64 0, u64 0, u64 0] : [u64; 6]
            inc_rc v3
            v4 = allocate -> &mut [u64; 6]
            store v3 at v4
            jmp b1(u32 0)
          b1(v1: u32):
            v7 = lt v1, u32 {num_iterations}
            jmpif v7 then: b3, else: b2
          b3():
            v9 = load v4 -> [u64; 6]
            v10 = array_get v0, index v1 -> u64
            v12 = add v10, u64 1
            v13 = array_set v9, index v1, value v12
            v15 = add v1, u32 1
            store v13 at v4
            v16 = add v1, u32 1 // duplicate
            jmp b1(v16)
          b2():
            v8 = load v4 -> [u64; 6]
            dec_rc v0
            return v8
        }}
        "
        );
        Ssa::from_str(&src).unwrap()
    }

    // Boilerplate stats of the first loop in the SSA.
    fn loop0_stats(ssa: &Ssa) -> BoilerplateStats {
        let function = ssa.main();
        let mut loops = Loops::find_all(function);
        let loop0 = loops.yet_to_unroll.pop().expect("there should be a loop");
        loop0.boilerplate_stats(function, &loops.cfg).expect("there should be stats")
    }

    #[test_case(1000, 700, 50, true; "size decreased")]
    #[test_case(1000, 1500, 50, true; "size increased just by the max")]
    #[test_case(1000, 1501, 50, false; "size increased over the max")]
    #[test_case(1000, 700, -50, false; "size decreased but not enough")]
    #[test_case(1000, 250, -50, true; "size decreased over expectations")]
    #[test_case(1000, 250, -1250, false; "demanding more than minus 100 is handled")]
    fn test_is_new_size_ok(old: usize, new: usize, max: i32, ok: bool) {
        assert_eq!(is_new_size_ok(old, new, max), ok);
    }
}
