//! Pre-process functions before inlining them into others.

use crate::ssa::Ssa;

use super::inlining;

impl Ssa {
    /// Run pre-processing steps on functions in isolation.
    pub(crate) fn preprocess_functions(mut self, aggressiveness: i64) -> Ssa {
        // Bottom-up order, starting with the "leaf" functions, so we inline already optimized code into the ones that call them.
        let bottom_up = inlining::compute_bottom_up_order(&self);

        // As a heuristic to avoid optimizing functions near the entry point, find a cutoff weight.
        let total_weight =
            bottom_up.iter().fold(0usize, |acc, (_, (_, w))| (acc.saturating_add(*w)));
        let mean_weight = total_weight / bottom_up.len();
        let cutoff_weight = mean_weight;

        // Preliminary inlining decisions.
        let inline_infos = inlining::compute_inline_infos(&self, false, aggressiveness);

        for (id, (own_weight, transitive_weight)) in bottom_up {
            // Skip preprocessing heavy functions that gained most of their weight from transitive accumulation.
            // These can be processed later by the regular SSA passes.
            if transitive_weight >= cutoff_weight && transitive_weight > own_weight * 2 {
                continue;
            }
            // Functions which are inline targets will be processed in later passes.
            // Here we want to treat the functions which will be inlined into them.
            if let Some(info) = inline_infos.get(&id) {
                if info.is_inline_target() {
                    continue;
                }
            }
            let function = &self.functions[&id];
            // Start with an inline pass.
            let mut function = function.inlined(&self, false, &inline_infos);
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
