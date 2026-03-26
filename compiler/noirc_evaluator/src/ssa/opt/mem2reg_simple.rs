//! Mem2reg algorithm adapted from the paper: <https://bernsteinbear.com/assets/img/bebenita-ssa.pdf>
//!
//! The goal is for this new, simpler to eventually replace our existing mem2reg algorithm in `ssa/opt/mem2reg.rs`.
//! The pre-existing pass however can optimize in more/different cases than this pass. For example,
//! it can still optimize out stores/loads in some cases even when the reference is aliased. That
//! other pass has a larger surface area for bugs though and this one is simpler so the goal is to
//! replace the old pass with this one plus any other, separate passes needed for the features
//! unhandled here (such as alias analysis).
use iter_extended::{btree_map, vecmap};
use rustc_hash::FxHashSet as HashSet;
use std::collections::BTreeMap;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        cfg::ControlFlowGraph,
        dfg::DataFlowGraph,
        dom::DominatorTree,
        function::Function,
        function_inserter::FunctionInserter,
        instruction::{Instruction, TerminatorInstruction},
        post_order::PostOrder,
        value::ValueId,
    },
    ssa_gen::Ssa,
};

/// Arbitrary limit for maximum variables optimized by this pass in each function.
///
/// This is because this pass can lead to regressions in certain cases (e.g. the hashmap test)
/// where variables are modified in inner loops but not outer ones, yet the outer loops would need
/// to pay for passing around the variables while with the `Load` approach, only the inner loops
/// paid previously.
const MAX_VARIABLES_OPTIMIZED: u32 = 10;

/// Maximum number of blocks a variable's declaration can dominate before we skip
/// promoting it in the pre-flattening pass.
///
/// The cost of promoting a variable before flattening is O(promoted_variables × dominated_blocks)
/// because each promoted variable adds a block parameter to every dominated block, and the
/// flattener converts each conditional block into ~5 predicate opcodes (not, mul,
/// enable_side_effects, etc.). A variable that spans many blocks (e.g. a byte in a 254-iteration
/// unrolled loop) can generate thousands of extra ACIR opcodes.
///
/// This limit filters out variables whose declaration dominates too many blocks,
/// keeping promotion beneficial for small CFGs (like if/else diamonds in `conditional_1`)
/// while avoiding regressions in deeply unrolled code (like `to_bytes_integration`).
const MAX_BLOCK_SPAN_PRE_FLATTENING: usize = 100;

impl Ssa {
    /// Run mem2reg_simple on all functions (both ACIR and Brillig).
    ///
    /// ACIR functions have no variable limit since they benefit more from full promotion.
    /// Brillig keeps the limit to avoid regressions in loop-heavy code.
    ///
    /// **Important:** This should only be used after flattening for ACIR functions.
    /// Before flattening, use `mem2reg_simple_pre_flattening` instead to avoid
    /// regressions from promoting variables that span too many blocks.
    pub(crate) fn mem2reg_simple(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            let max_vars =
                if function.runtime().is_brillig() { Some(MAX_VARIABLES_OPTIMIZED) } else { None };
            function.mem2reg_simple(max_vars, None);
        }
        self
    }

    /// Run mem2reg_simple only on Brillig functions.
    pub(crate) fn mem2reg_simple_brillig(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            if function.runtime().is_brillig() {
                function.mem2reg_simple(Some(MAX_VARIABLES_OPTIMIZED), None);
            }
        }
        self
    }

    /// Run mem2reg_simple on all functions before flattening.
    ///
    /// Brillig functions use the standard variable limit. ACIR functions use both
    /// a variable limit and a block span limit to avoid regressions: promoting a
    /// variable whose declaration dominates many blocks (e.g. across an unrolled loop)
    /// generates O(variables × blocks) extra predicate opcodes after flattening.
    pub(crate) fn mem2reg_simple_pre_flattening(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            if function.runtime().is_brillig() {
                function.mem2reg_simple(Some(MAX_VARIABLES_OPTIMIZED), None);
            } else {
                function.mem2reg_simple(
                    Some(MAX_VARIABLES_OPTIMIZED),
                    Some(MAX_BLOCK_SPAN_PRE_FLATTENING),
                );
            }
        }
        self
    }
}

impl Function {
    fn mem2reg_simple(&mut self, max_variables: Option<u32>, max_block_span: Option<usize>) {
        let cfg = ControlFlowGraph::with_function(self);
        let post_order = PostOrder::with_cfg(&cfg);
        let mut dom_tree = DominatorTree::with_cfg_and_post_order(&cfg, &post_order);
        let mut inserter = FunctionInserter::new(self);

        let blocks = post_order.into_vec_reverse();

        // Note that `variables` and `entry_values` in variable_states are all keyed by the original
        // ValueId of the `allocate` instruction result. These are all iterated over at some point
        // so it is important we use a deterministic order so that block arguments always correspond
        // to block parameters in the same order.
        let mut variables = collect_all_eligible_variables(inserter.function, &blocks);

        // Filter out variables whose declaration dominates too many blocks.
        // Each promoted variable adds a block parameter to every dominated block, and the
        // flattener converts each conditional into predicate opcodes, so the cost is
        // O(promoted_variables × dominated_blocks).
        //
        // We approximate this count by precomputing dominator-tree subtree
        // sizes in O(blocks): `blocks` is in RPO order, so iterating in reverse guarantees
        // each block is visited before its immediate dominator (dominators always have a
        // lower RPO index). One reverse pass accumulates subtree sizes bottom-up.
        if let Some(max_span) = max_block_span
            && blocks.len() > max_span
            && !variables.is_empty()
        {
            // Initialize each block's dom count to 1
            let mut subtree_size = btree_map(&blocks, |block| (*block, 1));

            for &block in blocks.iter().rev() {
                if let Some(idom) = dom_tree.immediate_dominator(block) {
                    let size = subtree_size[&block];
                    *subtree_size.entry(idom).or_insert(1) += size;
                }
            }
            variables.retain(|_var, decl_block| subtree_size[decl_block] <= max_span);
        }

        // Limit increase in memory usage and brillig regressions by arbitrarily limiting this pass to some variables
        if let Some(max) = max_variables {
            variables = variables.into_iter().take(max as usize).collect();
        }
        if variables.is_empty() {
            return;
        }

        let mut block_states = BlockStates::default();
        add_block_params_and_find_exit_states(
            &blocks,
            &variables,
            &mut dom_tree,
            &mut inserter,
            &mut block_states,
        );
        add_terminator_arguments(&blocks, &variables, &mut inserter, &block_states, &cfg);
        remove_params_from_blocks_with_identical_terminator_args(&blocks, &mut inserter, &cfg);
        commit(&mut inserter, &variables, blocks);
    }
}

/// Contains the starting & ending values of each variable in each block
#[derive(Default)]
struct BlockStates {
    blocks: BTreeMap<BasicBlockId, BlockState>,
}

/// Contains the starting & ending values of each variable in one block
#[derive(Default)]
struct BlockState {
    /// Maps each variable visible in this block to its starting value in the block. This is always
    /// a block parameter or a forwarded value from a previous block.
    entry_state: BTreeMap<ValueId, ValueId>,

    /// Maps each variable modified within this block to the value it is set to at the end of
    /// the block. Note that to save on memory, we do not store variables which were not modified
    /// within the block. Their values will be the same as the value in `entry_states`.
    exit_state: BTreeMap<ValueId, ValueId>,
}

/// Find the starting & ending states of each variable in each block.
///
/// This will add a block parameter for every variable in `variables` that
/// is alive in each block. This parameter will always be the entry state
/// of that variable, while the exit state will be empty (variable was not changed)
/// or filled with the most recent Store value to the variable in the block.
fn add_block_params_and_find_exit_states(
    blocks: &[BasicBlockId],
    variables: &BTreeMap<ValueId, BasicBlockId>,
    dom_tree: &mut DominatorTree,
    inserter: &mut FunctionInserter,
    variable_states: &mut BlockStates,
) {
    for block in blocks.iter().copied() {
        // All variables visible at the start of the current block
        let entry_state = add_visible_variables_as_block_parameters(
            variables,
            dom_tree,
            block,
            &mut inserter.function.dfg,
        );
        let exit_state = abstract_interpret_block(inserter, block, &entry_state);
        variable_states.blocks.insert(block, BlockState { entry_state, exit_state });
    }
}

/// Link entry & exit states by adding terminator arguments for every variable stored to.
fn add_terminator_arguments(
    blocks: &[BasicBlockId],
    variables: &BTreeMap<ValueId, BasicBlockId>,
    inserter: &mut FunctionInserter,
    variable_states: &BlockStates,
    cfg: &ControlFlowGraph,
) {
    for block in blocks.iter().copied() {
        let block_state = &variable_states.blocks[&block];

        for predecessor in cfg.predecessors(block) {
            let pred_state = &variable_states.blocks[&predecessor];
            let args = get_terminator_args_mut(&mut inserter.function.dfg, predecessor, block);
            for address in block_state.entry_state.keys() {
                // If the current block is this variable's source block, no merge is needed.
                if block != variables[address] {
                    args.push(pred_state.get_exit_value(*address));
                }
            }
        }
    }
}

/// For every block, remove any block parameters whose arguments (from predecessors' terminators) are all identical
fn remove_params_from_blocks_with_identical_terminator_args(
    blocks: &[BasicBlockId],
    inserter: &mut FunctionInserter,
    cfg: &ControlFlowGraph,
) {
    // Sort blocks such that we remove parameters from blocks with multiple predecessors last.
    // This helps remove some dependency on block ordering for optimizations. E.g. the
    // `read_only_loop` test requires 2 passes of mem2reg_simple without this.
    let mut blocks = blocks.to_vec();
    blocks.sort_unstable_by_key(|block| cfg.predecessors(*block).len());

    for block in blocks {
        remove_params_from_block_with_identical_terminator_args(block, inserter, cfg);
    }
}

/// Removes block parameters whose arguments (from predecessors' terminators) are all identical
fn remove_params_from_block_with_identical_terminator_args(
    block: BasicBlockId,
    inserter: &mut FunctionInserter,
    cfg: &ControlFlowGraph,
) {
    let parameters = inserter.function.dfg.block_parameters(block).to_vec();

    // Mask of whether each parameter has non-identical arguments,
    // E.g. if parameter 2's arguments are all identical, then `mask[2]` would be false
    let mask = keep_argument_mask(inserter, cfg, block, &parameters);

    // Remove unneeded parameters from the block
    retain_items_from_mask(inserter.function.dfg[block].parameters_mut(), &mask);

    // And remove the corresponding parameter's arguments from each predecessor
    for predecessor in cfg.predecessors(block) {
        let arguments = get_terminator_args_mut(&mut inserter.function.dfg, predecessor, block);
        retain_items_from_mask(arguments, &mask);
    }
}

/// Mapping from a variable to its value at a point in time.
type StateVec = BTreeMap<ValueId, ValueId>;

/// Filter `variables`, returning only those that are visible at the start of the given block.
/// Since we do not consider variables within a block, the visible variables at the start of a block
/// are any variables whose declaration block dominates the given block.
fn add_visible_variables_as_block_parameters(
    variables: &BTreeMap<ValueId, BasicBlockId>,
    dom_tree: &mut DominatorTree,
    block: BasicBlockId,
    dfg: &mut DataFlowGraph,
) -> StateVec {
    variables
        .iter()
        .filter_map(|(var, decl_block)| {
            if dom_tree.dominates(*decl_block, block) {
                let typ = dfg
                    .type_of_value(*var)
                    .reference_element_type()
                    .expect("All variables should be references")
                    .clone();

                let new_value =
                    if block == *decl_block { *var } else { dfg.add_block_parameter(block, typ) };
                Some((*var, new_value))
            } else {
                None
            }
        })
        .collect()
}

impl BlockState {
    /// Gets the "exit value" of a variable, which is its value at the end of the given block.
    ///
    /// `variable` should always be a reference.
    fn get_exit_value(&self, variable: ValueId) -> ValueId {
        *self
            .exit_state
            .get(&variable)
            // To save memory, `exit_states` only contains the value if it changed within the block.
            // So we have to check `entry_states` for the value if it went unchanged.
            .unwrap_or_else(|| &self.entry_state[&variable])
    }
}

/// Return a mask indicating whether to keep or remove the corresponding parameter.
///
/// For a given parameter at index i, index i of the resulting mask is false (indicating to remove the parameter)
/// if all arguments passed to that parameter from predecessor blocks are identical.
///
/// Each parameter that should be removed will be mapped to its single argument's value.
fn keep_argument_mask(
    inserter: &mut FunctionInserter,
    cfg: &ControlFlowGraph,
    block: BasicBlockId,
    parameters: &[ValueId],
) -> Vec<bool> {
    let entry = inserter.function.entry_block();
    if block == entry {
        // Entry block has no predecessors, so keep all parameters
        return vec![true; parameters.len()];
    }

    vecmap(parameters.iter().enumerate(), |(i, parameter)| {
        let mut args = cfg
            .predecessors(block)
            .map(|predecessor| get_terminator_args(&inserter.function.dfg, predecessor, block)[i])
            .map(|arg| inserter.resolve(arg))
            // Filtering here optimizes away cases where we have the parameter changed in one block
            // and unchanged in another (argument can only equal parameter in the case of back-edges)
            .filter(|arg| arg != parameter);

        let first_arg = args
            .next()
            .expect("Entry block is excluded so there should always be >= 1 predecessor");

        // keep the parameter if the arguments do not all match
        // unwrap safety: for all multi-predecessor blocks, each predecessor should end in a jmp, not jmpif
        let keep_param = !args.all(|arg| arg == first_arg);
        if !keep_param {
            // All arguments are identical, so the choice to map to the first is arbitrary
            // Do not `inserter.resolve()` this parameter! We want the original block parameter, but it
            // may already be mapped away in the inserter.
            inserter.map_value(*parameter, first_arg);
        }
        keep_param
    })
}

/// Get the terminator arguments for block `block` jumping to block `jmp_target`.
/// The `jmp_target` is relevant if `block` terminates in a jmpif terminator and may jmp to
/// multiple blocks. Panics if the given block does not have block arguments.
fn get_terminator_args(
    dfg: &DataFlowGraph,
    block: BasicBlockId,
    jmp_target: BasicBlockId,
) -> &[ValueId] {
    match dfg[block].unwrap_terminator() {
        TerminatorInstruction::Jmp { arguments, .. } => arguments,
        TerminatorInstruction::JmpIf {
            then_destination, then_arguments, else_arguments, ..
        } => {
            if jmp_target == *then_destination {
                then_arguments
            } else {
                else_arguments
            }
        }
        TerminatorInstruction::Return { .. } | TerminatorInstruction::Unreachable { .. } => panic!(
            "get_terminator_args called on block edge {block} -> {jmp_target} but {block} does not have any arguments"
        ),
    }
}

/// Get the terminator arguments for block `block` jumping to block `jmp_target`.
/// The `jmp_target` is relevant if `block` terminates in a jmpif terminator and may jmp to
/// multiple blocks. Panics if the given block does not have block arguments.
fn get_terminator_args_mut(
    dfg: &mut DataFlowGraph,
    block: BasicBlockId,
    jmp_target: BasicBlockId,
) -> &mut Vec<ValueId> {
    match dfg[block].unwrap_terminator_mut() {
        TerminatorInstruction::Jmp { arguments, .. } => arguments,
        TerminatorInstruction::JmpIf {
            then_destination, then_arguments, else_arguments, ..
        } => {
            if jmp_target == *then_destination {
                then_arguments
            } else {
                else_arguments
            }
        }
        TerminatorInstruction::Return { .. } | TerminatorInstruction::Unreachable { .. } => panic!(
            "get_terminator_args called on block edge {block} -> {jmp_target} but {block} does not have any arguments"
        ),
    }
}

/// For each index i of `items`, keep `items[i]` iff `mask[i]`
fn retain_items_from_mask(items: &mut Vec<ValueId>, mask: &[bool]) {
    debug_assert_eq!(items.len(), mask.len());
    let mut mask_iter = mask.iter();
    items.retain(|_| *mask_iter.next().unwrap());
    // Reclaim some memory, important in larger programs
    items.shrink_to_fit();
}

/// Abstractly interpret a block, collecting the value of each reference at the end of a block.
/// Any references not included in the result are assumed to be unchanged.
///
/// This function is very simple and will produce incorrect results for first-class references
/// that are aliased, passed to functions, or stored in arrays, etc.
///
/// Note that this function will also replace any instances of the reference being stored to
/// with its current value in the block. Most of the time, this locally renames the reference
/// in the block so that we can map its value in the block independent of other blocks.
fn abstract_interpret_block(
    inserter: &mut FunctionInserter,
    block: BasicBlockId,
    entry_state: &StateVec,
) -> StateVec {
    // Any variables not in the exit_state by function end are assumed to be unchanged from the entry_state
    let mut exit_state = StateVec::new();
    let instructions = inserter.function.dfg[block].take_instructions();

    for instruction_id in &instructions {
        match &inserter.function.dfg[*instruction_id] {
            Instruction::Store { address, value } => {
                // Only update the value if the address is already in the entry state, since only
                // those addresses are eligible for mem2reg optimization. If the address is
                // in the state, we remove it.
                if entry_state.contains_key(address) {
                    exit_state.insert(*address, *value);
                }
            }
            Instruction::Load { address } => {
                if let Some(value) = exit_state.get(address).or_else(|| entry_state.get(address)) {
                    let result = inserter.function.dfg.instruction_results(*instruction_id)[0];
                    inserter.map_value(result, *value);
                }
            }
            _ => (),
        }
    }

    *inserter.function.dfg[block].instructions_mut() = instructions;
    exit_state
}

/// Return a map from each variable to the block it was declared in.
/// This only includes variables that are eligible for mem2reg optimization,
/// i.e. those that are allocated but never used in a first-class manner.
fn collect_all_eligible_variables(
    function: &Function,
    blocks: &[BasicBlockId],
) -> BTreeMap<ValueId, BasicBlockId> {
    // Map each variable to the block it was declared in
    let mut variables = BTreeMap::default();

    // Workaround for https://github.com/noir-lang/noir/issues/11482
    // If the declaration block of an allocate has no starting store then it isn't eligible for mem2reg_simple.
    let mut variables_with_stores_in_decl_block = HashSet::default();

    for block_id in blocks.iter().copied() {
        let block = &function.dfg[block_id];
        for instruction_id in block.instructions() {
            let instruction = &function.dfg[*instruction_id];
            match instruction {
                Instruction::Allocate => {
                    let address = function.dfg.instruction_results(*instruction_id)[0];
                    variables.insert(address, block_id);
                }
                Instruction::Load { .. } => (),
                // Storing to an address is fine, but storing an address prevents optimizing it out.
                Instruction::Store { address, value } => {
                    variables.remove(value);

                    if variables.get(address) == Some(&block_id) {
                        variables_with_stores_in_decl_block.insert(*address);
                    }
                }
                // Any other use of an address (in arrays, functions, etc) is also first-class and prevents optimization.
                _ => instruction.for_each_value(|value| variables.remove(&value)),
            }
        }

        block.unwrap_terminator().for_each_value(|value| variables.remove(&value));
    }

    variables.retain(|address, _| variables_with_stores_in_decl_block.contains(address));
    variables
}

/// Commit to all changes made by the pass:
/// - Map any values mapped from the inserter to their new values in the function
/// - Remove all Allocate, Load, and Store instructions from the eligible variables
fn commit(
    inserter: &mut FunctionInserter,
    variables: &BTreeMap<ValueId, BasicBlockId>,
    blocks: Vec<BasicBlockId>,
) {
    for block in blocks {
        let mut instructions = inserter.function.dfg[block].take_instructions();

        // Remove any allocate, load, or store instructions for variables which were optimized out
        instructions.retain(|instruction_id| {
            let instruction = &inserter.function.dfg[*instruction_id];
            let keep = match instruction {
                Instruction::Allocate => {
                    let address = inserter.function.dfg.instruction_results(*instruction_id)[0];
                    !variables.contains_key(&address)
                }
                Instruction::Load { address } | Instruction::Store { address, value: _ } => {
                    !variables.contains_key(address)
                }
                _ => true,
            };

            if keep {
                inserter.map_instruction_in_place(*instruction_id);
            }
            keep
        });

        *inserter.function.dfg[block].instructions_mut() = instructions;

        let mut terminator = inserter.function.dfg[block].take_terminator();
        terminator.map_values_mut(|value| inserter.resolve(value));
        inserter.function.dfg[block].set_terminator(terminator);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_ssa_snapshot,
        ssa::{opt::assert_ssa_does_not_change, ssa_gen::Ssa},
    };

    #[test]
    fn test_simple() {
        let src = "
        brillig(inline) fn func f0 {
          b0():
            v0 = allocate -> &mut [Field; 2]
            v3 = make_array [Field 1, Field 1] : [Field; 2]
            store v3 at v0
            v4 = load v0 -> [Field; 2]
            v5 = array_get v4, index u32 1 -> Field
            return v5
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg_simple();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn func f0 {
          b0():
            v1 = make_array [Field 1, Field 1] : [Field; 2]
            v3 = array_get v1, index u32 1 -> Field
            return v3
        }
        ");
    }

    #[test]
    fn test_multi_block() {
        let src = "
        brillig(inline) fn func f0 {
          b0(v0: u1):
            v1 = allocate -> &mut Field
            store Field 0 at v1
            jmpif v0 then: b1(), else: b2()
          b1():
            store Field 1 at v1
            jmp b3()
          b2():
            store Field 2 at v1
            jmp b3()
          b3():
            v4 = load v1 -> Field
            return v4
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg_simple();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn func f0 {
          b0(v0: u1):
            jmpif v0 then: b1(), else: b2()
          b1():
            jmp b3(Field 1)
          b2():
            jmp b3(Field 2)
          b3(v1: Field):
            return v1
        }
        ");
    }

    #[test]
    fn test_single_predecessor_elimination() {
        let src = "
            brillig(inline) fn func f0 {
              b0():
                v1 = allocate -> &mut Field
                store Field 7 at v1
                jmp b1()
              b1():
                v2 = load v1 -> Field
                return v2
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg_simple();
        // Expect the allocate/load/store to be removed and the constant propagated to the return.
        assert_ssa_snapshot!(ssa, @r"
            brillig(inline) fn func f0 {
              b0():
                jmp b1()
              b1():
                return Field 7
            }
            ");
    }

    #[test]
    fn test_identical_branch_arguments_removed() {
        let src = "
            brillig(inline) fn func f0 {
              b0(v0: u1):
                v1 = allocate -> &mut Field
                store Field 0 at v1
                jmpif v0 then: b1(), else: b2()
              b1():
                store Field 1 at v1
                jmp b3()
              b2():
                store Field 1 at v1
                jmp b3()
              b3():
                v4 = load v1 -> Field
                return v4
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg_simple();
        // Both predecessors pass the same value, so the parameter should be removed and the value folded.
        assert_ssa_snapshot!(ssa, @r"
            brillig(inline) fn func f0 {
              b0(v0: u1):
                jmpif v0 then: b1(), else: b2()
              b1():
                jmp b3()
              b2():
                jmp b3()
              b3():
                return Field 1
            }
            ");
    }

    #[test]
    fn test_multiple_stores_same_block() {
        let src = "
            brillig(inline) fn func f0 {
              b0():
                v0 = allocate -> &mut Field
                store Field 5 at v0
                v1 = load v0 -> Field
                store Field 10 at v0
                v2 = load v0 -> Field
                v3 = add v1, v2
                return v3
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg_simple();
        assert_ssa_snapshot!(ssa, @r"
            brillig(inline) fn func f0 {
              b0():
                v2 = add Field 5, Field 10
                return v2
            }
            ");
    }

    #[test]
    fn test_three_way_merge() {
        let src = "
            brillig(inline) fn func f0 {
              b0(v0: u1, v1: u1):
                v2 = allocate -> &mut Field
                store Field 1 at v2
                jmpif v0 then: b1(), else: b2()
              b1():
                store Field 2 at v2
                jmp b5()
              b2():
                jmpif v1 then: b3(), else: b4()
              b3():
                store Field 3 at v2
                jmp b5()
              b4():
                jmp b5()
              b5():
                v3 = load v2 -> Field
                return v3
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg_simple();
        // Should handle merging from three different paths
        assert_ssa_snapshot!(ssa, @r"
            brillig(inline) fn func f0 {
              b0(v0: u1, v1: u1):
                jmpif v0 then: b1(), else: b2()
              b1():
                jmp b5(Field 2)
              b2():
                jmpif v1 then: b3(), else: b4()
              b3():
                jmp b5(Field 3)
              b4():
                jmp b5(Field 1)
              b5(v2: Field):
                return v2
            }
            ");
    }

    #[test]
    fn test_no_optimization_with_aliasing() {
        // This test ensures we don't try to optimize allocations that might be aliased
        let src = "
            brillig(inline) fn func f0 {
              b0():
                v0 = allocate -> &mut Field
                store Field 10 at v0
                v1 = make_array [v0] : [&mut Field; 1]
                v2 = load v0 -> Field
                v3 = array_get v1, index u32 0 -> &mut Field
                store Field 20 at v3
                v4 = load v0 -> Field
                v5 = add v2, v4
                return v5
            }
            ";
        assert_ssa_does_not_change(src, Ssa::mem2reg_simple);
    }

    #[test]
    fn test_variable_only_stored_in_one_branch() {
        // Variable is stored in one branch but not the other
        let src = "
            brillig(inline) fn func f0 {
              b0(v0: u1):
                v1 = allocate -> &mut Field
                store Field 5 at v1
                jmpif v0 then: b1(), else: b2()
              b1():
                store Field 10 at v1
                jmp b3()
              b2():
                jmp b3()
              b3():
                v2 = load v1 -> Field
                return v2
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg_simple();
        // b2 path should pass the initial value (Field 5), b1 passes (Field 10)
        assert_ssa_snapshot!(ssa, @r"
            brillig(inline) fn func f0 {
              b0(v0: u1):
                jmpif v0 then: b1(), else: b2()
              b1():
                jmp b3(Field 10)
              b2():
                jmp b3(Field 5)
              b3(v1: Field):
                return v1
            }
            ");
    }

    #[test]
    fn test_consecutive_stores_load() {
        // Multiple consecutive stores followed by a single load
        let src = "
            brillig(inline) fn func f0 {
              b0():
                v0 = allocate -> &mut Field
                store Field 1 at v0
                store Field 2 at v0
                store Field 3 at v0
                v1 = load v0 -> Field
                return v1
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg_simple();
        // Only the last store should matter
        assert_ssa_snapshot!(ssa, @r"
            brillig(inline) fn func f0 {
              b0():
                return Field 3
            }
            ");
    }

    #[test]
    fn test_deep_nesting_diamond() {
        // Deeply nested diamond patterns
        let src = "
            brillig(inline) fn func f0 {
              b0(v0: u1, v1: u1):
                v2 = allocate -> &mut Field
                store Field 1 at v2
                jmpif v0 then: b1(), else: b2()
              b1():
                jmpif v1 then: b3(), else: b4()
              b2():
                store Field 2 at v2
                jmp b5()
              b3():
                store Field 3 at v2
                jmp b5()
              b4():
                store Field 4 at v2
                jmp b5()
              b5():
                v3 = load v2 -> Field
                return v3
            }
            ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg_simple();
        // Should properly merge all three stores: from b2, b3, b4
        assert_ssa_snapshot!(ssa, @r"
            brillig(inline) fn func f0 {
              b0(v0: u1, v1: u1):
                jmpif v0 then: b1(), else: b2()
              b1():
                jmpif v1 then: b3(), else: b4()
              b2():
                jmp b5(Field 2)
              b3():
                jmp b5(Field 3)
              b4():
                jmp b5(Field 4)
              b5(v2: Field):
                return v2
            }
            ");
    }

    #[test]
    fn reference_not_found_regression() {
        // v119 in b24 was being improperly registered as valid at the
        // start of b24, and we were trying to look for its value in the predecessor blocks where
        // it doesn't exist, leading to a panic. This regression ensures we do not do that, and
        // gives us a test case with more complex control-flow.
        let src = "
brillig(inline) fn main f0 {
  b0(v0: [u32; 5], v1: [u32; 5], v2: u32, v4: u32):
    v3 = allocate -> &mut u32
    store v2 at v3
    v6 = allocate -> &mut u32
    store u32 2301 at v6
    v9 = array_get v1, index u32 4 -> u32
    store v9 at v3
    jmp b1(u32 0)
  b1(v12: u32):
    v13 = lt v12, u32 5
    jmpif v13 then: b2(), else: b3()
  b2():
    v14 = load v3 -> u32
    v15 = load v3 -> u32
    v16 = mul v14, v15
    v17 = array_get v1, index v12 -> u32
    v18 = mul v16, v17
    store v18 at v6
    v19 = load v3 -> u32
    v20 = load v6 -> u32
    v21 = sub v19, v20
    store v21 at v3
    v22 = unchecked_add v12, u32 1
    jmp b1(v22)
  b3():
    v23 = load v3 -> u32
    v24 = eq v23, u32 0
    constrain v23 == u32 0
    store u32 2301 at v6
    jmp b4(u32 0)
  b4(v27: u32):
    v28 = lt v27, u32 5
    jmpif v28 then: b5(), else: b6()
  b5():
    v31 = add v4, u32 2
    store v31 at v6
    v33 = load v3 -> u32
    v34 = load v3 -> u32
    v36 = array_get v0, index v27 -> u32
    store u32 0 at v6
    v39 = load v3 -> u32
    v41 = array_get v0, index v27 -> u32
    v42 = array_get v1, index v27 -> u32
    v43 = mul v41, v42
    v44 = load v6 -> u32
    store u32 1 at v3
    v47 = unchecked_add v27, u32 1
    jmp b4(v47)
  b6():
    v48 = load v3 -> u32
    v50 = eq v48, u32 3814912846
    constrain v48 == u32 3814912846
    store u32 2300001 at v6
    v53 = array_get v1, index u32 4 -> u32
    store v53 at v3
    jmp b7(u32 0)
  b7(v54: u32):
    v55 = lt v54, u32 5
    jmpif v55 then: b8(), else: b9()
  b8():
    v56 = load v3 -> u32
    v57 = array_get v0, index v54 -> u32
    v58 = array_get v1, index v54 -> u32
    v59 = mul v57, v58
    v60 = add v56, v59
    store v60 at v3
    jmp b10(u32 0)
  b9():
    v70 = load v3 -> u32
    v72 = eq v70, u32 41472
    constrain v70 == u32 41472
    v73 = array_get v1, index u32 4 -> u32
    store v73 at v3
    jmp b13(u32 0)
  b10(v62: u32):
    v63 = lt v62, u32 3
    jmpif v63 then: b11(), else: b12()
  b11():
    store u32 3 at v6
    v65 = load v3 -> u32
    v66 = load v6 -> u32
    store u32 4 at v3
    v68 = unchecked_add v62, u32 1
    jmp b10(v68)
  b12():
    v69 = unchecked_add v54, u32 1
    jmp b7(v69)
  b13(v74: u32):
    v75 = lt v74, u32 3
    jmpif v75 then: b14(), else: b15()
  b14():
    v76 = load v3 -> u32
    v77 = array_get v0, index v74 -> u32
    v78 = array_get v1, index v74 -> u32
    v79 = mul v77, v78
    v80 = add v76, v79
    store v80 at v3
    jmp b16(u32 0)
  b15():
    v92 = load v3 -> u32
    v94 = eq v92, u32 11539
    constrain v92 == u32 11539
    v95 = load v3 -> u32
    v96 = eq v95, u32 0
    jmpif v96 then: b19(), else: b20()
  b16(v81: u32):
    v82 = lt v81, u32 2
    jmpif v82 then: b17(), else: b18()
  b17():
    v83 = load v3 -> u32
    v84 = add v74, v81
    v85 = array_get v0, index v84 -> u32
    v86 = add v74, v81
    v87 = array_get v1, index v86 -> u32
    v88 = sub v85, v87
    v89 = add v83, v88
    store v89 at v3
    v90 = unchecked_add v81, u32 1
    jmp b16(v90)
  b18():
    v91 = unchecked_add v74, u32 1
    jmp b13(v91)
  b19():
    jmp b21(v0)
  b20():
    jmp b21(v1)
  b21(v97: [u32; 5]):
    v98 = array_get v97, index u32 0 -> u32
    v99 = array_get v1, index u32 0 -> u32
    v100 = eq v98, v99
    constrain v98 == v99
    jmp b22(u32 0)
  b22(v102: u32):
    v103 = lt v102, u32 5
    jmpif v103 then: b23(), else: b24()
  b23():
    v104 = array_get v1, index v102 -> u32
    jmp b25(u32 0)
  b24():
    v118 = make_array [Field 1, Field 2, Field 3, Field 4, Field 5, Field 6] : [(Field, Field); 3]
    v119 = allocate -> &mut [(Field, Field); 3]
    store v118 at v119
    v122 = load v119 -> [(Field, Field); 3]
    v123 = array_set v122, index u32 2, value Field 7
    v124 = array_set v123, index u32 3, value Field 8
    store v124 at v119
    v125 = load v119 -> [(Field, Field); 3]
    v126 = array_get v125, index u32 2 -> Field
    v127 = array_get v125, index u32 3 -> Field
    v128 = eq v127, Field 8
    constrain v127 == Field 8
    return
  b25(v105: u32):
    v106 = lt v105, u32 5
    jmpif v106 then: b26(), else: b27()
  b26():
    v107 = array_get v0, index v105 -> u32
    v108 = eq v107, v104
    v109 = not v108
    constrain v108 == u1 0
    v111 = unchecked_add v105, u32 1
    jmp b25(v111)
  b27():
    v112 = unchecked_add v102, u32 1
    jmp b22(v112)
}
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg_simple();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: [u32; 5], v1: [u32; 5], v2: u32, v3: u32):
            v27 = array_get v1, index u32 4 -> u32
            jmp b1(u32 0, v27, u32 2301)
          b1(v4: u32, v5: u32, v6: u32):
            v31 = lt v4, u32 5
            jmpif v31 then: b2(), else: b3()
          b2():
            v101 = mul v5, v5
            v102 = array_get v1, index v4 -> u32
            v103 = mul v101, v102
            v104 = sub v5, v103
            v105 = unchecked_add v4, u32 1
            jmp b1(v105, v104, v103)
          b3():
            v32 = eq v5, u32 0
            constrain v5 == u32 0
            jmp b4(u32 0, v5, u32 2301)
          b4(v7: u32, v8: u32, v9: u32):
            v33 = lt v7, u32 5
            jmpif v33 then: b5(), else: b6()
          b5():
            v95 = add v3, u32 2
            v96 = array_get v0, index v7 -> u32
            v97 = array_get v0, index v7 -> u32
            v98 = array_get v1, index v7 -> u32
            v99 = mul v97, v98
            v100 = unchecked_add v7, u32 1
            jmp b4(v100, u32 1, u32 0)
          b6():
            v35 = eq v8, u32 3814912846
            constrain v8 == u32 3814912846
            v36 = array_get v1, index u32 4 -> u32
            jmp b7(u32 0, v36, u32 2300001)
          b7(v10: u32, v11: u32, v12: u32):
            v38 = lt v10, u32 5
            jmpif v38 then: b8(), else: b9()
          b8():
            v88 = array_get v0, index v10 -> u32
            v89 = array_get v1, index v10 -> u32
            v90 = mul v88, v89
            v91 = add v11, v90
            jmp b10(u32 0, v91, v12)
          b9():
            v40 = eq v11, u32 41472
            constrain v11 == u32 41472
            v41 = array_get v1, index u32 4 -> u32
            jmp b13(u32 0, v41, v12)
          b10(v13: u32, v14: u32, v15: u32):
            v92 = lt v13, u32 3
            jmpif v92 then: b11(), else: b12()
          b11():
            v94 = unchecked_add v13, u32 1
            jmp b10(v94, u32 4, u32 3)
          b12():
            v93 = unchecked_add v10, u32 1
            jmp b7(v93, v14, v15)
          b13(v16: u32, v17: u32, v18: u32):
            v43 = lt v16, u32 3
            jmpif v43 then: b14(), else: b15()
          b14():
            v75 = array_get v0, index v16 -> u32
            v76 = array_get v1, index v16 -> u32
            v77 = mul v75, v76
            v78 = add v17, v77
            jmp b16(u32 0, v78)
          b15():
            v45 = eq v17, u32 11539
            constrain v17 == u32 11539
            v46 = eq v17, u32 0
            jmpif v46 then: b19(), else: b20()
          b16(v19: u32, v20: u32):
            v79 = lt v19, u32 2
            jmpif v79 then: b17(), else: b18()
          b17():
            v81 = add v16, v19
            v82 = array_get v0, index v81 -> u32
            v83 = add v16, v19
            v84 = array_get v1, index v83 -> u32
            v85 = sub v82, v84
            v86 = add v20, v85
            v87 = unchecked_add v19, u32 1
            jmp b16(v87, v86)
          b18():
            v80 = unchecked_add v16, u32 1
            jmp b13(v80, v20, v18)
          b19():
            jmp b21(v0)
          b20():
            jmp b21(v1)
          b21(v21: [u32; 5]):
            v47 = array_get v21, index u32 0 -> u32
            v48 = array_get v1, index u32 0 -> u32
            v49 = eq v47, v48
            constrain v47 == v48
            jmp b22(u32 0, v17, v18)
          b22(v22: u32, v23: u32, v24: u32):
            v50 = lt v22, u32 5
            jmpif v50 then: b23(), else: b24()
          b23():
            v66 = array_get v1, index v22 -> u32
            jmp b25(u32 0)
          b24():
            v57 = make_array [Field 1, Field 2, Field 3, Field 4, Field 5, Field 6] : [(Field, Field); 3]
            v60 = array_set v57, index u32 2, value Field 7
            v62 = array_set v60, index u32 3, value Field 8
            v63 = array_get v62, index u32 2 -> Field
            v64 = array_get v62, index u32 3 -> Field
            v65 = eq v64, Field 8
            constrain v64 == Field 8
            return
          b25(v25: u32):
            v67 = lt v25, u32 5
            jmpif v67 then: b26(), else: b27()
          b26():
            v70 = array_get v0, index v25 -> u32
            v71 = eq v70, v66
            v72 = not v71
            constrain v71 == u1 0
            v74 = unchecked_add v25, u32 1
            jmp b25(v74)
          b27():
            v69 = unchecked_add v22, u32 1
            jmp b22(v69, v23, v24)
        }
        ");
    }

    #[test]
    fn add_arg_to_jmpif_block_regression() {
        // Before JmpIf arguments were added, we used to require a separate pass
        // (Ssa::process_cfg_for_mem2reg_simple) for converting the SSA into a form where we
        // didn't need to add jmpif arguments. This test was erroring in a corner case of that
        // analysis previously. Now, we do have jmpif arguments and don't need a preprocessing
        // step but this test is kept anyway.
        let src = "
            brillig(inline) fn to_le_bits f19 {
              b0(v0: Field):
                v2 = call to_le_bits(v0) -> [u1; 32]
                v7 = make_array [u1 1, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 1, u1 1, u1 1, u1 1, u1 1, u1 1, u1 0, u1 0, u1 1, u1 0, u1 0, u1 1, u1 1, u1 0, u1 1, u1 0, u1 1, u1 1, u1 1, u1 1, u1 1, u1 0, u1 0, u1 0, u1 0, u1 1, u1 1, u1 1, u1 1, u1 1, u1 0, u1 0, u1 0, u1 0, u1 1, u1 0, u1 1, u1 0, u1 0, u1 0, u1 1, u1 0, u1 0, u1 1, u1 0, u1 0, u1 0, u1 0, u1 1, u1 1, u1 1, u1 0, u1 1, u1 0, u1 0, u1 1, u1 1, u1 1, u1 0, u1 1, u1 1, u1 0, u1 0, u1 1, u1 1, u1 1, u1 1, u1 0, u1 0, u1 0, u1 0, u1 1, u1 0, u1 0, u1 1, u1 0, u1 0, u1 0, u1 0, u1 1, u1 0, u1 1, u1 1, u1 1, u1 1, u1 1, u1 0, u1 0, u1 1, u1 1, u1 0, u1 0, u1 0, u1 0, u1 0, u1 1, u1 0, u1 1, u1 0, u1 0, u1 1, u1 0, u1 1, u1 1, u1 1, u1 0, u1 1, u1 0, u1 0, u1 0, u1 0, u1 1, u1 1, u1 0, u1 1, u1 0, u1 1, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 1, u1 1, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 1, u1 0, u1 1, u1 1, u1 0, u1 1, u1 1, u1 0, u1 1, u1 1, u1 0, u1 1, u1 0, u1 0, u1 0, u1 1, u1 0, u1 0, u1 0, u1 0, u1 0, u1 1, u1 0, u1 1, u1 0, u1 0, u1 0, u1 0, u1 1, u1 1, u1 1, u1 0, u1 1, u1 1, u1 0, u1 0, u1 1, u1 0, u1 1, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 1, u1 0, u1 1, u1 1, u1 0, u1 0, u1 0, u1 1, u1 1, u1 0, u1 0, u1 1, u1 0, u1 0, u1 0, u1 0, u1 1, u1 1, u1 1, u1 0, u1 1, u1 0, u1 0, u1 1, u1 1, u1 1, u1 0, u1 0, u1 1, u1 1, u1 1, u1 0, u1 0, u1 1, u1 0, u1 0, u1 0, u1 1, u1 0, u1 0, u1 1, u1 1, u1 0, u1 0, u1 0, u1 0, u1 0, u1 1, u1 1] : [u1]
                v10 = allocate -> &mut u1
                store u1 1 at v10
                jmp b1(u32 0)
              b1(v12: u32):
                v13 = lt v12, u32 32
                jmpif v13 then: b2(), else: b3()
              b2():
                v14 = load v10 -> u1
                v15 = not v14
                jmpif v15 then: b4(), else: b5()
              b3():
                v29 = load v10 -> u1
                constrain v29 == u1 1
                return v2
              b4():
                v18 = sub u32 31, v12
                v19 = array_get v2, index v18 -> u1
                v20 = sub u32 31, v12
                v21 = lt v20, u32 254
                constrain v21 == u1 1
                v22 = array_get v7, index v20 -> u1
                v23 = eq v19, v22
                v24 = not v23
                jmpif v24 then: b6(), else: b7()
              b5():
                v28 = unchecked_add v12, u32 1
                jmp b1(v28)
              b6():
                v25 = sub u32 31, v12
                v26 = lt v25, u32 254
                constrain v26 == u1 1
                v27 = array_get v7, index v25 -> u1
                constrain v27 == u1 1
                store u1 1 at v10
                jmp b7()
              b7():
                jmp b5()
            }";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg_simple();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn to_le_bits f0 {
          b0(v0: Field):
            v6 = call to_le_bits(v0) -> [u1; 32]
            v9 = make_array [u1 1, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 1, u1 1, u1 1, u1 1, u1 1, u1 1, u1 0, u1 0, u1 1, u1 0, u1 0, u1 1, u1 1, u1 0, u1 1, u1 0, u1 1, u1 1, u1 1, u1 1, u1 1, u1 0, u1 0, u1 0, u1 0, u1 1, u1 1, u1 1, u1 1, u1 1, u1 0, u1 0, u1 0, u1 0, u1 1, u1 0, u1 1, u1 0, u1 0, u1 0, u1 1, u1 0, u1 0, u1 1, u1 0, u1 0, u1 0, u1 0, u1 1, u1 1, u1 1, u1 0, u1 1, u1 0, u1 0, u1 1, u1 1, u1 1, u1 0, u1 1, u1 1, u1 0, u1 0, u1 1, u1 1, u1 1, u1 1, u1 0, u1 0, u1 0, u1 0, u1 1, u1 0, u1 0, u1 1, u1 0, u1 0, u1 0, u1 0, u1 1, u1 0, u1 1, u1 1, u1 1, u1 1, u1 1, u1 0, u1 0, u1 1, u1 1, u1 0, u1 0, u1 0, u1 0, u1 0, u1 1, u1 0, u1 1, u1 0, u1 0, u1 1, u1 0, u1 1, u1 1, u1 1, u1 0, u1 1, u1 0, u1 0, u1 0, u1 0, u1 1, u1 1, u1 0, u1 1, u1 0, u1 1, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 1, u1 1, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 1, u1 0, u1 1, u1 1, u1 0, u1 1, u1 1, u1 0, u1 1, u1 1, u1 0, u1 1, u1 0, u1 0, u1 0, u1 1, u1 0, u1 0, u1 0, u1 0, u1 0, u1 1, u1 0, u1 1, u1 0, u1 0, u1 0, u1 0, u1 1, u1 1, u1 1, u1 0, u1 1, u1 1, u1 0, u1 0, u1 1, u1 0, u1 1, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 0, u1 1, u1 0, u1 1, u1 1, u1 0, u1 0, u1 0, u1 1, u1 1, u1 0, u1 0, u1 1, u1 0, u1 0, u1 0, u1 0, u1 1, u1 1, u1 1, u1 0, u1 1, u1 0, u1 0, u1 1, u1 1, u1 1, u1 0, u1 0, u1 1, u1 1, u1 1, u1 0, u1 0, u1 1, u1 0, u1 0, u1 0, u1 1, u1 0, u1 0, u1 1, u1 1, u1 0, u1 0, u1 0, u1 0, u1 0, u1 1, u1 1] : [u1]
            jmp b1(u32 0, u1 1)
          b1(v1: u32, v2: u1):
            v12 = lt v1, u32 32
            jmpif v12 then: b2(), else: b3()
          b2():
            v13 = not v2
            jmpif v13 then: b4(), else: b5(v2)
          b3():
            constrain v2 == u1 1
            return v6
          b4():
            v15 = sub u32 31, v1
            v16 = array_get v6, index v15 -> u1
            v17 = sub u32 31, v1
            v19 = lt v17, u32 254
            constrain v19 == u1 1
            v20 = array_get v9, index v17 -> u1
            v21 = eq v16, v20
            v22 = not v21
            jmpif v22 then: b6(), else: b7(v2)
          b5(v3: u1):
            v27 = unchecked_add v1, u32 1
            jmp b1(v27, v3)
          b6():
            v23 = sub u32 31, v1
            v24 = lt v23, u32 254
            constrain v24 == u1 1
            v25 = array_get v9, index v23 -> u1
            constrain v25 == u1 1
            jmp b7(u1 1)
          b7(v4: u1):
            jmp b5(v4)
        }
        ");
    }

    #[test]
    fn read_only_loop() {
        // In the loop header b1 we have v1 incoming with a value of Field 0 from b0,
        // or the current block parameter that was added from the loop edge. Normally,
        // we only check if all arguments are equal to forward them, but it is sufficient
        // to check if all arguments are equal or equal to the original block parameter.
        let src = "
            brillig(inline) fn main f0 {
              b0(v0: u1):
                v1 = allocate -> &mut Field
                store Field 0 at v1
                jmp b1()
              b1():
                jmpif v0 then: b2(), else: b3()
              b2():
                jmp b1()
              b3():
                v2 = load v1 -> Field
                return v2
            }";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg_simple();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u1):
            jmp b1()
          b1():
            jmpif v0 then: b2(), else: b3()
          b2():
            jmp b1()
          b3():
            return Field 0
        }
        ");
    }

    #[test]
    fn nested_loops() {
        let src = "
            brillig(inline) predicate_pure fn main f0 {
              b0():
                v0 = allocate -> &mut Field
                store Field 0 at v0
                jmp b1()
              b1():
                v2 = load v0 -> Field
                v4 = eq v2, Field 6
                jmpif v4 then: b2(), else: b3()
              b2():
                return
              b3():
                v6 = add v2, Field 1
                store v6 at v0
                v7 = allocate -> &mut Field
                store Field 0 at v7
                jmp b4()
              b4():
                v8 = load v7 -> Field
                v10 = eq v8, Field 7
                jmpif v10 then: b5(), else: b6()
              b5():
                jmp b1()
              b6():
                v11 = add v8, Field 1
                store v11 at v7
                jmp b4()
            }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg_simple();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) predicate_pure fn main f0 {
          b0():
            jmp b1(Field 0)
          b1(v0: Field):
            v4 = eq v0, Field 6
            jmpif v4 then: b2(), else: b3()
          b2():
            return
          b3():
            v6 = add v0, Field 1
            jmp b4(Field 0)
          b4(v1: Field):
            v8 = eq v1, Field 7
            jmpif v8 then: b5(), else: b6()
          b5():
            jmp b1(v6)
          b6():
            v9 = add v1, Field 1
            jmp b4(v9)
        }
        ");
    }
}
