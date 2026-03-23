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
//!     which, when unrolled, is estimated to have the same or fewer total cost as it
//!     has when not unrolled.
//!     This cost estimation is analogous to LLVM's `analyzeLoopUnrollCost` which estimates
//!     which loads become constant after unrolling. See:
//!     <https://llvm.org/doxygen/LoopUnrollPass_8cpp_source.html>
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
//!   - Pre-condition: The first block parameter of each loop header is the induction variable.
//!     Loop headers may have additional parameters for promoted mutable variables (e.g. from mem2reg_simple).
//!   - Pre-condition: No loop header has a JmpIf with a constant condition (run simplify_cfg first).
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
            function::{Function, FunctionId, RuntimeType},
            function_inserter::FunctionInserter,
            instruction::{Binary, BinaryOp, Instruction, TerminatorInstruction},
            integer::IntegerConstant,
            post_order::PostOrder,
            value::{Value, ValueId, ValueMapping},
        },
        ssa_gen::Ssa,
    },
};
use rustc_hash::FxHashMap as HashMap;

/// Maximum Brillig-weighted cost (after unrolling) for Brillig loops to be
/// force-unrolled regardless of the cost model. Loops with constant bounds
/// and no breaks whose unrolled cost is at or below this threshold will
/// always be unrolled.
pub const FORCE_UNROLL_THRESHOLD: usize = 128;

/// Maximum number of iterations for Brillig loops to be unrolled.
/// Prevents code explosion from very large loops even if they pass the cost model.
pub const MAX_UNROLL_ITERATIONS: usize = 1000;

impl Ssa {
    /// Loop unrolling can return errors, since ACIR functions need to be fully unrolled.
    /// This meta-pass will keep trying to unroll loops and simplifying the SSA until no more errors are found.
    ///
    /// The `max_bytecode_incr_pct`, when given, is used to limit the growth of the Brillig bytecode size
    /// after unrolling small loops to some percentage of the original loop. For example a value of 150 would
    /// mean the new loop can be 150% (ie. 2.5 times) larger than the original loop. It will still contain
    /// fewer SSA instructions, but that can still result in more Brillig opcodes.
    ///
    /// The `force_unroll_threshold` overrides the default threshold for force-unrolling
    /// small Brillig loops. Set to 0 to disable force-unrolling.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn unroll_loops_iteratively(
        mut self,
        max_bytecode_increase_percent: Option<i32>,
        max_unroll_iterations: usize,
        force_unroll_threshold: usize,
    ) -> Result<Ssa, RuntimeError> {
        for function in self.functions.values_mut() {
            let is_brillig = function.runtime().is_brillig();

            // Take a snapshot in case we have to restore it.
            let orig_function =
                (max_bytecode_increase_percent.is_some() && is_brillig).then(|| function.clone());

            // We must be able to unroll ACIR loops at this point, so exit on failure to unroll.
            let no_callee_costs = HashMap::default();
            let has_unrolled = function.unroll_loops_iteratively(
                max_unroll_iterations,
                force_unroll_threshold,
                &no_callee_costs,
            )?;

            // Check if the size increase is acceptable
            // This is here now instead of in `Function::unroll_loops_iteratively` because we'd need
            // more finessing to convince the borrow checker that it's okay to share a read-only reference
            // to the globals and a mutable reference to the function at the same time, both part of the `Ssa`.
            if has_unrolled
                && is_brillig
                && let Some(max_incr_pct) = max_bytecode_increase_percent
            {
                let orig_function = orig_function.expect("took snapshot to compare");
                let new_size = function.num_instructions();
                let orig_size = orig_function.num_instructions();
                if !is_new_size_ok(orig_size, new_size, max_incr_pct) {
                    *function = orig_function;
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
    ///
    /// The `force_unroll_threshold` overrides the default threshold for
    /// force-unrolling small Brillig loops.
    pub(super) fn unroll_loops_iteratively(
        &mut self,
        max_unroll_iterations: usize,
        force_unroll_threshold: usize,
        callee_costs: &HashMap<FunctionId, usize>,
    ) -> Result<bool, RuntimeError> {
        #[cfg(debug_assertions)]
        unroll_loops_pre_check(self);

        let (mut has_unrolled, mut unroll_errors) =
            self.try_unroll_loops(max_unroll_iterations, force_unroll_threshold, callee_costs);

        match self.runtime() {
            RuntimeType::Acir(_) => {
                // Keep unrolling until no more errors are found
                while !unroll_errors.is_empty() {
                    let prev_unroll_err_count = unroll_errors.len();

                    // Simplify the SSA before retrying
                    simplify_between_unrolls(self);

                    // Unroll again
                    let (new_unrolled, new_errors) = self.try_unroll_loops(
                        max_unroll_iterations,
                        force_unroll_threshold,
                        callee_costs,
                    );
                    unroll_errors = new_errors;
                    has_unrolled |= new_unrolled;

                    // If we didn't manage to unroll any more loops, exit
                    if unroll_errors.len() >= prev_unroll_err_count {
                        return Err(unroll_errors.swap_remove(0));
                    }
                }
            }
            RuntimeType::Brillig(_) => loop {
                simplify_between_unrolls(self);
                let (unrolled, _) = self.try_unroll_loops(
                    max_unroll_iterations,
                    force_unroll_threshold,
                    callee_costs,
                );
                has_unrolled |= unrolled;
                if !unrolled {
                    break;
                }
            },
        }

        #[cfg(debug_assertions)]
        unroll_loops_post_check(self);

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
    fn try_unroll_loops(
        &mut self,
        max_unroll_iterations: usize,
        force_unroll_threshold: usize,
        callee_costs: &HashMap<FunctionId, usize>,
    ) -> (bool, Vec<RuntimeError>) {
        // The loops that failed to be unrolled so that we do not try to unroll them again.
        // Each loop is identified by its header block id.
        let mut failed_to_unroll = HashSet::new();
        // The reasons why loops in the above set failed to unroll.
        let mut unroll_errors = vec![];
        let mut has_unrolled = false;

        // Repeatedly find all loops as we unroll outer loops and go towards nested ones.
        loop {
            let order = if self.runtime().is_brillig() {
                LoopOrder::InsideOut
            } else {
                LoopOrder::OutsideIn
            };
            let mut loops = Loops::find_all(self, order);
            loops.callee_costs = callee_costs.clone();

            // Blocks which were part of loops we unrolled. Nested loops are included in the
            // outer loops, so if an outer loop is unrolled, we have to restart looking for
            // the nested ones.
            let mut modified_blocks = HashSet::new();
            // Blocks from loops that were skipped or failed to unroll. In InsideOut
            // ordering, if an inner loop can't be unrolled, any enclosing loop that
            // contains those blocks must also be skipped: unrolling visits each
            // block once and cannot traverse the inner loop's cycle.
            let mut failed_blocks: HashSet<BasicBlockId> = HashSet::new();
            let mut needs_refresh = false;
            // Accumulated header-param→final-value mappings from all unrolled loops
            // in this iteration. Applied in bulk after the loop processing is done,
            // avoiding O(loops * blocks) per-loop exit-block walks.
            let mut accumulated_mapping = ValueMapping::default();

            while let Some(next_loop) = loops.yet_to_unroll.pop() {
                // If we've previously modified a block in this loop we need to refresh.
                // This happens any time we have nested loops.
                if next_loop.blocks.iter().any(|block| modified_blocks.contains(block)) {
                    needs_refresh = true;
                    continue;
                }

                // InsideOut: skip if this loop contains blocks from an inner loop
                // that couldn't be unrolled. Unrolling visits each block once and
                // can't traverse an inner loop's cycle, so attempting to unroll an
                // outer loop with a non-unrolled inner loop would corrupt the SSA.
                // OutsideIn (ACIR) does not need this: outer loops are processed
                // first, and if they fail, inner loops are tried independently.
                if order == LoopOrder::InsideOut
                    && next_loop.blocks.iter().any(|block| failed_blocks.contains(block))
                {
                    continue;
                }

                // Don't try to unroll the loop again if it is known to fail.
                // Save loop blocks before `try_unroll_loop` takes ownership.
                let loop_blocks = next_loop.blocks.clone();
                let result = if failed_to_unroll.contains(&next_loop.header) {
                    LoopUnrollResult::Skipped
                } else {
                    self.try_unroll_loop(
                        next_loop,
                        &loops,
                        max_unroll_iterations,
                        force_unroll_threshold,
                    )
                };
                match result {
                    LoopUnrollResult::Skipped => {}
                    LoopUnrollResult::Failed(header, error) => {
                        failed_to_unroll.insert(header);
                        unroll_errors.push(error);
                        failed_blocks.extend(loop_blocks);
                    }
                    LoopUnrollResult::Unrolled(blocks, mapping) => {
                        has_unrolled = true;
                        modified_blocks.extend(blocks);
                        accumulated_mapping.extend(mapping);
                    }
                }
            }

            // Apply all header param->final value replacements in a single pass over
            // reachable blocks. This is O(blocks) total instead of O(loops * blocks).
            if !accumulated_mapping.is_empty() {
                for block_id in self.reachable_blocks() {
                    self.dfg.replace_values_in_block(block_id, &accumulated_mapping);
                }
            }

            // If we didn't need to refresh, we're done
            if !needs_refresh {
                break;
            }

            // In Brillig, simplify between inner and outer loop evaluations.
            // After unrolling inner loops, the expanded instructions need to be
            // constant-folded before the outer loop's cost model is evaluated,
            // otherwise useless_cost is inflated by un-simplified instructions.
            if self.runtime().is_brillig() {
                simplify_between_unrolls(self);
            }
        }
        (has_unrolled, unroll_errors)
    }

    /// Try to unroll a single loop.
    ///
    /// Returns the result: whether the loop was skipped, failed, or unrolled.
    fn try_unroll_loop(
        &mut self,
        loop_: Loop,
        loops: &Loops,
        max_unroll_iterations: usize,
        force_unroll_threshold: usize,
    ) -> LoopUnrollResult {
        // Only unroll small loops in Brillig.
        if self.runtime().is_brillig()
            && !loop_.should_unroll_in_brillig(
                self,
                loops,
                max_unroll_iterations,
                force_unroll_threshold,
            )
        {
            return LoopUnrollResult::Skipped;
        }

        // Check if we will be able to unroll this loop, before starting to modify the blocks.
        if loop_.has_const_back_edge_induction_value(&self.dfg) {
            // Don't try to unroll this.
            // If this is Brillig, we can still evaluate this loop at runtime.
            if self.runtime().is_acir() {
                return LoopUnrollResult::Failed(
                    loop_.header,
                    RuntimeError::UnknownLoopBound { call_stack: CallStack::new() },
                );
            }
            return LoopUnrollResult::Skipped;
        }

        // Try to unroll.
        match loop_.unroll(self, &loops.cfg) {
            Ok(mapping) => LoopUnrollResult::Unrolled(loop_.blocks, mapping),
            Err(call_stack) => LoopUnrollResult::Failed(
                loop_.header,
                RuntimeError::UnknownLoopBound { call_stack },
            ),
        }
    }
}

/// Result of trying to unroll a single loop.
enum LoopUnrollResult {
    /// Loop was skipped (not eligible for unrolling, or deferred for later).
    Skipped,
    /// Loop failed to unroll.
    Failed(BasicBlockId, RuntimeError),
    /// Loop was successfully unrolled. Contains the blocks that were part of the loop
    /// and a mapping from header params to their final values (to be applied in bulk).
    Unrolled(BTreeSet<BasicBlockId>, ValueMapping),
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

/// Order in which loops should be processed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LoopOrder {
    /// Process inner (smaller) loops first, then outer loops.
    /// Used for Brillig which can tolerate inner loops that reference outer induction variables.
    InsideOut,
    /// Process outer (larger) loops first, then inner loops.
    /// Used for ACIR which cannot tolerate inner loops that reference outer induction variables,
    /// so outer loops must be unrolled first.
    OutsideIn,
}

/// All the unrolled loops in the SSA.
pub(crate) struct Loops {
    /// Loops that haven't been unrolled yet, which is all the loops currently in the CFG.
    pub(crate) yet_to_unroll: Vec<Loop>,
    /// The CFG so we can query the predecessors of blocks when needed.
    pub(crate) cfg: ControlFlowGraph,
    /// The [DominatorTree] used during the discovery of loops.
    pub(crate) dom: DominatorTree,
    /// Body weights of callees that will be inlined, used to estimate the true cost
    /// of call instructions in loop bodies instead of using call overhead.
    /// Callers of unrolling set this to the
    /// actual map of inlineable callee body weights when available.
    pub(crate) callee_costs: HashMap<FunctionId, usize>,
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
    pub(crate) fn find_all(function: &Function, order: LoopOrder) -> Self {
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

        match order {
            LoopOrder::InsideOut => {
                // Sort by block size descending so we pop and unroll smaller, inner loops first.
                // This is safe for Brillig because if inner loop bounds depend on an outer
                // induction variable, `get_const_bounds` returns None, `is_small_loop` returns
                // false, and we skip it. After unrolling inner loops, outer loops have simpler
                // bodies and more accurate cost estimates for the `is_small_loop` heuristic.
                loops.sort_by_key(|loop_| std::cmp::Reverse(loop_.blocks.len()));
            }
            LoopOrder::OutsideIn => {
                // Sort by block size ascending so we unroll larger, outer loops of nested loops first.
                // This is needed because inner loops may use the induction variable from their
                // outer loops in their loop range.
                loops.sort_by_key(|loop_| loop_.blocks.len());
            }
        }

        Self { yet_to_unroll: loops, cfg, dom: dom_tree, callee_costs: HashMap::default() }
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
        assert!(!arguments.is_empty(), "back edge should have at least 1 argument");
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
    /// `resolve_value` maps ValueIds through an external substitution
    /// (e.g. `FunctionInserter::resolve`).
    /// If `get_const_upper_bound` is called within a pass that modifies instructions
    /// e.g through a `FunctionInserter`, the terminator check below might reference
    /// an old id that needs to be resolved.
    /// If not within a pass (e.g in a test), or if the caller does not use an inserter,
    /// we can safely use the identity `|v| v` instead.
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
    ///
    /// TODO(<https://github.com/noir-lang/noir/issues/11900>): Handle induction variable at any block parameter position
    fn get_const_upper_bound(
        &self,
        dfg: &DataFlowGraph,
        pre_header: BasicBlockId,
        resolve_value: impl Fn(ValueId) -> ValueId,
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

        // Verify that the jmpif condition actually uses the result of this instruction.
        // Without this check we could return a bogus upper bound from an unrelated instruction
        // that happens to be in the header.
        let Some(TerminatorInstruction::JmpIf { then_destination, condition, .. }) =
            header.terminator()
        else {
            return None;
        };
        // Resolve the condition through the provided mapping — during mid-pass
        // the terminator may still reference a pre-substitution ValueId.
        let condition = resolve_value(*condition);
        let results = dfg.instruction_results(instructions[0]);
        if results.first() != Some(&condition) {
            return None;
        }
        let then_branch_is_body = self.blocks.contains(then_destination);

        // The header's instruction must reference the induction variable (first block param).
        // Without this check, an unrelated instruction (e.g. `not` of a function parameter)
        // could be misinterpreted as a loop bound comparison.
        let induction_var = *dfg.block_parameters(self.header).first()?;

        match &dfg[instructions[0]] {
            // Most loops will expect the `then` block to be the body. In unconstrained code it is
            // possible to write `loop`s that use the else branch as a body. We return `None`
            // conservatively in this case.
            Instruction::Binary(Binary { lhs, operator: BinaryOp::Lt, rhs }) => {
                if *lhs != induction_var {
                    return None;
                }
                if then_branch_is_body { dfg.get_integer_constant(*rhs) } else { None }
            }
            Instruction::Binary(Binary { lhs, operator: BinaryOp::Eq, rhs }) => {
                if *lhs != induction_var {
                    return None;
                }
                // `for i in 0..1` is turned into:
                // b1(v0: u32):
                //   v12 = eq v0, u32 0
                //   jmpif v12 then: b2, else: b3
                //
                // If `b2` is the loop body: Loop exits when v == rhs; upper = rhs + 1.
                // If `b3` is the loop body: Loop exits when v == rhs; upper = rhs.
                let const_rhs = dfg.get_integer_constant(*rhs)?;
                if then_branch_is_body { Some(const_rhs.inc()) } else { Some(const_rhs) }
            }
            Instruction::Not(operand) => {
                if *operand != induction_var {
                    return None;
                }
                // We simplify equality operations with booleans like `(boolean == false)` into `!boolean`.
                // Thus, using a u1 in a loop bound can possibly lead to a Not instruction
                // as a loop header's jump condition.
                //
                // Standard (then=body): `for i in 0..1` is turned into:
                //  b1(v0: u1):
                //    v2 = eq v0, u32 0
                //    jmpif v2 then: b2, else: b3
                //
                // Which is further simplified into:
                //  b1(v0: u1):
                //    v2 = not v0
                //    jmpif v2 then: b2, else: b3
                if then_branch_is_body {
                    Some(IntegerConstant::Unsigned { value: 1, bit_size: 1 })
                } else {
                    None
                }
            }
            // A cast of a constant would already be simplified
            Instruction::Cast(_, _) => None,
            _ => {
                // Certain patterns can cause other instructions to be hoisted into the loop
                // header, or at least what looks to be the loop header.
                // `func_1` in `regression_mem2reg_unknown_array_aliases` is one such example
                // if `mem2reg_simple` is performed on it before unrolling.
                None
            }
        }
    }

    /// Get the lower and upper bounds of the loop if both are constant numeric values.
    /// See `get_const_upper_bound` for the role of `resolve_value`.
    pub(super) fn get_const_bounds(
        &self,
        dfg: &DataFlowGraph,
        pre_header: BasicBlockId,
        resolve_value: impl Fn(ValueId) -> ValueId,
    ) -> Option<(IntegerConstant, IntegerConstant)> {
        let lower = self.get_const_lower_bound(dfg, pre_header)?;
        let upper = self.get_const_upper_bound(dfg, pre_header, resolve_value)?;
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
    fn unroll(
        &self,
        function: &mut Function,
        cfg: &ControlFlowGraph,
    ) -> Result<ValueMapping, CallStack> {
        let mut unroll_into = self.get_pre_header(function, cfg)?;
        let mut header_args = get_header_arguments(&function.dfg, unroll_into)?;

        // Collect the original header parameters before unrolling.
        // The header may have extra parameters beyond the induction variable (promoted mutable variables).
        // Blocks outside the loop can reference these params directly, so after unrolling we need to
        // replace those references with the final iteration's values.
        let header_params: Vec<ValueId> = function.dfg[self.header].parameters().to_vec();

        while let Some((context, loop_header_id)) =
            self.unroll_header(function, unroll_into, &header_args)?
        {
            (unroll_into, header_args) = context.unroll_loop_iteration(loop_header_id);
        }

        // Build a mapping from header params to their final values.
        // The caller is responsible for applying this mapping to blocks outside the loop.
        let mut mapping = ValueMapping::default();
        if !header_params.is_empty() {
            mapping.batch_insert(&header_params, &header_args);
        }

        Ok(mapping)
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
        header_args: &[ValueId],
    ) -> Result<Option<(LoopIteration<'a>, BasicBlockId)>, CallStack> {
        // We insert into a fresh block first and move instructions into the unroll_into block later
        // only once we verify the jmpif instruction has a constant condition. If it does not, we can
        // just discard this fresh block and leave the loop unmodified.
        let fresh_block = function.dfg.make_block();

        let mut context = LoopIteration::new(function, self, fresh_block, self.header);
        let loop_header_id = context.source_block;

        // Collect all header parameters before mutably borrowing context.
        // The first parameter is the induction variable; additional parameters are
        // promoted mutable variables (e.g. from mem2reg_simple).
        let header_params: Vec<ValueId> = context.dfg()[loop_header_id].parameters().to_vec();

        // Map each header parameter to the corresponding argument value from the previous iteration
        // (or the initial values from the pre-header for the first iteration).
        for (param, &arg) in header_params.iter().zip(header_args) {
            context.inserter.try_map_value(*param, arg);
        }
        // Copy over all instructions and a fresh terminator.
        context.inline_instructions_from_block();
        context.visited_blocks.insert(loop_header_id);

        // Mutate the terminator if possible so that it points at the iteration block.
        match context.dfg()[fresh_block].unwrap_terminator() {
            TerminatorInstruction::JmpIf {
                condition,
                then_destination,
                then_arguments,
                else_destination,
                else_arguments,
                call_stack,
            } => {
                let condition = *condition;
                let then_destination = *then_destination;
                let then_arguments = then_arguments.clone();
                let else_destination = *else_destination;
                let else_arguments = else_arguments.clone();
                let call_stack = *call_stack;
                let next_blocks = context.handle_jmpif(
                    condition,
                    then_destination,
                    then_arguments,
                    else_destination,
                    else_arguments,
                    call_stack,
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
                    let is_in_loop = self.blocks.contains(&context.source_block);

                    Ok(is_in_loop
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
    /// Find reference values defined before the loop (allocations + reference params).
    ///
    /// Returns `(refs, constant_initial_refs)` where:
    /// - `refs`: all pre-header reference values
    /// - `constant_initial_refs`: the subset of refs whose pre-header stores all have constant values
    fn find_pre_header_reference_values(
        &self,
        function: &Function,
        cfg: &ControlFlowGraph,
    ) -> Option<(HashSet<ValueId>, HashSet<ValueId>)> {
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

        let refs: HashSet<ValueId> = params.chain(allocations).collect();

        // Find refs whose pre-header stores all have constant values.
        // A ref is "constant initial" if it has at least one store in the pre-header blocks
        // AND every such store has a constant value.
        //
        // We must exclude the loop's own blocks from this scan: for nested loops,
        // `find_blocks_in_loop(entry, pre_header)` traverses backward through the
        // outer loop's back-edge and re-enters the inner loop blocks. Without this
        // filter, stores *inside* the loop body (which are not initial values) would
        // incorrectly prevent the ref from being recognized as constant-initial.
        let mut has_store: HashSet<ValueId> = HashSet::default();
        let mut has_non_constant_store: HashSet<ValueId> = HashSet::default();
        for block in blocks.iter().filter(|b| !self.blocks.contains(b)) {
            for instruction_id in function.dfg[*block].instructions() {
                if let Instruction::Store { address, value } = &function.dfg[*instruction_id]
                    && refs.contains(address)
                {
                    has_store.insert(*address);
                    if !function.dfg.is_constant(*value) {
                        has_non_constant_store.insert(*address);
                    }
                }
            }
        }

        // Conservatively mark any reference that appears as a block terminator
        // argument within the loop as non-constant. Such refs can become aliased
        // via block parameters, meaning stores through the alias won't be visible
        // on the original ValueId. Since we only scan pre-header stores by address,
        // aliased refs would be incorrectly classified as constant-initial.
        for block_id in &self.blocks {
            for arg in function.dfg[*block_id].terminator_arguments() {
                has_non_constant_store.insert(*arg);
            }
        }

        let constant_initial_refs: HashSet<ValueId> =
            has_store.difference(&has_non_constant_store).copied().collect();

        Some((refs, constant_initial_refs))
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

    /// Count the total extra block parameter move costs from promoted mutable variables
    /// across all terminators in the loop.
    ///
    /// After unrolling, all Jmp terminators within the loop are eliminated, so every
    /// argument on them (except the induction variable on back-edge Jmps) is boilerplate.
    /// Similarly, JmpIf `then_arguments`/`else_arguments` that thread promoted values
    /// to loop-internal blocks are also boilerplate.
    /// Sum the Brillig-weighted cost of all terminators in the loop whose
    /// destinations are within the loop (including the header). These terminators
    /// are pure boilerplate that disappears entirely after unrolling.
    fn count_terminator_boilerplate(&self, function: &Function) -> usize {
        let mut cost = 0;
        for block_id in &self.blocks {
            match function.dfg[*block_id].unwrap_terminator() {
                t @ TerminatorInstruction::Jmp { destination, .. } => {
                    if self.blocks.contains(destination) || *destination == self.header {
                        cost += t.cost();
                    }
                }
                t @ TerminatorInstruction::JmpIf { then_destination, else_destination, .. } => {
                    // If either branch targets a loop block, the whole JmpIf is boilerplate.
                    if self.blocks.contains(then_destination)
                        || self.blocks.contains(else_destination)
                    {
                        cost += t.cost();
                    }
                }
                _ => {}
            }
        }
        cost
    }

    /// Count the Brillig-weighted cost of instructions in the loop, including terminators.
    ///
    /// When `callee_costs` is non-empty, calls to functions that will be inlined use
    /// the callee's body weight instead of the default call overhead estimate. This
    /// prevents underestimating loop cost when the loop contains calls to functions
    /// that will grow significantly after inlining.
    fn count_loop_cost(
        &self,
        function: &Function,
        callee_costs: &HashMap<FunctionId, usize>,
    ) -> usize {
        self.blocks
            .iter()
            .map(|block_id| {
                let block = &function.dfg[*block_id];
                let instr_cost: usize = block
                    .instructions()
                    .iter()
                    .map(|id| {
                        let instr = &function.dfg[*id];
                        if let Instruction::Call { func, .. } = instr
                            && let Value::Function(func_id) = function.dfg[*func]
                            && let Some(&body_cost) = callee_costs.get(&func_id)
                        {
                            return body_cost;
                        }

                        instr.cost(*id, &function.dfg)
                    })
                    .sum();
                let term_cost = block.terminator().map_or(0, |t| t.cost());
                instr_cost + term_cost
            })
            .sum()
    }

    /// Whether this loop should be unrolled when compiling to Brillig.
    ///
    /// A loop is unrolled if:
    /// 1. It has constant bounds and no breaks (`boilerplate_stats` + `is_fully_executed`)
    /// 2. The iteration count is within the `max_unroll_iterations` limit
    /// 3. AND either:
    ///    a. The cost model predicts unrolling reduces code size (`is_small`), OR
    ///    b. The total unrolled cost is within the force-unroll threshold
    fn should_unroll_in_brillig(
        &self,
        function: &Function,
        loops: &Loops,
        max_unroll_iterations: usize,
        force_unroll_threshold: usize,
    ) -> bool {
        self.boilerplate_stats(function, &loops.cfg, &loops.dom, &loops.callee_costs).is_some_and(
            |s| {
                let within_iteration_limit = s.iterations <= max_unroll_iterations;
                let force_unroll = s.unrolled_cost() <= force_unroll_threshold;
                let is_fully = self.is_fully_executed(&loops.cfg);
                (force_unroll || s.is_small()) && within_iteration_limit && is_fully
            },
        )
    }

    /// Compute the Brillig-weighted cost of instructions that become compile-time
    /// constants after unrolling.
    ///
    /// An instruction is "useless" (will be folded away) if ALL of its operands will be
    /// known constants once the loop is unrolled. We track this with a `constant_after_unroll`
    /// set, seeded with:
    /// - The induction variable (becomes a known constant per unrolled iteration)
    /// - Any value that is already a compile-time constant (`dfg.is_constant`)
    ///
    /// For each instruction in the loop body, if every operand is in `constant_after_unroll`,
    /// the result will also be constant after unrolling, so we add it to the set and
    /// accumulate its Brillig-weighted cost.
    ///
    /// For block parameters, a param is marked constant only when ALL forward
    /// (non-back-edge) in-loop predecessors send constant values at that
    /// position. Back-edges are identified via dominance (dest dominates pred)
    /// and excluded from the agreement check to avoid circular dependencies
    /// in nested loops. This is analogous to LLVM's per-iteration PHI
    /// simulation in `analyzeLoopUnrollCost`.
    fn count_useless_cost(
        &self,
        function: &Function,
        cfg: &ControlFlowGraph,
        dom: &DominatorTree,
        callee_costs: &HashMap<FunctionId, usize>,
        constant_initial_refs: &HashSet<ValueId>,
    ) -> usize {
        let mut useless_cost = 0;
        let Some(induction_var) = self.get_induction_variable(function) else {
            return 0;
        };

        let mut constant_after_unroll: HashSet<ValueId> = HashSet::default();
        constant_after_unroll.insert(induction_var);

        // Seed with header block parameters whose pre-header initial values are constant.
        // This is the promoted-variable analogue of the constant_initial_refs load propagation:
        // after mem2reg, what were loads from constant-initial refs become header params
        // initialized with constant values from the pre-header Jmp.
        if let Ok(pre_header) = self.get_pre_header(function, cfg)
            && let Some(TerminatorInstruction::Jmp { arguments, .. }) =
                function.dfg[pre_header].terminator()
        {
            let header_params = function.dfg[self.header].parameters();
            for (param, arg) in header_params.iter().zip(arguments.iter()) {
                if is_from_constant_source(*arg, &function.dfg) {
                    constant_after_unroll.insert(*param);
                }
            }
        }

        // Track which blocks have been processed so we only seed a param when
        // all its forward predecessors have been visited.
        let mut processed: HashSet<BasicBlockId> = HashSet::default();

        for block in &self.blocks {
            processed.insert(*block);

            for instruction_id in function.dfg[*block].instructions() {
                let results = function.dfg.instruction_results(*instruction_id);
                let instruction = &function.dfg[*instruction_id];

                // Load from a pre-header ref with constant initial store:
                // propagate the result into constant_after_unroll so downstream
                // instructions can cascade, but don't count the load as useless
                // (it's already counted as boilerplate via the load/store pair).
                if let Instruction::Load { address } = instruction
                    && constant_initial_refs.contains(address)
                {
                    for result in results {
                        constant_after_unroll.insert(*result);
                    }
                    continue;
                }

                let mut all_operands_constant = true;
                instruction.for_each_value(|value| {
                    all_operands_constant &= constant_after_unroll.contains(&value)
                        || is_from_constant_source(value, &function.dfg);
                });

                if all_operands_constant {
                    for result in results {
                        constant_after_unroll.insert(*result);
                    }
                    // Use callee body cost for calls, matching count_loop_cost.
                    // Without this, total_cost uses the callee body cost but
                    // useless_cost would only subtract the default call overhead,
                    // inflating useful_cost and preventing unrolling.
                    if let Instruction::Call { func, .. } = instruction
                        && let Value::Function(func_id) = function.dfg[*func]
                        && let Some(&body_cost) = callee_costs.get(&func_id)
                    {
                        useless_cost += body_cost;
                    } else {
                        useless_cost += instruction.cost(*instruction_id, &function.dfg);
                    }
                }
            }

            // Propagate constants through terminator arguments to in-loop
            // successor block parameters, checking that ALL forward (non-back-edge)
            // in-loop predecessors agree before marking a param as constant.
            let terminator = function.dfg[*block].unwrap_terminator();
            let successors = Self::terminator_successors(terminator);
            for (dest, _args) in successors {
                if !self.blocks.contains(&dest) {
                    continue;
                }
                let params = function.dfg[dest].parameters();
                for (i, param) in params.iter().enumerate() {
                    if constant_after_unroll.contains(param) {
                        continue;
                    }
                    // Collect forward (non-back-edge) in-loop predecessors.
                    // A pred forms a back-edge if dest dominates it.
                    let forward_preds: Vec<_> = cfg
                        .predecessors(dest)
                        .filter(|p| self.blocks.contains(p) && !dom.dominates_helper(dest, *p))
                        .collect();
                    if forward_preds.is_empty() {
                        continue;
                    }
                    // Only seed when all forward preds have been processed.
                    if !forward_preds.iter().all(|p| processed.contains(p)) {
                        continue;
                    }
                    let all_agree = forward_preds.iter().all(|&pred| {
                        Self::pred_sends_constant_at(
                            &function.dfg,
                            pred,
                            dest,
                            i,
                            &constant_after_unroll,
                        )
                    });
                    if all_agree {
                        constant_after_unroll.insert(*param);
                    }
                }
            }
        }
        useless_cost
    }

    /// Extract (destination, arguments) pairs from a terminator instruction.
    fn terminator_successors(
        terminator: &TerminatorInstruction,
    ) -> Vec<(BasicBlockId, &[ValueId])> {
        match terminator {
            TerminatorInstruction::Jmp { destination, arguments, .. } => {
                vec![(*destination, arguments)]
            }
            TerminatorInstruction::JmpIf {
                then_destination,
                then_arguments,
                else_destination,
                else_arguments,
                ..
            } => {
                vec![(*then_destination, then_arguments), (*else_destination, else_arguments)]
            }
            _ => vec![],
        }
    }

    /// Check whether `source`'s terminator sends a constant value at position
    /// `param_index` to `target`. For JmpIf where both branches go to the same
    /// target, both must send constants.
    fn pred_sends_constant_at(
        dfg: &DataFlowGraph,
        source: BasicBlockId,
        target: BasicBlockId,
        param_index: usize,
        constant_after_unroll: &HashSet<ValueId>,
    ) -> bool {
        let is_const =
            |v: &ValueId| constant_after_unroll.contains(v) || is_from_constant_source(*v, dfg);

        let Some(terminator) = dfg[source].terminator() else {
            return false;
        };
        match terminator {
            TerminatorInstruction::Jmp { destination, arguments, .. } => {
                *destination == target && arguments.get(param_index).is_some_and(is_const)
            }
            TerminatorInstruction::JmpIf {
                then_destination,
                then_arguments,
                else_destination,
                else_arguments,
                ..
            } => {
                let then_ok = *then_destination != target
                    || then_arguments.get(param_index).is_some_and(is_const);
                let else_ok = *else_destination != target
                    || else_arguments.get(param_index).is_some_and(is_const);
                // At least one branch must target this block
                (*then_destination == target || *else_destination == target) && then_ok && else_ok
            }
            _ => false,
        }
    }

    /// Collect boilerplate stats if we can figure out the upper and lower bounds of the loop,
    /// and the loop doesn't have multiple back-edges from breaks and continues.
    fn boilerplate_stats(
        &self,
        function: &Function,
        cfg: &ControlFlowGraph,
        dom: &DominatorTree,
        callee_costs: &HashMap<FunctionId, usize>,
    ) -> Option<BoilerplateStats> {
        let pre_header = self.get_pre_header(function, cfg).ok()?;
        let (lower, upper) = self.get_const_bounds(&function.dfg, pre_header, |v| v)?;
        let (refs, constant_initial_refs) = self.find_pre_header_reference_values(function, cfg)?;

        // If we have a break block, we can potentially directly use the induction variable in that break.
        // If we then unroll the loop, the induction variable will not exist anymore.
        let is_fully_executed = self.is_fully_executed(cfg);

        let (loads, stores) = self.count_loads_and_stores(function, &refs);
        let total_cost = self.count_loop_cost(function, callee_costs);

        let useless_cost = if !is_fully_executed {
            0
        } else {
            self.count_useless_cost(function, cfg, dom, callee_costs, &constant_initial_refs)
        };

        let terminator_boilerplate = self.count_terminator_boilerplate(function);
        let header_params = function.dfg[self.header].parameters().len();

        // Currently we don't iterate in reverse, so if upper <= lower it means 0 iterations.
        let iterations: usize = upper
            .reduce(
                lower,
                |u, l| u.saturating_sub(l).max(0) as usize,
                |u, l| u.saturating_sub(l) as usize,
            )
            .unwrap_or_default();

        let stats = BoilerplateStats {
            iterations,
            loads,
            stores,
            total_cost,
            useless_cost,
            terminator_boilerplate,
            header_params,
        };
        Some(stats)
    }
}

/// Check if a value ultimately comes from constant data by tracing through
/// `array_get` instructions. Returns true if:
/// - The value is a compile-time constant (`dfg.is_constant`), OR
/// - The value is the result of an `array_get` whose array operand
///   recursively traces back to a constant source (global, MakeArray)
///
/// This lets the cost model recognize that `array_get constant_array, index`
/// will fold away after unrolling, even when `constant_array` is itself
/// the result of indexing into a higher-dimensional constant.
fn is_from_constant_source(value: ValueId, dfg: &DataFlowGraph) -> bool {
    if dfg.is_constant(value) {
        return true;
    }
    match dfg.get_local_or_global_instruction(value) {
        Some(Instruction::ArrayGet { array, .. }) => is_from_constant_source(*array, dfg),
        _ => false,
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
    /// Number of loads of pre-header references in the loop.
    loads: usize,
    /// Number of stores into pre-header references in the loop.
    stores: usize,
    /// Brillig-weighted cost of instructions in the loop, including boilerplate,
    /// but excluding the boilerplate which is outside the loop.
    total_cost: usize,
    /// Brillig-weighted cost of "useless" instructions that become compile-time constants
    /// after unrolling. This includes the bound comparison (lt), induction variable
    /// increments, and any other instructions whose operands are all known after unrolling.
    useless_cost: usize,
    /// Sum of `TerminatorInstruction::cost()` for all terminators in the loop
    /// whose destinations are within the loop (including the header).
    /// These terminators disappear entirely after unrolling.
    terminator_boilerplate: usize,
    /// Number of header block parameters (including the induction variable).
    /// Used to compute the pre-header Jmp cost in `baseline_cost`.
    header_params: usize,
}

impl BoilerplateStats {
    /// Brillig-weighted cost if we leave the loop as-is.
    /// It's the cost of the loop body, plus the pre-header jmp that kicks it off.
    fn baseline_cost(&self) -> usize {
        // Pre-header jmp: 1 (jump) + N moves for header params
        let pre_header_jmp = 1 + self.header_params;
        self.total_cost + pre_header_jmp
    }

    /// Per-iteration cost excluding boilerplate but NOT subtracting useless_cost.
    /// This is the conservative estimate: it assumes no constant folding happens.
    fn conservative_useful_cost(&self) -> usize {
        let load_and_store = self.loads.min(self.stores) * 2;
        let total_boilerplate = load_and_store + self.terminator_boilerplate;
        assert!(
            total_boilerplate <= self.total_cost,
            "Boilerplate cost exceeds total cost in loop"
        );
        self.total_cost.saturating_sub(total_boilerplate)
    }

    /// Estimated Brillig-weighted cost of _useful_ instructions, which is the
    /// cost of the loop minus all in-loop boilerplate and useless (constant-foldable)
    /// instructions.
    fn useful_cost(&self) -> usize {
        self.conservative_useful_cost().saturating_sub(self.useless_cost)
    }

    /// Estimated Brillig-weighted cost if we unroll the loop.
    fn unrolled_cost(&self) -> usize {
        self.useful_cost().saturating_mul(self.iterations)
    }

    /// Conservative estimate of unrolled cost that excludes useless_cost.
    ///
    /// Unlike `unrolled_cost()` which assumes constant-foldable instructions will be
    /// eliminated, this gives the cost if NO folding happens. Used by `is_small()` to
    /// avoid over-aggressive unrolling of large loops whose `useless_cost` may be
    /// overestimated (e.g. loops containing previously-unrolled inner loops).
    /// The `force_unroll` path still uses `unrolled_cost()` with full useless_cost
    /// subtraction, ensuring genuinely tiny loops are still unrolled.
    fn conservative_unrolled_cost(&self) -> usize {
        self.conservative_useful_cost().saturating_mul(self.iterations)
    }

    /// A small loop is where if we unroll it into the pre-header then considering the
    /// number of iterations we still end up with a smaller bytecode than if we leave
    /// the blocks in tact with all the boilerplate involved in jumping, and the extra
    /// reference access overhead.
    ///
    /// Uses `conservative_unrolled_cost` (without useless_cost subtraction) to avoid
    /// false positives from overestimated constant folding, particularly for loops
    /// containing previously-unrolled inner loops.
    fn is_small(&self) -> bool {
        self.conservative_unrolled_cost() < self.baseline_cost()
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
///
/// TODO(<https://github.com/noir-lang/noir/issues/11900>): Handle induction variable at any block parameter position
fn get_induction_variable(dfg: &DataFlowGraph, block: BasicBlockId) -> Result<ValueId, CallStack> {
    match dfg[block].terminator() {
        Some(TerminatorInstruction::Jmp { arguments, call_stack: location, .. }) => {
            // This assumption will no longer be valid if e.g. mutable variables are represented as
            // block parameters. If that becomes the case we'll need to figure out which variable
            // is generally constant and increasing to guess which parameter is the induction
            // variable.
            if arguments.is_empty() {
                // It is expected that a loop's induction variable is the first block parameter of the loop header.
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

/// Get all arguments of the jump into the loop header from the pre-header block.
///
/// The first argument is the induction variable (must be a numeric constant).
/// Additional arguments are promoted mutable variables (e.g. from mem2reg_simple).
/// Returns `Err` if the block does not end with a `Jmp`, has no arguments, or the
/// first argument is not a numeric constant.
fn get_header_arguments(
    dfg: &DataFlowGraph,
    block: BasicBlockId,
) -> Result<Vec<ValueId>, CallStack> {
    match dfg[block].terminator() {
        Some(TerminatorInstruction::Jmp { arguments, call_stack: location, .. }) => {
            if arguments.is_empty() {
                return Err(dfg.get_call_stack(*location));
            }
            let induction = arguments[0];
            if dfg.get_numeric_constant(induction).is_none() {
                return Err(dfg.get_call_stack(*location));
            }
            Ok(arguments.clone())
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

    /// All back-edge arguments (and the block they were found in) for the next loop iteration.
    /// The first element is the new induction variable; additional elements are promoted
    /// mutable variables (e.g. from mem2reg_simple).
    /// This is None until we visit the block which jumps back to the start of the
    /// loop, at which point we record the arguments and the block they were found in.
    induction_value: Option<(BasicBlockId, Vec<ValueId>)>,
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
    fn unroll_loop_iteration(
        mut self,
        loop_header_id: BasicBlockId,
    ) -> (BasicBlockId, Vec<ValueId>) {
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
        let (end_block, all_args) = self
            .induction_value
            .expect("Expected to find the induction variable by end of loop iteration");

        assert!(
            self.encountered_loop_header,
            "expected to encounter loop header when visiting blocks"
        );

        (end_block, all_args)
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
                then_arguments,
                else_destination,
                else_arguments,
                call_stack,
            } => {
                let (condition, then_destination, else_destination, call_stack) =
                    (*condition, *then_destination, *else_destination, *call_stack);
                let then_arguments = then_arguments.clone();
                let else_arguments = else_arguments.clone();
                self.handle_jmpif(
                    condition,
                    then_destination,
                    then_arguments,
                    else_destination,
                    else_arguments,
                    call_stack,
                )
            }
            TerminatorInstruction::Jmp { destination, arguments, call_stack: _ } => {
                if *destination == self.loop_.header {
                    // We found the back-edge of the loop.
                    assert!(!arguments.is_empty(), "back-edge should have at least 1 argument");
                    assert!(self.induction_value.is_none(), "there should be only one back-edge");
                    self.induction_value = Some((self.insert_block, arguments.clone()));
                    self.encountered_loop_header = true;
                    // Don't enqueue the header as a next block: it was already visited
                    // at the start of this iteration. The next call to `unroll_header`
                    // in the `unroll()` while loop will handle the header for the next
                    // iteration.
                    vec![]
                } else {
                    vec![*destination]
                }
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
        then_arguments: Vec<ValueId>,
        else_destination: BasicBlockId,
        else_arguments: Vec<ValueId>,
        call_stack: CallStackId,
    ) -> Vec<BasicBlockId> {
        let condition = self.inserter.resolve(condition);

        match self.dfg().get_numeric_constant(condition) {
            Some(constant) => {
                let (destination, arguments) = if constant.is_zero() {
                    (else_destination, else_arguments)
                } else {
                    (then_destination, then_arguments)
                };

                self.source_block = self.get_original_block(destination);

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
    ///
    /// The loop header is also returned as-is: the header is handled separately
    /// by `unroll_header`, so creating a fresh block for it here would leave an
    /// orphan block with no terminator if unrolling is later aborted.
    fn get_or_insert_block(&mut self, block: BasicBlockId) -> BasicBlockId {
        if block == self.loop_.header {
            return block;
        }

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

/// Pre-check condition for [Function::unroll_loops_iteratively].
#[cfg(debug_assertions)]
fn unroll_loops_pre_check(function: &Function) {
    super::checks::assert_no_constant_jmpif(function);
}

/// Post-check condition for [Function::unroll_loops_iteratively].
///
/// Panics if:
///   - Any ACIR function still contains loops after unrolling.
///
/// Note: This check only runs for ACIR functions since Brillig functions
/// may intentionally retain loops that are too large to unroll.
#[cfg(debug_assertions)]
fn unroll_loops_post_check(function: &Function) {
    if function.runtime().is_acir() {
        // All loops should be unrolled in ACIR functions
        super::checks::assert_no_loops(function);
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use crate::assert_ssa_snapshot;
    use crate::errors::RuntimeError;
    use crate::ssa::interpreter::value::Value;
    use crate::ssa::ir::cfg::ControlFlowGraph;
    use crate::ssa::ir::integer::IntegerConstant;
    use crate::ssa::opt::assert_ssa_does_not_change;
    use crate::ssa::{Ssa, ir::value::ValueId, opt::assert_normalized_ssa_equals};

    use super::{
        BoilerplateStats, FORCE_UNROLL_THRESHOLD, HashMap, LoopOrder, Loops, MAX_UNROLL_ITERATIONS,
        is_new_size_ok,
    };

    /// Tries to unroll all loops in each SSA function once, calling the `Function` directly,
    /// bypassing the iterative loop done by the SSA which does further optimizations.
    ///
    /// If any loop cannot be unrolled, it is left as-is or in a partially unrolled state.
    fn try_unroll_loops(mut ssa: Ssa) -> (Ssa, Vec<RuntimeError>) {
        let mut errors = vec![];
        for function in ssa.functions.values_mut() {
            errors.extend(
                function
                    .try_unroll_loops(
                        MAX_UNROLL_ITERATIONS,
                        FORCE_UNROLL_THRESHOLD,
                        &HashMap::default(),
                    )
                    .1,
            );
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
                    jmpif v1 then: b2(), else: b3()
                b2():
                    jmp b4(u32 0)
                b4(v2: u32):  // header of inner loop
                    v3 = lt v2, u32 4
                    jmpif v3 then: b5(), else: b6()
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
            jmpif v2 then: b2(), else: b3()
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
        let loops = Loops::find_all(function, LoopOrder::OutsideIn);
        assert_eq!(loops.yet_to_unroll.len(), 1);

        let loop_ = &loops.yet_to_unroll[0];
        let pre_header =
            loop_.get_pre_header(function, &loops.cfg).expect("Should have a pre_header");
        let (lower, upper) = loop_
            .get_const_bounds(&function.dfg, pre_header, |v| v)
            .expect("bounds are numeric const");

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
            jmpif u1 0 then: b2(), else: b3()
          b2():
            v41 = unchecked_add v0, u32 1
            jmp b1(v41)
          b3():
            return
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let loops = Loops::find_all(function, LoopOrder::OutsideIn);
        assert_eq!(loops.yet_to_unroll.len(), 1);

        let loop_ = &loops.yet_to_unroll[0];
        let pre_header =
            loop_.get_pre_header(function, &loops.cfg).expect("Should have a pre_header");
        let (lower, upper) = loop_
            .get_const_bounds(&function.dfg, pre_header, |v| v)
            .expect("should use the lower for upper");

        assert_eq!(lower, IntegerConstant::Unsigned { value: 0, bit_size: 32 });
        assert_eq!(upper, lower);
    }

    #[test]
    fn test_find_pre_header_reference_values() {
        let ssa = brillig_unroll_test_case();
        let function = ssa.main();
        let mut loops = Loops::find_all(function, LoopOrder::OutsideIn);
        let loop0 = loops.yet_to_unroll.pop().unwrap();

        let (refs, constant_initial_refs) =
            loop0.find_pre_header_reference_values(function, &loops.cfg).unwrap();
        assert_eq!(refs.len(), 1);
        assert!(refs.contains(&ValueId::test_new(2)));
        assert_eq!(constant_initial_refs.len(), 1);
        assert!(constant_initial_refs.contains(&ValueId::test_new(2)));

        let (loads, stores) = loop0.count_loads_and_stores(function, &refs);
        assert_eq!(loads, 1);
        assert_eq!(stores, 1);

        let all = loop0.count_loop_cost(function, &HashMap::default());
        assert_eq!(all, 13);
    }

    #[test]
    fn test_boilerplate_stats() {
        let ssa = brillig_unroll_test_case();
        let stats = loop0_stats(&ssa);
        assert_eq!(stats.iterations, 4);
        assert_eq!(stats.total_cost, 3 + 10); // Brillig-weighted cost in b1 and b3
        assert_eq!(stats.loads, 1);
        assert_eq!(stats.stores, 1);
        // lt(1) + add of load+induction(3) + increment(3)
        assert_eq!(stats.useless_cost, 7);
        assert_eq!(stats.useful_cost(), 0);
        assert_eq!(stats.baseline_cost(), 15);
        // useful_cost = 0 → unrolled_cost = 0, force_unrolled via threshold.
        // is_small uses conservative_unrolled_cost (without useless subtraction)
        // which is higher, but force_unroll handles this case.
        assert_eq!(stats.unrolled_cost(), 0);
    }

    #[test]
    fn test_boilerplate_stats_i64_empty() {
        // Looping 0..-1, which should be 0 iterations.
        // u64::MAX is how -1 is represented as a Field.
        let ssa = Ssa::from_str(&brillig_unroll_test_case_6470_with_params(
            "i64",
            "0",
            &format!("{}", u64::MAX),
        ))
        .unwrap();

        let stats = loop0_stats(&ssa);
        assert_eq!(stats.iterations, 0);
        assert_eq!(stats.unrolled_cost(), 0);
    }

    #[test]
    fn test_boilerplate_stats_i64_non_empty() {
        // Looping -4..-1, which should be 3 iterations.
        // u64::MAX-3 is how -4 is represented as a Field.
        let ssa = Ssa::from_str(&brillig_unroll_test_case_6470_with_params(
            "i64",
            &format!("{}", u64::MAX - 3),
            &format!("{}", u64::MAX),
        ))
        .unwrap();
        let stats = loop0_stats(&ssa);
        assert_eq!(stats.iterations, 3);
    }

    #[test]
    fn test_boilerplate_stats_6470() {
        let ssa = Ssa::from_str(&brillig_unroll_test_case_6470(2)).unwrap();
        let stats = loop0_stats(&ssa);
        assert_eq!(stats.iterations, 2);
        assert_eq!(stats.total_cost, 3 + 19); // Brillig-weighted cost in b1 and b3
        assert_eq!(stats.loads, 1);
        assert_eq!(stats.stores, 1);
        // lt(1) + 2x unchecked_add(1 each) = 3; v0 is runtime so array_get/add/array_set are NOT useless
        assert_eq!(stats.useless_cost, 3);
        assert_eq!(stats.useful_cost(), 13); // array_get(3) + add(3) + array_set(7)
        assert_eq!(stats.baseline_cost(), 24);
        // Not small with Brillig weights: 13*2=26 > 24, but within force-unroll threshold
        assert!(!stats.is_small());
    }

    #[test]
    fn test_boilerplate_stats_const_zero_jump_condition() {
        let src = "
        brillig(inline) impure fn main f0 {
          b0():
            jmp b1(u32 0)
          b1(v0: u32):
            jmpif u1 0 then: b2(), else: b3()
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

    #[test]
    fn test_boilerplate_stats_constant_array_source() {
        // v3 is a constant array defined outside the loop.
        // Inside the loop, `array_get v3, index v0` should be recognized as
        // useless because v3 traces back to constant data.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v2 = allocate -> &mut u32
            store u32 0 at v2
            v3 = make_array [u32 10, u32 20, u32 30, u32 40] : [u32; 4]
            jmp b1(u32 0)
          b1(v0: u32):
            v5 = lt v0, u32 4
            jmpif v5 then: b2(), else: b3()
          b2():
            v6 = load v2 -> u32
            v7 = array_get v3, index v0 -> u32
            v8 = add v6, v7
            store v8 at v2
            v9 = unchecked_add v0, u32 1
            jmp b1(v9)
          b3():
            v10 = load v2 -> u32
            return v10
        }";
        let ssa = Ssa::from_str(src).unwrap();
        let stats = loop0_stats(&ssa);
        // is_from_constant_source recognizes v3 as constant even though
        // it's outside the loop and not in constant_after_unroll.
        // Load v6 from v2 (constant initial store u32 0) → v6 in constant_after_unroll.
        // Useless: lt, array_get (constant source + induction var), add (v6 + v7 both constant), unchecked_add = 4
        // all=7, boilerplate=2 (jmpif+jmp), loads=1, stores=1, useless=4
        // useful = 7 - 2 - 2 - 4 = 0 (store folds too but is already boilerplate)
        // lt(1) + array_get(3) + add(3) + unchecked_add(1) = 8
        assert_eq!(stats.useless_cost, 8);
        assert_eq!(stats.useful_cost(), 0);
        // useful_cost = 0 → unrolled_cost = 0, force_unrolled via threshold.
        assert_eq!(stats.unrolled_cost(), 0);
    }

    /// Regression test for nested loops with an accumulator (simplified regression_4709).
    ///
    /// The inner loop (b3, b4) accumulates values from a constant array indexed by
    /// the outer loop's induction variable. Without the filter that excludes `self.blocks`
    /// from the constant-initial store scan, the inner loop's own store (`store v10 at v2`)
    /// would be seen as a pre-header store with a non-constant value, preventing load
    /// propagation and making the `add` instruction appear "useful".
    ///
    /// With 35 inner iterations and 1 useful instruction: unrolled = 35 > baseline (8),
    /// so the loop would NOT be unrolled. With the fix, all instructions are useless,
    /// unrolled = 0, and the loop IS unrolled.
    #[test]
    fn test_boilerplate_stats_nested_loop_constant_initial_ref() {
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v2 = allocate -> &mut Field
            store Field 0 at v2
            v3 = make_array [Field 10, Field 20, Field 30, Field 40] : [Field; 4]
            jmp b1(u32 0)
          b1(v0: u32):
            v5 = lt v0, u32 4
            jmpif v5 then: b2(), else: b5()
          b2():
            v7 = array_get v3, index v0 -> Field
            jmp b3(u32 0)
          b3(v1: u32):
            v8 = lt v1, u32 35
            jmpif v8 then: b4(), else: b6()
          b4():
            v9 = load v2 -> Field
            v10 = add v9, v7
            store v10 at v2
            v11 = unchecked_add v1, u32 1
            jmp b3(v11)
          b6():
            v12 = unchecked_add v0, u32 1
            jmp b1(v12)
          b5():
            v13 = load v2 -> Field
            return v13
        }";
        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let mut loops = Loops::find_all(function, LoopOrder::OutsideIn);
        // OutsideIn puts outer loop last; remove(0) gets the inner loop.
        assert_eq!(loops.yet_to_unroll.len(), 2, "should find outer and inner loops");
        let inner = loops.yet_to_unroll.remove(0);
        let stats =
            inner.boilerplate_stats(function, &loops.cfg, &loops.dom, &loops.callee_costs).unwrap();
        // Inner loop: blocks b3 (lt + jmpif) and b4 (load + add + store + unchecked_add + jmp)
        assert_eq!(stats.iterations, 35);
        assert_eq!(stats.loads, 1);
        assert_eq!(stats.stores, 1);
        // v2 has constant initial store (Field 0 in b0). The filter excludes b4's
        // non-constant store from the scan, so load propagation works.
        // lt(1) + add (propagated load + constant source)(1) + unchecked_add(1) = 3
        assert_eq!(stats.useless_cost, 3);
        assert_eq!(stats.useful_cost(), 0);
        // useful_cost = 0 → unrolled_cost = 0, force_unrolled via threshold.
        assert_eq!(stats.unrolled_cost(), 0);
    }

    /// A reference passed as a block terminator argument is NOT classified
    /// as constant-initial.
    ///
    /// v2 is passed into the loop header as a block param and the back-edge swaps it with v4, creating an alias.
    /// Stores through the alias (v0, which takes on v4's value) are not visible when
    /// scanning pre-header stores for v4, so v4 would be incorrectly classified as
    /// constant-initial without the terminator filter.
    #[test]
    fn test_boilerplate_stats_ref_block_param_alias() {
        // Two allocations v2 and v4. v2 is passed as block param to loop header (b1),
        // and the back-edge (b2 -> b1) swaps to v4. This creates an alias: the loop
        // header param v0 can be either v2 or v4 depending on iteration.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v2 = allocate -> &mut Field
            store Field 0 at v2
            v4 = allocate -> &mut Field
            store Field 1 at v4
            jmp b1(v2, u32 0)
          b1(v0: &mut Field, v1: u32):
            v6 = lt v1, u32 4
            jmpif v6 then: b2(), else: b3()
          b2():
            v7 = load v0 -> Field
            v8 = add v7, Field 1
            store v8 at v0
            v10 = unchecked_add v1, u32 1
            jmp b1(v4, v10)
          b3():
            v11 = load v2 -> Field
            v12 = load v4 -> Field
            v13 = add v11, v12
            return v13
        }";
        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let mut loops = Loops::find_all(function, LoopOrder::OutsideIn);
        assert_eq!(loops.yet_to_unroll.len(), 1);
        let loop0 = loops.yet_to_unroll.pop().unwrap();

        let v2 = ValueId::test_new(2);
        let v4 = ValueId::test_new(4);

        let (refs, constant_initial_refs) =
            loop0.find_pre_header_reference_values(function, &loops.cfg).unwrap();
        // Both v2 and v4 are reference allocations visible in the pre-header.
        assert!(refs.contains(&v2), "v2 should be in refs");
        assert!(refs.contains(&v4), "v4 should be in refs");

        // v2 is only passed in b0's terminator (outside the loop), so it stays.
        assert!(
            constant_initial_refs.contains(&v2),
            "v2 should be constant-initial (store Field 0 in pre-header)"
        );
        // v4 appears in b2's `jmp b1(v4, v10)` — a terminator inside the loop —
        // so it must be removed by the filter.
        assert!(
            !constant_initial_refs.contains(&v4),
            "v4 should NOT be constant-initial (aliased via block param)"
        );
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
        let ssa = Ssa::from_str(&brillig_unroll_test_case_6470(2)).unwrap();
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
            return v15
        }
        ");
    }

    /// Test that with more iterations it's not unrolled.
    #[test]
    fn test_brillig_unroll_6470_large() {
        // 13 iterations × 13 useful Brillig cost = 169, above FORCE_UNROLL_THRESHOLD (128)
        let parse_ssa = || Ssa::from_str(&brillig_unroll_test_case_6470(13)).unwrap();
        let ssa = parse_ssa();
        let stats = loop0_stats(&ssa);
        assert!(!stats.is_small(), "the loop should be considered large");
        assert!(
            stats.unrolled_cost() > FORCE_UNROLL_THRESHOLD,
            "the loop should exceed the force-unroll threshold"
        );

        let (ssa, errors) = try_unroll_loops(ssa);
        assert_eq!(errors.len(), 0, "Unroll should have no errors");
        // Check that it's still the original
        assert_normalized_ssa_equals(ssa, &parse_ssa().print_without_locations().to_string());
    }

    #[test]
    fn test_brillig_unroll_iteratively_respects_max_increase() {
        let ssa = brillig_unroll_test_case();
        let ssa = ssa
            .unroll_loops_iteratively(Some(-90), MAX_UNROLL_ITERATIONS, FORCE_UNROLL_THRESHOLD)
            .unwrap();
        // Check that it's still the original
        let expected = brillig_unroll_test_case();
        assert_normalized_ssa_equals(ssa, &expected.print_without_locations().to_string());
    }

    #[test]
    fn test_brillig_unroll_iteratively_with_large_max_increase() {
        let ssa = brillig_unroll_test_case();
        let ssa = ssa
            .unroll_loops_iteratively(Some(50), MAX_UNROLL_ITERATIONS, FORCE_UNROLL_THRESHOLD)
            .unwrap();
        // Check that it did the unroll (simplification after unrolling may merge blocks)
        assert_eq!(ssa.main().reachable_blocks().len(), 1, "The loop should be unrolled");
    }

    /// Test that setting force_unroll_threshold to 0 disables force-unrolling.
    ///
    /// This uses a loop with 6 iterations where:
    /// - is_small() = false (unrolled cost exceeds baseline)
    /// - unrolled_cost = 78 (within default threshold of 128)
    ///
    /// With the default threshold, this loop would be force-unrolled.
    /// With threshold=0, it should NOT be unrolled.
    #[test]
    fn test_brillig_force_unroll_threshold_zero_disables_unrolling() {
        let parse_ssa = || Ssa::from_str(&brillig_unroll_test_case_6470(6)).unwrap();
        let ssa = parse_ssa();

        // Verify the loop's properties match our expectations
        let stats = loop0_stats(&ssa);
        assert!(!stats.is_small(), "loop should not be small according to cost model");
        assert!(
            stats.unrolled_cost() <= FORCE_UNROLL_THRESHOLD,
            "loop should be within default force-unroll threshold"
        );

        assert_ssa_does_not_change(&brillig_unroll_test_case_6470(6), |ssa| {
            // With threshold=0, the loop should NOT be unrolled
            ssa.unroll_loops_iteratively(None, 0, 0).unwrap()
        });
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
            jmpif v5 then: b2(), else: b6()
          b2():
            v7 = eq v0, u32 2
            jmpif v7 then: b7(), else: b3()
          b3():
            v11 = eq v0, u32 5
            jmpif v11 then: b5(), else: b4()
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
            jmpif v5 then: b3(), else: b2()
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
    fn brillig_unroll_test_case_6470(num_iterations: usize) -> String {
        brillig_unroll_test_case_6470_with_params("u32", "0", &format!("{num_iterations}"))
    }

    fn brillig_unroll_test_case_6470_with_params(
        idx_type: &str,
        lower: &str,
        upper: &str,
    ) -> String {
        if idx_type == "u32" {
            format!(
                "
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
            jmpif v7 then: b3(), else: b2()
          b3():
            v9 = load v4 -> [u64; 6]
            v11 = array_get v0, index v1 -> u64
            v12 = add v11, u64 1
            v13 = array_set v9, index v1, value v12
            v15 = unchecked_add v1, {idx_type} 1
            store v13 at v4
            v16 = unchecked_add v1, {idx_type} 1 // duplicate
            jmp b1(v16)
          b2():
            v8 = load v4 -> [u64; 6]
            return v8
        }}
        "
            )
        } else {
            format!(
                "
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
            jmpif v7 then: b3(), else: b2()
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
            return v8
        }}
        "
            )
        }
    }

    // Boilerplate stats of the first loop in the SSA.
    fn loop0_stats(ssa: &Ssa) -> BoilerplateStats {
        let function = ssa.main();
        let mut loops = Loops::find_all(function, LoopOrder::OutsideIn);
        let loop0 = loops.yet_to_unroll.pop().expect("there should be a loop");
        loop0
            .boilerplate_stats(function, &loops.cfg, &loops.dom, &HashMap::default())
            .expect("there should be stats")
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
            jmpif v3 then: b2(), else: b3()
          b2():
            jmpif u1 1 then: b4(), else: b5()
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

    /// Regression test: a for-loop with a conditional break from a body block
    /// should not be unrolled by Brillig. The break creates an exit edge from
    /// a non-header block to outside the loop. Previously, `is_fully_executed`
    /// only checked the header's exit block, missing body exits, which caused
    /// unrolling to panic with "destination not in original loop".
    #[test]
    fn do_not_unroll_loop_with_body_break() {
        // for i in 0..2 {
        //     if some_call() { break; }
        //     println(i);
        // }
        let src = r#"
        brillig(inline) impure fn main f0 {
          b0():
            jmp b1(u32 0)
          b1(v0: u32):
            v3 = lt v0, u32 2
            jmpif v3 then: b2(), else: b3()
          b2():
            v4 = call f1() -> u1
            jmpif v4 then: b4(), else: b5()
          b3():
            return
          b4():
            jmp b3()
          b5():
            v6 = unchecked_add v0, u32 1
            jmp b1(v6)
        }
        brillig(inline) fn f1 f1 {
          b0():
            return u1 0
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let (ssa, errors) = try_unroll_loops(ssa);
        assert_eq!(errors.len(), 0, "Unroll should have no errors");

        // The SSA is expected to be unchanged because the loop has a body break
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
            jmpif v3 then: b2(), else: b3()
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
            jmpif v2 then: b2(), else: b3()
          b2():
            jmp b1()
          b3():
            return
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let mut loops = Loops::find_all(function, LoopOrder::OutsideIn);
        let loop0 = loops.yet_to_unroll.pop().expect("there should be a loop");
        let pre_header = loop0.get_pre_header(function, &loops.cfg).unwrap();
        assert!(loop0.get_const_lower_bound(&function.dfg, pre_header).is_none());
        assert!(loop0.get_const_upper_bound(&function.dfg, pre_header, |v| v).is_none());
    }

    #[test]
    fn unroll_loop_upper_bound_saturated() {
        // We need to avoid overflow when the loop bounds is `u128::MAX`. In this case,
        // the loop body is in the `else` case so we fail to unroll entirely.
        let ssa = format!(
            r#"
        acir(inline) fn main f0 {{
          b0():
            jmp b1(u128 {0})
          b1(v0: u128):
            v3 = eq v0, u128 {0}
            jmpif v3 then: b3(), else: b2()
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

        let loops = Loops::find_all(function, LoopOrder::OutsideIn);
        assert_eq!(loops.yet_to_unroll.len(), 1);

        let loop_ = &loops.yet_to_unroll[0];
        let pre_header =
            loop_.get_pre_header(function, &loops.cfg).expect("Should have a pre_header");
        let (lower, upper) = loop_
            .get_const_bounds(&function.dfg, pre_header, |v| v)
            .expect("bounds are numeric const");
        assert_eq!(lower, upper);
    }

    /// Prior passes can place non-comparison instructions (like MakeArray) into a loop header block
    /// alongside a constant-condition JmpIf.
    ///
    /// The pre-check should catch this and require simplify_cfg to be run first.
    #[test]
    #[should_panic(expected = "has a JmpIf with a constant condition")]
    fn pre_check_rejects_const_condition_jmpif_in_loop_header() {
        let src = "
        acir(inline) impure fn main f0 {
          b0():
            call f1(u1 1)
            return
        }
        brillig(inline) impure fn func f1 {
          b0(v0: u1):
            v2 = not v0
            v3 = allocate -> &mut u1
            store u1 1 at v3
            jmp b1(u8 0)
          b1(v1: u8):
            v24 = make_array b\"unsignedinteger\"
            jmpif u1 0 then: b2(), else: b3()
          b2():
            inc_rc v24
            v29 = unchecked_add v1, u8 1
            jmp b1(v29)
          b3():
            v26 = load v3 -> u1
            jmpif v26 then: b4(), else: b5()
          b4():
            inc_rc v24
            jmp b5()
          b5():
            return
        }";
        let ssa = Ssa::from_str(src).unwrap();
        // This should panic because b1 has a constant-condition `jmpif u1 0`.
        let _ = ssa.unroll_loops_iteratively(None, MAX_UNROLL_ITERATIONS, FORCE_UNROLL_THRESHOLD);
    }

    #[test]
    fn handles_jmpif_args() {
        let src = r#"
            brillig(inline) predicate_pure fn main f0 {
              b0():
                v0 = make_array [] : [i32]
                call f1(u32 0, v0)
                return
            }
            brillig(inline) predicate_pure fn iter_0_times f1 {
              b0(v0: u32, v1: [i32]):
                jmp b1(u32 0)
              b1(v2: u32):
                v4 = eq v2, u32 0
                jmpif v4 then: b2(), else: b3()
              b2():
                jmp b4()
              b3():
                v6 = add v2, u32 1
                v8 = lt u32 10000, v0
                constrain v8 == u1 1, "Index out of bounds"
                v10 = array_get v1, index u32 10000 -> i32
                jmp b5()
              b4():
                return
              b5():
                jmp b1(v6)
            }
            "#;
        let (ssa, errors) = try_unroll_loops(Ssa::from_str(src).unwrap());
        assert_eq!(errors.len(), 0, "Unroll should have no errors");
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) predicate_pure fn main f0 {
          b0():
            v0 = make_array [] : [i32]
            call f1(u32 0, v0)
            return
        }
        brillig(inline) predicate_pure fn iter_0_times f1 {
          b0(v0: u32, v1: [i32]):
            jmp b1()
          b1():
            jmp b2()
          b2():
            return
        }
        ");
    }

    /// Test that `get_const_upper_bound` does not blindly trust the single
    /// instruction in the loop header without checking that the jmpif
    /// condition actually uses that instruction's result.
    ///
    /// Here the header has a `lt` with rhs=100, but the jmpif condition
    /// is a completely different value (`v10`) defined in the pre-header.
    /// `get_const_upper_bound` should return `None` (or at least not 100).
    #[test]
    fn get_const_upper_bound_ignores_unrelated_instruction() {
        // The loop header has a single `lt v0, u32 100` instruction
        // but the jmpif uses a constant `u1 1`, not the result of that lt.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            jmp b1(u32 0)
          b1(v0: u32):
            v1 = lt v0, u32 100
            jmpif u1 1 then: b3(), else: b2()
          b3():
            v2 = unchecked_add v0, u32 1
            jmp b1(v2)
          b2():
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let loops = Loops::find_all(function, LoopOrder::OutsideIn);
        assert_eq!(loops.yet_to_unroll.len(), 1);

        let loop_ = &loops.yet_to_unroll[0];
        let pre_header =
            loop_.get_pre_header(function, &loops.cfg).expect("Should have a pre_header");

        // The upper bound should be None because the lt instruction in the header
        // is not connected to the jmpif condition. If this returns Some(100),
        // the function is incorrectly assuming the header instruction feeds the jmpif.
        let upper = loop_.get_const_upper_bound(&function.dfg, pre_header, |v| v);
        assert!(
            upper.is_none(),
            "get_const_upper_bound should return None when the header's Lt instruction \
             does not feed the jmpif condition, but got: {upper:?}"
        );
    }

    /// Regression test: after mem2reg_simple, loop headers can have multiple parameters
    /// (induction variable + promoted mutable variables). Blocks outside the loop (like
    /// the exit block b3) reference these header params directly. After unrolling, these
    /// references must remain valid.
    #[test]
    fn unroll_loop_with_multi_param_header() {
        // Simplified main from vector_loop after mem2reg_simple.
        // b1 has 3 params: v1 (induction), v2 (promoted u32), v3 (promoted [Field]).
        // b3 (exit) references v2 and v3 from b1 directly.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: Field):
            jmp b1(u32 0, u32 0, Field 0)
          b1(v1: u32, v2: u32, v3: Field):
            v5 = lt v1, u32 3
            jmpif v5 then: b2(), else: b3()
          b2():
            v6 = add v3, v0
            v7 = unchecked_add v2, u32 1
            v8 = unchecked_add v1, u32 1
            jmp b1(v8, v7, v6)
          b3():
            v9 = lt u32 5, v2
            jmpif v9 then: b4(), else: b5(v3)
          b4():
            v10 = add v3, Field 1
            jmp b5(v10)
          b5(v4: Field):
            return v4
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        // Verify semantic preservation: interpret before and after unrolling
        let input = vec![Value::field(3u128.into())];
        let before = ssa.interpret(input.clone()).unwrap();

        let (ssa, errors) = try_unroll_loops(ssa);
        assert!(errors.is_empty(), "Unrolling should succeed: {errors:?}");

        let after = ssa.interpret(input).unwrap();
        assert_eq!(before, after, "Unrolling should preserve semantics");

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: Field):
            v2 = add v0, v0
            v3 = add v2, v0
            jmp b1()
          b1():
            v6 = lt u32 5, u32 3
            jmpif v6 then: b2(), else: b3(v3)
          b2():
            v8 = add v3, Field 1
            jmp b3(v8)
          b3(v1: Field):
            return v1
        }
        ");
    }

    /// Regression test: after mem2reg promotes loads/stores to block parameters,
    /// the loop should still be identified as small enough to unroll.
    /// This is the SSA for the `brillig_cow_assign` integration test post mem2reg.
    #[test]
    fn test_brillig_unroll_after_mem2reg_simple() {
        let src = "
            brillig(inline) predicate_pure fn main f0 {
                b0():
                    v6 = make_array [Field 0, Field 0, Field 0, Field 0, Field 0, Field 0, Field 0, Field 0, Field 0, Field 0] : [Field; 10]
                    inc_rc v6
                    jmp b1(u32 0, v6, v6)
                b1(v1: u32, v2: [Field; 10], v3: [Field; 10]):
                    v8 = lt v1, u32 10
                    jmpif v8 then: b2(), else: b3()
                b2():
                    v16 = eq v1, u32 5
                    jmpif v16 then: b4(), else: b5(v3)
                b3():
                    v10 = array_get v2, index u32 6 -> Field
                    constrain v10 == Field 27
                    v12 = array_get v3, index u32 6 -> Field
                    v13 = eq v12, Field 27
                    constrain v13 == u1 0
                    return
                b4():
                    inc_rc v2
                    jmp b5(v2)
                b5(v4: [Field; 10]):
                    v17 = array_set v2, index v1, value Field 27
                    v19 = unchecked_add v1, u32 1
                    jmp b1(v19, v17, v4)
            }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        // After mem2reg_simple, no loads/stores remain — the cost model must recognize
        // the loop-internal terminator costs as boilerplate.
        let stats = loop0_stats(&ssa);
        assert_eq!(
            stats.terminator_boilerplate, 11,
            "should count all loop-internal terminator costs as boilerplate"
        );
        assert_eq!(stats.header_params, 3, "header has induction var + 2 promoted params");
        assert!(
            stats.unrolled_cost() <= FORCE_UNROLL_THRESHOLD,
            "unrolled_cost {} should be within force-unroll threshold {}",
            stats.unrolled_cost(),
            FORCE_UNROLL_THRESHOLD,
        );

        let (ssa, errors) = try_unroll_loops(ssa);
        assert_eq!(errors.len(), 0, "Unroll should have no errors");
        // Loop has been unrolled
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) predicate_pure fn main f0 {
          b0():
            v11 = make_array [Field 0, Field 0, Field 0, Field 0, Field 0, Field 0, Field 0, Field 0, Field 0, Field 0] : [Field; 10]
            inc_rc v11
            jmp b2(v11)
          b1():
            v33 = array_get v32, index u32 6 -> Field
            constrain v33 == Field 27
            v34 = array_get v9, index u32 6 -> Field
            v35 = eq v34, Field 27
            constrain v35 == u1 0
            return
          b2(v0: [Field; 10]):
            v14 = array_set v11, index u32 0, value Field 27
            jmp b3(v0)
          b3(v1: [Field; 10]):
            v16 = array_set v14, index u32 1, value Field 27
            jmp b4(v1)
          b4(v2: [Field; 10]):
            v18 = array_set v16, index u32 2, value Field 27
            jmp b5(v2)
          b5(v3: [Field; 10]):
            v20 = array_set v18, index u32 3, value Field 27
            jmp b6(v3)
          b6(v4: [Field; 10]):
            v22 = array_set v20, index u32 4, value Field 27
            jmp b7()
          b7():
            inc_rc v22
            jmp b8(v22)
          b8(v5: [Field; 10]):
            v24 = array_set v22, index u32 5, value Field 27
            jmp b9(v5)
          b9(v6: [Field; 10]):
            v26 = array_set v24, index u32 6, value Field 27
            jmp b10(v6)
          b10(v7: [Field; 10]):
            v28 = array_set v26, index u32 7, value Field 27
            jmp b11(v7)
          b11(v8: [Field; 10]):
            v30 = array_set v28, index u32 8, value Field 27
            jmp b12(v8)
          b12(v9: [Field; 10]):
            v32 = array_set v30, index u32 9, value Field 27
            jmp b1()
        }
        ");
    }

    /// Regression test: after mem2reg_simple promotes loads/stores to block parameters,
    /// `count_useless_cost` must propagate constants through Jmp arguments to non-header
    /// block parameters. Without this, nested loops over constant 2D arrays won't see
    /// inner loop accumulators as constant, inflating useful_cost and preventing unrolling.
    ///
    /// This models the pattern from the regression_4709 integration test: outer loop indexes a constant 2D
    /// global array, inner loop accumulates over the row. After mem2reg, the row value
    /// is passed as a block parameter to the inner loop header.
    #[test]
    fn test_boilerplate_stats_nested_loop_block_param_propagation() {
        // Outer loop (b1) iterates i in 0..3.
        // b2 does array_get on a constant 2D array to get a row, then jumps to inner header b4
        // passing the row as a block param along with inner induction var 0 and accumulator 0.
        // Inner loop (b4) iterates j in 0..6, accumulating array_get row[j] into acc.
        // After inner loop, b6 increments outer induction var and loops back.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v100 = make_array [Field 1, Field 2, Field 3, Field 4, Field 5, Field 6] : [Field; 6]
            v101 = make_array [Field 7, Field 8, Field 9, Field 10, Field 11, Field 12] : [Field; 6]
            v102 = make_array [Field 13, Field 14, Field 15, Field 16, Field 17, Field 18] : [Field; 6]
            v103 = make_array [v100, v101, v102] : [[Field; 6]; 3]
            jmp b1(u32 0)
          b1(v0: u32):
            v5 = lt v0, u32 3
            jmpif v5 then: b2(), else: b3()
          b2():
            v6 = array_get v103, index v0 -> [Field; 6]
            jmp b4(u32 0, v6, Field 0)
          b4(v1: u32, v7: [Field; 6], v8: Field):
            v9 = lt v1, u32 6
            jmpif v9 then: b5(), else: b6()
          b5():
            v10 = array_get v7, index v1 -> Field
            v11 = add v8, v10
            v12 = unchecked_add v1, u32 1
            jmp b4(v12, v7, v11)
          b6():
            v13 = unchecked_add v0, u32 1
            jmp b1(v13)
          b3():
            return
        }";
        let ssa = Ssa::from_str(src).unwrap();
        let function = ssa.main();
        let mut loops = Loops::find_all(function, LoopOrder::OutsideIn);
        // OutsideIn: inner loop first, outer loop last.
        assert_eq!(loops.yet_to_unroll.len(), 2, "should find outer and inner loops");

        // Check that the outer loop has useful_cost = 0.
        let outer = loops.yet_to_unroll.pop().unwrap();
        let stats =
            outer.boilerplate_stats(function, &loops.cfg, &loops.dom, &loops.callee_costs).unwrap();
        assert_eq!(
            stats.useful_cost(),
            0,
            "all outer loop instructions should be useless after block param propagation"
        );

        // Also verify the loop would be unrolled (unrolled_cost <= baseline_cost).
        assert!(
            stats.unrolled_cost() <= stats.baseline_cost(),
            "outer loop should be unrolled: unrolled={} <= baseline={}",
            stats.unrolled_cost(),
            stats.baseline_cost()
        );
    }

    /// Regression test: nested loops where the inner loop header has multiple
    /// parameters.
    #[test]
    fn unroll_nested_loop_with_multi_param_inner_header() {
        // Outer loop (b1) iterates v2 in 0..3 with 3 header params.
        // b2 enters inner loop (b3) passing 4 params (3 outer + induction var).
        // Inner loop (b3) iterates v8 in 0..1.
        // On exit (b4) jumps back to outer header b1.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            jmp b1(u32 0, u32 0, u32 0)
          b1(v0: u32, v1: u32, v2: u32):
            v3 = lt v2, u32 3
            jmpif v3 then: b2(), else: b5()
          b2():
            v4 = unchecked_add v2, u32 1
            jmp b3(v0, v1, v4, u32 0)
          b3(v5: u32, v6: u32, v7: u32, v8: u32):
            v9 = lt v8, u32 1
            jmpif v9 then: b6(), else: b4()
          b4():
            jmp b1(v5, v6, v7)
          b5():
            return
          b6():
            v10 = unchecked_add v8, u32 1
            jmp b3(v5, v6, v7, v10)
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let (ssa, _errors) = try_unroll_loops(ssa);

        // TODO(https://github.com/noir-lang/noir/issues/11900): The inner loop is not unrolled
        // because `get_const_upper_bound` and `get_induction_variable` assume the induction
        // variable is always the first block parameter. For multi-param inner loop headers
        // (where outer loop variables are forwarded as earlier params), the actual induction
        // variable can be at a later position. A follow-up should identify the induction
        // variable by its increment pattern rather than by position.
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            jmp b1(u32 0, u32 0, u32 0)
          b1(v0: u32, v1: u32, v2: u32):
            v9 = lt v2, u32 3
            jmpif v9 then: b2(), else: b5()
          b2():
            v11 = unchecked_add v2, u32 1
            jmp b3(v0, v1, v11, u32 0)
          b3(v3: u32, v4: u32, v5: u32, v6: u32):
            v12 = lt v6, u32 1
            jmpif v12 then: b6(), else: b4()
          b4():
            jmp b1(v3, v4, v5)
          b5():
            return
          b6():
            v13 = unchecked_add v6, u32 1
            jmp b3(v3, v4, v5, v13)
        }
        ");
    }

    /// Regression test: a loop with a single header parameter (the induction variable)
    /// where the induction variable is referenced in post-loop blocks.
    ///
    /// After unrolling the first loop (b1), the induction variable v0 is used in b3
    /// (outside the loop) as an argument to the second loop header b4.
    #[test]
    fn unroll_single_param_header_referenced_in_post_loop() {
        // b1 is a simple loop 0..3, b3 uses v0 (b1's param) to enter second loop b4.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            jmp b1(u32 0)
          b1(v0: u32):
            v1 = eq v0, u32 3
            jmpif v1 then: b2(), else: b3()
          b2():
            v2 = unchecked_add v0, u32 1
            jmp b1(v2)
          b3():
            jmp b4(v0, u32 0)
          b4(v3: u32, v4: u32):
            v5 = eq v4, u32 2
            jmpif v5 then: b5(), else: b6()
          b5():
            return
          b6():
            v6 = unchecked_add v4, u32 1
            jmp b4(v3, v6)
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let (ssa, _errors) = try_unroll_loops(ssa);

        // This used to panic because v0 from b1 was referenced in b3 after
        // b1 was unrolled away, potentially leaving an orphan block parameter.
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            jmp b1()
          b1():
            jmp b2(u32 0, u32 0)
          b2(v0: u32, v1: u32):
            v4 = eq v1, u32 2
            jmpif v4 then: b3(), else: b4()
          b3():
            return
          b4():
            v6 = unchecked_add v1, u32 1
            jmp b2(v0, v6)
        }
        ");
    }

    #[test]
    fn unroll_nested_loop_with_break_to_outer_loop() {
        // Regression (fuzzer seed 0x4a6418c600059c93 for acir_vs_brillig): 3-nested-loop structure
        // where the inner loop has a non-constant lower bound and a break that
        // exits to the middle loop. In InsideOut ordering:
        //   1. Inner loop (b8<->b10) is skipped (non-constant lower bound)
        //   2. Middle loop (b4..b10) is skipped (contains inner loop's blocks)
        //   3. Outer loop (b1..b10) is skipped (contains inner loop's blocks)
        //
        // Without checking skipped or failed blocks, the middle loop would proceed
        // to unroll, fail to traverse the inner loop's cycle, and corrupt SSA.
        //
        // Reduced from:
        //   for idx_a in 0..1 {
        //     loop { if idx_b == 1 { break } else {
        //       loop { if idx_c == 1 { break } else { while false {} } }
        //     }}
        //   }
        let src = "
        brillig(inline) fn func_1 f0 {
          b0():
            jmp b1(u32 0)
          b1(v0: u32):
            v2 = eq v0, u32 0
            jmpif v2 then: b2(), else: b3()
          b2():
            jmp b4(u32 0)
          b3():
            return u1 1
          b4(v3: u32):
            v6 = eq v3, u32 1
            jmpif v6 then: b5(), else: b6()
          b5():
            v10 = unchecked_add v0, u32 1
            jmp b1(v10)
          b6():
            v7 = unchecked_add v3, u32 1
            jmp b8(v7, u32 0)
          b8(v8: u32, v9: u32):
            v11 = eq v9, u32 1
            jmpif v11 then: b9(), else: b10()
          b9():
            jmp b4(v8)
          b10():
            v12 = add v9, u32 1
            jmp b8(v8, v12)
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        // All loops aside the top-level for loop are skipped in a single pass.
        let (ssa, _errors) = try_unroll_loops(ssa);

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn func_1 f0 {
          b0():
            jmp b1(u32 0)
          b1(v0: u32):
            v7 = eq v0, u32 0
            jmpif v7 then: b2(), else: b3()
          b2():
            jmp b10(u32 1, u32 0)
          b3():
            return u1 1
          b4(v1: u32):
            v12 = eq v1, u32 1
            jmpif v12 then: b5(), else: b6()
          b5():
            v16 = unchecked_add v0, u32 1
            jmp b1(v16)
          b6():
            v13 = unchecked_add v1, u32 1
            jmp b7(v13, u32 0)
          b7(v2: u32, v3: u32):
            v14 = eq v3, u32 1
            jmpif v14 then: b8(), else: b9()
          b8():
            jmp b4(v2)
          b9():
            v15 = add v3, u32 1
            jmp b7(v2, v15)
          b10(v4: u32, v5: u32):
            v10 = eq v5, u32 1
            jmpif v10 then: b11(), else: b12()
          b11():
            jmp b4(v4)
          b12():
            v11 = add v5, u32 1
            jmp b10(v4, v11)
        }
        ");
    }

    /// Regression test: `get_const_upper_bound` must verify the header instruction
    /// references the induction variable. Without this check, a loop header with a
    /// single instruction like `not v0` (on a function parameter, not the induction
    /// variable) is misidentified as a bound check, producing bogus bounds that cause
    /// LICM to replace induction-variable-dependent expressions with constants.
    ///
    /// In this test, the loop header b1 has `not v0` (where v0 is a u1 parameter)
    /// and the actual loop exit is `eq v1, u32 1` in b2 (where v1 is the induction
    /// variable). Without the fix, `get_const_upper_bound` returns upper=1 (bit_size 1),
    /// and LICM's `simplify_induction_variable_in_binary` replaces `eq v1, u32 1` with
    /// constant `false`, creating an infinite loop.
    #[test]
    fn get_const_upper_bound_checks_induction_variable() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1):
            jmp b1(u32 0)
          b1(v1: u32):
            v2 = not v0
            jmpif v2 then: b2(), else: b3()
          b2():
            v3 = eq v1, u32 1
            jmpif v3 then: b3(), else: b4()
          b3():
            return
          b4():
            v4 = add v1, u32 1
            jmp b1(v4)
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        // The loop header's `not v0` does NOT reference the induction variable v1,
        // so get_const_upper_bound must return None (no known bounds).
        let function = ssa.main();
        let cfg = ControlFlowGraph::with_function(function);
        let loops = Loops::find_all(function, LoopOrder::InsideOut);
        assert_eq!(loops.yet_to_unroll.len(), 1, "should find exactly one loop");
        let the_loop = &loops.yet_to_unroll[0];
        let pre_header = the_loop.get_pre_header(function, &cfg).unwrap();
        let upper = the_loop.get_const_upper_bound(&function.dfg, pre_header, |v| v);
        assert!(
            upper.is_none(),
            "upper bound should be None when header instruction doesn't reference the induction variable, got {upper:?}"
        );

        // Verify semantics are preserved: interpret before and after LICM.
        let before = ssa.interpret(vec![Value::bool(false)]);
        let mut ssa_after = ssa;
        ssa_after = ssa_after.loop_invariant_code_motion();
        let after = ssa_after.interpret(vec![Value::bool(false)]);
        assert_eq!(before, after, "LICM should preserve semantics");
    }
}
