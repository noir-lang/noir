//! An SSA pass for ACIR functions that transforms "less than", "div" and "mod" operation on
//! signed integers into equivalent sequences of operations that rely on unsigned integers.
//!
//! The purpose of this pass is to avoid ACIR having to handle signed integers "less than",
//! "div" and "mod" operations (for simplicity), while also allowing further optimizations to
//! be done during subsequent SSA passes on the expanded instructions.
use acvm::FieldElement;

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
    /// Expands signed "less than", "div" and "mod" operations in ACIR to be done using
    /// unsigned operations.
    ///
    /// See [`expand_signed_math`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn expand_signed_math(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.expand_signed_math();
        }
        self
    }
}

impl Function {
    /// The structure of this pass is simple:
    /// Go through each block and re-insert all instructions, decomposing any signed
    /// "less than", "div" and "mod" operations to be done using unsigned types, but only if this
    /// is an ACIR function.
    fn expand_signed_math(&mut self) {
        if !self.dfg.runtime().is_acir() {
            return;
        }

        self.simple_optimization(|context| {
            let instruction_id = context.instruction_id;
            let instruction = context.instruction();

            // We only care about "less than"
            let Instruction::Binary(Binary {
                lhs,
                rhs,
                operator: operator @ (BinaryOp::Lt | BinaryOp::Div | BinaryOp::Mod),
            }) = instruction
            else {
                return;
            };

            // ... and it must be a signed integer operation.
            if !context.dfg.type_of_value(*lhs).is_signed() {
                return;
            }

            let lhs = *lhs;
            let rhs = *rhs;
            let operator = *operator;

            // We remove the current instruction, as we will need to replace it with multiple new instructions.
            context.remove_current_instruction();

            let [old_result] = context.dfg.instruction_result(instruction_id);

            let mut expansion_context = Context { context };
            let new_result = match operator {
                BinaryOp::Lt => expansion_context.insert_lt(lhs, rhs),
                BinaryOp::Div => expansion_context.insert_div(lhs, rhs),
                BinaryOp::Mod => expansion_context.insert_mod(lhs, rhs),
                _ => unreachable!("ICE: expand_signed_math called on non-lt/div/mod"),
            };

            context.replace_value(old_result, new_result);
        });

        #[cfg(debug_assertions)]
        expand_signed_math_post_check(self);
    }
}

struct Context<'m, 'dfg, 'mapping> {
    context: &'m mut SimpleOptimizationContext<'dfg, 'mapping>,
}

impl Context<'_, '_, '_> {
    fn insert_lt(&mut self, lhs: ValueId, rhs: ValueId) -> ValueId {
        // First cast lhs and rhs to their unsigned equivalents
        let bit_size = self.context.dfg.type_of_value(lhs).bit_size();
        let unsigned_typ = NumericType::unsigned(bit_size);
        let lhs_unsigned = self.insert_cast(lhs, unsigned_typ);
        let rhs_unsigned = self.insert_cast(rhs, unsigned_typ);

        // Check if lhs and rhs are positive or negative, respectively.
        // Values greater than or equal to 2^(bit_size-1) are negative so dividing by that would
        // give 0 (positive) or 1 (negative).
        let first_negative_value = self.numeric_constant(1_u128 << (bit_size - 1), unsigned_typ);
        let lhs_is_negative = self.insert_binary(lhs_unsigned, BinaryOp::Div, first_negative_value);
        let lhs_is_negative = self.insert_cast(lhs_is_negative, NumericType::bool());
        let rhs_is_negative = self.insert_binary(rhs_unsigned, BinaryOp::Div, first_negative_value);
        let rhs_is_negative = self.insert_cast(rhs_is_negative, NumericType::bool());

        // Do rhs and lhs have a different sign?
        let different_sign = self.insert_binary(lhs_is_negative, BinaryOp::Xor, rhs_is_negative);

        // Check lhs < rhs using their unsigned equivalents
        let unsigned_lt = self.insert_binary(lhs_unsigned, BinaryOp::Lt, rhs_unsigned);

        // It can be shown that the result is given by xor'ing the two results above:
        // - if lhs and rhs have the same sign (different_sign is 0):
        //   - if both are positive then the unsigned comparison is correct, xor'ing it with 0 gives
        //     the same result
        //   - if both are negative then the unsigned comparison is also correct, as, for example,
        //     for i8, -128 i8 is Field 128 and -1 i8 is Field 255 and `-128 < -1` and `128 < 255`
        // - if lhs and rhs have different signs (different_sign is 1):
        //   - if lhs is positive and rhs is negative then, as fields, rhs will be greater, but
        //     the result is the opposite (so xor'ing with 1 gives the correct result)
        //   - if lhs is negative and rhs is positive then, as fields, lhs will be greater, but
        //     the result is the opposite (so xor'ing with 1 gives the correct result)
        self.insert_binary(different_sign, BinaryOp::Xor, unsigned_lt)
    }

    fn insert_div(&mut self, lhs: ValueId, rhs: ValueId) -> ValueId {
        let is_division = true;
        self.insert_div_or_mod(lhs, rhs, is_division)
    }

    fn insert_mod(&mut self, lhs: ValueId, rhs: ValueId) -> ValueId {
        let is_division = false;
        self.insert_div_or_mod(lhs, rhs, is_division)
    }

    fn insert_div_or_mod(&mut self, lhs: ValueId, rhs: ValueId, is_division: bool) -> ValueId {
        // First cast lhs and rhs to their unsigned equivalents
        let bit_size = self.context.dfg.type_of_value(lhs).bit_size();
        let unsigned_typ = NumericType::unsigned(bit_size);
        let lhs_unsigned = self.insert_cast(lhs, unsigned_typ);
        let rhs_unsigned = self.insert_cast(rhs, unsigned_typ);

        // There's one condition that could generate an overflow: dividing the minimum
        // negative value by -1. For example dividing -128 i8 by -1 would give 128, but that
        // does not fit i8. So the first thing we do is check for this case.
        let min_negative_value = self.numeric_constant(1_u128 << (bit_size - 1), unsigned_typ);
        let minus_one = self.numeric_constant((1_u128 << bit_size) - 1, unsigned_typ);
        let lhs_is_min_negative_value =
            self.insert_binary(lhs_unsigned, BinaryOp::Eq, min_negative_value);
        let rhs_is_minus_one = self.insert_binary(rhs_unsigned, BinaryOp::Eq, minus_one);
        let min_overflow = self.insert_binary(
            lhs_is_min_negative_value,
            BinaryOp::Mul { unchecked: true },
            rhs_is_minus_one,
        );

        let zero = self.numeric_constant(0_u128, NumericType::bool());
        let message = if is_division {
            "Attempt to divide with overflow".to_string()
        } else {
            "Attempt to calculate the remainder with overflow".to_string()
        };
        self.insert_constrain(min_overflow, zero, Some(message.into()));

        // What about checking that the divisor is not zero? We don't need to explicitly check
        // this here because it'll be checked when doing the unsigned div/mod.

        // Check if lhs and rhs are positive or negative, respectively.
        // Values greater than or equal to 2^(bit_size-1) are negative so dividing by that would
        // give 0 (positive) or 1 (negative).
        let lhs_is_negative = self.insert_binary(lhs_unsigned, BinaryOp::Div, min_negative_value);
        let rhs_is_negative = self.insert_binary(rhs_unsigned, BinaryOp::Div, min_negative_value);

        // Here we compute the absolute values of lhs and rhs using their 2-complement
        let lhs_absolute =
            self.two_complement(lhs_unsigned, lhs_is_negative, unsigned_typ, bit_size);
        let rhs_absolute =
            self.two_complement(rhs_unsigned, rhs_is_negative, unsigned_typ, bit_size);

        // We then perform the division (or modulo) using the absolute values
        let operator = if is_division { BinaryOp::Div } else { BinaryOp::Mod };
        let absolute_result = self.insert_binary(lhs_absolute, operator, rhs_absolute);

        let lhs_is_negative = self.insert_cast(lhs_is_negative, NumericType::bool());

        // The result changes slightly depending on whether we are doing division or modulo.
        let result_is_negative = if is_division {
            // For division, the result is negative if lhs and rhs have different signs.
            let rhs_is_negative = self.insert_cast(rhs_is_negative, NumericType::bool());
            self.insert_binary(lhs_is_negative, BinaryOp::Xor, rhs_is_negative)
        } else {
            // For modulo, the result has the same sign as lhs
            lhs_is_negative
        };
        let result_is_negative = self.insert_cast(result_is_negative, unsigned_typ);

        // We return the 2-complement again if lhs and rhs have different signs, with the
        // intention of making the result be negative.
        let result_unsigned =
            self.two_complement(absolute_result, result_is_negative, unsigned_typ, bit_size);

        // If we divide, for example 4 i8 by -5, the absolute division will give 0.
        // Because the signs are different, if we do the two complement of 0 we'll get 256, which
        // is out of range. Here we take this case into account: if absolute_div is zero the result
        // should be zero, otherwise it should be that result.
        // Then, we need to multiply result_unsigned by `absolute_div != 0`.
        //
        // The same is true for modulo: -4 i8 mod 4 is 0, but taking its two-complement would give 256.
        let zero = self.numeric_constant(0_u128, unsigned_typ);
        let absolute_result_is_zero = self.insert_binary(absolute_result, BinaryOp::Eq, zero);
        let absolute_result_is_not_zero = self.insert_not(absolute_result_is_zero);
        let absolute_result_is_not_zero =
            self.insert_cast(absolute_result_is_not_zero, unsigned_typ);

        let result_unsigned = self.insert_binary(
            result_unsigned,
            BinaryOp::Mul { unchecked: true },
            absolute_result_is_not_zero,
        );

        // Make sure we return the signed type
        self.insert_cast(result_unsigned, NumericType::signed(bit_size))
    }

    /// Returns the 2-complement of `value`, given `value_is_negative` is 1 if the value is negative,
    /// and 0 if it's positive.
    ///
    /// The math here is:
    ///
    /// result = value + 2*((2^(bit_size - 1) - value)*value_is_negative)
    ///
    /// For example, for i8 we have bit_size = 8 so:
    ///
    /// result = value + 2*(128 - value)*value_is_negative
    ///
    /// If the value is positive, so value_is_negative = 0:
    ///
    /// result = value
    ///
    /// That is, the value stays the same.
    ///
    /// If value_is_negative = 1 we get:
    ///
    /// result = value + 2*(128 - value) = value + 256 - 2*value = 256 - value
    ///
    /// which effectively negates the value in 2-complement representation.
    fn two_complement(
        &mut self,
        value: ValueId,
        value_is_negative: ValueId,
        unsigned_type: NumericType,
        bit_size: u32,
    ) -> ValueId {
        let max_power_of_two = self.numeric_constant(1_u128 << (bit_size - 1), unsigned_type);

        let intermediate =
            self.insert_binary(max_power_of_two, BinaryOp::Sub { unchecked: true }, value);
        let intermediate =
            self.insert_binary(intermediate, BinaryOp::Mul { unchecked: true }, value_is_negative);
        let two = self.numeric_constant(2_u128, unsigned_type);
        let intermediate = self.insert_binary(intermediate, BinaryOp::Mul { unchecked: true }, two);
        self.insert_binary(value, BinaryOp::Add { unchecked: true }, intermediate)
    }

    /// Insert a numeric constant into the current function
    fn numeric_constant(&mut self, value: impl Into<FieldElement>, typ: NumericType) -> ValueId {
        self.context.dfg.make_constant(value.into(), typ)
    }

    /// Insert a not instruction at the end of the current block.
    /// Returns the result of the instruction.
    fn insert_not(&mut self, rhs: ValueId) -> ValueId {
        self.context.insert_instruction(Instruction::Not(rhs), None).first()
    }

    /// Insert a binary instruction at the end of the current block.
    /// Returns the result of the binary instruction.
    fn insert_binary(&mut self, lhs: ValueId, operator: BinaryOp, rhs: ValueId) -> ValueId {
        let instruction = Instruction::Binary(Binary { lhs, rhs, operator });
        self.context.insert_instruction(instruction, None).first()
    }

    /// Insert a cast instruction at the end of the current block.
    /// Returns the result of the cast instruction.
    fn insert_cast(&mut self, value: ValueId, typ: NumericType) -> ValueId {
        self.context.insert_instruction(Instruction::Cast(value, typ), None).first()
    }

    /// Insert a constrain instruction at the end of the current block.
    fn insert_constrain(
        &mut self,
        lhs: ValueId,
        rhs: ValueId,
        assert_message: Option<ConstrainError>,
    ) {
        self.context.insert_instruction(Instruction::Constrain(lhs, rhs, assert_message), None);
    }
}

/// Post-check condition for [Function::expand_signed_math].
///
/// Succeeds if:
///   - `func` does not contain any signed "less than" ops
///
/// Otherwise panics.
#[cfg(debug_assertions)]
fn expand_signed_math_post_check(func: &Function) {
    for block_id in func.reachable_blocks() {
        let instruction_ids = func.dfg[block_id].instructions();
        for instruction_id in instruction_ids {
            if let Instruction::Binary(binary) = &func.dfg[*instruction_id] {
                if func.dfg.type_of_value(binary.lhs).is_signed() && binary.operator == BinaryOp::Lt
                {
                    panic!("Checked signed 'less than' has not been removed")
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_ssa_snapshot,
        ssa::{interpreter::value::Value, opt::assert_ssa_does_not_change, ssa_gen::Ssa},
    };

    #[test]
    fn expands_signed_lt_in_acir() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: i8, v1: i8):
            v2 = lt v0, v1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.expand_signed_math();

        // Check that the expanded code works as expected
        let test_cases = [
            (10, 20, true),
            (20, 10, false),
            (-20, -10, true),
            (-10, -20, false),
            (-20, 10, true),
            (-10, 20, true),
            (10, -20, false),
            (20, -10, false),
        ];
        for (lhs, rhs, expected) in test_cases {
            let result = ssa.interpret(vec![Value::i8(lhs), Value::i8(rhs)]);
            assert!(result.is_ok());
            let result = result.unwrap();
            assert_eq!(result.len(), 1);
            assert_eq!(result[0], Value::bool(expected));
        }

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: i8, v1: i8):
            v2 = cast v0 as u8
            v3 = cast v1 as u8
            v5 = div v2, u8 128
            v6 = cast v5 as u1
            v7 = div v3, u8 128
            v8 = cast v7 as u1
            v9 = xor v6, v8
            v10 = lt v2, v3
            v11 = xor v9, v10
            return v11
        }
        ");
    }

    #[test]
    fn does_not_expand_signed_lt_in_brillig() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: i8, v1: i8):
            v2 = lt v0, v1
            return v2
        }
        ";
        assert_ssa_does_not_change(src, Ssa::expand_signed_math);
    }

    #[test]
    fn expands_signed_div_in_acir() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: i8, v1: i8):
            v2 = div v0, v1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.expand_signed_math();

        // Check that -128 i8 / -1 i8 overflows
        let result = ssa.interpret(vec![Value::i8(-128), Value::i8(-1)]);
        assert!(result.is_err());

        // Check that 10 i8 / 0 i8 overflows
        let result = ssa.interpret(vec![Value::i8(10), Value::i8(0)]);
        assert!(result.is_err());

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0(v0: i8, v1: i8):
            v2 = cast v0 as u8
            v3 = cast v1 as u8
            v5 = eq v2, u8 128
            v7 = eq v3, u8 255
            v8 = unchecked_mul v5, v7
            constrain v8 == u1 0, "Attempt to divide with overflow"
            v10 = div v2, u8 128
            v11 = div v3, u8 128
            v12 = unchecked_sub u8 128, v2
            v13 = unchecked_mul v12, v10
            v15 = unchecked_mul v13, u8 2
            v16 = unchecked_add v2, v15
            v17 = unchecked_sub u8 128, v3
            v18 = unchecked_mul v17, v11
            v19 = unchecked_mul v18, u8 2
            v20 = unchecked_add v3, v19
            v21 = div v16, v20
            v22 = cast v10 as u1
            v23 = cast v11 as u1
            v24 = xor v22, v23
            v25 = cast v24 as u8
            v26 = unchecked_sub u8 128, v21
            v27 = unchecked_mul v26, v25
            v28 = unchecked_mul v27, u8 2
            v29 = unchecked_add v21, v28
            v31 = eq v21, u8 0
            v32 = not v31
            v33 = cast v32 as u8
            v34 = unchecked_mul v29, v33
            v35 = cast v34 as i8
            return v35
        }
        "#);
    }

    #[test]
    fn does_not_expands_signed_div_in_brillig() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: i8, v1: i8):
            v2 = div v0, v1
            return v2
        }
        ";
        assert_ssa_does_not_change(src, Ssa::expand_signed_math);
    }

    #[test]
    fn expands_signed_mod_in_acir() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: i8, v1: i8):
            v2 = mod v0, v1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.expand_signed_math();

        // Check that -128 i8 / -1 i8 overflows
        let result = ssa.interpret(vec![Value::i8(-128), Value::i8(-1)]);
        assert!(result.is_err());

        // Check that 10 i8 / 0 i8 overflows
        let result = ssa.interpret(vec![Value::i8(10), Value::i8(0)]);
        assert!(result.is_err());

        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0(v0: i8, v1: i8):
            v2 = cast v0 as u8
            v3 = cast v1 as u8
            v5 = eq v2, u8 128
            v7 = eq v3, u8 255
            v8 = unchecked_mul v5, v7
            constrain v8 == u1 0, "Attempt to calculate the remainder with overflow"
            v10 = div v2, u8 128
            v11 = div v3, u8 128
            v12 = unchecked_sub u8 128, v2
            v13 = unchecked_mul v12, v10
            v15 = unchecked_mul v13, u8 2
            v16 = unchecked_add v2, v15
            v17 = unchecked_sub u8 128, v3
            v18 = unchecked_mul v17, v11
            v19 = unchecked_mul v18, u8 2
            v20 = unchecked_add v3, v19
            v21 = mod v16, v20
            v22 = cast v10 as u1
            v23 = unchecked_sub u8 128, v21
            v24 = unchecked_mul v23, v10
            v25 = unchecked_mul v24, u8 2
            v26 = unchecked_add v21, v25
            v28 = eq v21, u8 0
            v29 = not v28
            v30 = cast v29 as u8
            v31 = unchecked_mul v26, v30
            v32 = cast v31 as i8
            return v32
        }
        "#);
    }

    #[test]
    fn does_not_expands_signed_mod_in_brillig() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: i8, v1: i8):
            v2 = mod v0, v1
            return v2
        }
        ";
        assert_ssa_does_not_change(src, Ssa::expand_signed_math);
    }
}
