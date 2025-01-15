//! Pre-process functions before inlining them into others.

use crate::ssa::Ssa;

impl Ssa {
    /// Run pre-processing steps on functions in isolation.
    pub(crate) fn preprocess_functions(
        mut self,
        aggressiveness: i64,
        max_bytecode_increase_percent: Option<i32>,
    ) -> Ssa {
        // Ok(self
        //     .inline_functions_limited(aggressiveness)
        //     .as_slice_optimization()
        //     .try_unroll_loops_iteratively(max_bytecode_increase_percent, true)?
        //     .mem2reg())

        // TODO: Ideally we would go bottom-up, starting with the "leaf" functions, so we inline already optimized code into
        // the ones that call them, but for now just see what happens if we pre-process the "serialize" function.
        let to_preprocess = self
            .functions
            .iter()
            .filter_map(|(id, f)| (f.name() == "serialize").then_some(*id))
            .collect::<Vec<_>>();

        let not_to_inline =
            super::inlining::get_functions_to_inline_into(&self, false, aggressiveness);

        for id in to_preprocess {
            let function = &self.functions[&id];
            let mut function = function.inlined(&self, false, &not_to_inline);
            // Help unrolling determine bounds.
            function.as_slice_optimization();
            // We might not be able to unroll all loops without fully inlining them, so ignore errors.
            let _ = function.try_unroll_loops_iteratively(max_bytecode_increase_percent, true);
            // Reduce the number of redundant stores/loads after unrolling
            function.mem2reg();
            // Put it back into the SSA, so the next functions can pick it up.
            self.functions.insert(id, function);
        }

        self
    }
}
