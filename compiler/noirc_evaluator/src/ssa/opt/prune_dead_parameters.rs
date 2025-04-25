//! Prunes any dead block parameters from a block and correspondingly
//! adjusts the terminators of predecessor blocks.

use fxhash::FxHashSet as HashSet;

use crate::ssa::{
    ir::{
        cfg::ControlFlowGraph, function::Function, instruction::TerminatorInstruction,
        post_order::PostOrder,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn prune_dead_parameters(mut self) -> Self {
        for function in self.functions.values_mut() {
            function.prune_dead_parameters();
        }
        self
    }
}

impl Function {
    /// See [`prune_dead_parameters`][self] module for more information
    pub(crate) fn prune_dead_parameters(&mut self) {
        let cfg = ControlFlowGraph::with_function(self);
        let post_order = PostOrder::with_cfg(&cfg);

        for block in post_order.as_slice() {
            let block = *block;

            let unused_params = self.dfg[block].unused_parameters().to_vec();
            if unused_params.is_empty() {
                continue;
            }

            self.dfg[block].clear_unused_parameters();

            // We do not support to removing function arguments. This is because function signatures,
            // which are used for setting up the program artifact inputs, are set by the frontend.
            if block == self.entry_block() {
                continue;
            }

            let old_params = self.dfg[block].take_parameters();

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
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_ssa_snapshot;

    use crate::ssa::Ssa;

    #[test]
    fn prune_unused_block_params() {
        let src = r#"
        brillig(inline) fn test f0 {
          b0():
            jmp b1(Field 1, Field 2, Field 3)
          b1(v0: Field, v1: Field, v2: Field):
            return v1
        }"#;

        let mut ssa = Ssa::from_str(src).unwrap();
        // DIE is necessary to fetch the block parameters liveness information
        ssa = ssa.dead_instruction_elimination();
        ssa = ssa.prune_dead_parameters();

        assert_all_unused_parameters_cleared(&ssa);

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

        let mut ssa = Ssa::from_str(src).unwrap();
        // DIE is necessary to fetch the block parameters liveness information
        ssa = ssa.dead_instruction_elimination();
        ssa = ssa.prune_dead_parameters();

        assert_all_unused_parameters_cleared(&ssa);

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
            v8 = cast v7 as u64
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

        let mut ssa = Ssa::from_str(src).unwrap();
        ssa = ssa.dead_instruction_elimination();
        ssa = ssa.prune_dead_parameters();

        assert_all_unused_parameters_cleared(&ssa);

        // b0 still has both parameters even though v0 is unused
        assert_ssa_snapshot!(ssa, @r#"
        brillig(inline) fn test f0 {
          b0(v0: Field, v1: Field):
            jmp b1(Field 1)
          b1(v2: Field):
            return v2
        }"#);
    }

    fn assert_all_unused_parameters_cleared(ssa: &Ssa) {
        for function in ssa.functions.values() {
            for block in function.reachable_blocks() {
                let unused_params = function.dfg[block].unused_parameters();
                assert!(unused_params.is_empty());
            }
        }
    }
}
