//! This module implements the pruning of unused block parameters from functions.
//!
//! Blocks can accept parameters that are passed via terminator instructions (e.g., `jmp`).
//! During the compilation pipeline, it's common for some of these parameters to become unused.
//! This module eliminates those unused parameters and correspondingly
//! adjusts the terminators of predecessor blocks. This work is ultimately done to reduce code size.
//!
//! ## How this pass works:
//! - Iterates through all blocks in post-order (to ensure predecessors are visited after successors).
//! - Detects and removes unused block parameters, except for those on the entry block.
//! - Clears the list of unused block parameters after removing them from the block.
//! - **Entry block parameters** are never removed. These often correspond to function inputs and
//!   must remain to preserve the function's external interface, even if they're unused internally.
//! - Updates the corresponding argument lists in predecessor terminator instructions to keep
//!   them aligned with the new parameter lists.
//!
//! ## Preconditions:
//! - This pass should be run *after* [Dead Instruction Elimination (DIE)][super] so that parameter
//!   liveness is up-to-date.
//!
//! ## Panics
//! Return blocks are not expected to have successors, so encountering one as a predecessor
//! is treated as an internal compiler error (ICE).
//!
//! ## Example:
//!
//! Before pruning:
//! ```text
//! b0():
//!   jmp b1(Field 1, Field 2, Field 3)
//! b1(v0: Field, v1: Field, v2: Field):
//!   return v1
//! ```
//!
//! After pruning:
//! ```text
//! b0():
//!   jmp b1(Field 2)
//! b1(v0: Field):
//!   return v0
//! ```
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        cfg::ControlFlowGraph,
        function::{Function, FunctionId},
        instruction::TerminatorInstruction,
        post_order::PostOrder,
        value::ValueId,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// See [`prune_dead_parameters`][self] module for more information.
    #[tracing::instrument(level = "trace", skip_all)]
    pub(crate) fn prune_dead_parameters(
        mut self,
        unused_parameters: &HashMap<FunctionId, HashMap<BasicBlockId, Vec<ValueId>>>,
    ) -> Self {
        for (func_id, unused_parameters) in unused_parameters {
            let function = self.functions.get_mut(func_id).expect("ICE: Function should exist");
            function.prune_dead_parameters(unused_parameters);
        }
        self
    }
}

impl Function {
    /// See [`prune_dead_parameters`][self] module for more information
    fn prune_dead_parameters(&mut self, unused_params: &HashMap<BasicBlockId, Vec<ValueId>>) {
        let cfg = ControlFlowGraph::with_function(self);
        let post_order = PostOrder::with_cfg(&cfg);

        for &block in post_order.as_slice() {
            // We do not support to removing function arguments. This is because function signatures,
            // which are used for setting up the program artifact inputs, are set by the frontend.
            if block == self.entry_block() {
                continue;
            }

            let empty_params = Vec::new();
            let unused_params = unused_params.get(&block).unwrap_or(&empty_params);
            if unused_params.is_empty() {
                // Nothing to do if the block has no unused params
                continue;
            }

            let old_params = self.dfg[block].take_parameters();

            // Create the list of new params for updating the block with unused parameters
            // as well as an indexed list of the removed parameters to update each predecessor's terminator argument list.
            let mut keep_list = Vec::with_capacity(old_params.len());
            let mut new_params = Vec::with_capacity(old_params.len());
            let unused_set = unused_params.iter().copied().collect::<HashSet<_>>();
            for param in old_params {
                let keep = !unused_set.contains(&param);
                keep_list.push(keep);
                if keep {
                    new_params.push(param);
                }
            }

            self.dfg[block].set_parameters(new_params);

            // Update the predecessor argument list to match the new parameter list
            let predecessors = cfg.predecessors(block);
            for pred in predecessors {
                let terminator = self.dfg[pred].unwrap_terminator_mut();

                match terminator {
                    TerminatorInstruction::JmpIf { .. } => {
                        // No terminator arguments in a JmpIf, so we do nothing here
                    }
                    TerminatorInstruction::Jmp { destination, arguments, .. } => {
                        // Cannot place this guard on the pattern as we are matching by reference
                        if *destination == block {
                            let new_args = arguments
                                .iter()
                                .zip(keep_list.iter())
                                .filter_map(|(arg, &keep)| if keep { Some(*arg) } else { None })
                                .collect();
                            *arguments = new_args;
                        }
                    }
                    TerminatorInstruction::Return { .. } => {
                        unreachable!("ICE: A return block should not be a predecessor");
                    }
                    TerminatorInstruction::Unreachable { .. } => {
                        unreachable!("ICE: An unreachable block should not be a predecessor");
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_ssa_snapshot;

    use crate::ssa::Ssa;
    use crate::ssa::ir::map::Id;

    #[test]
    fn prune_unused_block_params() {
        let src = r#"
        brillig(inline) fn test f0 {
          b0():
            jmp b1(Field 1, Field 2, Field 3)
          b1(v0: Field, v1: Field, v2: Field):
            return v1
        }"#;

        let ssa = Ssa::from_str(src).unwrap();
        // DIE is necessary to fetch the block parameters liveness information
        let (ssa, die_result) = ssa.dead_instruction_elimination_inner(false, false);

        assert!(die_result.unused_parameters.len() == 1);
        let function = die_result
            .unused_parameters
            .get(&Id::test_new(0))
            .expect("Should have unused parameters");
        let b0_unused = function.get(&Id::test_new(0)).expect("Should have unused parameters");
        // b0 has no parameters
        assert!(b0_unused.is_empty());
        let b1_unused = function.get(&Id::test_new(1)).expect("Should have unused parameters");
        // We expect v0 and v2 to be unused, not v1
        assert_eq!(b1_unused.len(), 2);
        assert_eq!(b1_unused[0].to_u32(), 0);
        assert_eq!(b1_unused[1].to_u32(), 2);

        let ssa = ssa.prune_dead_parameters(&die_result.unused_parameters);

        assert_ssa_snapshot!(ssa, @r#"
        brillig(inline) fn test f0 {
          b0():
            jmp b1(Field 2)
          b1(v0: Field):
            return v0
        }"#);
    }

    #[test]
    fn prune_unused_block_params_multiple_predecessors() {
        // This SSA comes from a regression in https://github.com/noir-lang/noir/issues/8229
        let src = r#"
        g0 = u32 2825334515

        brillig(inline) predicate_pure fn main f0 {
          b0(v1: [[u1; 4]; 4]):
            v4 = array_get v1, index u32 0 -> [u1; 4]
            inc_rc v4
            v6 = array_get v4, index u32 3 -> u1
            jmpif v6 then: b1, else: b2
          b1():
            v9 = mul u32 601072115, u32 2825334515
            v10 = cast v9 as u64
            jmp b3(v10)
          b2():
            jmp b3(u64 3513574538769362461)
          b3(v2: u64):
            return u1 0
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        // DIE is necessary to fetch the block parameters liveness information
        let (ssa, die_result) = ssa.dead_instruction_elimination_inner(false, false);

        assert!(die_result.unused_parameters.len() == 1);
        let function = die_result
            .unused_parameters
            .get(&Id::test_new(0))
            .expect("Should have unused parameters");
        let b0_unused = function.get(&Id::test_new(0)).expect("Should have unused parameters");
        // b0 has one parameter but it is used
        assert!(b0_unused.is_empty());
        let b1_unused = function.get(&Id::test_new(1)).expect("Should have unused parameters");
        // b1 has no parameters
        assert!(b1_unused.is_empty());
        // b2 has no parameters
        let b2_unused = function.get(&Id::test_new(2)).expect("Should have unused parameters");
        assert!(b2_unused.is_empty());
        let b3_unused = function.get(&Id::test_new(3)).expect("Should have unused parameters");
        // b3 has `v2: u64` but it is unused
        assert_eq!(b3_unused.len(), 1);
        assert_eq!(b3_unused[0].to_u32(), 2);

        let ssa = ssa.prune_dead_parameters(&die_result.unused_parameters);
        let (ssa, _) = ssa.dead_instruction_elimination_inner(false, false);

        // We expect b3 to have no parameters anymore and both predecessors (b1 and b2)
        // should no longer pass any arguments to their terminator (which jumps to b3).
        assert_ssa_snapshot!(ssa, @r#"
        g0 = u32 2825334515
        
        brillig(inline) predicate_pure fn main f0 {
          b0(v1: [[u1; 4]; 4]):
            v3 = array_get v1, index u32 0 -> [u1; 4]
            v5 = array_get v3, index u32 3 -> u1
            jmpif v5 then: b1, else: b2
          b1():
            v7 = mul u32 601072115, u32 2825334515
            jmp b3()
          b2():
            jmp b3()
          b3():
            return u1 0
        }
        "#);
    }

    #[test]
    fn do_not_prune_dead_entry_block_params() {
        let src = r#"
        brillig(inline) fn test f0 {
          b0(v0: Field, v1: Field):
            jmp b1(Field 1)
          b1(v2: Field):
            return v2
        }"#;

        let ssa = Ssa::from_str(src).unwrap();
        // DIE is necessary to fetch the block parameters liveness information
        let (ssa, die_result) = ssa.dead_instruction_elimination_inner(false, false);

        assert!(die_result.unused_parameters.len() == 1);
        let function = die_result
            .unused_parameters
            .get(&Id::test_new(0))
            .expect("Should have unused parameters");
        let b0_unused = function.get(&Id::test_new(0)).expect("Should have unused parameters");
        // b0 has two parameters but they are unused
        assert!(b0_unused.len() == 2);
        let b1_unused = function.get(&Id::test_new(1)).expect("Should have unused parameters");
        assert!(b1_unused.is_empty());

        let ssa = ssa.prune_dead_parameters(&die_result.unused_parameters);

        // b0 still has both parameters even though v0 is unused
        // as b0 is the entry block which would also change the function signature.
        assert_ssa_snapshot!(ssa, @r#"
        brillig(inline) fn test f0 {
          b0(v0: Field, v1: Field):
            jmp b1(Field 1)
          b1(v2: Field):
            return v2
        }"#);
    }

    #[test]
    fn prune_parameter_used_in_a_separate_block_terminator() {
        // The following SSA comes this code:
        // ```noir
        // fn main(input: i16) {
        //     let result = if input > 3 {
        //         if input > 4 {
        //             1
        //         } else {
        //             2
        //         }
        //     } else {
        //         3
        //     };
        //     let result2 = if input > 5 {
        //         result
        //     } else {
        //         result + 1
        //     };
        // }
        // ```
        let src = r#"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: i16):
            v5 = lt i16 3, v0
            jmpif v5 then: b1, else: b2
          b1():
            v8 = lt i16 4, v0
            jmpif v8 then: b3, else: b4
          b2():
            jmp b5(Field 3)
          b3():
            jmp b6(Field 1)
          b4():
            jmp b6(Field 2)
          b5(v1: Field):
            v12 = lt i16 5, v0
            jmpif v12 then: b7, else: b8
          b6(v2: Field):
            jmp b5(v2)
          b7():
            jmp b9(v1)
          b8():
            v13 = add v1, Field 1
            jmp b9(v13)
          b9(v3: Field):
            return
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();

        // DIE is necessary to fetch the block parameters liveness information
        let (ssa, die_result) = ssa.dead_instruction_elimination_inner(false, false);

        assert!(die_result.unused_parameters.len() == 1);
        let function = die_result
            .unused_parameters
            .get(&Id::test_new(0))
            .expect("Should have unused parameters");
        for (block_id, unused_params) in function {
            if block_id.to_u32() == 9 {
                assert!(unused_params.len() == 1);
                assert_eq!(unused_params[0].to_u32(), 3);
            } else if block_id.to_u32() == 5 {
                assert!(unused_params.len() == 1);
                assert_eq!(unused_params[0].to_u32(), 1);
            } else if block_id.to_u32() == 6 {
                assert!(unused_params.len() == 1);
                assert_eq!(unused_params[0].to_u32(), 2);
            } else {
                assert!(unused_params.is_empty());
            }
        }

        let ssa = ssa.prune_dead_parameters(&die_result.unused_parameters);

        let (ssa, die_result) = ssa.dead_instruction_elimination_inner(false, false);

        assert!(die_result.unused_parameters.len() == 1);
        let function = die_result
            .unused_parameters
            .get(&Id::test_new(0))
            .expect("Should have unused parameters");
        for unused_params in function.values() {
            assert!(unused_params.is_empty());
        }

        assert_ssa_snapshot!(ssa, @r#"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: i16):
            v2 = lt i16 3, v0
            jmpif v2 then: b1, else: b2
          b1():
            v4 = lt i16 4, v0
            jmpif v4 then: b3, else: b4
          b2():
            jmp b5()
          b3():
            jmp b6()
          b4():
            jmp b6()
          b5():
            v6 = lt i16 5, v0
            jmpif v6 then: b7, else: b8
          b6():
            jmp b5()
          b7():
            jmp b9()
          b8():
            jmp b9()
          b9():
            return
        }
        "#);

        // Now check that calling the DIE -> parameter pruning feedback loop produces the same result
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.dead_instruction_elimination_with_pruning(false, false);
        assert_ssa_snapshot!(ssa, @r#"
        brillig(inline) predicate_pure fn main f0 {
          b0(v0: i16):
            v2 = lt i16 3, v0
            jmpif v2 then: b1, else: b2
          b1():
            v4 = lt i16 4, v0
            jmpif v4 then: b3, else: b4
          b2():
            jmp b5()
          b3():
            jmp b6()
          b4():
            jmp b6()
          b5():
            v6 = lt i16 5, v0
            jmpif v6 then: b7, else: b8
          b6():
            jmp b5()
          b7():
            jmp b9()
          b8():
            jmp b9()
          b9():
            return
        }
        "#);
    }
}
