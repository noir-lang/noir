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
                            .is_some_and(|constant| constant.is_zero())
                        {
                            if let Value::Instruction { instruction, .. } = &self.dfg[*lhs] {
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
