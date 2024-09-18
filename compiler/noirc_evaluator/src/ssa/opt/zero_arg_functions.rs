use crate::ssa::{
    ir::{
        function::Function,
        instruction::{Instruction, InstructionId, Intrinsic},
        types::Type,
        value::Value,
    },
    ssa_gen::Ssa,
    RuntimeError,
};

impl Ssa {
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn zero_arg_functions(mut self) -> Result<Ssa, RuntimeError> {
        for func in self.functions.values_mut() {
            // TODO cleanup
            dbg!((func.name(), func.parameters()));

            // TODO UNUSED:
            // bubble_up_constrains(&mut self) {
            if func.parameters().len() == 0 {
                // NOTE: these are run in the same order as in `optimize_into_acir`
                func.remove_paired_rc();
                func.replace_is_unconstrained_result();
                func.mem2reg();
                func.as_slice_optimization();
                func.evaluate_static_assert_and_assert_constant()?;
                func.unroll_loops_iteratively()?;
                func.simplify_function();
                func.remove_bit_shifts();
                func.mem2reg();
                func.remove_if_else();

                let without_constraint_info = false;
                func.constant_fold(without_constraint_info);
                func.remove_enable_side_effects();

                let with_constraint_info = true;
                func.constant_fold(with_constraint_info);

                let insert_out_of_bounds_checks = true;
                func.dead_instruction_elimination(insert_out_of_bounds_checks);
                func.array_set_optimization();
            }
        }

        Ok(self)
    }
}

