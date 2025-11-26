//! The loop invariant code motion (LICM) pass performs two related optimizations:
//! 1. Loop-Invariant Code Motion: Moves computations that produce the same result
//!    on every iteration out of the loop and into its pre-header.
//! 2. Loop-Bounds Simplification: Simplifies computations inside the loop body
//!    using information derived from induction variables and loop bounds. This optimization
//!    is essentially constant folding using induction variables.
//!
//! ## Design
//! To identify a loop invariant, check whether all of an instruction's values are:
//! - Outside of the loop
//! - Constant
//! - Already marked as loop invariants
//!
//! If we know that an invariant will always be executed (a loop's bounds are not dynamic and the upper bound is greater than its lower bounds)
//! we then hoist that invariant into the loop pre-header.
//!
//! We also check that we are not hoisting instructions with side effects.
//! However, there are certain instructions whose side effects are only activated
//! under a predicate (e.g. an array out of bounds error on a dynamic index).
//! Thus, we also track the control dependence of loop blocks to determine
//! whether these "pure with predicate instructions" can be hoisted.
//! We use post-dominance frontiers to determine control dependence.
//!
//! Let's look at definition 3 from the following paper:
//! Jeanne Ferrante, Karl J. Ottenstein, and Joe D. Warren. 1987.
//! The program dependence graph and its use in optimization. ACM
//! Trans. Program. Lang. Syst. 9, 3 (July 1987), 319–349.
//! <https://doi.org/10.1145/24039.24041>
//!
//! ```text
//! Let G be a control flow graph. Let X and Y be nodes in G. Y is
//! control dependent on X iff
//! (1) there exists a directed path P from X to Y with any Z in P (excluding X and Y) post-dominated by Y, and
//! (2) X is not post-dominated by Y.
//!
//! If Y is control dependent on X then X must have two exits. Following one of the
//! exits from X always results in Y being executed, while taking the other exit may
//! result in Y not being executed.
//! ```
//!
//! Verifying these conditions for every loop block would be quite inefficient.
//! For example, let's say we just want to check whether a given loop block is control dependent at all
//! after the loop preheader. We would have to to verify the conditions above for every block between the loop preheader
//! and the given loop block. This is n^2 complexity in the worst case.
//! To optimize the control dependence checks, we can use post-dominance frontiers (PDF).
//!
//! From Cooper, Keith D. et al. “A Simple, Fast Dominance Algorithm.” (1999).
//! ```text
//! A dominance frontier is the set of all CFG nodes, y, such that
//! b dominates a predecessor of y but does not strictly dominate y.
//! ```
//! Reversing this for post-dominance we can see that the conditions for control dependence
//! are the same as those for post-dominance frontiers: the post-dominance frontier of a block Y
//! is the set of blocks closest to Y where a choice was made of whether to reach Y or not.
//!
//! Thus, we rewrite our control dependence condition as Y is control dependent on X iff X is in PDF(Y).
//!
//! We then can store the PDFs for every block as part of the context of this pass, and use it for checking control dependence.
//! Using PDFs gets us from a worst case n^2 complexity to a worst case n.
//!
//! ### Simplification from Loop Bounds
//! We analyze induction variables and loop bounds to simplify instructions.
//! - Replacing conditions with constants when bounds make the condition always true/false.
//! - Simplifying arithmetic expressions involving induction variables (e.g., comparisons against loop bounds)
//!
//! Simplification is attempted before hoisting. This maximizes opportunities for
//! eliminating redundant computations entirely (by replacing them with constants),
//! and reduces the amount of code even considered for hoisting.
//!
//! ## Preconditions
//! - The pass will only be run on loops with a single pre-header. If a loop's header has multiple predecessors,
//!   the pass will skip that loop.
//!
//! ## Post-conditions
//! - All loop-invariant instructions that are safe to hoist are moved to the loop pre-header.
//! - Instructions inside loops may be simplified if loop bounds or induction variable
//!   constraints allow (e.g. replacing comparisons with constants).
//! - Control dependence is respected: instructions whose effects depend on runtime conditions
//!   remain in the loop unless proven safe for hoisting.
//!
//! ## ACIR vs Brillig
//! - On ACIR, LICM operates only on pure value computations.
//! - On Brillig, additional reference-counting rules apply. For example, hoisting [Instruction::MakeArray]
//!   requires inserting an [Instruction::IncrementRc] to preserve reference semantics if the array
//!   may later be mutated.
use std::collections::BTreeSet;

use crate::ssa::{
    Ssa,
    ir::{
        basic_block::BasicBlockId,
        cfg::ControlFlowGraph,
        dfg::DataFlowGraph,
        dom::DominatorTree,
        function::Function,
        function_inserter::FunctionInserter,
        instruction::{Instruction, InstructionId},
        integer::IntegerConstant,
        post_order::PostOrder,
        types::{NumericType, Type},
        value::{Value, ValueId},
    },
    opt::pure::Purity,
};
use acvm::{FieldElement, acir::AcirField};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::unrolling::{Loop, Loops};

mod simplify;

impl Ssa {
    /// See [`loop_invariant`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn loop_invariant_code_motion(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.loop_invariant_code_motion();
        }

        self
    }
}

impl Function {
    pub(super) fn loop_invariant_code_motion(&mut self) {
        Loops::find_all(self).hoist_loop_invariants(self);
    }
}

impl Loops {
    fn hoist_loop_invariants(mut self, function: &mut Function) {
        let mut context = LoopInvariantContext::new(function, &self.yet_to_unroll);

        // The loops should be sorted by the number of blocks.
        // We want to access outer nested loops first, which we do by popping
        // from the top of the list.
        while let Some(loop_) = self.yet_to_unroll.pop() {
            // If the loop does not have a preheader we skip hoisting loop invariants for this loop
            if let Ok(pre_header) = loop_.get_pre_header(context.inserter.function, &self.cfg) {
                context.hoist_loop_invariants(&loop_, &self.yet_to_unroll, pre_header);
            };
        }

        context.map_dependent_instructions();
        context.inserter.map_data_bus_in_place();
    }
}

impl Loop {
    /// Find the value that controls whether to perform a loop iteration.
    /// This is going to be the block parameter of the loop header.
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
    /// In the example above, `v1` is the induction variable.
    ///
    /// There is an example in the tests where a loop does not have an induction variable,
    /// but rather loads a reference in the header, in which case this will return `None`.
    fn get_induction_variable(&self, function: &Function) -> Option<ValueId> {
        function.dfg.block_parameters(self.header).iter().next().copied()
    }

    /// Check if the loop will be fully executed, that is, there is no early `break` in it.
    ///
    /// Our SSA code generation restricts loops to having one exit block.
    /// If the exit block has only one predecessor, that means there is no `break` in the loop.
    ///
    /// If a loop can have several exit blocks, we would need to update this function.
    ///
    /// If the loop header doesn't lead to an exit block, then it must be a `loop` or `while`,
    /// rather than a `for` loop. Even if such blocks don't have a `break` (e.g. they are infinite),
    /// we don't consider them fully executed.
    pub(super) fn is_fully_executed(&self, cfg: &ControlFlowGraph) -> bool {
        // A typical for-loop header has 2 successors: the loop body and the exit block.
        for block in cfg.successors(self.header) {
            // The exit block is not contained in the loop.
            if !self.blocks.contains(&block) {
                // If the exit block can be reached from the header and somewhere else in the loop,
                // then there must be a `break`.
                return cfg.predecessors(block).len() == 1;
            }
        }
        // If we are here then we haven't found an exit block from the header,
        // which means we must be dealing with a `loop` or `while`.
        false
    }
}

/// Context with the scope of an entire function.
struct LoopInvariantContext<'f> {
    inserter: FunctionInserter<'f>,

    /// Maps outer loop induction variable -> fixed lower and upper loop bound
    /// This will be used by inner loops to determine whether they
    /// have safe operations reliant upon an outer loop's maximum induction variable
    outer_induction_variables: HashMap<ValueId, (IntegerConstant, IntegerConstant)>,
    /// All induction variables collected up front.
    all_induction_variables: HashMap<ValueId, (IntegerConstant, IntegerConstant)>,

    cfg: ControlFlowGraph,

    /// Maps a block to its post-dominance frontiers
    post_dom_frontiers: PostDominanceFrontiers,

    // Helper constants
    true_value: ValueId,
    false_value: ValueId,
}

/// Context with the scope of just one loop.
struct LoopContext {
    pre_header: BasicBlockId,
    /// Maps current loop induction variable with a fixed lower and upper loop bound.
    /// If the loop doesn't have constant bounds then it's `None`.
    induction_variable: Option<(ValueId, (IntegerConstant, IntegerConstant))>,

    /// Indicate whether this loop has fixed bounds that are guaranteed to execute at least once.
    does_loop_execute: bool,

    /// Any values defined in the loop: block parameters and instruction results.
    defined_in_loop: HashSet<ValueId>,

    loop_invariants: HashSet<ValueId>,
    /// Caches all blocks that belong to nested loops determined to be control dependent
    /// on blocks in an outer loop. This allows short circuiting future control dependence
    /// checks during loop invariant analysis, as these blocks are guaranteed to be
    /// control dependent due to the entire nested loop being control dependent.
    ///
    /// Reset for each new loop as the set should not be shared across different outer loops.
    nested_loop_control_dependent_blocks: HashSet<BasicBlockId>,
    /// Indicates whether the current loop has break or early returns
    no_break: bool,
}

#[derive(Debug)]
struct BlockContext {
    /// Indicate that this block is the header of the current loop.
    is_header: bool,
    /// Stores whether the current block being processed is control dependent on any block
    /// between the loop header and itself. If it is, it means that even if the loop executes,
    /// this particular block may or may not do so.
    is_control_dependent: bool,
    /// Tracks whether the current block has a side-effectual instruction.
    /// This is maintained per instruction for hoisting control dependent instructions
    is_impure: bool,
    /// Stores whether the current loop has known fixed upper and lower bounds that
    /// indicate that it is guaranteed to execute at least once.
    does_execute: bool,
}

impl BlockContext {
    /// A control dependent instruction (e.g. constrain or division) has more strict conditions for hoisting.
    /// This function check the following conditions:
    /// - The current block is non control dependent
    /// - The loop body is guaranteed to be executed
    /// - The current block is not impure
    fn can_hoist_control_dependent_instruction(&self) -> bool {
        !self.is_control_dependent && self.does_execute && !self.is_impure
    }

    /// A control dependent instruction (e.g. constrain or division) has more strict conditions for simplifying.
    /// This function matches [Self::can_hoist_control_dependent_instruction] except
    /// that simplification does not require that the current block is pure to be simplified.
    fn can_simplify_control_dependent_instruction(&self) -> bool {
        !self.is_control_dependent && self.does_execute
    }
}

impl LoopContext {
    /// Create a `LoopContext`:
    /// * Gather the variables declared within the loop
    /// * Determine the induction variable bounds
    fn new(
        inserter: &FunctionInserter,
        cfg: &ControlFlowGraph,
        loop_: &Loop,
        pre_header: BasicBlockId,
    ) -> Self {
        let mut defined_in_loop = HashSet::default();
        for block in loop_.blocks.iter() {
            let params = inserter.function.dfg.block_parameters(*block);
            defined_in_loop.extend(params);
            for instruction_id in inserter.function.dfg[*block].instructions() {
                let results = inserter.function.dfg.instruction_results(*instruction_id);
                defined_in_loop.extend(results);
            }
        }
        let induction_variable = get_induction_var_bounds(inserter, loop_, pre_header);

        Self {
            // There is only ever one current induction variable for a loop.
            induction_variable,
            does_loop_execute: does_loop_execute(induction_variable.map(|(_, bounds)| bounds)),
            pre_header,
            defined_in_loop,
            loop_invariants: HashSet::default(),
            // Clear any cached control dependent nested loop blocks from the previous loop.
            // This set is only relevant within the scope of a single loop.
            // Keeping previous data would incorrectly classify blocks as control dependent,
            // leading to missed hoisting opportunities.
            nested_loop_control_dependent_blocks: HashSet::default(),
            no_break: loop_.is_fully_executed(cfg),
        }
    }

    fn pre_header(&self) -> BasicBlockId {
        self.pre_header
    }

    fn is_loop_invariant(&self, value_id: &ValueId) -> bool {
        !self.defined_in_loop.contains(value_id) || self.loop_invariants.contains(value_id)
    }

    /// Get the induction variable bounds if it the current variable matches the `id`.
    fn get_current_induction_variable_bounds(
        &self,
        id: ValueId,
    ) -> Option<(IntegerConstant, IntegerConstant)> {
        self.induction_variable.filter(|(val, _)| *val == id).map(|(_, bounds)| bounds)
    }

    /// Update any values defined in the loop and loop invariants after
    /// analyzing and re-inserting a loop's instruction.
    fn extend_values_defined_in_loop_and_invariants(
        &mut self,
        values: &[ValueId],
        hoist_invariant: bool,
    ) {
        self.defined_in_loop.extend(values.iter());

        // We also want the update value IDs when we are marking loop invariants as we may not
        // be going through the blocks of the loop in execution order
        if hoist_invariant {
            // Track already found loop invariants
            self.loop_invariants.extend(values.iter());
        }
    }
}

#[derive(Default)]
struct PostDominanceFrontiers {
    post_dom_frontiers: HashMap<BasicBlockId, HashSet<BasicBlockId>>,
}

impl PostDominanceFrontiers {
    fn with_function(func: &mut Function) -> Self {
        let reversed_cfg = ControlFlowGraph::extended_reverse(func);
        let post_order = PostOrder::with_cfg(&reversed_cfg);
        let mut post_dom = DominatorTree::with_cfg_and_post_order(&reversed_cfg, &post_order);
        let post_dom_frontiers = post_dom.compute_dominance_frontiers(&reversed_cfg);

        Self { post_dom_frontiers }
    }

    /// Checks whether a `block` is control dependent on a `parent_block`.
    /// Uses post-dominance frontiers to determine control dependence.
    /// Reference the doc comments at the top of the this module for more information
    /// regarding post-dominance frontiers and control dependence.
    fn is_control_dependent(&self, parent_block: BasicBlockId, block: BasicBlockId) -> bool {
        match self.post_dom_frontiers.get(&block) {
            Some(frontier) => frontier.contains(&parent_block),
            None => false,
        }
    }
}

impl<'f> LoopInvariantContext<'f> {
    fn new(function: &'f mut Function, loops: &[Loop]) -> Self {
        let cfg = ControlFlowGraph::with_function(function);
        let post_dom_frontiers = PostDominanceFrontiers::with_function(function);
        let true_value =
            function.dfg.make_constant(FieldElement::one(), NumericType::Unsigned { bit_size: 1 });
        let false_value =
            function.dfg.make_constant(FieldElement::zero(), NumericType::Unsigned { bit_size: 1 });
        let mut context = Self {
            inserter: FunctionInserter::new(function),
            outer_induction_variables: HashMap::default(),
            all_induction_variables: HashMap::default(),
            cfg,
            post_dom_frontiers,
            true_value,
            false_value,
        };

        // Insert all loop bounds up front, so we can inspect both outer and nested loops.
        for loop_ in loops {
            if let Some((induction_variable, bounds)) =
                loop_.get_pre_header(context.inserter.function, &context.cfg).ok().and_then(
                    |pre_header| get_induction_var_bounds(&context.inserter, loop_, pre_header),
                )
            {
                context.all_induction_variables.insert(induction_variable, bounds);
            };
        }

        context
    }

    /// Perform loop invariant code motion and induction variable analysis.
    /// See [`loop_invariant`][self] module for more information.
    fn hoist_loop_invariants(
        &mut self,
        loop_: &Loop,
        all_loops: &[Loop],
        pre_header: BasicBlockId,
    ) {
        let mut loop_context = LoopContext::new(&self.inserter, &self.cfg, loop_, pre_header);

        for block in loop_.blocks.iter() {
            let mut block_context =
                self.init_block_context(&mut loop_context, loop_, all_loops, *block);

            for instruction_id in self.inserter.function.dfg[*block].take_instructions() {
                if self.simplify_from_loop_bounds(&loop_context, &block_context, instruction_id) {
                    continue;
                }
                let (hoist_invariant, insert_rc) =
                    self.can_hoist_invariant(&loop_context, &block_context, instruction_id);

                if hoist_invariant {
                    self.inserter.push_instruction(instruction_id, pre_header, false);

                    // If we are hoisting a MakeArray instruction,
                    // we need to issue an extra inc_rc in case they are mutated afterward.
                    if insert_rc {
                        let [result] =
                            self.inserter.function.dfg.instruction_result(instruction_id);
                        let inc_rc = Instruction::IncrementRc { value: result };
                        let call_stack = self
                            .inserter
                            .function
                            .dfg
                            .get_instruction_call_stack_id(instruction_id);
                        self.inserter
                            .function
                            .dfg
                            .insert_instruction_and_results(inc_rc, *block, None, call_stack);
                    }
                } else {
                    let dfg = &self.inserter.function.dfg;
                    // If the block has already been labelled as impure, we don't need to check the current
                    // instruction's side effects.
                    if !block_context.is_impure {
                        block_context.is_impure = dfg[instruction_id].has_side_effects(dfg);
                    }
                    self.inserter.push_instruction(instruction_id, *block, true);
                }

                // We will have new IDs after pushing instructions.
                // We should mark the resolved result IDs as also being defined within the loop.
                let results = self
                    .inserter
                    .function
                    .dfg
                    .instruction_results(instruction_id)
                    .iter()
                    .map(|value| self.inserter.resolve(*value))
                    .collect::<Vec<_>>();

                loop_context
                    .extend_values_defined_in_loop_and_invariants(&results, hoist_invariant);
            }
        }

        // We're now done with this loop so it's now safe to insert its bounds into `outer_induction_variables`.
        if let Some((induction_variable, bounds)) =
            get_induction_var_bounds(&self.inserter, loop_, pre_header)
        {
            self.outer_induction_variables.insert(induction_variable, bounds);
        };
    }

    /// Checks whether a `block` is control dependent on any blocks after
    /// the given loop's header (excluding itself and the header).
    fn is_control_dependent_post_header(
        &self,
        loop_: &Loop,
        all_loops: &[Loop],
        block: BasicBlockId,
        all_predecessors: &BTreeSet<BasicBlockId>,
        nested_loop_control_dependent_blocks: &mut HashSet<BasicBlockId>,
    ) -> bool {
        // The block is already known to be in a control dependent nested loop
        // Thus, we can avoid checking for control dependence again.
        if nested_loop_control_dependent_blocks.contains(&block) {
            return true;
        }

        // Now check whether the current block is dependent on any blocks between
        // the current block and the loop header, exclusive of the current block and loop header themselves.
        if all_predecessors
            .iter()
            .any(|predecessor| self.post_dom_frontiers.is_control_dependent(*predecessor, block))
        {
            return true;
        }

        if let Some(nested) =
            self.find_control_dependent_nested_loop(loop_, block, all_loops, all_predecessors)
        {
            // Mark all blocks in the nested loop as control dependent to avoid redundant checks
            // for each of these blocks when they are later visited during hoisting.
            // This is valid because control dependence of the loop header implies dependence
            // for the entire loop body.
            nested_loop_control_dependent_blocks.extend(nested.blocks.iter().copied());
            return true;
        }

        false
    }

    /// Determines if the `block` is in a nested loop that is control dependent
    /// on a block in the outer loop. Returns the first such loop found, if any.
    ///
    /// If this is the case, we block hoisting as control is not guaranteed.
    /// If the block is not control dependent on the inner loop itself, it will be marked appropriately
    /// when the inner loop is processed later.
    ///
    /// Control dependence on a nested loop is determined by checking whether the nested loop's header
    /// is control dependent on any blocks between itself and the outer loop's header.
    /// It is expected that `all_predecessors` contains at least all of these blocks.
    fn find_control_dependent_nested_loop<'a>(
        &self,
        loop_: &Loop,
        block: BasicBlockId,
        all_loops: &'a [Loop],
        all_predecessors: &BTreeSet<BasicBlockId>,
    ) -> Option<&'a Loop> {
        // Now check for nested loops within the current loop
        for nested in all_loops.iter() {
            if !nested.blocks.contains(&block) {
                // Skip unrelated loops
                continue;
            }

            // We have found a nested loop if an inner loop shares blocks with the current loop
            // and they do not share a loop header.
            // `all_loops` should not contain the current loop but this extra check provides a sanity
            // check in case that ever changes.
            let nested_loop_is_control_dep = nested.header != loop_.header
                && all_predecessors
                    .iter()
                    // Check whether the nested loop's header is control dependent on any of its predecessors
                    .any(|predecessor| {
                        self.post_dom_frontiers.is_control_dependent(*predecessor, nested.header)
                    });

            if nested_loop_is_control_dep {
                return Some(nested);
            }
        }
        None
    }

    /// Determine whether a block in the loop body is guaranteed to execute.
    ///
    /// We know a loop body will execute if we have constant loop bounds where the upper bound
    /// is greater than the lower bound.
    ///
    /// The loop will never be executed if we have equal loop bounds
    /// or we are unsure if the loop will ever be executed (dynamic loop bounds).
    /// If certain instructions were to be hoisted out of a loop that never executed it
    /// could potentially cause the program to fail when it is not meant to fail.
    ///
    /// A block might be in a nested loop that isn't guaranteed to execute even if the current loop does.
    fn does_block_execute(
        &self,
        loop_context: &LoopContext,
        all_loops: &[Loop],
        block: BasicBlockId,
    ) -> bool {
        // If the current loop doesn't execute, then nothing does.
        if !loop_context.does_loop_execute {
            return false;
        }
        // If the block is part of any nested loop, they have to execute as well.
        for nested in all_loops.iter() {
            if !nested.blocks.contains(&block) {
                continue;
            }
            let Some(induction_variable) = get_induction_variable(&self.inserter, nested) else {
                // If we don't know what the induction variable is, we can't say if it executes.
                return false;
            };
            if !does_loop_execute(self.all_induction_variables.get(&induction_variable).copied()) {
                return false;
            }
        }
        true
    }

    /// Create a `BlockContext`.
    fn init_block_context(
        &self,
        loop_context: &mut LoopContext,
        loop_: &Loop,
        all_loops: &[Loop],
        block: BasicBlockId,
    ) -> BlockContext {
        let dfg = &self.inserter.function.dfg;

        // Find all blocks between the current block and the loop header.
        let mut all_predecessors = Loop::find_blocks_in_loop(loop_.header, block, &self.cfg).blocks;

        // For purity we consider the header as a predecessor as well (unless it's the block itself).
        // In practice it doesn't matter, because an impure header implies that `does_execute` will be false,
        // which already prevents hoisting instructions before we even look at `is_impure`, but it's clear
        // that the loop with side-effecting operations (typically a Load) in its header is impure.
        //
        // If the predecessors are all pure, the block might turn impure as and when we encounter
        // a side-effectful instruction in it later. Before that we can consider hoisting control
        // dependent instructions. But the block we are going to process is considered pure until
        // proven otherwise.
        all_predecessors.remove(&block);

        // When hoisting a control dependent instruction, if a side effectual instruction comes in the predecessor block
        // of that instruction we can no longer hoist the control dependent instruction.
        // This is important for maintaining the execution order and semantic correctness of the code.
        let is_impure = all_predecessors.iter().any(|block| {
            dfg[*block]
                .instructions()
                .iter()
                .any(|instruction| dfg[*instruction].has_side_effects(dfg))
        });

        // For control dependence we don't consider the header: all blocks are obviously control
        // dependent on it, as the header decides whether to loop one more time or not.
        all_predecessors.remove(&loop_.header);

        let is_control_dependent = self.is_control_dependent_post_header(
            loop_,
            all_loops,
            block,
            &all_predecessors,
            &mut loop_context.nested_loop_control_dependent_blocks,
        );

        let does_execute = self.does_block_execute(loop_context, all_loops, block);

        BlockContext {
            is_header: loop_.header == block,
            is_control_dependent,
            does_execute,
            is_impure,
        }
    }

    /// Decide if an in instruction can be hoisted into the pre-header of the loop.
    ///
    /// Returns 2 flags:
    /// 1. Whether the instruction can be hoisted
    /// 2. If it can be hoisted, does it require an `IncrementRc` instruction.
    fn can_hoist_invariant(
        &mut self,
        loop_context: &LoopContext,
        block_context: &BlockContext,
        instruction_id: InstructionId,
    ) -> (bool, bool) {
        use CanBeHoistedResult::*;

        let mut is_loop_invariant = true;
        // The list of blocks for a nested loop contain any inner loops as well.
        // We may have already re-inserted new instructions if two loops share blocks
        // so we need to map all the values in the instruction which we want to check.
        let (instruction, _) = self.inserter.map_instruction(instruction_id);
        instruction.for_each_value(|value| {
            // If an instruction value is defined in the loop and not already a loop invariant
            // the instruction results are not loop invariants.
            //
            // We are implicitly checking whether the values are constant as well.
            // The set of values defined in the loop only contains instruction results and block parameters
            // which cannot be constants.
            is_loop_invariant &= loop_context.is_loop_invariant(&value);
        });

        if !is_loop_invariant {
            return (false, false);
        }

        // Check if the operation depends only on the outer loop variable, in which case it can be hoisted
        // into the pre-header of a nested loop even if the nested loop does not execute.
        if self.can_be_hoisted_from_loop_bounds(loop_context, &instruction) {
            return (true, false);
        }

        match can_be_hoisted(&instruction, &self.inserter.function.dfg) {
            Yes => (true, false),
            No => (false, false),
            WithRefCount => (true, true),
            WithPredicate => (block_context.can_hoist_control_dependent_instruction(), false),
        }
    }

    /// Certain instructions can take advantage of that our induction variable has a fixed minimum/maximum.
    ///
    /// For example, an array access can usually only be safely deduplicated when we have a constant
    /// index that is below the length of the array.
    ///
    /// Checking an array get where the index is the loop's induction variable on its own
    /// would determine that the instruction is not safe for hoisting.
    ///
    /// However, if we know that the induction variable's upper bound will always be in bounds of the array
    /// we can safely hoist the array access.
    fn can_be_hoisted_from_loop_bounds(
        &self,
        loop_context: &LoopContext,
        instruction: &Instruction,
    ) -> bool {
        use Instruction::*;

        match instruction {
            ArrayGet { array, index } => {
                let array_typ = self.inserter.function.dfg.type_of_value(*array);
                let upper_bound = self.outer_induction_variables.get(index).map(|bounds| bounds.1);
                if let (Type::Array(_, len), Some(upper_bound)) = (array_typ, upper_bound) {
                    upper_bound.apply(|i| i <= len.into(), |i| i <= len.into())
                } else {
                    // We're dealing with a loop that doesn't have a fixed upper bound.
                    false
                }
            }
            Binary(binary) => self.can_evaluate_binary_op(loop_context, binary),
            // The rest of the instructions should not depend on the loop bounds.
            _ => false,
        }
    }

    /// Loop invariant hoisting only operates over loop instructions.
    /// The `FunctionInserter` is used for mapping old values to new values after
    /// re-inserting loop invariant instructions.
    /// However, there may be instructions which are not within loops that are
    /// still reliant upon the instruction results altered during the pass.
    /// This method re-inserts all instructions so that all instructions have
    /// correct new value IDs based upon the `FunctionInserter` internal map.
    /// Leaving out this mapping could lead to instructions with values that do not exist.
    fn map_dependent_instructions(&mut self) {
        let mut block_order = PostOrder::with_function(self.inserter.function).into_vec();
        block_order.reverse();

        for block in block_order {
            for instruction_id in self.inserter.function.dfg[block].take_instructions() {
                self.inserter.push_instruction(instruction_id, block, true);
            }
            self.inserter.map_terminator_in_place(block);
        }
    }
}

/// Get and resolve the induction variable of a loop.
fn get_induction_variable(inserter: &FunctionInserter, loop_: &Loop) -> Option<ValueId> {
    loop_.get_induction_variable(inserter.function).map(|v| inserter.resolve(v))
}

/// Check that a loop has have fixed bounds and upper is higher than lower, indicating that it would execute.
fn does_loop_execute(bounds: Option<(IntegerConstant, IntegerConstant)>) -> bool {
    bounds
        .and_then(|(lower_bound, upper_bound)| {
            upper_bound.reduce(lower_bound, |u, l| u > l, |u, l| u > l)
        })
        .unwrap_or(false)
}

/// Keep track of a loop induction variable and respective upper bound.
///
/// In the case of a nested loop, this will be used by later loops to determine
/// whether they have operations reliant upon the maximum induction variable.
///
/// When within the current loop, the known upper bound can be used to simplify instructions,
/// such as transforming a checked add to an unchecked add.
fn get_induction_var_bounds(
    inserter: &FunctionInserter,
    loop_: &Loop,
    pre_header: BasicBlockId,
) -> Option<(ValueId, (IntegerConstant, IntegerConstant))> {
    let bounds = loop_.get_const_bounds(&inserter.function.dfg, pre_header)?;
    let induction_variable = get_induction_variable(inserter, loop_)?;
    Some((induction_variable, bounds))
}

/// Indicate whether an instruction can be hoisted.
#[derive(Debug, PartialEq, Eq)]
enum CanBeHoistedResult {
    Yes,
    No,
    WithPredicate,
    WithRefCount,
}

impl From<bool> for CanBeHoistedResult {
    fn from(value: bool) -> Self {
        if value { Self::Yes } else { Self::No }
    }
}

/// Indicates if the instruction can be safely hoisted out of a loop.
///
/// If it returns `WithPredicate`, the instruction can only be hoisted together
/// with the predicate it relies on.
///
/// # Preconditions
/// Certain instructions can be hoisted because they implicitly depend on a predicate.
/// However, to avoid tight coupling between passes, we make the hoisting
/// conditional on whether the caller wants the predicate to be taken into account or not.
///
/// Even if we know the predicate is the same for an instruction's block and a loop's header block,
/// the caller of this methods needs to be careful as a loop may still never be executed.
/// This is because an loop with dynamic bounds may never execute its loop body.
/// If the instruction were to trigger a failure, our program may fail inadvertently.
/// If we know a loop's upper bound is greater than its lower bound we can hoist these instructions,
/// but it is left to the caller of this method to account for this case.
///
/// This differs from `can_be_deduplicated` as that method assumes there is a matching instruction
/// with the same inputs. Hoisting is for lone instructions, meaning a mislabeled hoist could cause
/// unexpected failures if the instruction was never meant to be executed.
fn can_be_hoisted(instruction: &Instruction, dfg: &DataFlowGraph) -> CanBeHoistedResult {
    use CanBeHoistedResult::*;
    use Instruction::*;

    match instruction {
        // These either have side-effects or interact with memory
        EnableSideEffectsIf { .. }
        | Allocate
        | Load { .. }
        | Store { .. }
        | IncrementRc { .. }
        | DecrementRc { .. } => No,

        Call { func, .. } => {
            let purity = match dfg[*func] {
                Value::Intrinsic(intrinsic) => Some(intrinsic.purity()),
                Value::Function(id) => dfg.purity_of(id),
                _ => None,
            };
            match purity {
                Some(Purity::Pure) => Yes,
                Some(Purity::PureWithPredicate) => WithPredicate,
                Some(Purity::Impure) => No,
                None => No,
            }
        }

        Cast(source, target_type) => {
            // A cast may have dependence on a range-check, which may not be hoisted, so we cannot always hoist a cast.
            // We can safely hoist a cast from a smaller to a larger type as no range check is necessary in this case.
            let source_type = dfg.type_of_value(*source).unwrap_numeric();
            (source_type.bit_size::<FieldElement>() <= target_type.bit_size::<FieldElement>())
                .into()
        }

        // These instructions can always be hoisted
        Not(_) | Truncate { .. } | IfElse { .. } => Yes,

        Constrain(..) | ConstrainNotEqual(..) | RangeCheck { .. } => WithPredicate,

        // Noop instructions can always be hoisted, although they're more likely to be
        // removed entirely.
        Noop => Yes,

        // Arrays can be mutated in unconstrained code so code that handles this case must
        // take care to track whether the array was possibly mutated or not before hoisted.
        // An ACIR it is always safe to hoist MakeArray.
        MakeArray { .. } => {
            if dfg.runtime().is_acir() {
                Yes
            } else {
                WithRefCount
            }
        }

        // These can have different behavior depending on the predicate.
        Binary(_) | ArraySet { .. } | ArrayGet { .. } => {
            if !instruction.requires_acir_gen_predicate(dfg) { Yes } else { WithPredicate }
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use crate::assert_ssa_snapshot;
    use crate::ssa::Ssa;
    use crate::ssa::ir::basic_block::BasicBlockId;
    use crate::ssa::ir::function::RuntimeType;
    use crate::ssa::ir::instruction::{Instruction, Intrinsic, TerminatorInstruction};
    use crate::ssa::ir::types::Type;
    use crate::ssa::opt::Loops;
    use crate::ssa::opt::loop_invariant::{
        CanBeHoistedResult, LoopContext, LoopInvariantContext, can_be_hoisted,
    };
    use crate::ssa::opt::pure::Purity;
    use crate::ssa::opt::{assert_normalized_ssa_equals, assert_ssa_does_not_change};
    use acvm::AcirField;
    use noirc_frontend::monomorphization::ast::InlineType;
    use test_case::test_case;

    #[test]
    fn hoists_casts() {
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            jmp b1(u32 2)
          b1(v1: u32):
            v5 = lt v1, u32 2
            jmpif v5 then: b2, else: b3
          b2():
            v6 = cast v0 as u64
            v7 = unchecked_add v1, u32 1
            jmp b1(v7)
          b3():
            return
        }";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v2 = cast v0 as u64
            jmp b1(u32 2)
          b1(v1: u32):
            v4 = lt v1, u32 2
            jmpif v4 then: b2, else: b3
          b2():
            v6 = unchecked_add v1, u32 1
            jmp b1(v6)
          b3():
            return
        }
        ");
    }

    #[test]
    fn simple_loop_invariant_code_motion() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: i32, v1: i32):
              jmp b1(i32 0)
          b1(v2: i32):
              v5 = lt v2, i32 4
              jmpif v5 then: b3, else: b2
          b2():
              return
          b3():
              v6 = unchecked_mul v0, v1
              constrain v6 == i32 6
              v8 = unchecked_add v2, i32 1
              jmp b1(v8)
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main();

        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 0); // The final return is not counted

        // From b3:
        // ```
        // v6 = mul v0, v1
        // constrain v6 == i32 6
        // ```
        // To b0:
        // ```
        // v3 = mul v0, v1
        // constrain v3 == i32 6
        // ```
        let ssa = ssa.loop_invariant_code_motion();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: i32, v1: i32):
            v3 = unchecked_mul v0, v1
            constrain v3 == i32 6
            jmp b1(i32 0)
          b1(v2: i32):
            v7 = lt v2, i32 4
            jmpif v7 then: b3, else: b2
          b2():
            return
          b3():
            v9 = unchecked_add v2, i32 1
            jmp b1(v9)
        }
        ");
    }

    #[test]
    fn simple_licm_one_value_invariant_one_value_local() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: i32, v1: i32):
            jmp b1(i32 0)
          b1(v2: i32):
            v5 = lt v2, i32 4
            jmpif v5 then: b3, else: b2
          b2():
            return
          b3():
            v6 = unchecked_mul v0, v1 // loop invariant
            constrain v6 == v2 // local loop instruction (checks against induction variable)
            v8 = unchecked_add v2, i32 1
            jmp b1(v8)
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();

        // From b3:
        // ```
        // v6 = unchecked_mul v0, v1
        // constrain v6 == v2
        // ```
        // To b0:
        // ```
        // v3 = unchecked_mul v0, v1
        // ```
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: i32, v1: i32):
            v3 = unchecked_mul v0, v1
            jmp b1(i32 0)
          b1(v2: i32):
            v6 = lt v2, i32 4
            jmpif v6 then: b3, else: b2
          b2():
            return
          b3():
            constrain v3 == v2
            v8 = unchecked_add v2, i32 1
            jmp b1(v8)
        }
        ");
    }

    #[test]
    fn nested_loop_invariant_code_motion() {
        // Check that a loop invariant in the inner loop of a nested loop
        // is hoisted to the parent loop's pre-header block.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: i32, v1: i32):
            jmp b1(i32 0)
          b1(v2: i32):
            v6 = lt v2, i32 4
            jmpif v6 then: b3, else: b2
          b2():
            return
          b3():
            jmp b4(i32 0)
          b4(v3: i32):
            v7 = lt v3, i32 4
            jmpif v7 then: b6, else: b5
          b5():
            v9 = unchecked_add v2, i32 1
            jmp b1(v9)
          b6():
            v10 = unchecked_mul v0, v1
            constrain v10 == i32 6
            v12 = unchecked_add v3, i32 1
            jmp b4(v12)
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main();

        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 0); // The final return is not counted

        // `v10 = mul v0, v1` in b6 should now be `v4 = mul v0, v1` in b0
        let ssa = ssa.loop_invariant_code_motion();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: i32, v1: i32):
            v4 = unchecked_mul v0, v1
            constrain v4 == i32 6
            jmp b1(i32 0)
          b1(v2: i32):
            v8 = lt v2, i32 4
            jmpif v8 then: b3, else: b2
          b2():
            return
          b3():
            jmp b4(i32 0)
          b4(v3: i32):
            v9 = lt v3, i32 4
            jmpif v9 then: b6, else: b5
          b5():
            v12 = unchecked_add v2, i32 1
            jmp b1(v12)
          b6():
            v11 = unchecked_add v3, i32 1
            jmp b4(v11)
        }
        ");
    }

    #[test]
    fn nested_inner_loop_invariant_from_outer_loop_local_variables() {
        // Check that a loop invariant in the inner loop, using only outer-loop locals,
        // is hoisted to the inner loop's pre-header, not the outer loop's pre-header.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: i32):
            jmp b1(i32 0)
          b1(v1: i32):
            v5 = lt v1, i32 3
            jmpif v5 then: b3, else: b2
          b2():
            return
          b3():
            jmp b4(i32 0)
          b4(v2: i32):
            v7 = lt v2, i32 2
            jmpif v7 then: b6, else: b5
          b5():
            v12 = unchecked_add v1, i32 1
            jmp b1(v12)
          b6():
            v9 = unchecked_mul v1, i32 5 // loop invariant using outer loop local variable
            v11 = unchecked_add v2, i32 1
            jmp b4(v11)
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();

        // After LICM, v9 should be hoisted to the inner loop pre-header (b5)
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: i32):
            jmp b1(i32 0)
          b1(v1: i32):
            v5 = lt v1, i32 3
            jmpif v5 then: b3, else: b2
          b2():
            return
          b3():
            v7 = unchecked_mul v1, i32 5
            jmp b4(i32 0)
          b4(v2: i32):
            v9 = lt v2, i32 2
            jmpif v9 then: b6, else: b5
          b5():
            v12 = unchecked_add v1, i32 1
            jmp b1(v12)
          b6():
            v11 = unchecked_add v2, i32 1
            jmp b4(v11)
        }
        ");
    }

    #[test]
    fn hoist_invariant_with_invariant_as_argument() {
        // Check that an instruction which has arguments defined in the loop
        // but which are already marked loop invariants is still hoisted to the preheader.
        //
        // For example, in b3 we have the following instructions:
        // ```text
        // v6 = mul v0, v1
        // v7 = mul v6, v0
        // ```
        // `v6` should be marked a loop invariants as `v0` and `v1` are both declared outside of the loop.
        // As we will be hoisting `v6 = mul v0, v1` to the loop preheader we know that we can also
        // hoist `v7 = mul v6, v0`.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: i32, v1: i32):
            jmp b1(i32 0)
          b1(v2: i32):
            v5 = lt v2, i32 4
            jmpif v5 then: b3, else: b2
          b2():
            return
          b3():
            v6 = unchecked_mul v0, v1
            v7 = unchecked_mul v6, v0
            v8 = eq v7, i32 12
            constrain v7 == i32 12
            v9 = unchecked_add v2, i32 1
            jmp b1(v9)
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main();

        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 0); // The final return is not counted

        let ssa = ssa.loop_invariant_code_motion();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: i32, v1: i32):
            v3 = unchecked_mul v0, v1
            v4 = unchecked_mul v3, v0
            v6 = eq v4, i32 12
            constrain v4 == i32 12
            jmp b1(i32 0)
          b1(v2: i32):
            v9 = lt v2, i32 4
            jmpif v9 then: b3, else: b2
          b2():
            return
          b3():
            v11 = unchecked_add v2, i32 1
            jmp b1(v11)
        }
        ");
    }

    #[test]
    fn do_not_hoist_instructions_with_side_effects() {
        // In `v12 = load v5` in `b3`, `v5` is defined outside the loop.
        // However, as the instruction has side effects, we want to make sure
        // we do not hoist the instruction to the loop preheader.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v4 = make_array [u32 0, u32 0, u32 0, u32 0, u32 0] : [u32; 5]
            inc_rc v4
            v5 = allocate -> &mut [u32; 5]
            store v4 at v5
            jmp b1(u32 0)
          b1(v2: u32):
            v7 = lt v2, u32 4
            jmpif v7 then: b3, else: b2
          b2():
            v12 = load v5 -> [u32; 5]
            v14 = array_get v12, index u32 2 -> u32
            constrain v14 == u32 3
            return
          b3():
            v8 = load v5 -> [u32; 5]
            v9 = array_set v8, index v0, value v1
            store v9 at v5
            v11 = unchecked_add v2, u32 1
            jmp b1(v11)
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let main = ssa.main();

        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 4); // The final return is not counted

        let ssa = ssa.loop_invariant_code_motion();
        // The code should be unchanged
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn hoist_array_gets_using_induction_variable_with_const_bound() {
        // SSA for the following program:
        //
        // fn triple_loop(x: u32) {
        //   let arr = [2; 5];
        //   for i in 0..4 {
        //       for j in 0..4 {
        //           for _ in 0..4 {
        //               assert_eq(arr[i], x);
        //               assert_eq(arr[j], x);
        //           }
        //       }
        //   }
        // }
        //
        // `arr[i]` and `arr[j]` are safe to hoist as we know the maximum possible index
        // to be used for both array accesses.
        // We want to make sure `arr[i]` is hoisted to the outermost loop body and that
        // `arr[j]` is hoisted to the second outermost loop body.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v6 = make_array [u32 2, u32 2, u32 2, u32 2, u32 2] : [u32; 5]
            inc_rc v6
            jmp b1(u32 0)
          b1(v2: u32):
            v9 = lt v2, u32 4
            jmpif v9 then: b3, else: b2
          b2():
            return
          b3():
            jmp b4(u32 0)
          b4(v3: u32):
            v10 = lt v3, u32 4
            jmpif v10 then: b6, else: b5
          b5():
            v12 = unchecked_add v2, u32 1
            jmp b1(v12)
          b6():
            jmp b7(u32 0)
          b7(v4: u32):
            v13 = lt v4, u32 4
            jmpif v13 then: b9, else: b8
          b8():
            v14 = unchecked_add v3, u32 1
            jmp b4(v14)
          b9():
            v15 = array_get v6, index v2 -> u32
            v16 = eq v15, v0
            constrain v15 == v0
            v17 = array_get v6, index v3 -> u32
            v18 = eq v17, v0
            constrain v17 == v0
            v19 = unchecked_add v4, u32 1
            jmp b7(v19)
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v6 = make_array [u32 2, u32 2, u32 2, u32 2, u32 2] : [u32; 5]
            inc_rc v6
            jmp b1(u32 0)
          b1(v2: u32):
            v9 = lt v2, u32 4
            jmpif v9 then: b3, else: b2
          b2():
            return
          b3():
            v10 = array_get v6, index v2 -> u32
            v11 = eq v10, v0
            constrain v10 == v0
            jmp b4(u32 0)
          b4(v3: u32):
            v12 = lt v3, u32 4
            jmpif v12 then: b6, else: b5
          b5():
            v19 = unchecked_add v2, u32 1
            jmp b1(v19)
          b6():
            v13 = array_get v6, index v3 -> u32
            v14 = eq v13, v0
            constrain v13 == v0
            jmp b7(u32 0)
          b7(v4: u32):
            v15 = lt v4, u32 4
            jmpif v15 then: b9, else: b8
          b8():
            v18 = unchecked_add v3, u32 1
            jmp b4(v18)
          b9():
            v17 = unchecked_add v4, u32 1
            jmp b7(v17)
        }
        ");
    }

    #[test]
    fn insert_inc_rc_when_moving_make_array() {
        // SSA for the following program:
        //
        // unconstrained fn main(x: u32, y: u32) {
        //   let mut a1 = [1, 2, 3, 4, 5];
        //   a1[x] = 64;
        //   for i in 0 .. 5 {
        //       let mut a2 = [1, 2, 3, 4, 5];
        //       a2[y + i] = 128;
        //       foo(a2);
        //   }
        //   foo(a1);
        // }
        //
        // We want to make sure move a loop invariant make_array instruction,
        // to account for whether that array has been marked as mutable.
        // To do so, we increment the reference counter on the array we are moving.
        // In the SSA below, we want to move `v42` out of the loop.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v8 = make_array [Field 1, Field 2, Field 3, Field 4, Field 5] : [Field; 5]
            v9 = allocate -> &mut [Field; 5]
            v11 = array_set v8, index v0, value Field 64
            v13 = add v0, u32 1
            store v11 at v9
            jmp b1(u32 0)
          b1(v2: u32):
            v16 = lt v2, u32 5
            jmpif v16 then: b3, else: b2
          b2():
            v17 = load v9 -> [Field; 5]
            call f1(v17)
            return
          b3():
            v19 = make_array [Field 1, Field 2, Field 3, Field 4, Field 5] : [Field; 5]
            v20 = allocate -> &mut [Field; 5]
            v21 = add v1, v2
            v23 = array_set v19, index v21, value Field 128
            call f1(v23)
            v24 = unchecked_add v2, u32 1
            jmp b1(v24)
        }
        brillig(inline) fn foo f1 {
          b0(v0: [Field; 5]):
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        // We expect the `make_array` at the top of `b3` to be replaced with an `inc_rc`
        // of the newly hoisted `make_array` at the end of `b0`.
        let ssa = ssa.loop_invariant_code_motion();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v8 = make_array [Field 1, Field 2, Field 3, Field 4, Field 5] : [Field; 5]
            v9 = allocate -> &mut [Field; 5]
            v11 = array_set v8, index v0, value Field 64
            v13 = add v0, u32 1
            store v11 at v9
            v14 = make_array [Field 1, Field 2, Field 3, Field 4, Field 5] : [Field; 5]
            jmp b1(u32 0)
          b1(v2: u32):
            v17 = lt v2, u32 5
            jmpif v17 then: b3, else: b2
          b2():
            v24 = load v9 -> [Field; 5]
            call f1(v24)
            return
          b3():
            inc_rc v14
            v18 = allocate -> &mut [Field; 5]
            v19 = add v1, v2
            v21 = array_set v14, index v19, value Field 128
            call f1(v21)
            v23 = unchecked_add v2, u32 1
            jmp b1(v23)
        }
        brillig(inline) fn foo f1 {
          b0(v0: [Field; 5]):
            return
        }
        ");
    }

    #[test]
    fn do_not_insert_inc_rc_when_moving_make_array_acir() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            jmp b1(u32 0)
          b1(v2: u32):
            v5 = lt v2, u32 5
            jmpif v5 then: b3, else: b2
          b2():
            return
          b3():
            v11 = make_array [Field 1, Field 2, Field 3, Field 4, Field 5] : [Field; 5]
            v12 = allocate -> &mut [Field; 5]
            v13 = add v1, v2
            v15 = array_set v11, index v13, value Field 128
            call f1(v15)
            v18 = unchecked_add v2, u32 1
            jmp b1(v18)
        }
        acir(inline) fn foo f1 {
          b0(v0: [Field; 5]):
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();

        // We expect the `make_array` at the top of `b3` to be hoisted to `b0`
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v8 = make_array [Field 1, Field 2, Field 3, Field 4, Field 5] : [Field; 5]
            jmp b1(u32 0)
          b1(v2: u32):
            v11 = lt v2, u32 5
            jmpif v11 then: b3, else: b2
          b2():
            return
          b3():
            v12 = allocate -> &mut [Field; 5]
            v13 = add v1, v2
            v15 = array_set v8, index v13, value Field 128
            call f1(v15)
            v18 = unchecked_add v2, u32 1
            jmp b1(v18)
        }
        acir(inline) fn foo f1 {
          b0(v0: [Field; 5]):
            return
        }
        ");
    }

    #[test]
    fn transform_safe_ops_to_unchecked_during_code_motion() {
        // This test is identical to `simple_loop_invariant_code_motion`, except this test
        // uses a checked add in `b3`.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
              jmp b1(u32 0)
          b1(v2: u32):
              v5 = lt v2, u32 4
              jmpif v5 then: b3, else: b2
          b2():
              return
          b3():
              v6 = unchecked_mul v0, v1
              constrain v6 == u32 6
              v8 = add v2, u32 1
              jmp b1(v8)
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        // `v8 = add v2, u32 1` in b3 should now be `v9 = unchecked_add v2, u32 1` in b3
        let ssa = ssa.loop_invariant_code_motion();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            v3 = unchecked_mul v0, v1
            constrain v3 == u32 6
            jmp b1(u32 0)
          b1(v2: u32):
            v7 = lt v2, u32 4
            jmpif v7 then: b3, else: b2
          b2():
            return
          b3():
            v9 = unchecked_add v2, u32 1
            jmp b1(v9)
        }
        ");
    }

    #[test]
    fn do_not_transform_unsafe_sub_to_unchecked() {
        // This test is identical to `simple_loop_invariant_code_motion`, except this test
        // uses a checked sub in `b3`.
        // We want to make sure that our sub operation has the induction variable (`v2`) on the lhs.
        // The induction variable `v2` is placed on the lhs of the sub operation
        // to test that we are checking against the loop's lower bound
        // rather than the upper bound (add/mul only check against the upper bound).
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            jmp b1(u32 0)
          b1(v2: u32):
            v5 = lt v2, u32 4
            jmpif v5 then: b3, else: b2
          b2():
            return
          b3():
            v7 = sub v2, u32 1
            jmp b1(v7)
        }
        ";
        assert_ssa_does_not_change(src, Ssa::loop_invariant_code_motion);
    }

    #[test]
    fn transform_safe_sub_to_unchecked() {
        // This test is identical to `do_not_transform_unsafe_sub_to_unchecked`, except the loop
        // in this test starts with a lower bound of `1`.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
              jmp b1(u32 1)
          b1(v2: u32):
              v5 = lt v2, u32 4
              jmpif v5 then: b3, else: b2
          b2():
              return
          b3():
              v8 = sub v2, u32 1
              jmp b1(v8)
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        // `v8 = sub v2, u32 1` in b3 should now be `v9 = unchecked_sub v2, u32 1` in b3
        let ssa = ssa.loop_invariant_code_motion();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32):
            jmp b1(u32 1)
          b1(v2: u32):
            v5 = lt v2, u32 4
            jmpif v5 then: b3, else: b2
          b2():
            return
          b3():
            v6 = unchecked_sub v2, u32 1
            jmp b1(v6)
        }
        ");
    }

    #[test]
    fn do_not_hoist_unsafe_div() {
        // This test is similar to `nested_loop_invariant_code_motion`, except that
        // the loop logic is under a dynamic predicate.
        // Divisions are only reliant upon predicates and do not have other side effects.
        //
        // If an unsafe division occurs in a loop block that is not control dependent,
        // we can still safely hoist that division as that instruction is always going to be hit.
        // Thus, we place the unsafe division under a predicate to ensure that we are testing
        // division hoisting based upon loop bounds and nothing else.
        //
        // The operation in question we are trying to hoist is `v12 = div u32 10, v1`.
        // Check whether the lower bound of the outer loop is zero and that we do not
        // hoist an operation that can potentially error with a division by zero.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v4 = eq v0, u32 5
            jmp b1(u32 0)
          b1(v1: u32):
            v7 = lt v1, u32 4
            jmpif v7 then: b2, else: b3
          b2():
            jmp b4(u32 0)
          b3():
            return
          b4(v2: u32):
            v8 = lt v2, u32 4
            jmpif v8 then: b5, else: b6
          b5():
            jmpif v4 then: b7, else: b8
          b6():
            v10 = unchecked_add v1, u32 1
            jmp b1(v10)
          b7():
            v12 = div u32 10, v1
            constrain v12 == u32 6
            jmp b8()
          b8():
            v14 = unchecked_add v2, u32 1
            jmp b4(v14)
        }
        ";
        assert_ssa_does_not_change(src, Ssa::loop_invariant_code_motion);
    }

    #[test]
    fn hoist_safe_div() {
        // This test is identical to `do_not_hoist_unsafe_div`, except the loop
        // in this test starts with a lower bound of `1`.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v4 = eq v0, u32 5
            jmp b1(u32 1)
          b1(v1: u32):
            v7 = lt v1, u32 4
            jmpif v7 then: b2, else: b3
          b2():
            jmp b4(u32 0)
          b3():
            return
          b4(v2: u32):
            v9 = lt v2, u32 4
            jmpif v9 then: b5, else: b6
          b5():
            jmpif v4 then: b7, else: b8
          b6():
            v10 = unchecked_add v1, u32 1
            jmp b1(v10)
          b7():
            v12 = div u32 10, v1
            constrain v12 == u32 6
            jmp b8()
          b8():
            v14 = unchecked_add v2, u32 1
            jmp b4(v14)
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.loop_invariant_code_motion();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v4 = eq v0, u32 5
            jmp b1(u32 1)
          b1(v1: u32):
            v7 = lt v1, u32 4
            jmpif v7 then: b2, else: b3
          b2():
            v9 = div u32 10, v1
            jmp b4(u32 0)
          b3():
            return
          b4(v2: u32):
            v11 = lt v2, u32 4
            jmpif v11 then: b5, else: b6
          b5():
            jmpif v4 then: b7, else: b8
          b6():
            v12 = unchecked_add v1, u32 1
            jmp b1(v12)
          b7():
            constrain v9 == u32 6
            jmp b8()
          b8():
            v14 = unchecked_add v2, u32 1
            jmp b4(v14)
        }
        ");
    }

    #[test]
    fn negative_lower_bound() {
        // Regression from issue #8858 (https://github.com/noir-lang/noir/issues/8858) that we
        // do not panic on a negative lower bound
        let src = "
      acir(inline) predicate_pure fn main f0 {
        b0():
          jmp b1(i32 4294967295)
        b1(v0: i32):
          v3 = lt v0, i32 0
          jmpif v3 then: b2, else: b3
        b2():
          v4 = truncate v0 to 32 bits, max_bit_size: 33
          v5 = cast v4 as u32
          v6 = cast v0 as u32
          v8 = lt v6, u32 2147483648
          v9 = lt v5, u32 2147483648
          v10 = eq v9, v8
          v11 = unchecked_mul v10, v8
          constrain v11 == v8
          v12 = lt v0, v4
          constrain v12 == u1 0
          v15 = unchecked_add v0, i32 1
          jmp b1(v15)
        b3():
          return
      }
      ";

        assert_ssa_does_not_change(src, Ssa::loop_invariant_code_motion);
    }

    /// Outer loop executes, nested loop executes;
    /// constraint from nested loop should be hoisted.
    #[test]
    fn hoist_from_nested_loop_known_to_execute() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            jmp b1(u32 0)
          b1(v1: u32):
            v2 = lt v1, u32 5
            jmpif v2 then: b2, else: b3
          b2():
            jmp b4(u32 0)
          b3():
            return
          b4(v3: u32):
            v4 = lt v3, u32 5
            jmpif v4 then: b5, else: b6
          b5():
            constrain v0 == u32 0
            v5 = unchecked_add v3, u32 1
            jmp b4(v5)
          b6():
            v6 = unchecked_add v1, u32 1
            jmp b1(v6)
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();
        let b0 = &ssa.main().dfg[BasicBlockId::new(0)];
        assert_eq!(b0.instructions().len(), 1);
    }

    /// Outer loop executes, nested loop does not;
    /// constraint from nested loop should not be hoisted.
    #[test]
    fn do_not_hoist_from_nested_loop_known_not_to_execute() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            jmp b1(u32 0)
          b1(v1: u32):
            v2 = lt v1, u32 5
            jmpif v2 then: b2, else: b3
          b2():
            jmp b4(u32 5)
          b3():
            return
          b4(v3: u32):
            v4 = lt v3, u32 5
            jmpif v4 then: b5, else: b6
          b5():
            constrain v0 == u32 0
            v5 = unchecked_add v3, u32 1
            jmp b4(v5)
          b6():
            v6 = unchecked_add v1, u32 1
            jmp b1(v6)
        }
        "#;

        assert_ssa_does_not_change(src, Ssa::loop_invariant_code_motion);
    }

    /// Outer loop executes, nested loop executes, but another nested loop does not;
    /// constraint should be hoisted.
    #[test]
    fn hoist_from_nested_loop_in_the_presence_of_unrelated_non_executed_nested_loops() {
        //          b0
        //          |
        // +------> b1
        // |       /  \
        // |      /    \
        // |     b2     b7
        // |     |      |
        // | +-> b4     b8 <-+
        // | |  /  \   /  \  |
        // | +-b5  b6 b3  b9-+
        // |       |
        // +-------+
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            jmp b1(u32 0)
          b1(v1: u32):
            v2 = lt v1, u32 5
            jmpif v2 then: b2, else: b7
          b2():
            jmp b4(u32 0)
          b3():
            return
          b4(v3: u32):
            v4 = lt v3, u32 5
            jmpif v4 then: b5, else: b6
          b5():
            constrain v0 == u32 0
            v5 = unchecked_add v3, u32 1
            jmp b4(v5)
          b6():
            v6 = unchecked_add v1, u32 1
            jmp b1(v6)
          b7():
            jmp b8(u32 5)
          b8(v7: u32):
            v8 = lt v7, u32 5
            jmpif v8 then: b9, else: b3
          b9():
            v9 = unchecked_add v7, u32 1
            jmp b8(v9)
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();
        let b0 = &ssa.main().dfg[BasicBlockId::new(0)];
        assert_eq!(b0.instructions().len(), 1);
    }

    /// Outer loop executes, nested loop doesn't have induction variable;
    /// constraint from nested loop should not be hoisted.
    #[test]
    fn do_not_hoist_from_nested_loop_without_induction_variable() {
        // This SSA is artificial: the induction variable was removed from b4,
        // but hasn't been replaced with e.g. the load of a reference, because
        // that would make it impure and the constraint wouldn't be hoisted for
        // that reason alone.
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            jmp b1(u32 0)
          b1(v1: u32):
            v2 = lt v1, u32 5
            jmpif v2 then: b2, else: b3
          b2():
            jmp b4()
          b3():
            return
          b4():
            constrain v0 == u32 0
            jmpif u1 0 then: b5, else: b6
          b5():
            jmp b4()
          b6():
            v6 = unchecked_add v1, u32 1
            jmp b1(v6)
        }
        "#;

        assert_ssa_does_not_change(src, Ssa::loop_invariant_code_motion);
    }

    #[test]
    fn impure_header_implies_impure_block() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v1 = allocate -> &mut u32
            store u32 0 at v1
            jmp b1()
          b1():
            v2 = load v1 -> u32
            v3 = lt v2, u32 5
            jmpif v3 then: b2, else: b3
          b2():
            constrain v0 == u32 0
            v4 = unchecked_add v2, u32 1
            store v4 at v1
            jmp b1()
          b3():
            return
        }
        "#;

        let mut ssa = Ssa::from_str(src).unwrap();
        let function = ssa.functions.get_mut(&ssa.main_id).unwrap();
        let mut loops = Loops::find_all(function);
        let ctx = LoopInvariantContext::new(function, &loops.yet_to_unroll);
        let pre_header = BasicBlockId::new(0);
        let loop_ = loops.yet_to_unroll.pop().unwrap();
        let mut loop_ctx = LoopContext::new(&ctx.inserter, &ctx.cfg, &loop_, pre_header);

        let mut get_block_ctx = |id| {
            ctx.init_block_context(
                &mut loop_ctx,
                &loop_,
                &loops.yet_to_unroll,
                BasicBlockId::new(id),
            )
        };

        let b1 = get_block_ctx(1);
        assert!(!b1.is_impure, "header starts as pure until processed");

        let b2_ctx = get_block_ctx(2);
        assert!(!b2_ctx.does_execute, "while loop execution treated as unknown");
        assert!(b2_ctx.is_impure, "header was impure");
    }

    /// Test cases where array_get should or shouldn't be hoisted from an inner loop
    /// when its indexed by the outer loop induction variable.
    #[test_case(4, 1, true; "upper less than size and both execute")]
    #[test_case(5, 1, true; "upper equal size and both execute")]
    #[test_case(6, 1, true; "upper greater than size and both execute")]
    #[test_case(0, 1, true; "upper less than size but outer does not execute")]
    #[test_case(4, 0, true; "upper less than size but inner does not execute")]
    #[test_case(6, 0, false; "upper greater than size but inner does not execute")]
    fn hoist_from_loop_bounds_array(outer_upper: u32, inner_upper: u32, should_hoist: bool) {
        let src = format!(
            r#"
        acir(inline) predicate_pure fn main f0 {{
          b0(v0: u32):
            v10 = make_array [u32 0, u32 0, u32 0, u32 0, u32 0] : [u32; 5]
            jmp b1(u32 0)
          b1(v1: u32):
            v2 = lt v1, u32 {outer_upper}
            jmpif v2 then: b2, else: b3
          b2():
            jmp b4(u32 0)
          b3():
            return
          b4(v3: u32):
            v4 = lt v3, u32 {inner_upper}
            jmpif v4 then: b5, else: b6
          b5():
            v11 = array_get v10, index v1 -> u32
            v5 = unchecked_add v3, u32 1
            jmp b4(v5)
          b6():
            v6 = unchecked_add v1, u32 1
            jmp b1(v6)
        }}
        "#
        );

        let ssa = Ssa::from_str(&src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();
        // The pre-header of the loop b4 is b2
        let pre_header = &ssa.main().dfg[BasicBlockId::new(2)];
        if !should_hoist {
            assert!(pre_header.instructions().is_empty(), "should not hoist");
        } else {
            assert_eq!(pre_header.instructions().len(), 1, "should hoist into nested pre-header");
        }
    }

    /// Test cases where +,-,*,/,% should or shouldn't be hoisted from an inner loop
    /// when its using by the outer loop induction variable.
    #[test_case("u32", 0, 10, 10, true, "add", 10, true, "loop executes, add cannot overflow")]
    #[test_case(
        "u32",
        0,
        10,
        0,
        true,
        "add",
        10,
        true,
        "nested loop empty, but add cannot overflow"
    )]
    #[test_case("u32", 0, 10, 10, true, "add", i64::from(u32::MAX - 5), true, "add overflows, and loop executes")]
    #[test_case("u32", 0, 10, 0, true, "add", i64::from(u32::MAX - 5), false, "add overflows, but loop empty")]
    #[test_case("u32", 0, 10, 0, true, "unchecked_add", i64::from(u32::MAX - 5), true, "loop empty, but add unchecked")]
    #[test_case("u32", 5, 10, 10, true, "sub", 5, true, "loop executes, sub cannot overflow")]
    #[test_case("u32", 0, 10, 10, true, "sub", 5, true, "sub overflows, and loop executes")]
    #[test_case("u32", 0, 10, 0, true, "sub", 5, false, "sub overflows, but loop empty")]
    #[test_case("u32", 0, 10, 0, true, "unchecked_sub", 5, true, "loop empty, but sub unchecked")]
    #[test_case("u32", 0, 10, 10, true, "mul", 10, true, "loop executes, mul cannot overflow")]
    #[test_case(
        "u32",
        0,
        10,
        10,
        true,
        "mul",
        i64::from(u32::MAX),
        true,
        "mul overflows, and loop executes"
    )]
    #[test_case(
        "u32",
        0,
        10,
        0,
        true,
        "mul",
        i64::from(u32::MAX),
        false,
        "mul overflows, but loop empty"
    )]
    #[test_case(
        "u32",
        0,
        10,
        0,
        true,
        "unchecked_mul",
        i64::from(u32::MAX),
        true,
        "loop empty, but mul unchecked"
    )]
    #[test_case("u32", 0, 10, 10, true, "div", 2, true, "loop executes, div ok")]
    #[test_case("u32", 0, 10, 0, true, "div", 2, true, "loop empty, div ok")]
    #[test_case("u32", 0, 10, 10, true, "div", 0, true, "div by zero, and loop executes")]
    #[test_case("u32", 0, 10, 0, true, "div", 0, false, "div by zero, but loop empty")]
    #[test_case("u32", 0, 10, 10, true, "mod", 2, true, "loop executes, mod ok")]
    #[test_case("u32", 0, 10, 0, true, "mod", 2, true, "loop empty, mod ok")]
    #[test_case("u32", 0, 10, 10, true, "mod", 0, true, "mod by zero, and loop executes")]
    #[test_case("u32", 0, 10, 0, true, "mod", 0, false, "mod by zero, but loop empty")]
    #[test_case("u32", 0, 10, 10, true, "eq", 5, true, "eq is safe")]
    #[test_case("u32", 0, 10, 0, true, "eq", 5, true, "loop empty, but eq is safe")]
    #[test_case("u32", 5, 10, 10, true, "shr", 1, true, "loop executes, shr ok")]
    #[test_case("u32", 5, 10, 0, true, "shr", 1, false, "loop empty, shr ok")]
    #[test_case("u32", 5, 10, 10, true, "shr", 32, true, "shr overflow, and loop executes")]
    #[test_case("u32", 5, 10, 0, true, "shr", 32, false, "shr overflow, but loop empty")]
    #[test_case("u32", 5, 10, 10, true, "shl", 1, true, "loop executes, shl ok")]
    #[test_case("u32", 5, 10, 0, true, "shl", 1, false, "loop empty, shl ok")]
    #[test_case("u32", 5, 10, 10, true, "shl", 32, true, "shl overflow, and loop executes")]
    #[test_case("u32", 5, 10, 0, true, "shl", 32, false, "shl overflow, but loop empty")]
    #[test_case("i32", -10, 10, 10, false, "div", 100, true, "div by zero (mid), and loop executes")]
    #[test_case("i32", 0, 10, 0, false, "div", 100, false, "div by zero (low), but loop empty")]
    #[test_case("i32", -10, 0, -10, false, "div", 100, false, "div by zero (up), but loop empty")]
    #[test_case("i32", -10, -5, -10, false, "div", 100, true, "loop empty, but div ok")]
    #[allow(clippy::too_many_arguments)]
    fn hoist_from_loop_bounds_binary(
        typ: &str,
        lower: i64,
        outer_upper: i64,
        inner_upper: i64,
        is_induction_left: bool,
        op: &str,
        constant: i64,
        should_hoist: bool,
        msg: &str,
    ) {
        let i = "v1".to_string();
        let c = format!("{typ} {constant}");
        let (lhs, rhs) = if is_induction_left { (i, c) } else { (c, i) };

        let src = format!(
            r#"
        acir(inline) predicate_pure fn main f0 {{
          b0(v0: {typ}):
            jmp b1({typ} {lower})
          b1(v1: {typ}):
            v2 = lt v1, {typ} {outer_upper}
            jmpif v2 then: b2, else: b3
          b2():
            jmp b4({typ} {lower})
          b3():
            return
          b4(v3: {typ}):
            v4 = lt v3, {typ} {inner_upper}
            jmpif v4 then: b5, else: b6
          b5():
            v7 = {op} {lhs}, {rhs}
            v5 = unchecked_add v3, {typ} 1
            jmp b4(v5)
          b6():
            v6 = unchecked_add v1, {typ} 1
            jmp b1(v6)
        }}
        "#
        );

        let ssa = Ssa::from_str(&src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();
        // The pre-header of the loop b4 is b2
        let pre_header = &ssa.main().dfg[BasicBlockId::new(2)];
        let instruction_count = pre_header.instructions().len();
        if !should_hoist {
            assert_eq!(instruction_count, 0, "should not hoist: {msg}");
        } else {
            assert_eq!(instruction_count, 1, "should hoist into nested pre-header: {msg}");
        }
    }

    enum TestCall {
        Function(Option<Purity>),
        ForeignFunction,
        Intrinsic(Intrinsic),
    }

    /// Test that calls to functions is hoisted into the pre-header based on their purity.
    #[test_case(1, TestCall::Function(Some(Purity::Pure)), true; "non-empty loop, pure function")]
    #[test_case(0, TestCall::Function(Some(Purity::Pure)), true; "empty loop, pure function")]
    #[test_case(1, TestCall::Function(Some(Purity::PureWithPredicate)), true; "non-empty loop, predicate pure function")]
    #[test_case(0, TestCall::Function(Some(Purity::PureWithPredicate)), false; "empty loop, predicate pure function")]
    #[test_case(1, TestCall::Function(Some(Purity::Impure)), false; "impure function")]
    #[test_case(1, TestCall::Function(None), false; "purity unknown")]
    #[test_case(1, TestCall::ForeignFunction, false; "foreign functions always impure")]
    #[test_case(0, TestCall::Intrinsic(Intrinsic::BlackBox(acvm::acir::BlackBoxFunc::Keccakf1600)), true; "empty loop, pure intrinsic")]
    #[test_case(1, TestCall::Intrinsic(Intrinsic::BlackBox(acvm::acir::BlackBoxFunc::Keccakf1600)), true; "non-empty loop, pure intrinsic")]
    fn hoist_from_loop_call_with_purity(upper: u32, test_call: TestCall, should_hoist: bool) {
        let dummy_purity = if let TestCall::Function(purity) = &test_call { *purity } else { None };
        let dummy_purity = dummy_purity.map_or("".to_string(), |p| format!("{p}"));

        // The arguments are not meant to make sense, just pass SSA validation and not be simplified out.
        let call_target = match test_call {
            TestCall::Function(_) => "f1".to_string(),
            TestCall::ForeignFunction => "print".to_string(), // The ony foreign function the SSA parser allows.
            TestCall::Intrinsic(intrinsic) => format!("{intrinsic}"),
        };

        let src = format!(
            r#"
        acir(inline) fn main f0 {{
          b0(v0: [u64; 25]):
            jmp b1(u32 0)
          b1(v1: u32):
            v2 = lt v1, u32 {upper}
            jmpif v2 then: b2, else: b3
          b2():
            v3 = call {call_target}(v0) -> [u64; 25]
            v4 = unchecked_add v1, u32 1
            jmp b1(v4)
          b3():
            return
        }}

        acir(inline) {dummy_purity} fn dummy f1 {{
          b0(v0: [u64; 25]):
            return v0
        }}
        "#,
        );

        let ssa = Ssa::from_str(&src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();

        // The pre-header of the loop b1 is b0
        let pre_header = &ssa.main().dfg[BasicBlockId::new(0)];
        if !should_hoist {
            assert!(pre_header.instructions().is_empty(), "should not hoist");
        } else {
            assert_eq!(pre_header.instructions().len(), 1, "should hoist");
        }
    }

    #[test_case(0, false; "empty loop; predicate pure intrinsic")]
    #[test_case(1, true; "non-empty loop; predicate pure intrinsic")]
    fn hoist_as_witness_from_loop_call_with_purity(upper: u32, should_hoist: bool) {
        let src = format!(
            r#"
        acir(inline) fn main f0 {{
          b0(v0: Field):
            jmp b1(u32 0)
          b1(v1: u32):
            v2 = lt v1, u32 {upper}
            jmpif v2 then: b2, else: b3
          b2():
            call as_witness(v0)
            v4 = unchecked_add v1, u32 1
            jmp b1(v4)
          b3():
            return
        }}
        "#,
        );

        let ssa = Ssa::from_str(&src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();

        // The pre-header of the loop b1 is b0
        let pre_header = &ssa.main().dfg[BasicBlockId::new(0)];
        if !should_hoist {
            assert!(pre_header.instructions().is_empty(), "should not hoist");
        } else {
            assert_eq!(pre_header.instructions().len(), 1, "should hoist");
        }
    }

    #[test]
    fn does_not_hoist_array_refcount_from_loop_call_with_purity() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 3]):
            jmp b1(u32 0)
          b1(v1: u32):
            v2 = lt v1, u32 1
            jmpif v2 then: b2, else: b3
          b2():
            v3 = call array_refcount(v0) -> u32
            v4 = unchecked_add v1, u32 1
            jmp b1(v4)
          b3():
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();

        // The pre-header of the loop b1 is b0
        let pre_header = &ssa.main().dfg[BasicBlockId::new(0)];
        assert!(pre_header.instructions().is_empty(), "should not hoist");
    }

    /// Test cases where `i < const` or `const < i` should or shouldn't be simplified.
    #[test_case(10, 20, true, 25, Some(true))]
    #[test_case(10, 20, true, 20, Some(true))]
    #[test_case(10, 10, true, 25, Some(true))]
    #[test_case(10, 20, true, 15, None)]
    #[test_case(10, 20, true, 10, Some(false))]
    #[test_case(10, 20, true, 5, Some(false))]
    #[test_case(10, 10, true, 5, Some(false))]
    #[test_case(10, 20, false, 25, Some(false))]
    #[test_case(10, 20, false, 15, None)]
    #[test_case(10, 20, false, 5, Some(true))]
    #[test_case(10, 20, false, 20, Some(false))]
    #[test_case(10, 20, false, 19, Some(false))]
    #[test_case(10, 20, false, 18, None)]
    fn hoist_from_loop_bounds_binary_lt(
        lower: u32,
        upper: u32,
        induction_is_left: bool,
        constant: u32,
        simplify_to: Option<bool>,
    ) {
        let i = "v0".to_string();
        let c = format!("u32 {constant}");
        let (lhs, rhs) = if induction_is_left { (i, c) } else { (c, i) };
        let src = format!(
            r#"
            brillig(inline) impure fn main f0 {{
              b0():
                jmp b1(u32 {lower})
              b1(v0: u32):
                v3 = lt v0, u32 {upper}
                jmpif v3 then: b2, else: b3
              b2():
                v5 = lt {lhs}, {rhs}
                jmpif v5 then: b4, else: b5
              b3():
                return
              b4():
                jmp b5()
              b5():
                v30 = unchecked_add v0, u32 1
                jmp b1(v30)
            }}
        "#
        );
        // If the condition simplifies, then the `jmpif v5` either becomes `jmpif u1 1` or `jmpif u1 0`

        let ssa = Ssa::from_str(&src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();
        let body = &ssa.main().dfg[BasicBlockId::new(2)];

        let TerminatorInstruction::JmpIf { condition, .. } = body.unwrap_terminator() else {
            unreachable!("body should end in jmpif");
        };

        let dfg = &ssa.main().dfg;
        let is_var = || dfg.get_numeric_constant(*condition).is_none();
        let is_one = || dfg.get_numeric_constant(*condition).is_some_and(|c| c.is_one());
        let is_zero = || dfg.get_numeric_constant(*condition).is_some_and(|c| c.is_zero());

        match simplify_to {
            None => {
                assert!(is_var(), "shouldn't simplify");
            }
            Some(true) => {
                assert!(is_one(), "should simplify to constant true");
            }
            Some(false) => {
                assert!(is_zero(), "should simplify to constant false");
            }
        }
    }

    #[test]
    fn do_not_hoist_array_get_from_loop_without_induction() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32, v11: u32):
            v10 = make_array [u32 0, u32 0, u32 0, u32 0, u32 0] : [u32; 5]
            v1 = allocate -> &mut u32
            store u32 0 at v1
            jmp b1()
          b1():
            v2 = load v1 -> u32
            v3 = lt v2, u32 5
            jmpif v3 then: b2, else: b3
          b2():
            v12 = array_get v10, index v11 -> u32
            v4 = unchecked_add v2, u32 1
            store v4 at v1
            jmp b1()
          b3():
            return
        }
        "#;

        assert_ssa_does_not_change(src, Ssa::loop_invariant_code_motion);
    }

    #[test]
    fn do_not_hoist_div_by_zero_from_non_executed_nested_loop() {
        let src = r#"
          brillig(inline) predicate_pure fn main f0 {
            b0():
              jmp b1(u32 0)
            b1(v0: u32):
              v4 = lt v0, u32 10
              jmpif v4 then: b2, else: b3
            b2():
              jmp b4(u32 10)
            b3():
              return
            b4(v1: u32):
              v6 = lt v1, u32 10
              jmpif v6 then: b5, else: b6
            b5():
              v9 = div v0, u32 0
              v10 = unchecked_add v1, u32 1
              jmp b4(v10)
            b6():
              v8 = unchecked_add v0, u32 1
              jmp b1(v8)
          }
        "#;

        assert_ssa_does_not_change(src, Ssa::loop_invariant_code_motion);
    }

    #[test]
    fn do_not_hoist_signed_div_by_minus_one_from_non_executed_nested_loop() {
        let src = r#"
          brillig(inline) predicate_pure fn main f0 {
            b0():
              jmp b1(i32 0)
            b1(v0: i32):
              v4 = lt v0, i32 10
              jmpif v4 then: b2, else: b3
            b2():
              jmp b4(i32 10)
            b3():
              return
            b4(v1: i32):
              v6 = lt v1, i32 10
              jmpif v6 then: b5, else: b6
            b5():
              v9 = div v0, i32 -1
              v10 = unchecked_add v1, i32 1
              jmp b4(v10)
            b6():
              v8 = unchecked_add v0, i32 1
              jmp b1(v8)
          }
        "#;

        assert_ssa_does_not_change(src, Ssa::loop_invariant_code_motion);
    }

    /// Test that in itself `MakeArray` is only safe to be hoisted in ACIR.
    #[test_case(RuntimeType::Brillig(InlineType::default()), CanBeHoistedResult::WithRefCount)]
    #[test_case(RuntimeType::Acir(InlineType::default()), CanBeHoistedResult::Yes)]
    fn make_array_can_be_hoisted(runtime: RuntimeType, result: CanBeHoistedResult) {
        // This is just a stub to create a function with the expected runtime.
        let src = format!(
            r#"
        {runtime} predicate_pure fn main f0 {{
          b0():
            return
        }}
        "#
        );
        let ssa = Ssa::from_str(&src).unwrap();
        let function = ssa.main();

        let instruction = Instruction::MakeArray {
            elements: Default::default(),
            typ: Type::Array(Arc::new(vec![]), 0),
        };

        assert_eq!(can_be_hoisted(&instruction, &function.dfg), result);
    }
}

#[cfg(test)]
mod control_dependence {
    use crate::{
        assert_ssa_snapshot,
        ssa::{
            interpreter::{errors::InterpreterError, tests::from_constant},
            ir::{function::RuntimeType, types::NumericType},
            opt::{assert_normalized_ssa_equals, assert_ssa_does_not_change, unrolling::Loops},
            ssa_gen::Ssa,
        },
    };
    use noirc_frontend::monomorphization::ast::InlineType;
    use test_case::test_case;

    #[test]
    fn do_not_hoist_unsafe_mul_in_control_dependent_block() {
        let src = "
        brillig(inline) fn main f0 {
          entry(v0: u32, v1: u32):
            v4 = eq v0, u32 5
            jmp loop(u32 0)
          loop(v2: u32):
            v7 = lt v2, u32 4
            jmpif v7 then: loop_cond, else: exit
          loop_cond():
            jmpif v4 then: loop_body, else: loop_end
          exit():
            return
          loop_body():
            v8 = mul v0, v1
            constrain v8 == u32 12
            jmp loop_end()
          loop_end():
            v11 = unchecked_add v2, u32 1
            jmp loop(v11)
        }
        ";
        assert_ssa_does_not_change(src, Ssa::loop_invariant_code_motion);
    }

    #[test]
    fn hoist_safe_mul_that_is_non_control_dependent() {
        let src = "
        brillig(inline) fn main f0 {
          entry(v0: u32, v1: u32):
            jmp loop(u32 0)
          loop(v2: u32):
            v3 = lt v2, u32 4
            jmpif v3 then: loop_body, else: exit
          loop_body():
            v6 = mul v0, v1
            v7 = mul v6, v0
            constrain v7 == u32 12
            v10 = unchecked_add v2, u32 1
            jmp loop(v10)
          exit():
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();

        let expected = "
        brillig(inline) fn main f0 {
          entry(v0: u32, v1: u32):
            v3 = mul v0, v1
            v4 = mul v3, v0
            constrain v4 == u32 12
            jmp loop(u32 0)
          loop(v2: u32):
            v8 = lt v2, u32 4
            jmpif v8 then: loop_body, else: exit
          loop_body():
            v10 = unchecked_add v2, u32 1
            jmp loop(v10)
          exit():
            return
        }
        ";

        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn non_control_dependent_loop_follows_control_dependent_loop() {
        // Test that we appropriately reset the control dependence status.
        // This program first has a loop with a control dependent body, thus preventing hoisting instructions.
        // There is then a separate second loop which is non control dependent for which
        // we expect instructions to be hoisted.
        let src = "
      brillig(inline) fn main f0 {
        entry(v0: u32, v1: u32):
          v5 = eq v0, u32 5
          jmp loop_1(u32 0)
        loop_1(v2: u32):
          v8 = lt v2, u32 4
          jmpif v8 then: loop_1_cond, else: loop_1_exit
        loop_1_cond():
          jmpif v5 then: loop_1_body, else: loop_1_end
        loop_1_exit():
          jmp loop_2(u32 0)
        loop_1_body():
          v15 = mul v0, v1
          constrain v15 == u32 12
          jmp loop_1_end()
        loop_1_end():
          v16 = unchecked_add v2, u32 1
          jmp loop_1(v16)
        loop_2(v3: u32):
          v10 = lt v3, u32 4
          jmpif v10 then: loop_2_body, else: exit
        loop_2_body():
          v9 = mul v0, v1
          v11 = mul v9, v0
          constrain v11 == u32 12
          v14 = unchecked_add v3, u32 1
          jmp loop_2(v14)
        exit():
          return
      }
      ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();

        // From loop_2_body:
        // ```
        // v9 = mul v0, v1
        // v11 = mul v9, v0
        // constrain v11 == u32 12
        // ```
        // To loop_1_exit:
        // ```
        // v9 = mul v0, v1
        // v10 = mul v9, v0
        // constrain v10 == u32 12
        // ```
        let expected = "
      brillig(inline) fn main f0 {
        entry(v0: u32, v1: u32):
          v5 = eq v0, u32 5
          jmp loop_1(u32 0)
        loop_1(v2: u32):
          v8 = lt v2, u32 4
          jmpif v8 then: loop_1_cond, else: loop_1_exit
        loop_1_cond():
          jmpif v5 then: loop_1_body, else: loop_1_end
        loop_1_exit():
          v9 = mul v0, v1
          v10 = mul v9, v0
          constrain v10 == u32 12
          jmp loop_2(u32 0)
        loop_1_body():
          v15 = mul v0, v1
          constrain v15 == u32 12
          jmp loop_1_end()
        loop_1_end():
          v16 = unchecked_add v2, u32 1
          jmp loop_1(v16)
        loop_2(v3: u32):
          v12 = lt v3, u32 4
          jmpif v12 then: loop_2_body, else: exit
        loop_2_body():
          v14 = unchecked_add v3, u32 1
          jmp loop_2(v14)
        exit():
          return
      }
      ";

        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn do_not_hoist_constrain_in_loop_with_zero_upper_bound() {
        // This test is the same as `hoist_safe_mul_that_is_non_control_dependent` except
        // that the upper loop bound is zero
        let src = "
        brillig(inline) fn main f0 {
          entry(v0: u32, v1: u32):
            jmp loop(u32 0)
          loop(v2: u32):
            v3 = lt v2, u32 0
            jmpif v3 then: loop_body, else: exit
          loop_body():
            v6 = unchecked_mul v0, v1
            v7 = unchecked_mul v6, v0
            constrain v7 == u32 12
            v10 = unchecked_add v2, u32 1
            jmp loop(v10)
          exit():
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.loop_invariant_code_motion();

        // We expect the constrain to remain inside of `loop_body`
        // as the loop is never going to be executed.
        // If the constrain were to be hoisted out it could potentially
        // cause the program to fail when it is not meant to fail.
        let expected = "
        brillig(inline) fn main f0 {
          entry(v0: u32, v1: u32):
            v3 = unchecked_mul v0, v1
            v4 = unchecked_mul v3, v0
            jmp loop(u32 0)
          loop(v2: u32):
            jmpif u1 0 then: loop_body, else: exit
          loop_body():
            constrain v4 == u32 12
            v10 = unchecked_add v2, u32 1
            jmp loop(v10)
          exit():
            return
        }
        ";

        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn do_not_hoist_constrain_in_loop_with_equal_non_zero_loop_bounds() {
        // This test is the same as `hoist_safe_mul_that_is_non_control_dependent` except
        // that the lower and upper loop bounds are the same and greater than zero
        let src = "
        brillig(inline) fn main f0 {
          entry(v0: u32, v1: u32):
            jmp loop(u32 1)
          loop(v2: u32):
            v3 = lt v2, u32 1
            jmpif v3 then: loop_body, else: exit
          loop_body():
            v6 = unchecked_mul v0, v1
            v7 = unchecked_mul v6, v0
            constrain v7 == u32 12
            v10 = unchecked_add v2, u32 1
            jmp loop(v10)
          exit():
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.loop_invariant_code_motion();
        // We expect the constrain to remain inside of `loop_body`
        // as the loop is never going to be executed.
        // If the constrain were to be hoisted out it could potentially
        // cause the program to fail when it is not meant to fail.
        let expected = "
        brillig(inline) fn main f0 {
          entry(v0: u32, v1: u32):
            v3 = unchecked_mul v0, v1
            v4 = unchecked_mul v3, v0
            jmp loop(u32 1)
          loop(v2: u32):
            v7 = eq v2, u32 0
            jmpif v7 then: loop_body, else: exit
          loop_body():
            constrain v4 == u32 12
            v10 = unchecked_add v2, u32 1
            jmp loop(v10)
          exit():
            return
        }
        ";

        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn do_not_hoist_constrain_in_loop_with_dynamic_upper_bound() {
        // This test is the same as `hoist_safe_mul_that_is_non_control_dependent` except
        // that the upper loop bound is dynamic
        let src = "
        brillig(inline) fn main f0 {
          entry(v0: u32, v1: u32):
            jmp loop(u32 0)
          loop(v2: u32):
            v3 = lt v2, v1
            jmpif v3 then: loop_body, else: exit
          loop_body():
            v6 = unchecked_mul v0, v1
            v7 = unchecked_mul v6, v0
            constrain v7 == u32 12
            v10 = unchecked_add v2, u32 1
            jmp loop(v10)
          exit():
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.loop_invariant_code_motion();

        // We expect the constrain to remain inside of `loop_body`
        // as that block may potentially never be executed.
        // If the constrain were to be hoisted out it could potentially
        // cause the program to fail when it is not meant to fail.
        let expected = "
        brillig(inline) fn main f0 {
          entry(v0: u32, v1: u32):
            v3 = unchecked_mul v0, v1
            v4 = unchecked_mul v3, v0
            jmp loop(u32 0)
          loop(v2: u32):
            v6 = lt v2, v1
            jmpif v6 then: loop_body, else: exit
          loop_body():
            constrain v4 == u32 12
            v10 = unchecked_add v2, u32 1
            jmp loop(v10)
          exit():
            return
        }
        ";

        assert_normalized_ssa_equals(ssa, expected);
    }

    #[test]
    fn do_not_hoist_pure_with_predicate_call_in_non_executed_loop_body() {
        // This test is the same as `hoist_safe_mul_that_is_non_control_dependent` except
        // that the upper loop bound is dynamic and the constrain inside the loop body
        // is replaced with a call to pure with predicates functions.
        // We cannot guarantee that the loop body will be executed when we have dynamic bounds.
        let src = "
        brillig(inline) fn main f0 {
          entry(v0: u32, v1: u32):
            jmp loop(u32 0)
          loop(v2: u32):
            v3 = lt v2, v1
            jmpif v3 then: loop_body, else: exit
          loop_body():
            v6 = unchecked_mul v0, v1
            v7 = unchecked_mul v6, v0
            call f1(v7)
            v10 = unchecked_add v2, u32 1
            jmp loop(v10)
          exit():
            return
        }
        brillig(inline) fn foo f1 {
          entry(v0: u32):
            constrain v0 == u32 12
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();
        let ssa = ssa.loop_invariant_code_motion();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: u32, v1: u32):
            v3 = unchecked_mul v0, v1
            v4 = unchecked_mul v3, v0
            jmp b1(u32 0)
          b1(v2: u32):
            v6 = lt v2, v1
            jmpif v6 then: b2, else: b3
          b2():
            call f1(v4)
            v9 = unchecked_add v2, u32 1
            jmp b1(v9)
          b3():
            return
        }
        brillig(inline) predicate_pure fn foo f1 {
          b0(v0: u32):
            constrain v0 == u32 12
            return
        }
        ");
    }

    #[test]
    fn hoist_pure_with_predicate_call_in_executed_loop_body() {
        // This test is the same as `do_not_hoist_pure_with_predicate_call_in_non_executed_loop_body`
        // except that the loop bounds are guaranteed to execute.
        let src = "
        brillig(inline) fn main f0 {
          entry(v0: u32, v1: u32):
            jmp loop(u32 0)
          loop(v2: u32):
            v3 = lt v2, u32 4
            jmpif v3 then: loop_body, else: exit
          loop_body():
            v6 = mul v0, v1
            v7 = mul v6, v0
            call f1(v7)
            v10 = unchecked_add v2, u32 1
            jmp loop(v10)
          exit():
            return
        }
        brillig(inline) fn foo f1 {
          entry(v0: u32):
            constrain v0 == u32 12
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.purity_analysis();
        let ssa = ssa.loop_invariant_code_motion();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: u32, v1: u32):
            v3 = mul v0, v1
            v4 = mul v3, v0
            call f1(v4)
            jmp b1(u32 0)
          b1(v2: u32):
            v8 = lt v2, u32 4
            jmpif v8 then: b2, else: b3
          b2():
            v10 = unchecked_add v2, u32 1
            jmp b1(v10)
          b3():
            return
        }
        brillig(inline) predicate_pure fn foo f1 {
          b0(v0: u32):
            constrain v0 == u32 12
            return
        }
        ");
    }

    #[test]
    fn simplify_constraint() {
        // This test shows the constraint constrain v17 == u1 1 is not simplified into constrain u1 0 == u1 1
        let src = "
        brillig(inline) fn main f0 {
          entry(v0: u32, v1: u32, v2: u32):
            v4 = allocate -> &mut u32
            store v0 at v4
            jmp b1(u32 0)
          b1(v3: u32):
            v7 = lt v3, u32 5
            jmpif v7 then: b2, else: b3
          b2():
            v12 = lt v3, u32 8
            jmpif v12 then: b4, else: b5
          b3():
            v8 = load v4 -> u32
            v9 = lt v1, v8
            constrain v9 == u1 1
            return
          b4():
            v13 = load v4 -> u32
            v15 = add v13, u32 1
            store v15 at v4
            jmp b5()
          b5():
            v17 = lt v3, u32 4
            constrain v17 == u1 1
            v18 = unchecked_add v3, u32 1
            jmp b1(v18)
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();

        // Despite the loop is guaranteed to fully execute, which implies that the constrain will fail at some iteration,
        // the constraint is not simplified in case some side-effect instruction would run in the previous iterations.
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32, v2: u32):
            v4 = allocate -> &mut u32
            store v0 at v4
            jmp b1(u32 0)
          b1(v3: u32):
            v7 = lt v3, u32 5
            jmpif v7 then: b2, else: b3
          b2():
            jmpif u1 1 then: b4, else: b5
          b3():
            v8 = load v4 -> u32
            v9 = lt v1, v8
            constrain v9 == u1 1
            return
          b4():
            v11 = load v4 -> u32
            v13 = add v11, u32 1
            store v13 at v4
            jmp b5()
          b5():
            v15 = lt v3, u32 4
            constrain v15 == u1 1
            v16 = unchecked_add v3, u32 1
            jmp b1(v16)
        }
        ");
    }

    #[test]
    fn do_not_simplify_constraint() {
        // This test is similar to simplify_constraint but does not simplify because loop_exit has 2 predecessors
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32, v2: u32):
            v4 = allocate -> &mut u32
            store v0 at v4
            v6 = eq v1, u32 4
            jmp b1(u32 0)
          b1(v3: u32):
            v9 = lt v3, u32 5
            jmpif v9 then: b2, else: loop_exit
          b2():
            jmpif u1 1 then: b4, else: b5
          loop_exit():
            v19 = load v4 -> u32
            v20 = lt v1, v19
            constrain v20 == u1 1
            return
          b4():
            v11 = load v4 -> u32
            v13 = add v11, u32 1
            store v13 at v4
            jmp b5()
          b5():
            v15 = lt u32 2, v3
            v16 = unchecked_mul v6, v15
            jmpif v16 then: loop_exit, else: b6
          b6():
            v17 = lt v3, u32 4
            constrain v17 == u1 1
            v18 = unchecked_add v3, u32 1
            jmp b1(v18)
        }
        ";
        assert_ssa_does_not_change(src, Ssa::loop_invariant_code_motion);
    }

    /// Change `constrain v0 != i` into `constrain v0 < 10 or v0 > 19`
    #[test]
    fn simplify_constrain_not_equal() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            jmp b1(u32 10)
          b1(v1: u32):
            v2 = lt v1, u32 20
            jmpif v2 then: b2, else: b3
          b2():
            constrain v0 != v1
            v3 = unchecked_add v1, u32 1
            jmp b1(v3)
          b3():
            return
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32):
            v3 = lt v0, u32 10
            v5 = lt u32 19, v0
            v6 = or v3, v5
            constrain v6 == u1 1
            jmp b1(u32 10)
          b1(v1: u32):
            v9 = lt v1, u32 20
            jmpif v9 then: b2, else: b3
          b2():
            v11 = unchecked_add v1, u32 1
            jmp b1(v11)
          b3():
            return
        }
        ");
    }

    #[test]
    fn simplify_comparison() {
        // This tests shows that the comparison v12 = lt v3, u32 8 is simplified because v3 is bounded by 5
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32, v2: u32):
            v4 = allocate -> &mut u32
            store v0 at v4
            jmp b1(u32 0)
          b1(v3: u32):
            v7 = lt v3, u32 5
            jmpif v7 then: b2, else: b3
          b2():
            v12 = lt v3, u32 8
            jmpif v12 then: b4, else: b5
          b3():
            v8 = load v4 -> u32
            v9 = lt v1, v8
            constrain v9 == u1 1
            return
          b4():
            v13 = load v4 -> u32
            v15 = add v13, u32 1
            store v15 at v4
            jmp b5()
          b5():
            v16 = unchecked_add v3, u32 1
            jmp b1(v16)
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.loop_invariant_code_motion();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32, v2: u32):
            v4 = allocate -> &mut u32
            store v0 at v4
            jmp b1(u32 0)
          b1(v3: u32):
            v7 = lt v3, u32 5
            jmpif v7 then: b2, else: b3
          b2():
            jmpif u1 1 then: b4, else: b5
          b3():
            v8 = load v4 -> u32
            v9 = lt v1, v8
            constrain v9 == u1 1
            return
          b4():
            v11 = load v4 -> u32
            v13 = add v11, u32 1
            store v13 at v4
            jmp b5()
          b5():
            v14 = unchecked_add v3, u32 1
            jmp b1(v14)
        }
        ");
    }

    #[test]
    fn simplify_not_equal_constraint() {
        // This tests shows that the not equal on v3 is simplified due to the loop bounds
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32, v2: u32):
            v4 = allocate -> &mut u32
            jmp b1(u32 0)
          b1(v3: u32):
            v7 = lt v3, u32 5
            jmpif v7 then: b2, else: b3
          b2():
            v9 = eq v3, u32 10
            v10 = not v9
            constrain v9 == u1 0
            v13 = unchecked_add v3, u32 1
            jmp b1(v13)
          b3():
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32, v2: u32):
            v4 = allocate -> &mut u32
            jmp b1(u32 0)
          b1(v3: u32):
            v7 = lt v3, u32 5
            jmpif v7 then: b2, else: b3
          b2():
            v9 = unchecked_add v3, u32 1
            jmp b1(v9)
          b3():
            return
        }
        ");
    }

    #[test]
    fn do_not_hoist_non_control_dependent_div_in_non_executed_loop() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v1 = allocate -> &mut Field
            store Field 0 at v1
            jmp b1()
          b1():
            v3 = load v1 -> Field
            v4 = eq v3, Field 0
            jmpif v4 then: b2, else: b3
          b2():
            return
          b3():
            v6 = div u32 5, v0
            jmp b1()
        }
        ";
        assert_ssa_does_not_change(src, Ssa::loop_invariant_code_motion);
    }

    #[test]
    fn do_not_hoist_from_outer_loop_when_inner_loop_is_control_dependent() {
        // We want to check the case when an entire inner loop is under a predicate
        // that we do not still hoist with respect to control dependence on the outer
        // loop's block header.
        // This is the SSA for the following program:
        // ```noir
        // fn main(a: pub bool) {
        //     for _ in 0..1 {
        //         if a {
        //             for _ in 0..1 {
        //                 let _ = (1 / (a as Field));
        //             }
        //         };
        //     }
        // }
        // ```
        let src = r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            jmp b1(u32 0)
          b1(v1: u32):
            v4 = eq v1, u32 0
            jmpif v4 then: b2, else: b3
          b2():
            jmpif v0 then: b4, else: b5
          b3():
            return
          b4():
            jmp b6(u32 0)
          b5():
            v7 = unchecked_add v1, u32 1
            jmp b1(v7)
          b6(v2: u32):
            v5 = eq v2, u32 0
            jmpif v5 then: b7, else: b8
          b7():
            v8 = cast v0 as Field
            v10 = div Field 1, v8
            v11 = unchecked_add v2, u32 1
            jmp b6(v11)
          b8():
            jmp b5()
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();

        // We expect `v10 = div Field 1, v8` to be hoisted, but only to the inner loop's header.
        // If we were to hoist that div to the outer loop's header, we will fail inadvertently
        // if `v0 == false`.
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u1):
            v3 = cast v0 as Field
            jmp b1(u32 0)
          b1(v1: u32):
            v5 = eq v1, u32 0
            jmpif v5 then: b2, else: b3
          b2():
            jmpif v0 then: b4, else: b5
          b3():
            return
          b4():
            v7 = div Field 1, v3
            jmp b6(u32 0)
          b5():
            v10 = unchecked_add v1, u32 1
            jmp b1(v10)
          b6(v2: u32):
            v8 = eq v2, u32 0
            jmpif v8 then: b7, else: b8
          b7():
            v11 = unchecked_add v2, u32 1
            jmp b6(v11)
          b8():
            jmp b5()
        }
        ");
    }

    #[test]
    fn do_not_hoist_constrain_with_preceding_side_effects() {
        let src = r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32, v1: u32):
            v3 = cast v0 as Field
            jmp b1(u32 0)
          b1(v2: u32):
            v6 = lt v2, u32 4
            jmpif v6 then: b2, else: b3
          b2():
            v7 = cast v2 as Field
            v8 = add v7, v3
            range_check v8 to 1 bits
            constrain v0 == u32 12
            v11 = unchecked_add v2, u32 1
            jmp b1(v11)
          b3():
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let expected = ssa
            .interpret(vec![
                from_constant(2_u128.into(), NumericType::unsigned(32)),
                from_constant(3_u128.into(), NumericType::unsigned(32)),
            ])
            .expect_err("Should have error");
        assert!(matches!(expected, InterpreterError::RangeCheckFailed { .. }));

        let mut ssa = ssa.loop_invariant_code_motion();
        ssa.normalize_ids();

        let got = ssa
            .interpret(vec![
                from_constant(2_u128.into(), NumericType::unsigned(32)),
                from_constant(3_u128.into(), NumericType::unsigned(32)),
            ])
            .expect_err("Should have error");
        assert_eq!(expected, got);

        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn do_not_hoist_constrain_with_preceding_side_effects_in_another_block() {
        // The SSA for this program where x = 2 and y = 3:
        // ```noir
        // fn main(x: u32, y: u32) {
        //     for i in 0..4 {
        //         if x == 2 {
        //           let y = i + x;
        //           assert_eq(y, 12);
        //         }
        //         assert_eq(x, 12);
        //     }
        //  }
        //
        // ```
        // We expect to fail on assert_eq(y, 12) rather than assert_eq(x, 12);
        let src = r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32, v1: u32):
            v4 = eq v0, u32 2
            jmp b1(u32 0)
          b1(v2: u32):
            v7 = lt v2, u32 4
            jmpif v7 then: b2, else: b3
          b2():
            jmpif v4 then: b4, else: b5
          b3():
            return
          b4():
            v8 = add v2, v0
            constrain v8 == u32 12
            jmp b5()
          b5():
            constrain v0 == u32 12
            v11 = unchecked_add v2, u32 1
            jmp b1(v11)
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let expected = ssa
            .interpret(vec![
                from_constant(2_u128.into(), NumericType::Unsigned { bit_size: 32 }),
                from_constant(3_u128.into(), NumericType::Unsigned { bit_size: 32 }),
            ])
            .expect_err("Should have error");
        let InterpreterError::ConstrainEqFailed { lhs_id, .. } = expected else {
            panic!("Expected ConstrainEqFailed");
        };
        // Make sure that the constrain on v8 is the on that failed
        assert_eq!(lhs_id.to_u32(), 8);

        let mut ssa = ssa.loop_invariant_code_motion();
        ssa.normalize_ids();

        let got = ssa
            .interpret(vec![
                from_constant(2_u128.into(), NumericType::Unsigned { bit_size: 32 }),
                from_constant(3_u128.into(), NumericType::Unsigned { bit_size: 32 }),
            ])
            .expect_err("Should have error");
        let InterpreterError::ConstrainEqFailed { lhs_id, .. } = got else {
            panic!("Expected ConstrainEqFailed");
        };
        // Make sure that the constrain on v8 is the on that failed
        assert_eq!(lhs_id.to_u32(), 8);

        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn do_not_hoist_constrain_with_break() {
        // There is a constraint in the loop which we know would fail on some induction values,
        // however because of the `break` we won't necessarily reach all values, so we don't hoist.
        // unconstrained fn main() {
        //     for i in 0..10 {
        //         assert(i < 5);
        //         if i == 1 {
        //             break;
        //         }
        //     }
        // }
        let src = r"
        brillig(inline) predicate_pure fn main f0 {
          b0():
            jmp b1(u32 0)
          b1(v0: u32):
            v3 = lt v0, u32 10
            jmpif v3 then: b2, else: b3
          b2():
            v5 = lt v0, u32 5
            constrain v5 == u1 1
            v8 = eq v0, u32 1
            jmpif v8 then: b4, else: b5
          b3():
            return
          b4():
            jmp b3()
          b5():
            v9 = unchecked_add v0, u32 1
            jmp b1(v9)
        }
        ";

        assert_ssa_does_not_change(src, Ssa::loop_invariant_code_motion);
    }

    #[test]
    fn do_not_hoist_control_dependent_cast() {
        // We want to check the case that a cast under a predicate in a loop is not hoisted
        //
        // This is the SSA for the following program:
        // ```noir
        // fn main(a: bool, c: i8) -> pub i16 {
        //     for _ in 0..1 {
        //         if a {
        //             let _ = c * 127;
        //         };
        //     }
        //     3
        // }
        // ```
        // Although `c*127` is loop invariant, the overflow checks of the multiplication must not be hoisted from the conditional `if a {..}`
        // They are code-gen as:
        //    `range_check v23 to 8 bits`
        //    `v24 = cast v23 as u8`
        let src = r"
        acir(inline) impure fn main f0 {
          b0(v0: u1, v1: i8):
            jmp b1(u32 0)
          b1(v2: u32):
            v4 = eq v2, u32 0
            jmpif v4 then: b2, else: b3
          b2():
            jmpif v0 then: b4, else: b5
          b3():
            return i16 3
          b4():
            v7 = mul v1, i8 127
            v8 = cast v7 as u16
            v9 = truncate v8 to 8 bits, max_bit_size: 16
            v10 = cast v1 as u8
            v12 = lt v10, u8 128
            v13 = not v12
            v14 = cast v1 as Field
            v15 = cast v12 as Field
            v16 = mul v15, v14
            v18 = sub Field 256, v14
            v19 = cast v13 as Field
            v20 = mul v19, v18
            v21 = add v16, v20
            v23 = mul v21, Field 127
            range_check v23 to 8 bits
            v24 = cast v23 as u8
            v25 = not v12
            v26 = cast v25 as u8
            v27 = unchecked_add u8 128, v26
            v28 = lt v24, v27
            constrain v28 == u1 1
            v30 = cast v9 as i8
            jmp b5()
          b5():
            v32 = unchecked_add v2, u32 1
            jmp b1(v32)
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();

        // We expect `v24 = cast v23 as u8` not to be hoisted and be kept in block `b4`.
        // If we were to hoist that cast to the outer loop's header, we would get potentially
        // an unsafe cast. It must stay just after the range-check.
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) impure fn main f0 {
          b0(v0: u1, v1: i8):
            v4 = mul v1, i8 127
            v5 = cast v4 as u16
            v6 = truncate v5 to 8 bits, max_bit_size: 16
            v7 = cast v1 as u8
            v9 = lt v7, u8 128
            v10 = not v9
            v11 = cast v1 as Field
            v12 = cast v9 as Field
            v13 = mul v12, v11
            v15 = sub Field 256, v11
            v16 = cast v10 as Field
            v17 = mul v16, v15
            v18 = add v13, v17
            v20 = mul v18, Field 127
            v21 = not v9
            v22 = cast v21 as u8
            v23 = unchecked_add u8 128, v22
            jmp b1(u32 0)
          b1(v2: u32):
            v25 = eq v2, u32 0
            jmpif v25 then: b2, else: b3
          b2():
            jmpif v0 then: b4, else: b5
          b3():
            return i16 3
          b4():
            range_check v20 to 8 bits
            v27 = cast v20 as u8
            v28 = lt v27, v23
            constrain v28 == u1 1
            v30 = cast v6 as i8
            jmp b5()
          b5():
            v32 = unchecked_add v2, u32 1
            jmp b1(v32)
        }
        ");
    }

    #[test]
    fn loop_with_break_is_not_fully_executed() {
        // Based on the SSA for the following program:
        // ```noir
        // unconstrained fn main(x: u32) -> pub u32 {
        //     let mut s = 0;
        //     for i in 0..10 {
        //         s += i;
        //         if i >= x {
        //             break;
        //         }
        //     }
        //     s
        // }
        // ```
        // The ID of blocks and their order has been altered,
        // so that the header is not the first block or the one
        // with the lowest ID.
        let src = r"
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v2 = allocate -> &mut u32
            store u32 0 at v2
            jmp b2(u32 0)
          b1():
            v6 = load v2 -> u32
            v7 = add v6, v1
            store v7 at v2
            v8 = lt v1, v0
            v9 = not v8
            jmpif v9 then: b4, else: b5
          b2(v1: u32):
            v5 = lt v1, u32 10
            jmpif v5 then: b1, else: b3
          b4():
            jmp b3()
          b5():
            v11 = unchecked_add v1, u32 1
            jmp b2(v11)
          b3():
            v12 = load v2 -> u32
            return v12
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let mut loops = Loops::find_all(ssa.main());
        let loop_ = loops.yet_to_unroll.pop().unwrap();
        assert!(!loop_.is_fully_executed(&loops.cfg));
    }

    #[test]
    fn infinite_loop_is_not_fully_executed() {
        let src = r"
        brillig(inline) impure fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 0 at v0
            jmp b1()
          b1():
            v1 = load v0 -> Field
            v2 = add v1, Field 1
            store v2 at v0
            jmp b1()
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let mut loops = Loops::find_all(ssa.main());
        let loop_ = loops.yet_to_unroll.pop().unwrap();
        assert!(!loop_.is_fully_executed(&loops.cfg));
    }

    #[test]
    fn while_loop_with_break_is_not_fully_executed() {
        // SSA from a program such as this:
        // let mut idx_d: u32 = 0_u32;
        // while true {
        //     if (idx_d == 3_u32) {
        //         break
        //     } else {
        //         ...
        //     }
        // }
        let src = r"
        brillig(inline) fn main f0 {
          b0(v0: u1, v1: &mut u32):
            v17 = allocate -> &mut u32
            store u32 0 at v17
            jmp b8()
          b1():
            return
          b8():
            jmp b9()
          b9():
            v18 = load v17 -> u32
            v20 = eq v18, u32 3
            jmpif v20 then: b10, else: b11
          b10():
            jmp b1()
          b11():
            v21 = load v17 -> u32
            v22 = add v21, u32 1
            store v22 at v17
            jmp b8()
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let mut loops = Loops::find_all(ssa.main());
        let loop_ = loops.yet_to_unroll.pop().unwrap();
        assert!(!loop_.is_fully_executed(&loops.cfg));
    }

    #[test_case(RuntimeType::Brillig(InlineType::default()))]
    #[test_case(RuntimeType::Acir(InlineType::default()))]
    fn do_not_hoist_unsafe_array_get_from_control_dependent_block(runtime: RuntimeType) {
        // We use an unknown index v0 to index an array, but only if v1 is true,
        // so we should not hoist the constraint or the array_get into the header.
        // Hoisting the array operation and indexing with an invalid `v0` would
        // not cause an OOB in Brillig, however the returned value would be invalid,
        // causing knockdown loop invariant instructions to fail when the loop is not meant to fail.
        let src = format!(
            r#"
          {runtime} impure fn main f0 {{
            b0(v0: u32, v1: u1):
              v2 = make_array [u8 0, u8 1] : [u8; 2]
              jmp b1(u32 0)
            b1(v3: u32):
              v4 = lt v3, u32 2
              jmpif v4 then: b2, else: b3
            b2():
              jmpif v1 then: b4, else: b5
            b3():
              return
            b4():
              constrain v0 == u32 0, "Index out of bounds"
              v5 = array_get v2, index v0 -> u8
              jmp b5()
            b5():
              v6 = unchecked_add v3, u32 1
              jmp b1(v6)
          }}
          "#
        );
        assert_ssa_does_not_change(&src, Ssa::loop_invariant_code_motion);
    }
}
