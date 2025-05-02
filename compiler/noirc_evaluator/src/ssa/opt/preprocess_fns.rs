//! Pre-process functions before inlining them into others.

use crate::ssa::{
    Ssa,
    ir::function::{Function, RuntimeType},
};

use super::inlining::{self, InlineInfo};

impl Ssa {
    /// Run pre-processing steps on functions in isolation.
    pub(crate) fn preprocess_functions(mut self, aggressiveness: i64) -> Ssa {
        // Bottom-up order, starting with the "leaf" functions, so we inline already optimized code into the ones that call them.
        let bottom_up = inlining::inline_info::compute_bottom_up_order(&self);

        // Preliminary inlining decisions.
        let inline_infos =
            inlining::inline_info::compute_inline_infos(&self, false, aggressiveness);

        let should_inline_call = |callee: &Function| -> bool {
            match callee.runtime() {
                RuntimeType::Acir(_) => {
                    // Functions marked to not have predicates should be preserved.
                    !callee.is_no_predicates()
                }
                RuntimeType::Brillig(_) => {
                    // We inline inline if the function called wasn't ruled out as too costly or recursive.
                    InlineInfo::should_inline(&inline_infos, callee.id())
                }
            }
        };

        for (id, (own_weight, transitive_weight)) in bottom_up {
            let function = &self.functions[&id];

            // Skip preprocessing heavy functions that gained most of their weight from transitive accumulation, which tend to be near the entry.
            // These can be processed later by the regular SSA passes.
            let is_heavy = transitive_weight > own_weight * 10;

            // Functions which are inline targets will be processed in later passes.
            // Here we want to treat the functions which will be inlined into them.
            let is_target =
                inline_infos.get(&id).map(|info| info.is_inline_target()).unwrap_or_default();

            if is_heavy || is_target {
                continue;
            }

            // Start with an inline pass.
            let mut function = function.inlined(&self, &should_inline_call);
            // Help unrolling determine bounds.
            function.as_slice_optimization();
            // Prepare for unrolling
            function.loop_invariant_code_motion();
            // We might not be able to unroll all loops without fully inlining them, so ignore errors.
            let _ = function.unroll_loops_iteratively();
            // Reduce the number of redundant stores/loads after unrolling
            function.mem2reg();

            // Try to reduce the number of blocks.
            function.simplify_function();

            // Put it back into the SSA, so the next functions can pick it up.
            self.functions.insert(id, function);
        }

        // Remove any functions that have been inlined into others already.
        let ssa = self.remove_unreachable_functions();
        // Remove leftover instructions.
        ssa.dead_instruction_elimination_pre_flattening()
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::ssa_gen::Ssa};

    #[test]
    fn dead_block_params() {
        // This test makes sure that we are appropriately triggering DIE+parameter pruning.
        //
        // DIE must be run over the full SSA to correctly identify unused block parameters.
        // If it's run in isolation on a single function (e.g., during preprocessing),
        // it may leave dangling block parameters.
        //
        // We need to call f0 from an entry point as inline targets are not preprocessed.
        let src = r#"
        acir(inline) fn main f0 {
          b0():
            call f0()
            return
        }
        acir(inline) fn foo f0 {
          b0(v0: u32, v1: Field):
            jmpif v0 then: b1, else: b2
          b1():
            v6 = add v0, u32 1
            jmp b3(v6, v1)
          b2():
            v5 = sub v0, u32 1
            jmp b3(v5, v1)
          b3(v2: u32, v3: Field):
            return
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.preprocess_functions(i64::MAX);

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0():
            call f1()
            return
        }
        acir(inline) fn foo f1 {
          b0(v0: u32, v1: Field):
            jmpif v0 then: b1, else: b2
          b1():
            v4 = add v0, u32 1
            jmp b3()
          b2():
            v3 = sub v0, u32 1
            jmp b3()
          b3():
            return
        }
        "#);
    }
}
