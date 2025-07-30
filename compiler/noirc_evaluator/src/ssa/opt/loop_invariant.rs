//! The loop invariant code motion (LICM) pass moves code from inside a loop to before the loop
//! if that code will always have the same result on every iteration of the loop.
//!
//! To identify a loop invariant, check whether all of an instruction's values are:
//! - Outside of the loop
//! - Constant
//! - Already marked as loop invariants
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
//! (1) there exists a directed path P from X to Y with any 2 in P (excluding X
//! and Y) post-dominated by Y and
//! (2) X is not post-dominated by Y.
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
//! are the same as those for post-dominance frontiers.
//! Thus, we rewrite our control dependence condition as Y is control dependent on X iff X is in PDF(Y).
//!
//! We then can store the PDFs for every block as part of the context of this pass, and use it for checking control dependence.
//! Using PDFs gets us from a worst case n^2 complexity to a worst case n.
use crate::ssa::{
    Ssa,
    ir::{
        basic_block::BasicBlockId,
        cfg::ControlFlowGraph,
        dfg::simplify::SimplifyResult,
        dom::DominatorTree,
        function::Function,
        function_inserter::FunctionInserter,
        instruction::{
            Binary, BinaryOp, ConstrainError, Instruction, InstructionId,
            binary::{BinaryEvaluationResult, eval_constant_binary_op},
        },
        integer::IntegerConstant,
        post_order::PostOrder,
        types::{NumericType, Type},
        value::{Value, ValueId},
    },
    opt::pure::{FunctionPurities, Purity},
};
use acvm::{FieldElement, acir::AcirField};
use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};
use noirc_errors::call_stack::CallStackId;

use super::unrolling::{Loop, Loops};

impl Ssa {
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn loop_invariant_code_motion(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.loop_invariant_code_motion(&self.function_purities);
        }
        self
    }
}

impl Function {
    pub(super) fn loop_invariant_code_motion(&mut self, purities: &FunctionPurities) {
        Loops::find_all(self).hoist_loop_invariants(self, purities);
    }
}

impl Loops {
    fn hoist_loop_invariants(mut self, function: &mut Function, purities: &FunctionPurities) {
        let mut context = LoopInvariantContext::new(function, purities);

        // Insert all loop bounds up front, so we can inspect both outer and nested loops.
        for loop_ in &self.yet_to_unroll {
            context.set_induction_var_bounds(loop_, None);
        }

        // The loops should be sorted by the number of blocks.
        // We want to access outer nested loops first, which we do by popping
        // from the top of the list.
        while let Some(loop_) = self.yet_to_unroll.pop() {
            // If the loop does not have a preheader we skip hoisting loop invariants for this loop
            let Ok(pre_header) = loop_.get_pre_header(context.inserter.function, &self.cfg) else {
                continue;
            };
            context.current_pre_header = Some(pre_header);
            context.hoist_loop_invariants(&loop_, &self.yet_to_unroll);
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

    // Check if the loop will be fully executed by checking the number of predecessors of the loop exit
    // Our SSA code generation restricts loops to having one exit block except in the case of a `break`.
    // If a loop can have several exit blocks, we would need to update this function
    pub(super) fn is_fully_executed(&self, cfg: &ControlFlowGraph) -> bool {
        let Some(header) = self.blocks.first() else {
            return true;
        };
        for block in cfg.successors(*header) {
            if !self.blocks.contains(&block) {
                return cfg.predecessors(block).len() == 1;
            }
        }
        true
    }
}

struct LoopInvariantContext<'f, 'purities> {
    inserter: FunctionInserter<'f>,
    defined_in_loop: HashSet<ValueId>,
    loop_invariants: HashSet<ValueId>,
    // Maps current loop induction variable -> fixed lower and upper loop bound
    // This map is expected to only ever contain a singular value.
    // However, we store it in a map in order to match the definition of
    // `outer_induction_variables` as both maps share checks for evaluating binary operations.
    current_induction_variables: HashMap<ValueId, (IntegerConstant, IntegerConstant)>,
    // Maps outer loop induction variable -> fixed lower and upper loop bound
    // This will be used by inner loops to determine whether they
    // have safe operations reliant upon an outer loop's maximum induction variable
    outer_induction_variables: HashMap<ValueId, (IntegerConstant, IntegerConstant)>,
    // All induction variables collected up front.
    all_induction_variables: HashMap<ValueId, (IntegerConstant, IntegerConstant)>,
    // This context struct processes runs across all loops.
    // This stores the current loop's pre-header block.
    // It is wrapped in an Option as our SSA `Id<T>` does not allow dummy values.
    current_pre_header: Option<BasicBlockId>,

    cfg: ControlFlowGraph,

    /// Caches all blocks that belong to nested loops determined to be control dependent
    /// on blocks in an outer loop. This allows short circuiting future control dependence
    /// checks during loop invariant analysis, as these blocks are guaranteed to be
    /// control dependent due to the entire nested loop being control dependent.
    ///
    /// Reset for each new loop as the set should not be shared across different outer loops.
    nested_loop_control_dependent_blocks: HashSet<BasicBlockId>,

    // Maps a block to its post-dominance frontiers
    // This map should be precomputed a single time and used for checking control dependence.
    post_dom_frontiers: HashMap<BasicBlockId, HashSet<BasicBlockId>>,

    // Stores whether the current block being processed is control dependent
    current_block_control_dependent: bool,

    // Tracks whether the current block has a side-effectual instruction.
    // This is maintained per instruction for hoisting control dependent instructions
    current_block_impure: bool,

    // Stores whether the current block has known fix upper and lower bounds that
    // indicate that it is guaranteed to execute at least once.
    current_block_executes: bool,

    // Indicates whether the current loop has break or early returns
    no_break: bool,
    // Helper constants
    true_value: ValueId,
    false_value: ValueId,

    purities: &'purities FunctionPurities,
}

impl<'f, 'purities> LoopInvariantContext<'f, 'purities> {
    fn new(function: &'f mut Function, purities: &'purities FunctionPurities) -> Self {
        let cfg = ControlFlowGraph::with_function(function);
        let reversed_cfg = ControlFlowGraph::extended_reverse(function);
        let post_order = PostOrder::with_cfg(&reversed_cfg);
        let mut post_dom = DominatorTree::with_cfg_and_post_order(&reversed_cfg, &post_order);
        let post_dom_frontiers = post_dom.compute_dominance_frontiers(&reversed_cfg);
        let true_value =
            function.dfg.make_constant(FieldElement::one(), NumericType::Unsigned { bit_size: 1 });
        let false_value =
            function.dfg.make_constant(FieldElement::zero(), NumericType::Unsigned { bit_size: 1 });
        Self {
            inserter: FunctionInserter::new(function),
            defined_in_loop: HashSet::default(),
            loop_invariants: HashSet::default(),
            current_induction_variables: HashMap::default(),
            outer_induction_variables: HashMap::default(),
            all_induction_variables: HashMap::default(),
            current_pre_header: None,
            cfg,
            nested_loop_control_dependent_blocks: HashSet::default(),
            post_dom_frontiers,
            current_block_control_dependent: false,
            current_block_impure: false,
            current_block_executes: false,
            true_value,
            false_value,
            no_break: false,
            purities,
        }
    }

    fn pre_header(&self) -> BasicBlockId {
        self.current_pre_header.expect("ICE: Pre-header block should have been set")
    }

    fn hoist_loop_invariants(&mut self, loop_: &Loop, all_loops: &[Loop]) {
        self.set_values_defined_in_loop(loop_);

        for block in loop_.blocks.iter() {
            // Reset the per block state
            self.current_block_executes = self.does_block_execute(*block, all_loops);
            self.current_block_impure = false;
            self.is_control_dependent_post_pre_header(loop_, *block, all_loops);

            for instruction_id in self.inserter.function.dfg[*block].take_instructions() {
                if self.simplify_from_loop_bounds(instruction_id, loop_, block) {
                    continue;
                }
                let hoist_invariant = self.can_hoist_invariant(instruction_id);

                if hoist_invariant {
                    self.inserter.push_instruction(instruction_id, self.pre_header());

                    // If we are hoisting a MakeArray instruction,
                    // we need to issue an extra inc_rc in case they are mutated afterward.
                    if self.inserter.function.runtime().is_brillig()
                        && matches!(
                            self.inserter.function.dfg[instruction_id],
                            Instruction::MakeArray { .. }
                        )
                    {
                        let result =
                            self.inserter.function.dfg.instruction_results(instruction_id)[0];
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
                    // If the block has already been labelled as impure, we do need to check the current
                    // instruction's side effects.
                    if !self.current_block_impure {
                        self.current_block_impure =
                            dfg[instruction_id].has_side_effects(dfg, self.purities);
                    }
                    self.inserter.push_instruction(instruction_id, *block);
                }
                self.extend_values_defined_in_loop_and_invariants(instruction_id, hoist_invariant);
            }
        }

        self.set_induction_var_bounds(loop_, Some(false));
    }

    /// Checks whether a `block` is control dependent on any blocks after
    /// the given loop's header.
    fn is_control_dependent_post_pre_header(
        &mut self,
        loop_: &Loop,
        block: BasicBlockId,
        all_loops: &[Loop],
    ) {
        // The block is already known to be in a control dependent nested loop
        // Thus, we can avoid checking for control dependence again.
        if self.nested_loop_control_dependent_blocks.contains(&block) {
            self.current_block_control_dependent = true;
            return;
        }

        // Find all blocks between the current block and the loop header, exclusive of the current block and loop header themselves.
        let all_predecessors = Loop::find_blocks_in_loop(loop_.header, block, &self.cfg).blocks;
        let all_predecessors = all_predecessors
            .into_iter()
            .filter(|&predecessor| predecessor != block && predecessor != loop_.header)
            .collect::<Vec<_>>();

        let dfg = &self.inserter.function.dfg;
        // When hoisting a control dependent instruction, if a side effectual instruction comes in the predecessor block
        // of that instruction we can no longer hoist the control dependent instruction.
        // This is important for maintaining ordering semantic correctness of the code.
        assert!(!self.current_block_impure, "ICE: Block impurity should be defaulted to false");
        self.current_block_impure = all_predecessors.iter().any(|block| {
            dfg[*block]
                .instructions()
                .iter()
                .any(|instruction| dfg[*instruction].has_side_effects(dfg, self.purities))
        });

        // Reset the current block control dependent flag, the check will set it to true if needed.
        // If we fail to reset it, a block may be inadvertently labelled
        // as control dependent thus preventing optimizations.
        self.current_block_control_dependent = false;

        // Now check whether the current block is dependent on any blocks between
        // the current block and the loop header, exclusive of the current block and loop header themselves
        if all_predecessors.iter().any(|predecessor| self.is_control_dependent(*predecessor, block))
        {
            self.current_block_control_dependent = true;
            return;
        }

        self.is_nested_loop_control_dependent(loop_, block, all_loops, all_predecessors);
    }

    /// Determines if the `block` is in a nested loop that is control dependent
    /// on a block in the outer loop.
    /// If this is the case, we block hoisting as control is not guaranteed.
    /// If the block is not control dependent on the inner loop itself, it will be marked appropriately
    /// when the inner loop is processed later.
    ///
    /// Control dependence on a nested loop is determined by checking whether the nested loop's header
    /// is control dependent on any blocks between itself and the outer loop's header.
    /// It is expected that `all_predecessors` contains at least all of these blocks.
    fn is_nested_loop_control_dependent(
        &mut self,
        loop_: &Loop,
        block: BasicBlockId,
        all_loops: &[Loop],
        all_predecessors: Vec<BasicBlockId>,
    ) {
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
                    .any(|predecessor| self.is_control_dependent(*predecessor, nested.header));
            if nested_loop_is_control_dep {
                self.current_block_control_dependent = true;
                // Mark all blocks in the nested loop as control dependent to avoid redundant checks
                // for each of these blocks when they are later visited during hoisting.
                // This is valid because control dependence of the loop header implies dependence
                // for the entire loop body.
                self.nested_loop_control_dependent_blocks.extend(nested.blocks.iter());
                return;
            }
        }
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
    fn does_block_execute(&mut self, block: BasicBlockId, all_loops: &[Loop]) -> bool {
        /// Check that we have fixed bounds and upper is higher than lower.
        fn check_bounds(bounds: Option<&(IntegerConstant, IntegerConstant)>) -> bool {
            bounds
                .copied()
                .and_then(|(lower_bound, upper_bound)| {
                    upper_bound.reduce(lower_bound, |u, l| u > l, |u, l| u > l)
                })
                .unwrap_or(false)
        }

        // If the current loop doesn't execute, then nothing does.
        if !check_bounds(self.current_induction_variables.values().next()) {
            return false;
        }

        // If the block is part of any nested loop, they have to execute as well.
        for nested in all_loops.iter() {
            if !nested.blocks.contains(&block) {
                continue;
            }
            let Some(induction_variable) = self.get_induction_variable(nested) else {
                // If we don't know what the induction variable is, we can't say if it executes.
                return false;
            };
            if !check_bounds(self.all_induction_variables.get(&induction_variable)) {
                return false;
            }
        }

        true
    }

    /// Get and resolve the induction variable of a loop.
    fn get_induction_variable(&mut self, loop_: &Loop) -> Option<ValueId> {
        loop_.get_induction_variable(self.inserter.function).map(|v| self.inserter.resolve(v))
    }

    /// Checks whether a `block` is control dependent on a `parent_block`.
    /// Uses post-dominance frontiers to determine control dependence.
    /// Reference the doc comments at the top of the this module for more information
    /// regarding post-dominance frontiers and control dependence.
    fn is_control_dependent(&self, parent_block: BasicBlockId, block: BasicBlockId) -> bool {
        match self.post_dom_frontiers.get(&block) {
            Some(dependent_blocks) => dependent_blocks.contains(&parent_block),
            None => false,
        }
    }

    /// Gather the variables declared within the loop
    fn set_values_defined_in_loop(&mut self, loop_: &Loop) {
        // Clear any values that may be defined in previous loops, as the context is per function.
        self.defined_in_loop.clear();
        // These are safe to keep per function, but we want to be clear that these values
        // are used per loop.
        self.loop_invariants.clear();
        // There is only ever one current induction variable for a loop.
        // For a new loop, we clear the previous induction variable and then
        // set the new current induction variable.
        self.current_induction_variables.clear();
        self.set_induction_var_bounds(loop_, Some(true));
        self.no_break = loop_.is_fully_executed(&self.cfg);
        // Clear any cached control dependent nested loop blocks from the previous loop.
        // This set is only relevant within the scope of a single loop.
        // Keeping previous data would incorrectly classify blocks as control dependent,
        // leading to missed hoisting opportunities.
        self.nested_loop_control_dependent_blocks.clear();

        for block in loop_.blocks.iter() {
            let params = self.inserter.function.dfg.block_parameters(*block);
            self.defined_in_loop.extend(params);
            for instruction_id in self.inserter.function.dfg[*block].instructions() {
                let results = self.inserter.function.dfg.instruction_results(*instruction_id);
                self.defined_in_loop.extend(results);
            }
        }
    }

    /// Update any values defined in the loop and loop invariants after
    /// analyzing and re-inserting a loop's instruction.
    fn extend_values_defined_in_loop_and_invariants(
        &mut self,
        instruction_id: InstructionId,
        hoist_invariant: bool,
    ) {
        let results = self.inserter.function.dfg.instruction_results(instruction_id).to_vec();
        // We will have new IDs after pushing instructions.
        // We should mark the resolved result IDs as also being defined within the loop.
        let results =
            results.into_iter().map(|value| self.inserter.resolve(value)).collect::<Vec<_>>();
        self.defined_in_loop.extend(results.iter());

        // We also want the update result IDs when we are marking loop invariants as we may not
        // be going through the blocks of the loop in execution order
        if hoist_invariant {
            // Track already found loop invariants
            self.loop_invariants.extend(results.iter());
        }
    }

    fn can_hoist_invariant(&mut self, instruction_id: InstructionId) -> bool {
        use Instruction::*;

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
            is_loop_invariant &= self.is_loop_invariant(&value);
        });

        if !is_loop_invariant {
            return false;
        }

        matches!(instruction, MakeArray { .. })
            || can_be_hoisted(&instruction, self.inserter.function, false, self.purities)
            || self.can_be_hoisted_with_control_dependence(&instruction, self.inserter.function)
            || self.can_be_hoisted_from_loop_bounds(&instruction)
    }

    /// Check [can_be_hoisted] with extra control dependence information that
    /// lives within the context of this pass.
    fn can_be_hoisted_with_control_dependence(
        &self,
        instruction: &Instruction,
        function: &Function,
    ) -> bool {
        can_be_hoisted(instruction, function, true, self.purities)
            && self.can_hoist_control_dependent_instruction()
    }

    /// Keep track of a loop induction variable and respective upper bound.
    ///
    /// In the case of a nested loop, this will be used by later loops to determine
    /// whether they have operations reliant upon the maximum induction variable.
    ///
    /// When within the current loop, the known upper bound can be used to simplify instructions,
    /// such as transforming a checked add to an unchecked add.
    ///
    /// Depending on the value of `current_loop` this has 3 modes:
    /// * `true` sets it in `current_induction_variables`
    /// * `false` sets it for the `outer_induction_variables`, but this must only happen after the current one is finished
    /// * `None` sets it in `all_induction_variables`, which is used to make decisions about inner nested loops
    fn set_induction_var_bounds(&mut self, loop_: &Loop, current_loop: Option<bool>) {
        let pre_header = if current_loop.is_some() {
            self.pre_header()
        } else if let Ok(pre_header) = loop_.get_pre_header(self.inserter.function, &self.cfg) {
            pre_header
        } else {
            // We will skip this loop.
            return;
        };

        let bounds = loop_.get_const_bounds(self.inserter.function, pre_header);

        if let Some((lower_bound, upper_bound)) = bounds {
            let Some(induction_variable) = self.get_induction_variable(loop_) else {
                return;
            };
            match current_loop {
                Some(true) => {
                    self.current_induction_variables
                        .insert(induction_variable, (lower_bound, upper_bound));
                }
                Some(false) => {
                    self.outer_induction_variables
                        .insert(induction_variable, (lower_bound, upper_bound));
                }
                None => {
                    self.all_induction_variables
                        .insert(induction_variable, (lower_bound, upper_bound));
                }
            }
        }
    }

    /// Certain instructions can take advantage of that our induction variable has a fixed minimum/maximum.
    ///
    /// For example, an array access can usually only be safely deduplicated when we have a constant
    /// index that is below the length of the array.
    /// Checking an array get where the index is the loop's induction variable on its own
    /// would determine that the instruction is not safe for hoisting.
    /// However, if we know that the induction variable's upper bound will always be in bounds of the array
    /// we can safely hoist the array access.
    fn can_be_hoisted_from_loop_bounds(&self, instruction: &Instruction) -> bool {
        use Instruction::*;

        match instruction {
            ArrayGet { array, index, offset: _ } => {
                let array_typ = self.inserter.function.dfg.type_of_value(*array);
                let upper_bound = self.outer_induction_variables.get(index).map(|bounds| bounds.1);
                if let (Type::Array(_, len), Some(upper_bound)) = (array_typ, upper_bound) {
                    upper_bound.apply(|i| i <= len.into(), |i| i <= len.into())
                } else {
                    false
                }
            }
            Binary(binary) => self.can_evaluate_binary_op(binary),
            Constrain(..) | ConstrainNotEqual(..) | RangeCheck { .. } => {
                self.can_hoist_control_dependent_instruction()
            }
            Call { func, .. } => {
                let purity = match self.inserter.function.dfg[*func] {
                    Value::Intrinsic(intrinsic) => Some(intrinsic.purity()),
                    Value::Function(id) => self.purities.get(&id).copied(),
                    _ => None,
                };
                matches!(purity, Some(Purity::PureWithPredicate))
                    && self.can_hoist_control_dependent_instruction()
            }
            _ => false,
        }
    }

    /// A control dependent instruction (e.g. constrain or division) has more strict conditions for hoisting.
    /// This function check the following conditions:
    /// - The current block is non control dependent
    /// - The loop body is guaranteed to be executed
    /// - The current block is not impure
    fn can_hoist_control_dependent_instruction(&self) -> bool {
        !self.current_block_control_dependent
            && self.current_block_executes
            && !self.current_block_impure
    }

    /// A control dependent instruction (e.g. constrain or division) has more strict conditions for simplifying.
    /// This function matches [LoopInvariantContext::can_hoist_control_dependent_instruction] except
    /// that simplification does not require that current block is pure to be simplified.
    fn can_simplify_control_dependent_instruction(&self) -> bool {
        !self.current_block_control_dependent && self.current_block_executes
    }

    /// Some instructions can take advantage of that our induction variable has a fixed minimum/maximum,
    /// For instance operations can be transformed from a checked operation to an unchecked operation.
    ///
    /// Checked operations require more bytecode and thus we aim to minimize their usage wherever possible.
    ///
    /// For example, if one side of an add/mul operation is a constant and the other is an induction variable
    /// with a known upper bound, we know whether that binary operation will ever overflow.
    /// If we determine that an overflow is not possible we can convert the checked operation to unchecked.
    ///
    /// The function returns false if the instruction must be added to the block
    fn simplify_from_loop_bounds(
        &mut self,
        instruction_id: InstructionId,
        loop_: &Loop,
        current_block: &BasicBlockId,
    ) -> bool {
        // Simplify the instruction and update it in the DFG.
        match self.simplify_induction_variable(instruction_id, loop_, current_block) {
            SimplifyResult::SimplifiedTo(id) => {
                let results =
                    self.inserter.function.dfg.instruction_results(instruction_id).to_vec();
                assert!(results.len() == 1);
                self.inserter.map_value(results[0], id);
                true
            }
            SimplifyResult::SimplifiedToInstruction(instruction) => {
                self.inserter.function.dfg[instruction_id] = instruction;
                false
            }
            SimplifyResult::Remove => true,
            SimplifyResult::None => false,
            _ => unreachable!(
                "ICE - loop bounds simplification should only simplify to a value or an instruction"
            ),
        }
    }

    /// Simplify 'assert(lhs < rhs)' into 'assert(max(lhs) < rhs)' if lhs is an induction variable and rhs a loop invariant
    /// For this simplification to be valid, we need to ensure that the induction variable takes all the values from min(induction) up to max(induction)
    /// This means that the assert must be executed  at each loop iteration, and that the loop processes all the iteration space
    /// This is ensured via control dependence and the check for break patterns, before calling this function.
    fn simplify_induction_in_constrain(
        &mut self,
        lhs: ValueId,
        rhs: ValueId,
        err: &Option<ConstrainError>,
        call_stack: CallStackId,
    ) -> SimplifyResult {
        let mut extract_variables = |rhs, lhs| {
            let rhs_true =
                self.inserter.function.dfg.get_numeric_constant(rhs)? == FieldElement::one();
            let (is_left, min, max, binary) = self.extract_induction_and_invariant(lhs)?;
            match (is_left, rhs_true) {
                (true, true) => Some((max, binary.rhs)),
                (false, true) => Some((binary.lhs, min)),
                _ => None,
            }
        };
        let Some((new_lhs, new_rhs)) = extract_variables(rhs, lhs) else {
            return SimplifyResult::None;
        };
        let new_binary =
            Instruction::Binary(Binary { lhs: new_lhs, rhs: new_rhs, operator: BinaryOp::Lt });
        // The new comparison can be safely hoisted to the pre-header because it is loop invariant and control independent
        let comparison_results = self
            .inserter
            .function
            .dfg
            .insert_instruction_and_results(new_binary, self.pre_header(), None, call_stack)
            .results();
        assert!(comparison_results.len() == 1);
        SimplifyResult::SimplifiedToInstruction(Instruction::Constrain(
            comparison_results[0],
            rhs,
            err.clone(),
        ))
    }

    /// Replace 'assert(invariant != induction)' with assert((invariant < min(induction) || (invariant > max(induction)))
    /// For this simplification to be valid, we need to ensure that the induction variable takes all the values from min(induction) up to max(induction)
    /// This means that the assert must be executed  at each loop iteration, and that the loop processes all the iteration space
    /// This is ensured via control dependence and the check for break patterns, before calling this function.
    fn simplify_not_equal_constraint(
        &mut self,
        lhs: &ValueId,
        rhs: &ValueId,
        err: &Option<ConstrainError>,
        call_stack: CallStackId,
    ) -> SimplifyResult {
        let (invariant, upper, lower) = match self.match_induction_and_invariant(lhs, rhs) {
            Some((true, upper, lower)) => (rhs, upper, lower),
            Some((false, upper, lower)) => (lhs, upper, lower),
            _ => return SimplifyResult::None,
        };

        let mut insert_binary_to_preheader = |lhs, rhs, operator| {
            let binary = Instruction::Binary(Binary { lhs, rhs, operator });
            let results = self
                .inserter
                .function
                .dfg
                .insert_instruction_and_results(binary, self.pre_header(), None, call_stack)
                .results();
            assert!(results.len() == 1);
            results[0]
        };
        // The comparisons can be safely hoisted to the pre-header because they are loop invariant and control independent
        let check_lower_bound = insert_binary_to_preheader(*invariant, lower, BinaryOp::Lt);
        let check_upper_bound = insert_binary_to_preheader(upper, *invariant, BinaryOp::Lt);
        let check_bounds =
            insert_binary_to_preheader(check_lower_bound, check_upper_bound, BinaryOp::Or);

        SimplifyResult::SimplifiedToInstruction(Instruction::Constrain(
            check_bounds,
            self.true_value,
            err.clone(),
        ))
    }

    /// Returns the binary instruction only if the input value refers to a binary instruction with invariant and induction variables as operands
    /// The return values are:
    /// - a boolean indicating if the induction variable is on the lhs
    /// - the minimum and maximum values of the induction variable, coming from the loop bounds
    /// - the binary instruction itself
    fn extract_induction_and_invariant(
        &mut self,
        value: ValueId,
    ) -> Option<(bool, ValueId, ValueId, Binary)> {
        let Value::Instruction { instruction, .. } = &self.inserter.function.dfg[value] else {
            return None;
        };
        let Instruction::Binary(binary) = self.inserter.function.dfg[*instruction].clone() else {
            return None;
        };
        self.match_induction_and_invariant(&binary.lhs, &binary.rhs)
            .map(|(is_left, min, max)| (is_left, min, max, binary))
    }

    /// If the inputs are an induction and loop invariant variables, it returns
    /// the maximum and minimum values of the induction variable, based on the loop bounds,
    /// and a boolean indicating if the induction variable is on the lhs or rhs (true for lhs)
    fn match_induction_and_invariant(
        &mut self,
        lhs: &ValueId,
        rhs: &ValueId,
    ) -> Option<(bool, ValueId, ValueId)> {
        let (is_left, lower, upper) = match (
            self.current_induction_variables.get(lhs),
            self.current_induction_variables.get(rhs),
        ) {
            (_, Some((lower, upper))) => Some((false, lower, upper)),
            (Some((lower, upper)), _) => Some((true, lower, upper)),
            _ => None,
        }?;

        assert!(self.current_block_executes, "executing a non executable loop");

        let (upper_field, upper_type) = upper.dec().into_numeric_constant();
        let (lower_field, lower_type) = lower.into_numeric_constant();

        let min_iter = self.inserter.function.dfg.make_constant(lower_field, lower_type);
        let max_iter = self.inserter.function.dfg.make_constant(upper_field, upper_type);
        if (is_left && self.is_loop_invariant(rhs)) || (!is_left && self.is_loop_invariant(lhs)) {
            return Some((is_left, min_iter, max_iter));
        }
        None
    }

    /// Simplify certain instructions using the lower/upper bounds of induction variables
    fn simplify_induction_variable(
        &mut self,
        instruction_id: InstructionId,
        loop_: &Loop,
        block: &BasicBlockId,
    ) -> SimplifyResult {
        let header = loop_.header == *block;

        let (instruction, call_stack) = self.inserter.map_instruction(instruction_id);
        match &instruction {
            Instruction::Binary(binary) => {
                self.simplify_induction_variable_in_binary(binary, header)
            }
            Instruction::Constrain(x, y, err) => {
                // Ensure the loop is fully executed
                if self.no_break && self.can_simplify_control_dependent_instruction() {
                    self.simplify_induction_in_constrain(*x, *y, err, call_stack)
                } else {
                    SimplifyResult::None
                }
            }
            Instruction::ConstrainNotEqual(x, y, err) => {
                // Ensure the loop is fully executed
                if self.no_break && self.can_simplify_control_dependent_instruction() {
                    self.simplify_not_equal_constraint(x, y, err, call_stack)
                } else {
                    SimplifyResult::None
                }
            }
            _ => SimplifyResult::None,
        }
    }

    fn is_loop_invariant(&self, value_id: &ValueId) -> bool {
        !self.defined_in_loop.contains(value_id) || self.loop_invariants.contains(value_id)
    }

    /// If the inputs are an induction variable and a constant, it returns
    /// the constant value, the maximum and minimum values of the induction variable, based on the loop bounds,
    /// and a boolean indicating if the induction variable is on the lhs or rhs (true for lhs)
    /// if `only_outer_induction`, we only consider outer induction variables, else we also consider the induction variables from the current loop.
    fn match_induction_and_constant(
        &self,
        lhs: &ValueId,
        rhs: &ValueId,
        only_outer_induction: bool,
    ) -> Option<(bool, IntegerConstant, IntegerConstant, IntegerConstant)> {
        let lhs_const = self.inserter.function.dfg.get_integer_constant(*lhs);
        let rhs_const = self.inserter.function.dfg.get_integer_constant(*rhs);
        match (
            lhs_const,
            rhs_const,
            self.current_induction_variables
                .get(lhs)
                .and_then(|v| if only_outer_induction { None } else { Some(v) })
                .or(self.outer_induction_variables.get(lhs)),
            self.current_induction_variables
                .get(rhs)
                .and_then(|v| if only_outer_induction { None } else { Some(v) })
                .or(self.outer_induction_variables.get(rhs)),
        ) {
            // LHS is a constant, RHS is the induction variable with a known lower and upper bound.
            (Some(lhs), None, None, Some((lower_bound, upper_bound))) => {
                Some((false, lhs, *lower_bound, *upper_bound))
            }
            // RHS is a constant, LHS is the induction variable with a known lower an upper bound
            (None, Some(rhs), Some((lower_bound, upper_bound)), None) => {
                Some((true, rhs, *lower_bound, *upper_bound))
            }
            _ => None,
        }
    }

    /// Given a constant `c` and an induction variable `i`:
    /// - Replace comparisons `i < c` by true if `max(i) < c`, and false if `min(i) >= c`
    /// - Replace comparisons `c < i` by true if `min(i) > c`, and false if `max(i) <= c`
    /// - Replace equalities `i == c` by false if `min(i) > c or max(i) < c`
    /// - Replace checked operations with unchecked version if the induction variable bounds prove that the operation will not overflow
    ///
    /// `header` indicates if we are in the loop header where loop bounds do not apply yet
    fn simplify_induction_variable_in_binary(
        &mut self,
        binary: &Binary,
        header: bool,
    ) -> SimplifyResult {
        // Checks the operands are an induction variable and a constant
        // Note that here we allow all_induction_variables
        let operand_type = self.inserter.function.dfg.type_of_value(binary.lhs).unwrap_numeric();

        let Some((is_induction_var_lhs, value, lower_bound, upper_bound)) =
            self.match_induction_and_constant(&binary.lhs, &binary.rhs, header)
        else {
            return SimplifyResult::None;
        };

        // Handle arithmetic operations
        if let Some((lhs, rhs)) = match binary.operator {
            BinaryOp::Add { unchecked }
            | BinaryOp::Sub { unchecked }
            | BinaryOp::Mul { unchecked }
                if unchecked =>
            {
                return SimplifyResult::None;
            }
            BinaryOp::Sub { .. } => {
                if is_induction_var_lhs {
                    Some((lower_bound, value))
                } else {
                    Some((value, upper_bound))
                }
            }
            BinaryOp::Add { .. } | BinaryOp::Mul { .. } => Some((value, upper_bound)),
            BinaryOp::Div | BinaryOp::Mod => return SimplifyResult::None,
            _ => None,
        } {
            // We evaluate this expression using the upper bounds (or lower in the case of sub)
            // of its inputs to check whether it will ever overflow.
            // If `eval_constant_binary_op` won't overflow we can simplify the instruction to an unchecked version.
            let lhs = lhs.into_numeric_constant().0;
            let rhs = rhs.into_numeric_constant().0;
            match eval_constant_binary_op(lhs, rhs, binary.operator, operand_type) {
                BinaryEvaluationResult::Success(..) => {
                    // Unchecked version of the binary operation
                    let unchecked = Instruction::Binary(Binary {
                        operator: binary.operator.into_unchecked(),
                        lhs: binary.lhs,
                        rhs: binary.rhs,
                    });
                    return SimplifyResult::SimplifiedToInstruction(unchecked);
                }
                BinaryEvaluationResult::CouldNotEvaluate | BinaryEvaluationResult::Failure(..) => {
                    return SimplifyResult::None;
                }
            }
        }

        // Handle comparisons
        match binary.operator {
            BinaryOp::Eq => {
                if value >= upper_bound || value < lower_bound {
                    SimplifyResult::SimplifiedTo(self.false_value)
                } else {
                    SimplifyResult::None
                }
            }
            BinaryOp::Lt => match is_induction_var_lhs {
                true if upper_bound <= value => SimplifyResult::SimplifiedTo(self.true_value),
                true if lower_bound >= value => SimplifyResult::SimplifiedTo(self.false_value),
                false if lower_bound > value => SimplifyResult::SimplifiedTo(self.true_value),
                false if upper_bound <= value.inc() => {
                    SimplifyResult::SimplifiedTo(self.false_value)
                }
                _ => SimplifyResult::None,
            },
            _ => SimplifyResult::None,
        }
    }

    /// Checks whether a binary operation can be evaluated using the bounds of a given loop induction variables.
    ///
    /// If it cannot be evaluated, it means that we either have a dynamic loop bound or
    /// that the operation can potentially overflow during a given loop iteration.
    fn can_evaluate_binary_op(&self, binary: &Binary) -> bool {
        match binary.operator {
            // An unchecked operation cannot overflow, so it can be safely evaluated
            BinaryOp::Add { unchecked: true }
            | BinaryOp::Mul { unchecked: true }
            | BinaryOp::Sub { unchecked: true } => true,
            BinaryOp::Div | BinaryOp::Mod => {
                // Division can be evaluated if we ensure that the divisor cannot be zero
                let Some((left, value, lower, upper)) =
                    self.match_induction_and_constant(&binary.lhs, &binary.rhs, true)
                else {
                    // Not a constant vs non-constant case, we cannot evaluate it.
                    return false;
                };

                if left {
                    // If the induction variable is on the LHS, we're dividing with a constant.
                    if !value.is_zero() {
                        return true;
                    }
                } else {
                    // Otherwise we are dividing a constant with the induction variable, and we have to check whether
                    // at any point in the loop the induction variable can be zero.
                    let can_be_zero = lower.is_negative() && !upper.is_negative()
                        || lower.is_zero()
                        || upper.is_zero();

                    if !can_be_zero {
                        return true;
                    }
                }

                false
            }
            // Some checked operations can be safely evaluated, depending on the loop bounds, but in that case,
            // they would have been already converted to unchecked operation in `simplify_induction_variable_in_binary()`
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
                self.inserter.push_instruction(instruction_id, block);
            }
            self.inserter.map_terminator_in_place(block);
        }
    }
}

/// Indicates if the instruction can be safely hoisted out of a loop.
/// If `hoist_with_predicate` is set, we assume we're hoisting the instruction
/// and its predicate, rather than just the instruction. Setting this means instructions that
/// rely on predicates can be hoisted as well.
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
fn can_be_hoisted(
    instruction: &Instruction,
    function: &Function,
    hoist_with_predicate: bool,
    purities: &FunctionPurities,
) -> bool {
    use Instruction::*;

    match instruction {
        // These either have side-effects or interact with memory
        EnableSideEffectsIf { .. }
        | Allocate
        | Load { .. }
        | Store { .. }
        | IncrementRc { .. }
        | DecrementRc { .. } => false,

        Call { func, .. } => {
            let purity = match function.dfg[*func] {
                Value::Intrinsic(intrinsic) => Some(intrinsic.purity()),
                Value::Function(id) => purities.get(&id).copied(),
                _ => None,
            };
            match purity {
                Some(Purity::Pure) => true,
                Some(Purity::PureWithPredicate) => hoist_with_predicate,
                Some(Purity::Impure) => false,
                None => false,
            }
        }

        // These instructions can always be hoisted
        Cast(_, NumericType::NativeField) | Not(_) | Truncate { .. } | IfElse { .. } => true,

        // A cast may have dependence on a range-check, which may not be hoisted, so we cannot always hoist a cast.
        Cast(_, _) | Constrain(..) | ConstrainNotEqual(..) | RangeCheck { .. } => {
            hoist_with_predicate
        }

        // Noop instructions can always be hoisted, although they're more likely to be
        // removed entirely.
        Noop => true,

        // Arrays can be mutated in unconstrained code so code that handles this case must
        // take care to track whether the array was possibly mutated or not before
        // hoisted. Since we don't know if the containing pass checks for this, we
        // can only assume these are safe to hoist in constrained code.
        MakeArray { .. } => function.runtime().is_acir(),

        // These can have different behavior depending on the predicate.
        Binary(_) | ArrayGet { .. } | ArraySet { .. } => {
            hoist_with_predicate
                || !instruction.requires_acir_gen_predicate(&function.dfg, purities)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ssa::Ssa;
    use crate::ssa::opt::assert_normalized_ssa_equals;

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
        let expected = "
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
        ";

        let ssa = ssa.loop_invariant_code_motion();
        assert_normalized_ssa_equals(ssa, expected);
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
        let expected = "
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
        ";

        let ssa = ssa.loop_invariant_code_motion();
        assert_normalized_ssa_equals(ssa, expected);
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

        let expected = "
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
        ";

        let ssa = ssa.loop_invariant_code_motion();
        assert_normalized_ssa_equals(ssa, expected);
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

        let expected = "
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
        ";

        let ssa = ssa.loop_invariant_code_motion();
        assert_normalized_ssa_equals(ssa, expected);
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
        let expected = "
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
        ";

        let ssa = ssa.loop_invariant_code_motion();
        assert_normalized_ssa_equals(ssa, expected);
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
        let expected = "
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
        ";

        let ssa = ssa.loop_invariant_code_motion();
        assert_normalized_ssa_equals(ssa, expected);
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

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();
        assert_normalized_ssa_equals(ssa, src);
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
        let expected = "
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
        ";

        let ssa = ssa.loop_invariant_code_motion();
        assert_normalized_ssa_equals(ssa, expected);
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

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.loop_invariant_code_motion();
        assert_normalized_ssa_equals(ssa, src);
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
        let expected = "
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
        ";

        assert_normalized_ssa_equals(ssa, expected);
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

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn do_not_hoist_from_non_executed_nested_loop() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            jmp b1(u128 2)
          b1(v0: u128):
            v4 = lt v0, u128 5
            jmpif v4 then: b2, else: b3
          b2():
            v6 = mod v0, u128 9
            jmp b4(u128 4)
          b3():
            return
          b4(v1: u128):
            v8 = lt v1, v6
            jmpif v8 then: b5, else: b6
          b5():
            v12 = lt u128 0, v0
            constrain v12 == u1 0
            v14 = unchecked_add v1, u128 1
            jmp b4(v14)
          b6():
            v10 = unchecked_add v0, u128 1
            jmp b1(v10)
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();

        // We expect that the constraint will be turned into an always-fail one,
        // but not be hoisted into the pre-header.
        let expected = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            jmp b1(u128 2)
          b1(v0: u128):
            v4 = lt v0, u128 5
            jmpif v4 then: b2, else: b3
          b2():
            v6 = mod v0, u128 9
            jmp b4(u128 4)
          b3():
            return
          b4(v1: u128):
            v8 = lt v1, v6
            jmpif v8 then: b5, else: b6
          b5():
            constrain u1 1 == u1 0
            v14 = unchecked_add v1, u128 1
            jmp b4(v14)
          b6():
            v10 = unchecked_add v0, u128 1
            jmp b1(v10)
        }
        "#;
        assert_normalized_ssa_equals(ssa, expected);
    }
}

#[cfg(test)]
mod control_dependence {
    use crate::{
        assert_ssa_snapshot,
        ssa::{
            interpreter::{errors::InterpreterError, tests::from_constant},
            ir::types::NumericType,
            opt::assert_normalized_ssa_equals,
            ssa_gen::Ssa,
        },
    };

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

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.loop_invariant_code_motion();
        assert_normalized_ssa_equals(ssa, src);
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
            call f1()
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
            call f1()
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
            call f1()
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
            call f1()
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
        // This test shows the simplification of the constraint constrain v17 == u1 1 which is converted into constrain u1 0 == u1 1 in b5
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
        // The loop is guaranteed to fully execute, so we expect the constrain to be simplified into constrain u1 0 == u1 1
        // However, even though the constrain is not a loop invariant we expect it to remain in place
        // as it is control dependent upon its predecessor blocks which are not pure.
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
            constrain u1 0 == u1 1
            v17 = unchecked_add v3, u32 1
            jmp b1(v17)
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

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.loop_invariant_code_motion();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u32, v1: u32, v2: u32):
            v4 = allocate -> &mut u32
            store v0 at v4
            v6 = eq v1, u32 4
            jmp b1(u32 0)
          b1(v3: u32):
            v9 = lt v3, u32 5
            jmpif v9 then: b2, else: b3
          b2():
            jmpif u1 1 then: b4, else: b5
          b3():
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
            jmpif v16 then: b3, else: b6
          b6():
            v17 = lt v3, u32 4
            constrain v17 == u1 1
            v18 = unchecked_add v3, u32 1
            jmp b1(v18)
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

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.loop_invariant_code_motion();

        // We expect the SSA to be unchanged
        assert_normalized_ssa_equals(ssa, src);
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
    v33 = unchecked_add v2, u32 1
    jmp b1(v33)
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
            v5 = cast v1 as Field
            v7 = sub Field 256, v5
            jmp b1(u32 0)
          b1(v2: u32):
            v9 = eq v2, u32 0
            jmpif v9 then: b2, else: b3
          b2():
            jmpif v0 then: b4, else: b5
          b3():
            return i16 3
          b4():
            v11 = cast v4 as u16
            v12 = truncate v11 to 8 bits, max_bit_size: 16
            v13 = cast v1 as u8
            v15 = lt v13, u8 128
            v16 = not v15
            v17 = cast v15 as Field
            v18 = mul v17, v5
            v19 = cast v16 as Field
            v20 = mul v19, v7
            v21 = add v18, v20
            v23 = mul v21, Field 127
            range_check v23 to 8 bits
            v24 = cast v23 as u8
            v25 = not v15
            v26 = cast v25 as u8
            v27 = unchecked_add u8 128, v26
            v28 = lt v24, v27
            constrain v28 == u1 1
            v30 = cast v12 as i8
            jmp b5()
          b5():
            v32 = unchecked_add v2, u32 1
            jmp b1(v32)
        }
        ");
    }
}
