//! The goal of this SSA pass is to go through each [`Instruction::Constrain`],
//! determine whether it's asserting two values are not equal, and if so replace it
//! with a [`Instruction::ConstrainNotEqual`].
//!
//! This pass is only applied to ACIR functions.
//!
//! For example, this SSA code:
//!
//! ```ssa
//! v2 = eq v0, v1
//! constrain v2 == u1 0
//! ```
//!
//! will be replaced with this one:
//!
//! ```ssa
//! v2 = eq v0, v1
//! constrain v0 != v1
//! ```
//!
//! When constraining with an equality in ACIR generation we need all the handling for the
//! case where the two values ARE equal. Rather than just asserting that an inverse
//! exists for the difference between these two values we need to create two
//! unnecessary witnesses - one which is unconstrained and the other constrained to
//! zero. This is unnecessary as we want the circuit to just fail in this case.
//!
//! ## Preconditions:
//! - this pass must be placed after [`CFG flattening`](super::flatten_cfg)
//!   as the flattening pass cannot handle this instruction.
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
    /// Replaces [`Instruction::Constrain`] asserting two values are not equal with [`Instruction::ConstrainNotEqual`].
    ///
    /// See the [`make_constrain_not_equal`](self) module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn make_constrain_not_equal(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.make_constrain_not_equal();
        }
        self
    }
}

impl Function {
    fn make_constrain_not_equal(&mut self) {
        if !self.runtime().is_acir() {
            return;
        }

        self.simple_optimization(|context| {
            // This Noir code:
            //
            // ```noir
            // assert(x != y)
            // ```
            //
            // always translates to an SSA like this:
            //
            // ```ssa
            // v0 = eq x, y
            // constrain v0 == u1 0
            // ```
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
    use crate::{
        assert_ssa_snapshot,
        ssa::{
            interpreter::tests::from_constant, ir::types::NumericType,
            opt::assert_ssa_does_not_change, ssa_gen::Ssa,
        },
    };

    #[test]
    fn replaces_constrain_with_constrain_not_equal_in_acir() {
        let src = "
        acir(inline) fn main f1 {
          b0(v0: Field, v1: Field):
            v2 = eq v0, v1
            constrain v2 == u1 0
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.make_constrain_not_equal();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: Field, v1: Field):
            v2 = eq v0, v1
            constrain v0 != v1
            return
        }
        ");
    }

    #[test]
    fn does_not_replace_constrain_with_constrain_not_equal_in_brillig() {
        let src = "
        brillig(inline) fn main f1 {
          b0(v0: Field, v1: Field):
            v2 = eq v0, v1
            constrain v2 == u1 0
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::make_constrain_not_equal);
    }

    #[test]
    /// https://github.com/noir-lang/noir/issues/10929
    fn regression_10929() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field, v1: Field, v2: u1):
            enable_side_effects v2
            v3 = eq v0, v1
            constrain v3 == u1 0, "QPA"
            return
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();

        let inputs = vec![
            from_constant(1_u128.into(), NumericType::NativeField),
            from_constant(1_u128.into(), NumericType::NativeField),
            from_constant(false.into(), NumericType::bool()),
        ];

        let expected = ssa.interpret(inputs.clone());

        let ssa = ssa.make_constrain_not_equal();

        let got = ssa.interpret(inputs);
        assert_eq!(expected, got);

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: Field, v1: Field, v2: u1):
            enable_side_effects v2
            v3 = eq v0, v1
            constrain v0 != v1, "QPA"
            return
        }
        "#);
    }
}
