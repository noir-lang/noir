//! Pre-process functions before inlining them into others.

use crate::ssa::Ssa;

use super::inlining;

impl Ssa {
    /// Run pre-processing steps on functions in isolation.
    pub(crate) fn preprocess_functions(
        mut self,
        aggressiveness: i64,
        max_bytecode_increase_percent: Option<i32>,
    ) -> Ssa {
        // No point pre-processing the functions that will never be inlined into others.
        let not_to_inline = inlining::get_functions_to_inline_into(&self, false, aggressiveness);
        // Bottom-up order, starting with the "leaf" functions, so we inline already optimized code into the ones that call them.
        let bottom_up = inlining::compute_bottom_up_order(&self);

        // As a heuristic to avoid optimizing functions near the entry point, find a cutoff weight.
        let total_weight = bottom_up.iter().fold(0usize, |acc, (_, w)| acc.saturating_add(*w));
        let mean_weight = total_weight / bottom_up.len();
        let cutoff_weight = mean_weight;

        for (id, _) in bottom_up
            .into_iter()
            .filter(|(id, _)| !not_to_inline.contains(id))
            .filter(|(_, weight)| *weight < cutoff_weight)
        {
            let function = &self.functions[&id];
            let mut function = function.inlined(&self, false, &not_to_inline);
            // Help unrolling determine bounds.
            function.as_slice_optimization();
            // We might not be able to unroll all loops without fully inlining them, so ignore errors.
            let _ = function.unroll_loops_iteratively(max_bytecode_increase_percent);
            // Reduce the number of redundant stores/loads after unrolling
            function.mem2reg();
            // Try to reduce the number of blocks.
            function.simplify_function();

            // Remove leftover instructions.
            // XXX: Doing this would currently integration test failures,
            // for example with `traits_in_crates_1` it eliminates a store to a mutable input reference.
            // function.dead_instruction_elimination(true);

            // Put it back into the SSA, so the next functions can pick it up.
            self.functions.insert(id, function);
        }

        self
    }
}
