//! This SSA pass removes `RangeCheck` instructions whose value the range analysis already proves
//! fits within the checked bit size.
//!
//! Instruction simplification only elides range checks that are redundant by type, since it runs on
//! every insertion and cannot afford the range analysis. The tighter, analysis-based elisions are
//! done here instead: once per function, so the range analysis is built a single time rather than
//! re-run for every instruction.

use crate::ssa::{
    ir::{function::Function, instruction::Instruction},
    ssa_gen::Ssa,
};

impl Ssa {
    /// See [`range_check_elision`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn remove_redundant_range_checks(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.remove_redundant_range_checks();
        }
        self
    }
}

impl Function {
    fn remove_redundant_range_checks(&mut self) {
        if !self.runtime().is_acir() {
            return;
        }

        // A range check only constrains its own value; removing one never changes any value's
        // inferred bit width (the analysis does not use range checks on the unconstrained `bits`
        // path), so a single snapshot stays valid for the whole pass.
        let max_num_bits = self.dfg.value_max_num_bits();

        self.simple_optimization(|context| {
            let Instruction::RangeCheck { value, max_bit_size, .. } = context.instruction() else {
                return;
            };
            let value = *value;
            let max_bit_size = *max_bit_size;

            // `u32::MAX` keeps the check when the value is somehow absent from the snapshot.
            if max_num_bits.get(&value).copied().unwrap_or(u32::MAX) <= max_bit_size {
                context.remove_current_instruction();
            }
        });
    }
}
