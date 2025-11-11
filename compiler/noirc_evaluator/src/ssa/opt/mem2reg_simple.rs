//! Mem2reg algorithm adapted from the paper: https://bernsteinbear.com/assets/img/bebenita-ssa.pdf
use std::collections::{BTreeSet, VecDeque};

use iter_extended::vecmap;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        cfg::ControlFlowGraph,
        dom::DominatorTree,
        function::Function,
        function_inserter::FunctionInserter,
        instruction::{Instruction, TerminatorInstruction},
        post_order::{self, PostOrder},
        value::ValueId,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    pub(crate) fn mem2reg_simple(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.mem2reg_simple();
        }
        self
    }
}

impl Function {
    fn mem2reg_simple(&mut self) {
        let blocks = self.reachable_blocks();
        let cfg = ControlFlowGraph::with_function(self);
        let post_order = PostOrder::with_cfg(&cfg);
        let mut dom_tree = DominatorTree::with_cfg_and_post_order(&cfg, &post_order);
        let mut inserter = FunctionInserter::new(self);

        let mut entry_states = FxHashMap::default();
        let mut exit_states = FxHashMap::default();

        let variables = collect_all_eligible_allocates(inserter.function, post_order);

        // Find the starting & ending states of each variable in each block
        for block in blocks.iter().copied() {
            // All variables visible at the start of the current block
            let entry_state = filter_visible_variables(&variables, &mut dom_tree, block);
            entry_states.insert(block, entry_state.clone());

            let exit_state = get_variable_values_in_block(&inserter.function, block, entry_state);
            exit_states.insert(block, exit_state);
        }

        // Link entry & exit states by adding block parameters & terminator arguments for every variable stored to
        for block in blocks.iter().copied() {
            for (address, entry_value) in entry_states[&block].iter() {
                // Remember any value stored to address in this block. The initial value given here is unused and thus arbitrary.
                let mut last_value = *entry_value;
                let predecessors = cfg.predecessors(block);
                let predecessor_count = predecessors.len();

                for predecessor in predecessors {
                    let exit_value = exit_states[&predecessor][address];
                    last_value = exit_value;

                    if predecessor_count > 1 {
                        add_terminator_argument(inserter.function, exit_value, predecessor);
                    }
                }

                if predecessor_count == 1 {
                    println!("pred count = 1, mapping {} to {}", address, last_value);
                    inserter.map_value(*address, last_value);
                } else if predecessor_count > 1 {
                    let param = add_block_param(inserter.function, last_value, block);
                    println!("pred count > 1, mapping {} to {}", address, param);
                    inserter.map_value(*address, param);
                }
            }
        }

        // Simplify block parameters where all arguments are identical
        for block in blocks.iter().copied() {
            let parameters = inserter.function.dfg.block_parameters(block).to_vec();

            // Mask of whether each parameter has non-identical arguments,
            // E.g. if parameter 2's arguments are all identical, then keep_parameters[2] would be false
            let mask = keep_argument_mask(&mut inserter, &cfg, block, &parameters);

            // Remove unneeded parameters from the block
            retain_items_from_mask(inserter.function.dfg[block].parameters_mut(), &mask);

            // And remove the corresponding parameter's arguments from each predecessor
            for predecessor in cfg.predecessors(block) {
                let terminator = inserter.function.dfg[predecessor].unwrap_terminator_mut();
                if let TerminatorInstruction::Jmp { arguments, .. } = terminator {
                    retain_items_from_mask(arguments, &mask);
                }
            }
        }

        commit(&mut inserter, &variables, blocks);
    }
}

/// Mapping from a variable to its value at a point in time
type StateVec = FxHashMap<ValueId, ValueId>;

/// Filter `variables`, returning only those that are visible at the start of the given block.
/// Since we do not consider variables within a block, the visible variables at the start of a block
/// are any variables whose declaration block dominates the given block.
fn filter_visible_variables(
    variables: &FxHashMap<ValueId, BasicBlockId>,
    dom_tree: &mut DominatorTree,
    block: BasicBlockId,
) -> StateVec {
    variables
        .iter()
        .filter_map(|(var, decl_block)| {
            dom_tree.dominates(*decl_block, block).then(|| (*var, *var))
        })
        .collect()
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
    vecmap(parameters.iter().enumerate(), |(i, parameter)| {
        let mut args = cfg.predecessors(block).map(|predecessor| {
            match inserter.function.dfg[predecessor].unwrap_terminator() {
                TerminatorInstruction::Jmp { arguments, .. } => inserter.resolve(arguments[i]),
                other => {
                    panic!("Unexpected terminator when checking block argument: {other:?}")
                }
            }
        });

        let Some(first_arg) = args.next() else {
            return true;
        };

        // keep the parameter if the arguments do not all match
        let keep_param = !args.all(|arg| arg == first_arg);
        if !keep_param {
            // All arguments are identical, so the choice to map to the first is arbitrary
            inserter.map_value(*parameter, first_arg);
        }
        keep_param
    })
}

// For each index i of `items`, keep `items[i]` iff `mask[i]`
fn retain_items_from_mask(items: &mut Vec<ValueId>, mask: &Vec<bool>) {
    let mut i = 0;
    items.retain(|_| {
        i += 1;
        mask[i - 1]
    });
}

fn add_block_param(function: &mut Function, arg: ValueId, block: BasicBlockId) -> ValueId {
    let parameter_type = function.dfg.type_of_value(arg);
    function.dfg.add_block_parameter(block, parameter_type)
}

fn add_terminator_argument(function: &mut Function, arg: ValueId, block: BasicBlockId) {
    match function.dfg[block].unwrap_terminator_mut() {
        TerminatorInstruction::Jmp { arguments, .. } => arguments.push(arg),
        other => panic!("Unexpected terminator when adding block argument: {other:?}"),
    }
}

/// Abstractly interpret a block, collecting the value of each reference at the end of a block.
/// Any references not included in the result are assumed to be unchanged.
///
/// This function is very simple and will produce incorrect results for first-class references
/// that are aliased, passed to functions, or stored in arrays, etc.
fn get_variable_values_in_block(
    function: &Function,
    block: BasicBlockId,
    entry_state: StateVec,
) -> StateVec {
    // To calculate the exit state, we start with the entry state
    let mut exit_state = entry_state;

    for instruction in function.dfg[block].instructions() {
        match &function.dfg[*instruction] {
            Instruction::Store { address, value } => {
                // Only update the value if the address is already in the state, since only
                // those addresses are eligible for mem2reg optimization.
                exit_state.entry(*address).and_modify(|v| *v = *value);
            }
            _ => {}
        }
    }

    exit_state
}

/// Return a map from each variable to the block it was declared in.
/// This only includes variables that are eligible for mem2reg optimization,
/// i.e. those that are allocated but never used in a first-class manner.
fn collect_all_eligible_allocates(
    function: &Function,
    post_order: PostOrder,
) -> FxHashMap<ValueId, BasicBlockId> {
    // Map each variable to the block it was declared in
    let mut variables = FxHashMap::default();
    let forward_order = post_order.into_vec_reverse();

    for block in forward_order {
        for instruction_id in function.dfg[block].instructions() {
            let instruction = &function.dfg[*instruction_id];
            match instruction {
                Instruction::Allocate => {
                    let address = function.dfg.instruction_results(*instruction_id)[0];
                    variables.insert(address, block);
                }
                Instruction::Load { .. } => (),
                // Storing to an address is fine, but storing an address prevents optimizing it out.
                Instruction::Store { address: _, value } => {
                    variables.remove(value);
                }
                // Any other use of an address (in arrays, functions, etc) is also first-class and prevents optimization.
                _ => {
                    instruction.for_each_value(|value| variables.remove(&value));
                }
            }
        }
    }

    variables
}

/// Commit to all changes made by the pass:
/// - Map any values mapped from the inserter to their new values in the function
/// - Remove all Allocate, Load, and Store instructions from the eligible variables
fn commit(
    inserter: &mut FunctionInserter,
    variables: &FxHashMap<ValueId, BasicBlockId>,
    blocks: BTreeSet<BasicBlockId>,
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
                Instruction::Load { address } => {
                    if variables.contains_key(address) {
                        let result = inserter.function.dfg.instruction_results(*instruction_id)[0];
                        inserter.map_value(result, inserter.resolve(*address));
                        false
                    } else {
                        true
                    }
                }
                Instruction::Store { address, value: _ } => !variables.contains_key(address),
                _ => true,
            };

            if keep {
                let instruction = &mut inserter.function.dfg[*instruction_id];
                instruction.map_values_mut(|value| {
                    FunctionInserter::resolve_detached(value, &inserter.values)
                });
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
    use crate::{assert_ssa_snapshot, ssa::ssa_gen::Ssa};

    #[test]
    fn test_simple() {
        let src = "
        acir(inline) fn func f0 {
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
        acir(inline) fn func f0 {
          b0():
            v0 = allocate -> &mut [Field; 2]
            v2 = make_array [Field 1, Field 1] : [Field; 2]
            return Field 1
        }
        ");
    }

    #[test]
    fn test_multiblock() {
        let src = "
        acir(inline) fn func f0 {
          b0(v0: u1):
            v1 = allocate -> &mut Field
            store Field 0 at v1
            jmpif v0 then: b1, else: b2
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
        acir(inline) fn func f0 {
          b0(v0: u1):
            jmpif v0 then: b1, else: b2
          b1():
            jmp b3(Field 1)
          b2():
            jmp b3(Field 2)
          b3(v1: Field):
            return v1
        }
        ");
    }
}
