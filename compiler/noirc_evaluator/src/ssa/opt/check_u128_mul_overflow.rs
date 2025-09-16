//! An SSA pass that operates on ACIR functions that checks that multiplying two u128 doesn't
//! overflow because both operands are greater or equal than 2^64.
//! If both are, then the result is surely greater or equal than 2^128 so it would overflow.
//! The operands can still overflow if just one of them is less than 2^64, but in that case
//! the result will be less than 2^192 so it fits in a Field value, and acir will check that
//! it fits in a u128.
//!
//! In Brillig an overflow check is automatically performed on unsigned binary operations
//! so this SSA pass has no effect for Brillig functions.
use acvm::{AcirField, FieldElement};

use crate::ssa::{
    ir::{
        function::Function,
        instruction::{Binary, BinaryOp, ConstrainError, Instruction},
        types::NumericType,
        value::ValueId,
    },
    ssa_gen::Ssa,
};

use super::simple_optimization::SimpleOptimizationContext;

impl Ssa {
    /// See [`check_u128_mul_overflow`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn check_u128_mul_overflow(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.check_u128_mul_overflow();
        }
        self
    }
}

impl Function {
    fn check_u128_mul_overflow(&mut self) {
        if !self.runtime().is_acir() {
            return;
        }

        self.simple_optimization(|context| {
            context.insert_current_instruction();

            let Instruction::Binary(Binary {
                lhs,
                rhs,
                operator: BinaryOp::Mul { unchecked: false },
            }) = context.instruction()
            else {
                return;
            };

            let binary_type = context.dfg.type_of_value(*lhs).unwrap_numeric();
            let NumericType::Unsigned { bit_size: 128 } = binary_type else {
                return;
            };

            check_u128_mul_overflow(*lhs, *rhs, context);
        });
    }
}

fn check_u128_mul_overflow(
    lhs: ValueId,
    rhs: ValueId,
    context: &mut SimpleOptimizationContext<'_, '_>,
) {
    let dfg = &mut context.dfg;
    let lhs_value = dfg.get_numeric_constant(lhs);
    let rhs_value = dfg.get_numeric_constant(rhs);

    // If we multiply a constant value 2^n by an unknown u128 value we get at most `2^(n+128) - 2`.
    // If `n+128` does not overflow the maximum Field element value, there's no need to check for overflow.
    let max_const_value_that_does_not_overflow = 1_u128 << (FieldElement::max_num_bits() - 128);
    if lhs_value.is_some_and(|value| value.to_u128() < max_const_value_that_does_not_overflow)
        || rhs_value.is_some_and(|value| value.to_u128() < max_const_value_that_does_not_overflow)
    {
        return;
    }

    let block = context.block_id;
    let call_stack = dfg.get_instruction_call_stack_id(context.instruction_id);

    let u128 = NumericType::unsigned(128);
    let two_pow_64 = 1_u128 << 64;
    let two_pow_64 = dfg.make_constant(two_pow_64.into(), u128);
    let mul = BinaryOp::Mul { unchecked: true };

    // To check if a value is less than 2^64 we divide it by 2^64 and expect the result to be zero.
    let res = match (lhs_value, rhs_value) {
        (Some(_), Some(_)) => {
            // If both values are known at compile time, at this point we know it overflows
            dfg.make_constant(FieldElement::one(), u128)
        }
        (Some(_), None) => {
            // If only the left-hand side is known we just need to check that the right-hand side
            // isn't greater than 2^64
            let instruction =
                Instruction::Binary(Binary { lhs: rhs, rhs: two_pow_64, operator: BinaryOp::Div });
            dfg.insert_instruction_and_results(instruction, block, None, call_stack).first()
        }
        (None, Some(_)) => {
            // Same goes for the other side
            let instruction =
                Instruction::Binary(Binary { lhs, rhs: two_pow_64, operator: BinaryOp::Div });
            dfg.insert_instruction_and_results(instruction, block, None, call_stack).first()
        }
        (None, None) => {
            // Check both sides
            let instruction =
                Instruction::Binary(Binary { lhs, rhs: two_pow_64, operator: BinaryOp::Div });
            let divided_lhs =
                dfg.insert_instruction_and_results(instruction, block, None, call_stack).first();

            let instruction =
                Instruction::Binary(Binary { lhs: rhs, rhs: two_pow_64, operator: BinaryOp::Div });
            let divided_rhs =
                dfg.insert_instruction_and_results(instruction, block, None, call_stack).first();

            // Unchecked as operands are restricted to be less than 2^64 so multiplying them cannot overflow.
            let instruction =
                Instruction::Binary(Binary { lhs: divided_lhs, rhs: divided_rhs, operator: mul });
            dfg.insert_instruction_and_results(instruction, block, None, call_stack).first()
        }
    };

    // We must only check for overflow if the side effects var is active
    let predicate = Instruction::Cast(context.enable_side_effects, u128);
    let predicate = dfg.insert_instruction_and_results(predicate, block, None, call_stack).first();
    let res = Instruction::Binary(Binary { lhs: res, rhs: predicate, operator: mul });
    let res = dfg.insert_instruction_and_results(res, block, None, call_stack).first();

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
        ssa::{opt::assert_ssa_does_not_change, ssa_gen::Ssa},
    };

    #[test]
    fn does_not_insert_check_if_multiplying_lhs_will_not_overflow_field_element() {
        // The big value here is 2^254 - 2^128 - 1, which, when multiplied by any u128
        // won't overflow a Field element max value.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u128):
            v2 = mul u128 85070591730234615865843651857942052863, v0
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::check_u128_mul_overflow);
    }

    #[test]
    fn does_not_insert_check_if_multiplying_rhs_will_not_overflow_field_element() {
        // The big value here is 2^254 - 2^128 - 1, which, when multiplied by any u128
        // won't overflow a Field element max value.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u128):
            v2 = mul v0, u128 85070591730234615865843651857942052863
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::check_u128_mul_overflow);
    }

    #[test]
    fn inserts_check_for_lhs() {
        // The big value here is 2^254 - 2^128, which, when multiplied by any u128
        // might overflow a Field element max value.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u128):
            v2 = mul v0, u128 85070591730234615865843651857942052864
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.check_u128_mul_overflow();
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0(v0: u128):
            v2 = mul v0, u128 85070591730234615865843651857942052864
            v4 = div v0, u128 18446744073709551616
            constrain v4 == u128 0, "attempt to multiply with overflow"
            return
        }
        "#);
    }

    #[test]
    fn inserts_check_for_rhs() {
        // The big value here is 2^254 - 2^128, which, when multiplied by any u128
        // might overflow a Field element max value.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u128):
            v2 = mul u128 85070591730234615865843651857942052864, v0
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.check_u128_mul_overflow();
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0(v0: u128):
            v2 = mul u128 85070591730234615865843651857942052864, v0
            v4 = div v0, u128 18446744073709551616
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
            v4 = div v0, u128 18446744073709551616
            v5 = div v1, u128 18446744073709551616
            v6 = unchecked_mul v4, v5
            constrain v6 == u128 0, "attempt to multiply with overflow"
            return
        }
        "#);
    }

    #[test]
    fn inserts_assertion_failure_if_overflow_is_guaranteed() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v2 = mul u128 85070591730234615865843651857942052864, u128 85070591730234615865843651857942052865
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.check_u128_mul_overflow();
        // The multiplication remains, but it will be later removed by DIE
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0():
            v2 = mul u128 85070591730234615865843651857942052864, u128 85070591730234615865843651857942052865
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
        assert_ssa_does_not_change(src, Ssa::check_u128_mul_overflow);
    }

    #[test]
    fn predicate_overflow_on_lhs_potentially_overflowing() {
        // This code performs a u128 multiplication that overflows, under a condition.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u128, v1: u1):
            enable_side_effects v1
            v2 = mul v0, u128 85070591730234615865843651857942052864
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.flatten_cfg().check_u128_mul_overflow();
        // Below, the overflow check takes the 'enable_side_effects' value into account
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0(v0: u128, v1: u1):
            enable_side_effects v1
            v3 = mul v0, u128 85070591730234615865843651857942052864
            v5 = div v0, u128 18446744073709551616
            v6 = cast v1 as u128
            v7 = unchecked_mul v5, v6
            constrain v7 == u128 0, "attempt to multiply with overflow"
            return v3
        }
        "#);
    }

    #[test]
    fn predicate_overflow_on_guaranteed_overflow() {
        // This code performs a u128 multiplication that overflows, under a condition.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            jmpif v0 then: b1, else: b2
          b1():
            v2 = mul u128 340282366920938463463374607431768211455, u128 340282366920938463463374607431768211455
            jmp b2()
          b2():
            return v0
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.flatten_cfg().check_u128_mul_overflow();
        // Below, the overflow check takes the 'enable_side_effects' value into account
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0(v0: u1):
            enable_side_effects v0
            v2 = mul u128 340282366920938463463374607431768211455, u128 340282366920938463463374607431768211455
            v3 = cast v0 as u128
            constrain v0 == u1 0, "attempt to multiply with overflow"
            v5 = not v0
            enable_side_effects u1 1
            return v0
        }
        "#);
    }
}
