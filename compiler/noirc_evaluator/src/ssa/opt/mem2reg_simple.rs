//! Mem2reg algorithm adapted from the paper: https://bernsteinbear.com/assets/img/bebenita-ssa.pdf
use std::collections::{BTreeSet, hash_map::Entry};

use iter_extended::vecmap;
use rustc_hash::FxHashMap;

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

        let variables = collect_all_eligible_variables(inserter.function, post_order);

        // Find the starting & ending states of each variable in each block
        for block in blocks.iter().copied() {
            // All variables visible at the start of the current block
            let entry_state = add_visible_variables_as_block_arguments(
                &variables,
                &mut dom_tree,
                block,
                &mut inserter.function.dfg,
            );
            entry_states.insert(block, entry_state.clone());

            let exit_state = abstract_interpret_block(&mut inserter, block, entry_state);
            exit_states.insert(block, exit_state);
        }

        // Link entry & exit states by adding block parameters & terminator arguments for every variable stored to
        for block in blocks.iter().copied() {
            for (address, entry_value) in entry_states[&block].iter() {
                // If the current block is this variable's source block, no merge
                // is needed, skip the block.
                if block == variables[address] {
                    continue;
                }

                // Remember any value stored to address in this block. The initial value given here is unused and thus arbitrary.
                let mut last_value = *entry_value;
                let predecessors = cfg.predecessors(block);
                let predecessor_count = predecessors.len();

                for predecessor in predecessors {
                    let exit_value = *exit_states
                        .get(&predecessor)
                        .unwrap_or_else(|| panic!("No entry for {predecessor}"))
                        .get(address)
                        .unwrap_or_else(|| {
                            panic!(
                                "No entry in {predecessor} for {address}, current block = {block}"
                            )
                        });
                    last_value = exit_value;

                    if predecessor_count > 1 {
                        add_terminator_argument(inserter.function, exit_value, predecessor);
                    }
                }

                if predecessor_count == 1 {
                    inserter.map_value(*entry_value, last_value);
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
fn add_visible_variables_as_block_arguments(
    variables: &FxHashMap<ValueId, BasicBlockId>,
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
        let mut args = cfg.predecessors(block).map(|predecessor| {
            let terminator = inserter.function.dfg[predecessor].unwrap_terminator();
            if let TerminatorInstruction::Jmp { arguments, .. } = terminator {
                arguments.get(i).copied()
            } else {
                None
            }
        });

        let Some(Some(first_arg)) = args.next() else {
            return false;
        };

        // keep the parameter if the arguments do not all match
        let keep_param = !args.all(|arg| arg == Some(first_arg));
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
///
/// Note that this function will also replace any instances of the reference being stored to
/// with its current value in the block. Most of the time, this locally renames the reference
/// in the block so that we can map its value in the block independent of other blocks.
fn abstract_interpret_block(
    inserter: &mut FunctionInserter,
    block: BasicBlockId,
    entry_state: StateVec,
) -> StateVec {
    // To calculate the exit state, we start with the entry state
    let mut exit_state = entry_state;
    let mut instructions = inserter.function.dfg[block].take_instructions();

    for (old_address, new_address) in exit_state.iter() {
        inserter.map_value(*old_address, *new_address);
    }

    // We're going to remove any load/store instructions we already know the answer of,
    // just with what we know in this single block.
    instructions.retain(|instruction_id| {
        let keep = match &inserter.function.dfg[*instruction_id] {
            Instruction::Store { address, value } => {
                // Only update the value if the address is already in the state, since only
                // those addresses are eligible for mem2reg optimization. If the address is
                // in the state, we remove it.
                if let Entry::Occupied(mut entry) = exit_state.entry(*address) {
                    *entry.get_mut() = *value;
                    inserter.map_value(*address, *value);
                    false
                } else {
                    true
                }
            }
            Instruction::Load { address } => {
                if let Some(value) = exit_state.get(address) {
                    let result = inserter.function.dfg.instruction_results(*instruction_id)[0];
                    inserter.map_value(result, *value);
                    false
                } else {
                    true
                }
            }
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

    // Remove the local block mappings. This shouldn't be necessary, but avoids other blocks
    // accidentally referring to these mappings in the case of a bug.
    for old_address in exit_state.keys() {
        inserter.map_value(*old_address, *old_address);
    }

    *inserter.function.dfg[block].instructions_mut() = instructions;
    exit_state
}

/// Return a map from each variable to the block it was declared in.
/// This only includes variables that are eligible for mem2reg optimization,
/// i.e. those that are allocated but never used in a first-class manner.
fn collect_all_eligible_variables(
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
            v1 = make_array [Field 1, Field 1] : [Field; 2]
            v3 = array_get v1, index u32 1 -> Field
            return v3
        }
        ");
    }

    #[test]
    fn test_multi_block() {
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

    #[test]
    fn test_single_predecessor_elimination() {
        let src = "
            acir(inline) fn func f0 {
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
            acir(inline) fn func f0 {
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
            acir(inline) fn func f0 {
              b0(v0: u1):
                v1 = allocate -> &mut Field
                store Field 0 at v1
                jmpif v0 then: b1, else: b2
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
            acir(inline) fn func f0 {
              b0(v0: u1):
                jmpif v0 then: b1, else: b2
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
            acir(inline) fn func f0 {
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
            acir(inline) fn func f0 {
              b0():
                v2 = add Field 5, Field 10
                return v2
            }
            ");
    }

    #[test]
    fn test_three_way_merge() {
        let src = "
            acir(inline) fn func f0 {
              b0(v0: u1, v1: u1):
                v2 = allocate -> &mut Field
                store Field 1 at v2
                jmpif v0 then: b1, else: b2
              b1():
                store Field 2 at v2
                jmp b5()
              b2():
                jmpif v1 then: b3, else: b4
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
            acir(inline) fn func f0 {
              b0(v0: u1, v1: u1):
                jmpif v0 then: b1, else: b2
              b1():
                jmp b5(Field 2)
              b2():
                jmpif v1 then: b3, else: b4
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
            acir(inline) fn func f0 {
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
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg_simple();
        // Should optimize separate allocations
        assert_ssa_snapshot!(ssa, @r"
            acir(inline) fn func f0 {
              b0():
                v0 = allocate -> &mut Field
                store Field 10 at v0
                v2 = make_array [v0] : [&mut Field; 1]
                v3 = load v0 -> Field
                v5 = array_get v2, index u32 0 -> &mut Field
                store Field 20 at v5
                v7 = load v0 -> Field
                v8 = add v3, v7
                return v8
            }
            ");
    }

    // #[test]
    // fn test_loop_with_back_edge() {
    //     // Simple loop: allocate outside, store/load inside loop
    //     let src = "
    //         acir(inline) fn func f0 {
    //           b0(v0: u32):
    //             v1 = allocate -> &mut u32
    //             store u32 0 at v1
    //             jmp b1(v0)
    //           b1(v2: u32):
    //             v3 = load v1 -> u32
    //             v4 = add v3, u32 1
    //             store v4 at v1
    //             v5 = lt v2, u32 10
    //             jmpif v5 then: b1(v2), else: b2
    //           b2():
    //             v6 = load v1 -> u32
    //             return v6
    //         }
    //         ";
    //     let ssa = Ssa::from_str(src).unwrap();
    //     let ssa = ssa.mem2reg_simple();
    //     // The load in b1 should be replaced with the parameter, and loop iterations tracked
    //     assert_ssa_snapshot!(ssa, @r"
    //         acir(inline) fn func f0 {
    //           b0(v0: u32):
    //             jmp b1(u32 0, v0)
    //           b1(v3: u32, v2: u32):
    //             v4 = add v3, u32 1
    //             v5 = lt v2, u32 10
    //             jmpif v5 then: b1(v4, v2), else: b2
    //           b2():
    //             return v4
    //         }
    //         ");
    // }

    #[test]
    fn test_variable_only_stored_in_one_branch() {
        // Variable is stored in one branch but not the other
        let src = "
            acir(inline) fn func f0 {
              b0(v0: u1):
                v1 = allocate -> &mut Field
                store Field 5 at v1
                jmpif v0 then: b1, else: b2
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
            acir(inline) fn func f0 {
              b0(v0: u1):
                jmpif v0 then: b1, else: b2
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
            acir(inline) fn func f0 {
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
            acir(inline) fn func f0 {
              b0():
                return Field 3
            }
            ");
    }

    #[test]
    fn test_deep_nesting_diamond() {
        // Deeply nested diamond patterns
        let src = "
            acir(inline) fn func f0 {
              b0(v0: u1, v1: u1):
                v2 = allocate -> &mut Field
                store Field 1 at v2
                jmpif v0 then: b1, else: b2
              b1():
                jmpif v1 then: b3, else: b4
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
            acir(inline) fn func f0 {
              b0(v0: u1, v1: u1):
                jmpif v0 then: b1, else: b2
              b1():
                jmpif v1 then: b3, else: b4
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
acir(inline) fn main f0 {
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
    jmpif v13 then: b2, else: b3
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
    jmpif v28 then: b5, else: b6
  b5():
    v31 = add v4, u32 2
    store v31 at v6
    v33 = load v3 -> u32
    v34 = load v3 -> u32
    v35 = call f1(v33, v34) -> u32
    v36 = array_get v0, index v27 -> u32
    v37 = call f1(v35, v36) -> u32
    store v37 at v6
    v39 = load v3 -> u32
    v41 = array_get v0, index v27 -> u32
    v42 = array_get v1, index v27 -> u32
    v43 = mul v41, v42
    v44 = load v6 -> u32
    v45 = call f3(v43, v44) -> u32
    v46 = call f2(v39, v45) -> u32
    store v46 at v3
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
    jmpif v55 then: b8, else: b9
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
    jmpif v63 then: b11, else: b12
  b11():
    v64 = call f3(v54, u32 2) -> u32
    store v64 at v6
    v65 = load v3 -> u32
    v66 = load v6 -> u32
    v67 = call f1(v65, v66) -> u32
    store v67 at v3
    v68 = unchecked_add v62, u32 1
    jmp b10(v68)
  b12():
    v69 = unchecked_add v54, u32 1
    jmp b7(v69)
  b13(v74: u32):
    v75 = lt v74, u32 3
    jmpif v75 then: b14, else: b15
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
    jmpif v96 then: b19, else: b20
  b16(v81: u32):
    v82 = lt v81, u32 2
    jmpif v82 then: b17, else: b18
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
    jmpif v103 then: b23, else: b24
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
    jmpif v106 then: b26, else: b27
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
        // Should properly merge all three stores: from b2, b3, b4
        assert_ssa_snapshot!(ssa, @r"
acir(inline) fn main f0 {
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
    jmpif v13 then: b2, else: b3
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
    jmpif v28 then: b5, else: b6
  b5():
    v31 = add v4, u32 2
    store v31 at v6
    v33 = load v3 -> u32
    v34 = load v3 -> u32
    v35 = call f1(v33, v34) -> u32
    v36 = array_get v0, index v27 -> u32
    v37 = call f1(v35, v36) -> u32
    store v37 at v6
    v39 = load v3 -> u32
    v41 = array_get v0, index v27 -> u32
    v42 = array_get v1, index v27 -> u32
    v43 = mul v41, v42
    v44 = load v6 -> u32
    v45 = call f3(v43, v44) -> u32
    v46 = call f2(v39, v45) -> u32
    store v46 at v3
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
    jmpif v55 then: b8, else: b9
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
    jmpif v63 then: b11, else: b12
  b11():
    v64 = call f3(v54, u32 2) -> u32
    store v64 at v6
    v65 = load v3 -> u32
    v66 = load v6 -> u32
    v67 = call f1(v65, v66) -> u32
    store v67 at v3
    v68 = unchecked_add v62, u32 1
    jmp b10(v68)
  b12():
    v69 = unchecked_add v54, u32 1
    jmp b7(v69)
  b13(v74: u32):
    v75 = lt v74, u32 3
    jmpif v75 then: b14, else: b15
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
    jmpif v96 then: b19, else: b20
  b16(v81: u32):
    v82 = lt v81, u32 2
    jmpif v82 then: b17, else: b18
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
    jmpif v103 then: b23, else: b24
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
    jmpif v106 then: b26, else: b27
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
            ");
    }
}
