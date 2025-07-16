use acvm::AcirField;
use num_traits::Zero;

use crate::ssa::{
    ir::{
        function::Function,
        instruction::{Binary, BinaryOp, Instruction},
        value::Value,
    },
    ssa_gen::Ssa,
};

impl Ssa {
    /// A simple SSA pass to go through each [`Instruction::Constrain`], determine whether it's asserting
    /// two values are not equal, and if so replace it with a [`Instruction::ConstrainNotEqual`].
    ///
    /// Note that this pass must be placed after CFG flattening as the flattening pass cannot
    /// handle this instruction.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn make_constrain_not_equal_instructions(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.make_constrain_not_equal();
        }
        self
    }
}

impl Function {
    pub(crate) fn make_constrain_not_equal(&mut self) {
        if !self.runtime().is_acir() {
            return;
        }

        self.simple_reachable_blocks_optimization(|context| {
            let instruction = context.instruction();

            let Instruction::Constrain(lhs, rhs, msg) = instruction else {
                return;
            };

            if !context.dfg.get_numeric_constant(*rhs).is_some_and(|constant| constant.is_zero()) {
                return;
            }

            let Value::Instruction { instruction, .. } = &context.dfg[*lhs] else {
                return;
            };

            let Instruction::Binary(Binary { lhs, rhs, operator: BinaryOp::Eq, .. }) =
                context.dfg[*instruction]
            else {
                return;
            };

            let new_instruction = Instruction::ConstrainNotEqual(lhs, rhs, msg.clone());
            context.replace_current_instruction_with(new_instruction);
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::ssa_gen::Ssa};

    #[test]
    fn test_make_constrain_not_equals() {
        let src = "
        acir(inline) fn main f1 {
          b0(v0: Field, v1: Field):
            v2 = eq v0, v1
            constrain v2 == u1 0
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.make_constrain_not_equal_instructions();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v2 = eq v0, v1
            constrain v0 != v1
            return
        }
        ");
    }
}
