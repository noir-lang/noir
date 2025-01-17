use acvm::AcirField;

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

        for block in self.reachable_blocks() {
            let instructions = self.dfg[block].instructions().to_vec();

            for instruction in instructions {
                let constrain_ne: Instruction = match &self.dfg[instruction] {
                    Instruction::Constrain(lhs, rhs, msg) => {
                        if self
                            .dfg
                            .get_numeric_constant(*rhs)
                            .map_or(false, |constant| constant.is_zero())
                        {
                            if let Value::Instruction { instruction, .. } =
                                &self.dfg[self.dfg.resolve(*lhs)]
                            {
                                if let Instruction::Binary(Binary {
                                    lhs,
                                    rhs,
                                    operator: BinaryOp::Eq,
                                    ..
                                }) = self.dfg[*instruction]
                                {
                                    Instruction::ConstrainNotEqual(lhs, rhs, msg.clone())
                                } else {
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                };

                self.dfg[instruction] = constrain_ne;
            }
        }
    }
}
