use acvm::AcirField;
use noirc_errors::call_stack::CallStackId;
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use num_traits::{One, Zero};

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
    if let Some(value) = lhs_value.clone() {
        if value.to_u128().expect("lhs is not a u128") < two_pow_64 {
            return;
        }
    }

    // Same goes for rhs
    if let Some(value) = rhs_value.clone() {
        if value.to_u128().expect("rhs is not a u128") < two_pow_64 {
            return;
        }
    }

    let u128 = NumericType::unsigned(128);
    let two_pow_64 = dfg.make_constant(two_pow_64.into(), u128);
    let mul = BinaryOp::Mul { unchecked: true };

    let res = if lhs_value.is_some() && rhs_value.is_some() {
        // If both values are known at compile time, at this point we know it overflows
        dfg.make_constant(BigInt::one(), u128)
    } else if lhs_value.is_some() {
        // If only the left-hand side is known we just need to check that the right-hand side
        // isn't greater than 2^64
        let instruction =
            Instruction::Binary(Binary { lhs: rhs, rhs: two_pow_64, operator: BinaryOp::Div });
        dfg.insert_instruction_and_results(instruction, block, None, call_stack).first()
    } else if rhs_value.is_some() {
        // Same goes for the other side
        let instruction =
            Instruction::Binary(Binary { lhs, rhs: two_pow_64, operator: BinaryOp::Div });
        dfg.insert_instruction_and_results(instruction, block, None, call_stack).first()
    } else {
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
    };

    let zero = dfg.make_constant(BigInt::zero(), u128);
    let instruction = Instruction::Cast(context.enable_side_effects, u128);
    let predicate =
        dfg.insert_instruction_and_results(instruction, block, None, call_stack).first();
    let instruction = Instruction::Binary(Binary { lhs: res, rhs: predicate, operator: mul });
    let res = dfg.insert_instruction_and_results(instruction, block, None, call_stack).first();
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
            v4 = div v0, u128 18446744073709551616
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
    #[test]
    fn predicate_overflow() {
        // This code performs a u128 multiplication that overflows, under a condition.
        let src = "
        acir(inline) fn main f0 {
        b0(v0: u1):
            jmpif v0 then: b1, else: b2
        b1():
            v2 = mul u128 340282366920938463463374607431768211455, u128 340282366920938463463374607431768211455	// src/main.nr:17:13
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
            constrain v3 == u128 0, "attempt to multiply with overflow"
            v5 = not v0
            enable_side_effects u1 1
            return v0
        }
        "#);
    }
}
