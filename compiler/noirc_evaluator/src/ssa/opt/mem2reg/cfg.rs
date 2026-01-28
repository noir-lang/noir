//! Process the cfg for mem2reg_simple to ensure that for each node with multiple predecessors,
//! all of those predecessors have only a single child node. This ensures all of these predecessors
//! will end in [TerminatorInstruction::Jmp] rather than [TerminatorInstruction::JmpIf], allowing
//! arguments to be passed to the merge node.
use crate::ssa::ir::function::Function;
use crate::ssa::ir::{cfg::ControlFlowGraph, instruction::TerminatorInstruction};
use crate::ssa::ssa_gen::Ssa;

impl Ssa {
    pub(crate) fn process_cfg_for_mem2reg_simple(mut self) -> Self {
        // For each function, ensure that any block with multiple predecessors has those
        // predecessors end in a single-successor jump. If a predecessor currently has
        // multiple successors (e.g. due to a JmpIf), create a new forwarding block that
        // contains only a Jmp terminator to the original target and retarget the
        // predecessor to jump to the new forwarding block. This guarantees that the
        // predecessor's terminator can become a single-successor Jmp, allowing later
        // passes to append arguments to the jump.
        for function in self.functions.values_mut() {
            function.process_cfg();
        }
        self
    }
}

impl Function {
    /// Goal: For every block with multiple predecessors, each predecessor should only have a single successor.
    /// To accomplish this, when we find such a predecessor with multiple successors, we create a new block in-between
    /// such that the predecessor now targets this new block and the new block targets only the original block.
    fn process_cfg(&mut self) {
        // We don't need to update this cfg as we create new blocks since we only process
        // each block once and the newly created blocks will not need to be processed either.
        let cfg = ControlFlowGraph::with_function(self);

        for block in self.reachable_blocks() {
            let predecessors = cfg.predecessors(block);
            if predecessors.len() <= 1 {
                continue;
            }

            for predecessor in predecessors {
                if cfg.successors(predecessor).len() <= 1 {
                    continue;
                }

                // We have `predecessor -> block`, create a new block such that `predecessor -> new_block -> block`.
                // This block doesn't need any arguments since `predecessor` is a jmpif and can't
                // pass arguments to `block` anyway.
                let new_block = self.dfg.make_block();
                let call_stack = self.dfg[predecessor].terminator_call_stack();
                self.dfg[new_block].set_terminator(TerminatorInstruction::Jmp {
                    destination: block,
                    arguments: Vec::new(),
                    call_stack,
                });

                // And change the predecessor to jump to `new_block` instead of `block`
                self.dfg[predecessor].unwrap_terminator_mut().mutate_blocks(|destination| {
                    if destination == block { new_block } else { destination }
                });
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{assert_ssa_snapshot, ssa::ssa_gen::Ssa};

    #[test]
    fn add_arg_to_jmpif_block_regression() {
        // b0 -> b1 -> b2 -> b4 -> b6
        //       | ^   |      \    /
        //       V  \  |       \  /
        //       b3  \ V        VV
        //            b5 <------b7
        //
        // b5 has multiple predecessors: b2 and b7, but b2 has multiple successors.
        // we need to create a new block b8 such that `b2 -> b8 -> b5` so that each predecessor of b5
        // only has one successor.
        // Similarly, b7 also has multiple predecessors, but b4 has multiple successors.
        let src = "
            acir(inline) fn to_le_bits f19 {
              b0(v0: Field):
                jmp b1(u32 0)
              b1(v12: u32):
                jmpif u1 0 then: b2, else: b3
              b2():
                jmpif u1 0 then: b4, else: b5
              b3():
                return v0
              b4():
                jmpif u1 0 then: b6, else: b7
              b5():
                jmp b1(u32 0)
              b6():
                jmp b7()
              b7():
                jmp b5()
            }";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.process_cfg_for_mem2reg_simple();

        // The addition of b8 and b9 are the only changes
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn to_le_bits f0 {
          b0(v0: Field):
            jmp b1(u32 0)
          b1(v1: u32):
            jmpif u1 0 then: b2, else: b3
          b2():
            jmpif u1 0 then: b4, else: b8
          b3():
            return v0
          b4():
            jmpif u1 0 then: b6, else: b9
          b5():
            jmp b1(u32 0)
          b6():
            jmp b7()
          b7():
            jmp b5()
          b8():
            jmp b5()
          b9():
            jmp b7()
        }
        ");
    }
}
