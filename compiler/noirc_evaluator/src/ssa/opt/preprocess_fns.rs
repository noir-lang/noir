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
        // Bottom-up order, starting with the "leaf" functions, so we inline already optimized code into the ones that call them.
        let bottom_up = inlining::compute_bottom_up_order(&self);
        let not_to_inline = inlining::get_functions_to_inline_into(&self, false, aggressiveness);

        for id in bottom_up.into_iter().filter(|id| !not_to_inline.contains(id)) {
            let function = &self.functions[&id];
            let mut function = function.inlined(&self, false, &not_to_inline);
            // Help unrolling determine bounds.
            function.as_slice_optimization();
            // We might not be able to unroll all loops without fully inlining them, so ignore errors.
            let _ = function.try_unroll_loops_iteratively(max_bytecode_increase_percent, true);
            // Reduce the number of redundant stores/loads after unrolling
            function.mem2reg();
            // Try to reduce the number of blocks.
            function.simplify_function();
            // Remove leftover instructions.
            function.dead_instruction_elimination(true);
            // Put it back into the SSA, so the next functions can pick it up.
            self.functions.insert(id, function);
        }

        self
    }
}
