use crate::ssa_refactor::Ssa;

impl Ssa {
    /// Simplifies the function's control flow graph by removing blocks
    pub(crate) fn simplify_cfg(mut self) -> Ssa {
        for _function in self.functions.values_mut() {
            // Context::new(function).simplify_function_cfg();
        }
        self
    }
}
