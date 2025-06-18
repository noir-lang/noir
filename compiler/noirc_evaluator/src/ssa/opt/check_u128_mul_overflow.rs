use acvm::{AcirField, FieldElement};
use noirc_errors::call_stack::CallStackId;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        function::Function,
        instruction::{Binary, BinaryOp, ConstrainError, Instruction},
        types::NumericType,
        value::ValueId,
    },
    ssa_gen::Ssa,
};

use super::simple_optimization::SimpleOptimizationContext;

impl Ssa {
    /// An SSA pass that checks that multiplying two u128 doesn't overflow because
    /// both operands are greater or equal than 2^64.
    /// If both are, then the result is surely greater or equal than 2^128 so it would overflow.
    /// The operands can still overflow if just one of them is less than 2^64, but in that case the result
    /// will be less than 2^192 so it fits in a Field value, and acir will check that it fits in a u128.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn check_u128_mul_overflow(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.check_u128_mul_overflow();
        }
        self
    }
}

impl Function {
    pub(crate) fn check_u128_mul_overflow(&mut self) {
        if !self.runtime().is_acir() {
            return;
        }

        self.simple_reachable_blocks_optimization(|context| {
            context.insert_current_instruction();

            let block_id = context.block_id;
            let instruction_id = context.instruction_id;
            let instruction = context.instruction();
            let Instruction::Binary(Binary {
                lhs,
                rhs,
                operator: BinaryOp::Mul { unchecked: false },
            }) = instruction
            else {
                return;
            };

            let binary_type = context.dfg.type_of_value(*lhs).unwrap_numeric();
            let NumericType::Unsigned { bit_size: 128 } = binary_type else {
                return;
            };

            let call_stack = context.dfg.get_instruction_call_stack_id(instruction_id);
            check_u128_mul_overflow(*lhs, *rhs, block_id, context, call_stack);
        });
    }
}

fn check_u128_mul_overflow(
    lhs: ValueId,
    rhs: ValueId,
    block: BasicBlockId,
    context: &mut SimpleOptimizationContext<'_, '_>,
    call_stack: CallStackId,
) {
    let dfg = &mut context.dfg;
    let lhs_value = dfg.get_numeric_constant(lhs);
    let rhs_value = dfg.get_numeric_constant(rhs);

    let two_pow_64 = 1_u128 << 64;

    // If lhs is less than 2^64 then the condition trivially holds.
    if let Some(value) = lhs_value {
        if value.to_u128() < two_pow_64 {
            return;
        }
    }

    // Same goes for rhs
    if let Some(value) = rhs_value {
        if value.to_u128() < two_pow_64 {
            return;
        }
    }

    let u128 = NumericType::unsigned(128);
    let two_pow_64_plus_1 = dfg.make_constant((two_pow_64 + 1).into(), u128);

    let res = if lhs_value.is_some() && rhs_value.is_some() {
        // If both values are known at compile time, at this point we know it overflows
        dfg.make_constant(FieldElement::one(), u128)
    } else if lhs_value.is_some() {
        // If only the left-hand side is known we just need to check that the right-hand side
        // isn't greater than 2^64
        let instruction = Instruction::Binary(Binary {
            lhs: rhs,
            rhs: two_pow_64_plus_1,
            operator: BinaryOp::Lt,
        });
        let rhs_less_than_two_pow_64_plus_1 =
            dfg.insert_instruction_and_results(instruction, block, None, call_stack).first();
        let not = Instruction::Not(rhs_less_than_two_pow_64_plus_1);
        dfg.insert_instruction_and_results(not, block, None, call_stack).first()
    } else if rhs_value.is_some() {
        // Same goes for the other side
        let instruction =
            Instruction::Binary(Binary { lhs, rhs: two_pow_64_plus_1, operator: BinaryOp::Lt });
        let lhs_less_than_two_pow_64_plus_1 =
            dfg.insert_instruction_and_results(instruction, block, None, call_stack).first();
        let not = Instruction::Not(lhs_less_than_two_pow_64_plus_1);
        dfg.insert_instruction_and_results(not, block, None, call_stack).first()
    } else {
        // Check both sides
        let instruction =
            Instruction::Binary(Binary { lhs, rhs: two_pow_64_plus_1, operator: BinaryOp::Lt });
        let lhs_less_than_two_pow_64_plus_1 =
            dfg.insert_instruction_and_results(instruction, block, None, call_stack).first();
        let not = Instruction::Not(lhs_less_than_two_pow_64_plus_1);
        let lhs_greater_than_two_pow_64 =
            dfg.insert_instruction_and_results(not, block, None, call_stack).first();

        let instruction = Instruction::Binary(Binary {
            lhs: rhs,
            rhs: two_pow_64_plus_1,
            operator: BinaryOp::Lt,
        });
        let rhs_less_than_two_pow_64_plus_1 =
            dfg.insert_instruction_and_results(instruction, block, None, call_stack).first();
        let not = Instruction::Not(rhs_less_than_two_pow_64_plus_1);
        let rhs_greater_than_two_pow_64 =
            dfg.insert_instruction_and_results(not, block, None, call_stack).first();

        // Unchecked as operands are restricted to be less than 2^64 so multiplying them cannot overflow.
        let mul = BinaryOp::Mul { unchecked: true };
        let instruction = Instruction::Binary(Binary {
            lhs: lhs_greater_than_two_pow_64,
            rhs: rhs_greater_than_two_pow_64,
            operator: mul,
        });
        dfg.insert_instruction_and_results(instruction, block, None, call_stack).first()
    };

    let zero = dfg.make_constant(FieldElement::zero(), u128);
    let instruction = Instruction::Constrain(
        res,
        zero,
        Some(ConstrainError::StaticString("attempt to multiply with overflow".to_string())),
    );
    dfg.insert_instruction_and_results(instruction, block, None, call_stack);
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_ssa_snapshot,
        ssa::{opt::assert_normalized_ssa_equals, ssa_gen::Ssa},
    };

    #[test]
    fn does_not_insert_check_if_lhs_is_less_than_two_pow_64() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u128):
            v2 = mul u128 18446744073709551615, v0
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.check_u128_mul_overflow();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn does_not_insert_check_if_rhs_is_less_than_two_pow_64() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u128):
            v2 = mul v0, u128 18446744073709551615
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.check_u128_mul_overflow();
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn inserts_check_for_lhs() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u128):
            v2 = mul v0, u128 18446744073709551617
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.check_u128_mul_overflow();
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0(v0: u128):
            v2 = mul v0, u128 18446744073709551617
            v3 = lt v0, u128 18446744073709551617
            v4 = not v3
            constrain v4 == u128 0, "attempt to multiply with overflow"
            return
        }
        "#);
    }

    #[test]
    fn inserts_check_for_rhs() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u128):
            v2 = mul u128 18446744073709551617, v0
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.check_u128_mul_overflow();
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0(v0: u128):
            v2 = mul u128 18446744073709551617, v0
            v3 = lt v0, u128 18446744073709551617
            v4 = not v3
            constrain v4 == u128 0, "attempt to multiply with overflow"
            return
        }
        "#);
    }

    #[test]
    fn inserts_check_for_both_operands() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u128, v1: u128):
            v2 = mul v0, v1
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.check_u128_mul_overflow();
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0(v0: u128, v1: u128):
            v2 = mul v0, v1
            v4 = lt v0, u128 18446744073709551617
            v5 = not v4
            v6 = lt v1, u128 18446744073709551617
            v7 = not v6
            v8 = unchecked_mul v5, v7
            constrain v8 == u128 0, "attempt to multiply with overflow"
            return
        }
        "#);
    }

    #[test]
    fn inserts_assertion_failure_if_overflow_is_guaranteed() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v2 = mul u128 18446744073709551617, u128 18446744073709551616
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.check_u128_mul_overflow();
        // The multiplication remains, but it will be later removed by DIE
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0():
            v2 = mul u128 18446744073709551617, u128 18446744073709551616
            constrain u128 1 == u128 0, "attempt to multiply with overflow"
            return
        }
        "#);
    }

    #[test]
    fn does_nothing_for_brillig() {
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v2 = mul u128 18446744073709551617, u128 18446744073709551616
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.check_u128_mul_overflow();
        assert_normalized_ssa_equals(ssa, src);
    }
}
