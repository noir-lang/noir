//! This file contains the loop unrolling pass for the new SSA IR.
//!
//! This pass is divided into a few steps:
//! 1. Find all loops in the program (`find_all_loops`)
//! 2. For each loop:
//!    1. If the loop is in our list of loops that previously failed to unroll, skip it.
//!    2. If we have previously modified any of the blocks in the loop,
//!       restart from step 1 to refresh the context.
//!    3. If not, try to unroll the loop. If successful, remember the modified
//!       blocks. If unsuccessful either error if the abort_on_error flag is set,
//!       or otherwise remember that the loop failed to unroll and leave it unmodified.
//!
//! Note that this pass also often creates superfluous jmp instructions in the
//! program that will need to be removed by a later simplify CFG pass.
//!
//! ACIR/Brillig differences within this pass:
//!   - Brillig functions may contain loops using `continue` or `break` which this pass does not
//!     support the unrolling of (running the pass on such functions is not an error).
//!   - Brillig functions only have small loops unrolled, where a small loop is defined as a loop
//!     which, when unrolled, is estimated to have the same or fewer total instructions as it
//!     has when not unrolled.
//!   - Unrolling may be reverted for brillig functions if the increase in instruction count is
//!     greater than `max_bytecode_increase_percent` (if set).
//!   - Differing post-conditions (see below).
//!
//! Relevance to other passes:
//!   - Loop unrolling is a required pass for constrained code (ACIR functions) since ACIR itself
//!     does not contain any branching constructs.
//!   - Loop unrolling must occur before flattening on ACIR functions
//!   - Since unrolling may fail on loops that fail to unroll, any simplification passes before it
//!     which affect loop bounds may affect which code fails to compile. One important optimization
//!     in this category is mem2reg which may simplify mutable variables, including those potentially
//!     used as loop bounds, into a form which loop unrolling may better identify.
//!
//! Conditions:
//!   - Pre-condition: All loop headers have a single induction variable.
//!   - Pre-condition: The SSA must be optimized to a point at which loop bounds are known.
//!     Some passes such as inlining and mem2reg are de-facto required before running this pass on arbitrary noir code.
//!   - Post-condition (ACIR-only): All loops in ACIR functions should be unrolled when this pass is
//!     completed successfully. Any loops that are not unrolled (e.g. because of a mutable variable
//!     used in the loop condition whose value is unknown) will result in an error.
//!   - Post-condition (Brillig-only): If `max_bytecode_increase_percent` is set, the instruction count
//!     of each function should increase by no more than that percentage compared to before the pass.
use std::collections::{BTreeSet, HashSet};

use acvm::acir::AcirField;
use noirc_errors::call_stack::{CallStack, CallStackId};

use crate::{
    errors::RuntimeError,
    ssa::{
        ir::{
            basic_block::BasicBlockId,
            cfg::ControlFlowGraph,
            dfg::DataFlowGraph,
            dom::DominatorTree,
            function::Function,
            function_inserter::FunctionInserter,
            instruction::{Binary, BinaryOp, Instruction, TerminatorInstruction},
            integer::IntegerConstant,
            post_order::PostOrder,
            value::ValueId,
        },
        ssa_gen::Ssa,
    },
};
use rustc_hash::FxHashMap as HashMap;

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
        mut self,
        max_bytecode_increase_percent: Option<i32>,
    ) -> Result<Ssa, RuntimeError> {
        for function in self.functions.values_mut() {
            let is_brillig = function.runtime().is_brillig();

            // Take a snapshot in case we have to restore it.
            let orig_function =
                (max_bytecode_increase_percent.is_some() && is_brillig).then(|| function.clone());

            // We must be able to unroll ACIR loops at this point, so exit on failure to unroll.
            let has_unrolled = function.unroll_loops_iteratively()?;

            // Check if the size increase is acceptable
            // This is here now instead of in `Function::unroll_loops_iteratively` because we'd need
            // more finessing to convince the borrow checker that it's okay to share a read-only reference
            // to the globals and a mutable reference to the function at the same time, both part of the `Ssa`.
            if has_unrolled && is_brillig {
                if let Some(max_incr_pct) = max_bytecode_increase_percent {
                    let orig_function = orig_function.expect("took snapshot to compare");
                    let new_size = function.num_instructions();
                    let orig_size = orig_function.num_instructions();
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
    /// Try to unroll loops in the function.
    ///
    /// Returns an `Err` if it cannot be done, for example because the loop bounds
    /// cannot be determined at compile time. This can happen during pre-processing,
    /// but it should still leave the function in a partially unrolled, but valid state.
    ///
    /// If successful, returns a flag indicating whether any loops have been unrolled.
    pub(super) fn unroll_loops_iteratively(&mut self) -> Result<bool, RuntimeError> {
        // Try to unroll loops first:
        let (mut has_unrolled, mut unroll_errors) = self.try_unroll_loops();

        // Keep unrolling until no more errors are found
        while !unroll_errors.is_empty() {
            let prev_unroll_err_count = unroll_errors.len();

            // Simplify the SSA before retrying
            simplify_between_unrolls(self);

            // Unroll again
            let (new_unrolled, new_errors) = self.try_unroll_loops();
            unroll_errors = new_errors;
            has_unrolled |= new_unrolled;

            // If we didn't manage to unroll any more loops, exit
            if unroll_errors.len() >= prev_unroll_err_count {
                return Err(unroll_errors.swap_remove(0));
            }
        }

        Ok(has_unrolled)
    }

    /// Unroll all loops within the function.
    /// Any loops which fail to be unrolled (due to using non-constant indices) will be unmodified.
    /// Returns a flag indicating whether any blocks have been modified.
    ///
    /// Loop unrolling in brillig can lead to a code explosion currently.
    /// This can also be true for ACIR, but we have no alternative to unrolling in ACIR.
    /// Brillig also generally prefers smaller code rather than faster code,
    /// so we only attempt to unroll small loops, which we decide on a case-by-case basis.
    fn try_unroll_loops(&mut self) -> (bool, Vec<RuntimeError>) {
        // The loops that failed to be unrolled so that we do not try to unroll them again.
        // Each loop is identified by its header block id.
        let mut failed_to_unroll = HashSet::new();
        // The reasons why loops in the above set failed to unroll.
        let mut unroll_errors = vec![];
        let mut has_unrolled = false;

        // Repeatedly find all loops as we unroll outer loops and go towards nested ones.
        loop {
            let mut loops = Loops::find_all(self);
            // Blocks which were part of loops we unrolled. Nested loops are included in the outer loops,
            // so if an outer loop is unrolled, we have to restart looking for the nested ones.
            let mut modified_blocks = HashSet::new();
            // Indicate whether we will have to have another go looking for loops, to deal with nested ones.
            let mut needs_refresh = false;

            while let Some(next_loop) = loops.yet_to_unroll.pop() {
                // Don't try to unroll the loop again if it is known to fail
                if failed_to_unroll.contains(&next_loop.header) {
                    continue;
                }

                // Only unroll small loops in Brillig.
                if self.runtime().is_brillig() && !next_loop.is_small_loop(self, &loops.cfg) {
                    continue;
                }

                // Check if we will be able to unroll this loop, before starting to modify the blocks.
                if next_loop.has_const_back_edge_induction_value(&self.dfg) {
                    // Don't try to unroll this.
                    failed_to_unroll.insert(next_loop.header);
                    // If this is Brillig, we can still evaluate this loop at runtime.
                    if self.runtime().is_acir() {
                        unroll_errors
                            .push(RuntimeError::UnknownLoopBound { call_stack: CallStack::new() });
                    }
                    continue;
                }

                // If we've previously modified a block in this loop we need to refresh the context.
                // This happens any time we have nested loops.
                if next_loop.blocks.iter().any(|block| modified_blocks.contains(block)) {
                    needs_refresh = true;
                    // Carry on unrolling the loops which weren't related to the ones we have already done.
                    continue;
                }

                // Try to unroll.
                match next_loop.unroll(self, &loops.cfg) {
                    Ok(_) => {
                        has_unrolled = true;
                        modified_blocks.extend(next_loop.blocks);
                    }
                    Err(call_stack) => {
                        failed_to_unroll.insert(next_loop.header);
                        unroll_errors.push(RuntimeError::UnknownLoopBound { call_stack });
                    }
                }
            }
            // Once we have no more nested loops, we are done.
            if !needs_refresh {
                break;
            }
        }
        (has_unrolled, unroll_errors)
    }
}

/// Describe the blocks that constitute up a loop.
#[derive(Debug)]
pub(crate) struct Loop {
    /// The header block of a loop is the block which dominates all the
    /// other blocks in the loop.
    pub(crate) header: BasicBlockId,

    /// The start of the back_edge n -> d is the block n at the end of
    /// the loop that jumps back to the header block d which restarts the loop.
    pub(crate) back_edge_start: BasicBlockId,

    /// All the blocks contained within the loop, including `header` and `back_edge_start`.
    pub(crate) blocks: BTreeSet<BasicBlockId>,
}

/// All the unrolled loops in the SSA.
pub(crate) struct Loops {
    /// Loops that haven't been unrolled yet, which is all the loops currently in the CFG.
    pub(crate) yet_to_unroll: Vec<Loop>,
    /// The CFG so we can query the predecessors of blocks when needed.
    pub(crate) cfg: ControlFlowGraph,
    /// The [DominatorTree] used during the discovery of loops.
    pub(crate) dom: DominatorTree,
}

impl Loops {
    /// Find all loops in the program by finding a node that dominates any predecessor node.
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
    ///
    /// Returns all groups of blocks that look like a loop, even if we might not be able to unroll them,
    /// which we can use to check whether we were able to unroll all blocks.
    pub(crate) fn find_all(function: &Function) -> Self {
        let cfg = ControlFlowGraph::with_function(function);
        let post_order = PostOrder::with_function(function);
        let mut dom_tree = DominatorTree::with_cfg_and_post_order(&cfg, &post_order);

        let mut loops = vec![];

        // Iterating over blocks in reverse-post-order, ie. forward order, just because it's already available.
        for block in post_order.into_vec_reverse() {
            for predecessor in cfg.predecessors(block) {
                // In the above example, we're looking for when `block` is `loop_entry` and `predecessor` is `loop_body`.
                if dom_tree.dominates(block, predecessor) {
                    // predecessor -> block is the back-edge of a loop
                    loops.push(Loop::find_blocks_in_loop(block, predecessor, &cfg));
                }
            }
        }

        // Sort loops by block size so that we unroll the larger, outer loops of nested loops first.
        // This is needed because inner loops may use the induction variable from their outer loops in
        // their loop range. We will start popping loops from the back.
        loops.sort_by_key(|loop_| loop_.blocks.len());

        Self { yet_to_unroll: loops, cfg, dom: dom_tree }
    }
}

impl Loop {
    /// Return each block that is in a loop starting in the given header block.
    /// Expects back_edge_start -> header to be the back edge of the loop.
    pub(crate) fn find_blocks_in_loop(
        header: BasicBlockId,
        back_edge_start: BasicBlockId,
        cfg: &ControlFlowGraph,
    ) -> Self {
        let mut blocks = BTreeSet::default();
        // Insert the header so we don't go past it when traversing backwards from the back-edge.
        blocks.insert(header);

        let mut insert = |block, stack: &mut Vec<BasicBlockId>| {
            if !blocks.contains(&block) {
                blocks.insert(block);
                stack.push(block);
            }
        };

        // Starting from the back edge of the loop, enqueue each predecessor of this block until we reach the header.
        let mut stack = vec![];
        insert(back_edge_start, &mut stack);

        while let Some(block) = stack.pop() {
            for predecessor in cfg.predecessors(block) {
                insert(predecessor, &mut stack);
            }
        }

        Self { header, back_edge_start, blocks }
    }

    /// Check that the loop does not end with a constant value passed to the header
    /// from the back-edge, which would result in a loop we would never finish unrolling.
    ///
    /// This can happen if constant folding replaces a variable with a constant it is
    /// constrained to equal (which doesn't even have to fall into the loop bounds).
    ///
    /// For example:
    /// ```text
    /// brillig(inline) predicate_pure fn main f0 {
    ///   b0():
    ///     jmp b1(u32 1)                // Pre-header
    ///   b1(v0: u32):                   // Header
    ///     v3 = lt v0, u32 20
    ///     jmpif v3 then: b2, else: b3
    ///   b2():                          // Back edge
    ///     constrain v0 == u32 1        // Constrain the induction variable to a known value.
    ///     jmp b1(u32 2)                // `v1 = unchecked_add v0, u32 1; jmp b1(v1)` replaced by `jmp b1 (1+1)`
    ///   b3():
    ///     return
    /// }
    /// ```
    fn has_const_back_edge_induction_value(&self, dfg: &DataFlowGraph) -> bool {
        let back_edge = &dfg[self.back_edge_start];
        let Some(TerminatorInstruction::Jmp { destination, arguments, .. }) =
            back_edge.terminator()
        else {
            unreachable!("the back edge is expected to end in a `Jmp`");
        };
        assert_eq!(*destination, self.header, "back edge goes to the header");
        assert_eq!(arguments.len(), 1);
        dfg.get_numeric_constant(arguments[0]).is_some()
    }

    /// Check if the loop header has a constant zero jump condition, which indicates an empty loop.
    ///
    /// This can happen if a jump condition has been simplified out.
    ///
    /// For example:
    /// ```text
    /// brillig(inline) predicate_pure fn main f0 {
    ///   b0():
    ///     jmp b1(u32 10)               // Pre-header
    ///   b1(v0: u32):                   // Header
    ///     // v3 = lt v0, u32 0         // Simplified to `u1 0`
    ///     jmpif u1 0 then: b2, else: b3
    ///   b2():                          // Back edge
    ///     v1 = unchecked_add v0, u32 1 // Increment induction value
    ///     jmp b1(v1)
    ///   b3():
    ///     return
    /// }
    /// ```
    fn has_const_zero_jump_condition(&self, dfg: &DataFlowGraph) -> bool {
        let header = &dfg[self.header];
        let Some(TerminatorInstruction::JmpIf { condition, .. }) = header.terminator() else {
            return false;
        };
        let Some(condition) = dfg.get_numeric_constant(*condition) else {
            return false;
        };
        condition.is_zero()
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
        dfg: &DataFlowGraph,
        pre_header: BasicBlockId,
    ) -> Option<IntegerConstant> {
        let jump_value = get_induction_variable(dfg, pre_header).ok()?;
        dfg.get_integer_constant(jump_value)
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
    fn get_const_upper_bound(
        &self,
        dfg: &DataFlowGraph,
        pre_header: BasicBlockId,
    ) -> Option<IntegerConstant> {
        let header = &dfg[self.header];

        // If the header has no parameters then this must be a `loop` or `while`.
        if header.parameters().is_empty() {
            return None;
        }

        let instructions = header.instructions();
        if instructions.is_empty() {
            // If the loop condition is constant, the loop header will be
            // simplified to a simple jump.
            if self.has_const_zero_jump_condition(dfg) {
                // There are cases where the upper bound jmpif degenerates into a constant `false`;
                // in that case we can just return the `lower` to emulate a known empty loop.
                return self.get_const_lower_bound(dfg, pre_header);
            } else {
                return None;
            };
        }

        if instructions.len() != 1 {
            // The header should just compare the induction variable and jump.
            // If that's not the case, this might be a `loop` and not a `for` loop.
            return None;
        }

        match &dfg[instructions[0]] {
            Instruction::Binary(Binary { lhs: _, operator: BinaryOp::Lt, rhs }) => {
                dfg.get_integer_constant(*rhs)
            }
            Instruction::Binary(Binary { lhs: _, operator: BinaryOp::Eq, rhs }) => {
                // `for i in 0..1` is turned into:
                // b1(v0: u32):
                //   v12 = eq v0, u32 0
                //   jmpif v12 then: b2, else: b3
                dfg.get_integer_constant(*rhs).map(|c| c.inc())
            }
            Instruction::Not(_) => {
                // We simplify equality operations with booleans like `(boolean == false)` into `!boolean`.
                // Thus, using a u1 in a loop bound can possibly lead to a Not instruction
                // as a loop header's jump condition.
                //
                // `for i in 0..1` is turned into:
                //  b1(v0: u1):
                //    v2 = eq v0, u32 0
                //    jmpif v2 then: b2, else: b3
                //
                // Which is further simplified into:
                //  b1(v0: u1):
                //    v2 = not v0
                //    jmpif v2 then: b2, else: b3
                Some(IntegerConstant::Unsigned { value: 1, bit_size: 1 })
            }
            Instruction::Cast(_, _) => {
                // A cast of a constant would already be simplified
                None
            }
            other => panic!("Unexpected instruction in header: {other:?}"),
        }
    }

    /// Get the lower and upper bounds of the loop if both are constant numeric values.
    pub(super) fn get_const_bounds(
        &self,
        dfg: &DataFlowGraph,
        pre_header: BasicBlockId,
    ) -> Option<(IntegerConstant, IntegerConstant)> {
        let lower = self.get_const_lower_bound(dfg, pre_header)?;
        let upper = self.get_const_upper_bound(dfg, pre_header)?;
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
    ///   v2 = lt i, v1
    ///   jmpif v2, then: loop_body, else: loop_end
    /// ```
    ///
    /// The first step is to unroll the header by recognizing that jump condition
    /// is a constant, which means it will go to `loop_body`:
    /// ```text
    /// main():
    ///   v0 = 0
    ///   v1 = 2
    ///   v2 = lt v0, v1
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
    ///   v2 = lt v0, v1
    ///   v3 = ... body ...
    ///   v4 = add v0, u32 1
    ///   jmp loop_entry(v4)
    /// ```
    ///
    /// At the end we reach a point where the condition evaluates to 0 and we jump to the end.
    /// ```text
    /// main():
    ///   v0 = 0
    ///   v1 = 2
    ///   v2 = lt v0, v1
    ///   v3 = ... body ...
    ///   v4 = add u32 1, v0
    ///   v5 = lt v4, v1
    ///   v6 = ... body ...
    ///   v7 = add v4, u32 1
    ///   v8 = lt v7, v1
    ///   jmp loop_end
    /// ```
    ///
    /// When e.g. `v8 = lt v7, v1` cannot be evaluated to a constant, the loop signals by returning `Err`
    /// that a few SSA passes are required to evaluate and simplify these values.
    fn unroll(&self, function: &mut Function, cfg: &ControlFlowGraph) -> Result<(), CallStack> {
        let mut unroll_into = self.get_pre_header(function, cfg)?;
        let mut jump_value = get_induction_variable(&function.dfg, unroll_into)?;

        while let Some((context, loop_header_id)) =
            self.unroll_header(function, unroll_into, jump_value)?
        {
            (unroll_into, jump_value) = context.unroll_loop_iteration(loop_header_id);
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
    /// Returns Some((iteration context, loop_header_id)) if we should perform another iteration.
    fn unroll_header<'a>(
        &'a self,
        function: &'a mut Function,
        unroll_into: BasicBlockId,
        induction_value: ValueId,
    ) -> Result<Option<(LoopIteration<'a>, BasicBlockId)>, CallStack> {
        // We insert into a fresh block first and move instructions into the unroll_into block later
        // only once we verify the jmpif instruction has a constant condition. If it does not, we can
        // just discard this fresh block and leave the loop unmodified.
        let fresh_block = function.dfg.make_block();

        let mut context = LoopIteration::new(function, self, fresh_block, self.header);
        let loop_header_id = context.source_block;
        let source_block = &context.dfg()[loop_header_id];
        assert_eq!(source_block.parameters().len(), 1, "Expected only 1 argument in loop header");

        // Insert the current value of the loop induction variable into our context.
        let first_param = source_block.parameters()[0];
        context.inserter.try_map_value(first_param, induction_value);
        // Copy over all instructions and a fresh terminator.
        context.inline_instructions_from_block();
        context.visited_blocks.insert(loop_header_id);

        // Mutate the terminator if possible so that it points at the iteration block.
        match context.dfg()[fresh_block].unwrap_terminator() {
            TerminatorInstruction::JmpIf {
                condition,
                then_destination,
                else_destination,
                call_stack,
            } => {
                let condition = *condition;
                let next_blocks = context.handle_jmpif(
                    condition,
                    *then_destination,
                    *else_destination,
                    *call_stack,
                );

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
                    Ok(self
                        .blocks
                        .contains(&context.source_block)
                        .then_some(context)
                        .map(|iteration_context| (iteration_context, loop_header_id)))
                } else {
                    // If this case is reached the loop either uses non-constant indices or we need
                    // another pass, such as mem2reg to resolve them to constants.
                    Err(context.inserter.function.dfg.get_value_call_stack(condition))
                }
            }
            other => unreachable!(
                "Expected loop header to terminate in a JmpIf to the loop body, but found {other:?} instead"
            ),
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
    ///   v16, v17 = call vector_push_back(v13, v14, v12) -> (u32, [u32]) // builtin to push, will store to storage and length references
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
                .map(|i| function.dfg.instruction_result::<1>(*i)[0])
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
            block.instructions().len() + usize::from(block.terminator().is_some())
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
        self.boilerplate_stats(function, cfg)
            .map(|s| s.is_small() && self.is_fully_executed(cfg))
            .unwrap_or_default()
    }

    /// Collect boilerplate stats if we can figure out the upper and lower bounds of the loop,
    /// and the loop doesn't have multiple back-edges from breaks and continues.
    fn boilerplate_stats(
        &self,
        function: &Function,
        cfg: &ControlFlowGraph,
    ) -> Option<BoilerplateStats> {
        let pre_header = self.get_pre_header(function, cfg).ok()?;
        let (lower, upper) = self.get_const_bounds(&function.dfg, pre_header)?;
        let refs = self.find_pre_header_reference_values(function, cfg)?;

        let (loads, stores) = self.count_loads_and_stores(function, &refs);
        let increments = self.count_induction_increments(function);
        let all_instructions = self.count_all_instructions(function);

        // Currently we don't iterate in reverse, so if upper <= lower it means 0 iterations.
        let iterations: usize = upper
            .reduce(
                lower,
                |u, l| u.saturating_sub(l).max(0) as usize,
                |u, l| u.saturating_sub(l) as usize,
            )
            .unwrap_or_default();

        Some(BoilerplateStats {
            iterations,
            loads,
            stores,
            increments,
            all_instructions,
            has_const_zero_jump_condition: self.has_const_zero_jump_condition(&function.dfg),
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
    /// Indicate whether the comparison with the upper bound has been simplified out.
    has_const_zero_jump_condition: bool,
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
        // Two jumps + plus the comparison with the upper bound.
        // This could be just 2 if the comparison has been simplified out.
        let boilerplate = if self.has_const_zero_jump_condition { 2 } else { 3 };
        // Be conservative and only assume that mem2reg gets rid of load followed by store.
        // NB we have not checked that these are actual pairs.
        let load_and_store = self.loads.min(self.stores) * 2;
        let total_boilerplate = self.increments + load_and_store + boilerplate;
        debug_assert!(
            total_boilerplate <= self.all_instructions,
            "Boilerplate instructions exceed total instructions in loop"
        );
        self.all_instructions.saturating_sub(total_boilerplate)
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
fn get_induction_variable(dfg: &DataFlowGraph, block: BasicBlockId) -> Result<ValueId, CallStack> {
    match dfg[block].terminator() {
        Some(TerminatorInstruction::Jmp { arguments, call_stack: location, .. }) => {
            // This assumption will no longer be valid if e.g. mutable variables are represented as
            // block parameters. If that becomes the case we'll need to figure out which variable
            // is generally constant and increasing to guess which parameter is the induction
            // variable.
            if arguments.len() != 1 {
                // It is expected that a loop's induction variable is the only block parameter of the loop header.
                // If there's no variable this might be a `loop`.
                let call_stack = dfg.get_call_stack(*location);
                return Err(call_stack);
            }

            let value = arguments[0];
            if dfg.get_numeric_constant(value).is_some() {
                Ok(value)
            } else {
                let call_stack = dfg.get_call_stack(*location);
                Err(call_stack)
            }
        }
        Some(terminator) => Err(dfg.get_call_stack(terminator.call_stack())),
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

    /// Has `unroll_loop_iteration` reached the `loop_header_id`?
    encountered_loop_header: bool,

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
            encountered_loop_header: false,

            induction_value: None,
        }
    }

    /// Unroll a single iteration of the loop.
    ///
    /// Note that after unrolling a single iteration, the loop is _not_ in a valid state.
    /// It is expected the terminator instructions are set up to branch into an empty block
    /// for further unrolling. When the loop is finished this will need to be mutated to
    /// jump to the end of the loop instead.
    fn unroll_loop_iteration(mut self, loop_header_id: BasicBlockId) -> (BasicBlockId, ValueId) {
        // Kick off the unrolling from the initial source block.
        let mut next_blocks = self.unroll_loop_block();

        while let Some(block) = next_blocks.pop() {
            self.insert_block = block;
            self.source_block = self.get_original_block(block);
            self.encountered_loop_header |= loop_header_id == self.source_block;

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

        assert!(
            self.encountered_loop_header,
            "expected to encounter loop header when visiting blocks"
        );

        (end_block, induction_value)
    }

    /// Unroll a single block in the current iteration of the loop.
    ///
    /// Returns the next blocks to unroll, based on whether the jmp terminator has 1 or 2 destinations.
    fn unroll_loop_block(&mut self) -> Vec<BasicBlockId> {
        self.visited_blocks.insert(self.source_block);

        // Copy instructions from the loop body to the unroll destination, replacing the terminator.
        self.inline_instructions_from_block();

        let terminator = self.inserter.function.dfg[self.insert_block].unwrap_terminator();

        let next_blocks = match terminator {
            TerminatorInstruction::JmpIf {
                condition,
                then_destination,
                else_destination,
                call_stack,
            } => self.handle_jmpif(*condition, *then_destination, *else_destination, *call_stack),
            TerminatorInstruction::Jmp { destination, arguments, call_stack: _ } => {
                if self.get_original_block(*destination) == self.loop_.header {
                    // We found the back-edge of the loop.
                    assert_eq!(arguments.len(), 1, "back-edge should only have 1 argument");
                    assert!(self.induction_value.is_none(), "there should be only one back-edge");
                    self.induction_value = Some((self.insert_block, arguments[0]));
                }
                vec![*destination]
            }
            TerminatorInstruction::Return { .. } => {
                // Early returns from loops are not implemented.
                unreachable!("unexpected return terminator in loop body");
            }
            TerminatorInstruction::Unreachable { .. } => {
                // The SSA pass that adds unreachable terminators must come after unrolling.
                unreachable!("unexpected unreachable terminator in loop body");
            }
        };

        // Guarantee that the next blocks we set up to be unrolled, are actually part of the loop,
        // which we recorded while inlining the instructions of the blocks already processed.
        // Since we only call `unroll_loop_block` from `unroll_loop_iteration`, which we only call
        // if the single destination in `unroll_header` is *not* outside the loop, this should hold.
        next_blocks.iter().for_each(|block| {
            let b = self.get_original_block(*block);
            assert!(self.loop_.blocks.contains(&b), "destination not in original loop");
        });

        next_blocks
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

    /// Translate a block id to a block id in the unrolled loop.
    ///
    /// If the given block id is not within the loop, it is returned as-is,
    /// which is the case for when the header jumps to the block following the loop.
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
            self.inserter.push_instruction(instruction, self.insert_block, false);
        }
        let mut terminator = self.dfg()[self.source_block].unwrap_terminator().clone();

        terminator.map_values_mut(|value| self.inserter.resolve(value));

        // Replace the blocks in the terminator with fresh one with the same parameters,
        // while remembering which were the original block IDs.
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

/// Unrolling leaves some duplicate instructions which can potentially be removed.
fn simplify_between_unrolls(function: &mut Function) {
    // Do a mem2reg after the last unroll to aid simplify_cfg
    function.mem2reg();
    function.simplify_function();
    // Do another mem2reg after simplify_cfg to aid the next unroll
    function.mem2reg();
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
    use test_case::test_case;

    use crate::assert_ssa_snapshot;
    use crate::errors::RuntimeError;
    use crate::ssa::ir::integer::IntegerConstant;
    use crate::ssa::{Ssa, ir::value::ValueId, opt::assert_normalized_ssa_equals};

    use super::{BoilerplateStats, Loops, is_new_size_ok};

    /// Tries to unroll all loops in each SSA function once, calling the `Function` directly,
    /// bypassing the iterative loop done by the SSA which does further optimizations.
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
                    jmp b1(u32 0)
                b1(v0: u32):  // header of outer loop
                    v1 = lt v0, u32 3
                    jmpif v1 then: b2, else: b3
                b2():
                    jmp b4(u32 0)
                b4(v2: u32):  // header of inner loop
                    v3 = lt v2, u32 4
                    jmpif v3 then: b5, else: b6
                b5():
                    v4 = add v0, v2
                    v5 = lt u32 10, v4
                    constrain v5 == u1 1
                    v6 = add v2, u32 1
                    jmp b4(v6)
                b6(): // end of inner loop
                    v7 = add v0, u32 1
                    jmp b1(v7)
                b3(): // end of outer loop
                    return u32 0
            }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        // The final block count is not 1 because unrolling creates some unnecessary jmps.
        // If a simplify cfg pass is ran afterward, the expected block count will be 1.
        let (ssa, errors) = try_unroll_loops(ssa);
        assert_eq!(errors.len(), 0, "All loops should be unrolled");
        assert_eq!(ssa.main().reachable_blocks().len(), 5);

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            constrain u1 0 == u1 1
            constrain u1 0 == u1 1
            constrain u1 0 == u1 1
            constrain u1 0 == u1 1
            jmp b2()
          b1():
            return u32 0
          b2():
            constrain u1 0 == u1 1
            constrain u1 0 == u1 1
            constrain u1 0 == u1 1
            constrain u1 0 == u1 1
            jmp b3()
          b3():
            constrain u1 0 == u1 1
            constrain u1 0 == u1 1
            constrain u1 0 == u1 1
            constrain u1 0 == u1 1
            jmp b4()
          b4():
            jmp b1()
        }
        ");
    }

    // Test that the pass can still be run on loops which fail to unroll properly
    #[test]
    fn fail_to_unroll_loop() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            jmp b1(v0)
          b1(v1: u32):
            v2 = lt v1, u32 5
            jmpif v2 then: b2, else: b3
          b2():
            v3 = add v1, u32 1
            jmp b1(v3)
          b3():
            return u32 0
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

        let loop_ = &loops.yet_to_unroll[0];
        let pre_header =
            loop_.get_pre_header(function, &loops.cfg).expect("Should have a pre_header");
        let (lower, upper) =
            loop_.get_const_bounds(&function.dfg, pre_header).expect("bounds are numeric const");

        assert_eq!(lower, IntegerConstant::Unsigned { value: 0, bit_size: 32 });
        assert_eq!(upper, IntegerConstant::Unsigned { value: 4, bit_size: 32 });
    }

    #[test]
    fn test_get_const_bounds_empty_simplified() {
        // The following is an empty loop where the jmpif in b1 was simplified
        // from `v1 = lt v0, u32 0` into `u1 0`.
        let src = r#"
        acir(inline) fn main f0 {
          b0():
            jmp b1(u32 0)
          b1(v0: u32):
            jmpif u1 0 then: b2, else: b3
          b2():
            v41 = unchecked_add v0, u32 1
            jmp b1(v41)
          b3():
            return
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let loops = Loops::find_all(function);
        assert_eq!(loops.yet_to_unroll.len(), 1);

        let loop_ = &loops.yet_to_unroll[0];
        let pre_header =
            loop_.get_pre_header(function, &loops.cfg).expect("Should have a pre_header");
        let (lower, upper) = loop_
            .get_const_bounds(&function.dfg, pre_header)
            .expect("should use the lower for upper");

        assert_eq!(lower, IntegerConstant::Unsigned { value: 0, bit_size: 32 });
        assert_eq!(upper, lower);
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
    fn test_boilerplate_stats_i64_empty() {
        // Looping 0..-1, which should be 0 iterations.
        // u64::MAX is how -1 is represented as a Field.
        let ssa = brillig_unroll_test_case_6470_with_params("i64", "0", &format!("{}", u64::MAX));
        let stats = loop0_stats(&ssa);
        assert_eq!(stats.iterations, 0);
        assert_eq!(stats.unrolled_instructions(), 0);
    }

    #[test]
    fn test_boilerplate_stats_i64_non_empty() {
        // Looping -4..-1, which should be 3 iterations.
        // u64::MAX-3 is how -4 is represented as a Field.
        let ssa = brillig_unroll_test_case_6470_with_params(
            "i64",
            &format!("{}", u64::MAX - 3),
            &format!("{}", u64::MAX),
        );
        let stats = loop0_stats(&ssa);
        assert_eq!(stats.iterations, 3);
    }

    #[test]
    fn test_boilerplate_stats_6470() {
        let ssa = brillig_unroll_test_case_6470(2);
        let stats = loop0_stats(&ssa);
        assert_eq!(stats.iterations, 2);
        assert_eq!(stats.all_instructions, 2 + 9); // Instructions in b1 and b3
        assert_eq!(stats.increments, 2);
        assert_eq!(stats.loads, 1);
        assert_eq!(stats.stores, 1);
        assert_eq!(stats.useful_instructions(), 4); // cast, array get, add, array set
        assert_eq!(stats.baseline_instructions(), 12);
        assert!(stats.is_small());
    }

    #[test]
    fn test_boilerplate_stats_const_zero_jump_condition() {
        let src = "
        brillig(inline) impure fn main f0 {
          b0():
            jmp b1(u32 0)
          b1(v0: u32):
            jmpif u1 0 then: b2, else: b3
          b2():
            v1 = unchecked_add v0, u32 1
            jmp b1(v1)
          b3():
            return
        }";
        let ssa = Ssa::from_str(src).unwrap();
        let stats = loop0_stats(&ssa);
        assert!(stats.is_small());
    }

    /// Test that we can unroll a small loop.
    #[test]
    fn test_brillig_unroll_small_loop() {
        let ssa = brillig_unroll_test_case();

        let (ssa, errors) = try_unroll_loops(ssa);
        assert_eq!(errors.len(), 0, "Unroll should have no errors");
        assert_eq!(ssa.main().reachable_blocks().len(), 2, "The loop should be unrolled");

        // Expectation taken by compiling the Noir program as ACIR,
        // ie. by removing the `unconstrained` from `main`.
        assert_ssa_snapshot!(ssa, @r"
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
        ");
    }

    /// Test that we can unroll the loop in the ticket if we don't have too many iterations.
    #[test]
    fn test_brillig_unroll_6470_small() {
        // Few enough iterations so that we can perform the unroll.
        let ssa = brillig_unroll_test_case_6470(2);
        let (ssa, errors) = try_unroll_loops(ssa);
        assert_eq!(errors.len(), 0, "Unroll should have no errors");
        assert_eq!(ssa.main().reachable_blocks().len(), 2, "The loop should be unrolled");

        assert_ssa_snapshot!(ssa, @r"
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
            jmp b1()
          b1():
            v15 = load v3 -> [u64; 6]
            dec_rc v0
            return v15
        }
        ");
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
        assert_normalized_ssa_equals(ssa, &parse_ssa().print_without_locations().to_string());
    }

    #[test]
    fn test_brillig_unroll_iteratively_respects_max_increase() {
        let ssa = brillig_unroll_test_case();
        let ssa = ssa.unroll_loops_iteratively(Some(-90)).unwrap();
        // Check that it's still the original
        let expected = brillig_unroll_test_case();
        assert_normalized_ssa_equals(ssa, &expected.print_without_locations().to_string());
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
            v11 = eq v0, u32 5
            jmpif v11 then: b5, else: b4
          b4():
            v15 = load v1 -> Field
            v17 = add v15, Field 1
            store v17 at v1
            v18 = add v0, u32 1
            jmp b1(v18)
          b5():
            jmp b6()
          b6():
            v12 = load v1 -> Field
            v14 = eq v12, Field 4
            constrain v12 == Field 4
            return
          b7():
            v9 = add v0, u32 1
            jmp b1(v9)
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
        brillig_unroll_test_case_with_params("u32", "0", "4")
    }

    fn brillig_unroll_test_case_with_params(idx_type: &str, lower: &str, upper: &str) -> Ssa {
        let src = format!(
            "
        // After `static_assert` and `assert_constant`:
        brillig(inline) fn main f0 {{
          b0(v0: u32):
            v2 = allocate -> &mut u32
            store u32 0 at v2
            jmp b1({idx_type} {lower})
          b1(v1: {idx_type}):
            v5 = lt v1, {idx_type} {upper}
            jmpif v5 then: b3, else: b2
          b3():
            v8 = load v2 -> u32
            v9 = add v8, v1
            store v9 at v2
            v11 = add v1, {idx_type} 1
            jmp b1(v11)
          b2():
            v6 = load v2 -> u32
            v7 = eq v6, v0
            constrain v6 == v0
            return
        }}
        "
        );
        Ssa::from_str(&src).unwrap()
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
        brillig_unroll_test_case_6470_with_params("u32", "0", &format!("{num_iterations}"))
    }

    fn brillig_unroll_test_case_6470_with_params(idx_type: &str, lower: &str, upper: &str) -> Ssa {
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
            jmp b1({idx_type} {lower})
          b1(v1: {idx_type}):
            v7 = lt v1, {idx_type} {upper}
            jmpif v7 then: b3, else: b2
          b3():
            v9 = load v4 -> [u64; 6]
            v10 = cast v1 as u32
            v11 = array_get v0, index v10 -> u64
            v12 = add v11, u64 1
            v13 = array_set v9, index v10, value v12
            v15 = unchecked_add v1, {idx_type} 1
            store v13 at v4
            v16 = unchecked_add v1, {idx_type} 1 // duplicate
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

    #[test]
    fn do_not_unroll_loop_with_break() {
        // One of the loop header's (b1) successors (b3) has multiple predecessors (b1 and b4).
        // This logic is how we identify a loop with a break expression.
        // We do not support unrolling these types of loops.
        let src = r#"
        brillig(inline) predicate_pure fn main f0 {
          b0():
            jmp b1(u32 0)
          b1(v0: u32):
            v3 = lt v0, u32 5
            jmpif v3 then: b2, else: b3
          b2():
            jmpif u1 1 then: b4, else: b5
          b3():
            return u1 1
          b4():
            jmp b3()
          b5():
            v6 = unchecked_add v0, u32 1
            jmp b1(v6)
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let (ssa, errors) = try_unroll_loops(ssa);
        assert_eq!(errors.len(), 0, "All loops should be unrolled");

        // The SSA is expected to be unchanged
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn test_brillig_unroll_with_const_back_edge() {
        // The loop is small enough that Brillig wants to unroll it,
        // but the back edge passes a constant that would result in
        // an infinite loop of attempting to unroll.
        let src = "
        brillig(inline) predicate_pure fn main f0 {
          b0():
            jmp b1(u32 10)
          b1(v0: u32):
            v3 = lt v0, u32 12
            jmpif v3 then: b2, else: b3
          b2():
            constrain v0 == u32 1
            jmp b1(u32 2)
          b3():
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let (ssa, errors) = try_unroll_loops(ssa);
        assert_eq!(errors.len(), 0, "Unroll should have no errors");
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn while_loop_has_empty_bounds() {
        // SSA of a program such as:
        // unconstrained fn main() {
        //     let mut run = true;
        //     while run { }
        // }
        let src = r#"
        brillig(inline) impure fn main f0 {
          b0():
            v0 = allocate -> &mut u1
            store u1 1 at v0
            jmp b1()
          b1():
            v2 = load v0 -> u1
            jmpif v2 then: b2, else: b3
          b2():
            jmp b1()
          b3():
            return
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let mut loops = Loops::find_all(function);
        let loop0 = loops.yet_to_unroll.pop().expect("there should be a loop");
        let pre_header = loop0.get_pre_header(function, &loops.cfg).unwrap();
        assert!(loop0.get_const_lower_bound(&function.dfg, pre_header).is_none());
        assert!(loop0.get_const_upper_bound(&function.dfg, pre_header).is_none());
    }

    #[test]
    #[should_panic(expected = "ICE: overflow while incrementing constant")]
    fn unroll_loop_upper_bound_saturated() {
        let ssa = format!(
            r#"
        acir(inline) fn main f0 {{
          b0():
            jmp b1(u128 {0})
          b1(v0: u128):
            v3 = eq v0, u128 {0}
            jmpif v3 then: b3, else: b2
          b2():
            v6 = unchecked_add v0, u128 1
            jmp b1(v6)
          b3():
            return
    }}"#,
            u128::MAX
        );

        let ssa = Ssa::from_str(&ssa).unwrap();
        let function = ssa.main();

        let loops = Loops::find_all(function);
        assert_eq!(loops.yet_to_unroll.len(), 1);

        let loop_ = &loops.yet_to_unroll[0];
        let pre_header =
            loop_.get_pre_header(function, &loops.cfg).expect("Should have a pre_header");
        let (lower, upper) =
            loop_.get_const_bounds(&function.dfg, pre_header).expect("bounds are numeric const");
        assert_ne!(lower, upper);
    }
}
