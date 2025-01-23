//! Pre-process functions before inlining them into others.

use crate::ssa::{
    ir::function::{Function, RuntimeType},
    Ssa,
};

use super::inlining::{self, InlineInfo};

impl Ssa {
    /// Run pre-processing steps on functions in isolation.
    pub(crate) fn preprocess_functions(mut self, aggressiveness: i64) -> Ssa {
        // Bottom-up order, starting with the "leaf" functions, so we inline already optimized code into the ones that call them.
        let bottom_up = inlining::compute_bottom_up_order(&self);

        // Preliminary inlining decisions.
        let inline_infos = inlining::compute_inline_infos(&self, false, aggressiveness);

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
            // Remove leftover instructions.
            function.dead_instruction_elimination(true, false);

            // Put it back into the SSA, so the next functions can pick it up.
            self.functions.insert(id, function);
        }

        // Remove any functions that have been inlined into others already.
        self.remove_unreachable_functions()
    }
}
