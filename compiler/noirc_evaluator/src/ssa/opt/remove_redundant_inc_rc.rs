//! This pass removes `inc_rc` instructions made redundant by an earlier `inc_rc` of the
//! same value.
//!
//! `inc_rc` only has a runtime effect in Brillig, where it bumps an array/vector's
//! reference count so that a later `array_set` copies instead of mutating in place.
//! The copy-on-write decision only distinguishes a reference count of exactly 1 from
//! anything greater (see the `ArrayCopy` Brillig procedure), so a second `inc_rc` on a
//! value whose count is already at least 2 makes no difference.
//!
//! Reference counts are only ever incremented: `dec_rc` is not generated, and the
//! decrement that `array_set`'s copy-on-write path would otherwise perform on its source
//! is disabled (see `codegen_decrement_rc`). A reference count is therefore monotonically
//! non-decreasing, so once an `inc_rc v` has executed the count of `v` stays at least 2
//! for the rest of the function — regardless of any `array_set`, `call`, or other
//! instruction that follows.
//!
//! An `inc_rc v` is thus redundant whenever another `inc_rc v` is guaranteed to have
//! already executed. Within a block that is any earlier `inc_rc v`; across blocks it is
//! an `inc_rc v` in a *dominating* block, since every path reaching the current block
//! runs the dominator's straight-line body in full first. We therefore walk blocks in
//! reverse post-order and seed each block with the set of already-incremented values from
//! its immediate dominator.
//!
//! The pass keys on the exact `ValueId`, so a loop-carried value (a block parameter, i.e.
//! a distinct value each iteration) is never merged with an increment outside the loop;
//! only increments of the *same* value — a loop-invariant value, a parameter, or a value
//! defined in a common dominator — are removed.
//!
//! Functions that read a reference count directly via the `array_refcount` /
//! `vector_refcount` intrinsics are skipped, since removing increments would change the
//! value they observe.

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet, FxHashSet};

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        cfg::ControlFlowGraph,
        dom::DominatorTree,
        function::Function,
        instruction::{Instruction, InstructionId, Intrinsic},
        post_order::PostOrder,
        value::ValueId,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// Remove `inc_rc` instructions made redundant by an earlier `inc_rc` of the same
    /// value in the same or a dominating block.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn remove_redundant_inc_rc(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.remove_redundant_inc_rc();
        }
        self
    }
}

impl Function {
    fn remove_redundant_inc_rc(&mut self) {
        if !self.runtime().is_brillig() {
            // inc_rc only has an effect in Brillig.
            return;
        }

        // A program that observes reference counts directly relies on their exact
        // value, so we must not remove increments in such functions.
        if self.dfg.get_intrinsic(Intrinsic::ArrayRefCount).is_some()
            || self.dfg.get_intrinsic(Intrinsic::VectorRefCount).is_some()
        {
            return;
        }

        let to_remove = self.find_redundant_inc_rcs();
        if to_remove.is_empty() {
            return;
        }

        for block in self.reachable_blocks() {
            self.dfg[block]
                .instructions_mut()
                .retain(|instruction| !to_remove.contains(instruction));
        }
    }

    /// Find every `inc_rc` of a value that was already incremented earlier in the same
    /// block or in a dominating block.
    fn find_redundant_inc_rcs(&self) -> FxHashSet<InstructionId> {
        let cfg = ControlFlowGraph::with_function(self);
        let post_order = PostOrder::with_cfg(&cfg);
        let dom_tree = DominatorTree::with_cfg_and_post_order(&cfg, &post_order);

        let mut to_remove = HashSet::default();
        // Per block, the set of values whose reference count is known to be at least 2 on
        // exit from the block (including everything inherited from its dominators).
        let mut incremented_on_exit: HashMap<BasicBlockId, HashSet<ValueId>> = HashMap::default();

        // Reverse post-order visits a block only after all of its dominators, so the
        // immediate dominator's exit set is always available when we reach a block.
        for block in post_order.into_vec_reverse() {
            let mut incremented = match dom_tree.immediate_dominator(block) {
                Some(idom) => incremented_on_exit[&idom].clone(),
                None => HashSet::default(),
            };
            for instruction in self.dfg[block].instructions() {
                match &self.dfg[*instruction] {
                    Instruction::IncrementRc { value } => {
                        if !incremented.insert(*value) {
                            to_remove.insert(*instruction);
                        }
                    }
                    // `dec_rc` is not generated during normal compilation, but if one is
                    // present it lowers the count of its value, so an earlier increment can
                    // no longer be assumed to keep it at least 2.
                    Instruction::DecrementRc { value } => {
                        incremented.remove(value);
                    }
                    _ => {}
                }
            }
            incremented_on_exit.insert(block, incremented);
        }

        to_remove
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::opt::assert_ssa_does_not_change, ssa::ssa_gen::Ssa};

    #[test]
    fn removes_later_inc_rc_on_same_value() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 2]):
            inc_rc v0
            inc_rc v0
            inc_rc v0
            return v0
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_redundant_inc_rc();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: [Field; 2]):
            inc_rc v0
            return v0
        }
        ");
    }

    #[test]
    fn removes_later_inc_rc_even_with_instructions_in_between() {
        // Nothing decrements a reference count, so the increment survives across the
        // array_set and the second `inc_rc v0` is still redundant.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 2]):
            inc_rc v0
            v3 = array_set v0, index u32 0, value Field 1
            inc_rc v0
            v5 = array_set v0, index u32 1, value Field 2
            return v0
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_redundant_inc_rc();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: [Field; 2]):
            inc_rc v0
            v3 = array_set v0, index u32 0, value Field 1
            v6 = array_set v0, index u32 1, value Field 2
            return v0
        }
        ");
    }

    #[test]
    fn keeps_one_inc_rc_per_distinct_value() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 2], v1: [Field; 2]):
            inc_rc v0
            inc_rc v1
            inc_rc v0
            inc_rc v1
            return v0
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_redundant_inc_rc();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: [Field; 2], v1: [Field; 2]):
            inc_rc v0
            inc_rc v1
            return v0
        }
        ");
    }

    #[test]
    fn removes_inc_rc_in_dominated_block() {
        // b0 dominates b1, so b1's `inc_rc v0` is redundant.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 2]):
            inc_rc v0
            jmp b1()
          b1():
            inc_rc v0
            return v0
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_redundant_inc_rc();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: [Field; 2]):
            inc_rc v0
            jmp b1()
          b1():
            return v0
        }
        ");
    }

    #[test]
    fn keeps_inc_rc_when_predecessor_does_not_dominate() {
        // b3 is reachable from both b1 and b2, so the `inc_rc v0` in b1 does not dominate
        // b3 and the `inc_rc v0` in b3 must be kept.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [Field; 2], v1: u1):
            jmpif v1 then: b1(), else: b2()
          b1():
            inc_rc v0
            jmp b3()
          b2():
            jmp b3()
          b3():
            inc_rc v0
            return v0
        }
        ";
        assert_ssa_does_not_change(src, Ssa::remove_redundant_inc_rc);
    }

    #[test]
    fn does_not_run_on_acir() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: [Field; 2]):
            inc_rc v0
            inc_rc v0
            return v0
        }
        ";
        assert_ssa_does_not_change(src, Ssa::remove_redundant_inc_rc);
    }
}
