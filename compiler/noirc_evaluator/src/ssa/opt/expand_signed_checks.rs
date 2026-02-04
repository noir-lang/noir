//! An SSA pass that transforms the checked signed arithmetic operations add, sub and mul
//! into unchecked operations followed by explicit overflow checks.
//!
//! The purpose of this pass is to avoid ACIR and Brillig having to handle checked signed arithmetic
//! operations, while also allowing further optimizations to be done during subsequent
//! SSA passes on the expanded instructions.
use acvm::{FieldElement, acir::AcirField};

use crate::ssa::{
    ir::{
        function::Function,
        instruction::{Binary, BinaryOp, ConstrainError, Instruction},
        types::{NumericType, Type},
        value::ValueId,
    },
    ssa_gen::Ssa,
};

use super::simple_optimization::SimpleOptimizationContext;

impl Ssa {
    /// Expands signed arithmetic operations to include explicit overflow checks.
    ///
    /// See [`expand_signed_checks`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn expand_signed_checks(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.expand_signed_checks();
        }
        self
    }
}

impl Function {
    /// The structure of this pass is simple:
    /// Go through each block and re-insert all instructions, decomposing any checked signed arithmetic to have explicit
    /// overflow checks.
    fn expand_signed_checks(&mut self) {
        // TODO: consider whether we can implement this more efficiently in brillig.

        self.simple_optimization(|context| {
            let instruction_id = context.instruction_id;
            let instruction = context.instruction();

            // We only care about binary instructions that are signed arithmetic operations...
            let Instruction::Binary(Binary {
                lhs,
                rhs,
                operator:
                    operator @ (BinaryOp::Add { unchecked: false }
                    | BinaryOp::Sub { unchecked: false }
                    | BinaryOp::Mul { unchecked: false }),
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
                BinaryOp::Add { .. } => expansion_context.insert_add(lhs, rhs),
                BinaryOp::Sub { .. } => expansion_context.insert_sub(lhs, rhs),
                BinaryOp::Mul { .. } => expansion_context.insert_mul(lhs, rhs),
                _ => unreachable!("ICE: expand_signed_checks called on non-add/sub/mul"),
            };

            context.replace_value(old_result, new_result);
        });

        #[cfg(debug_assertions)]
        expand_signed_checks_post_check(self);
    }
}

struct Context<'m, 'dfg, 'mapping> {
    context: &'m mut SimpleOptimizationContext<'dfg, 'mapping>,
}

impl Context<'_, '_, '_> {
    fn insert_add(&mut self, lhs: ValueId, rhs: ValueId) -> ValueId {
        let bit_size = self.context.dfg.type_of_value(lhs).bit_size();
        let unchecked_result = self.insert_binary(lhs, BinaryOp::Add { unchecked: true }, rhs);
        let truncated = self.insert_truncate(unchecked_result, bit_size, bit_size + 1);
        let truncated = self.insert_safe_cast(truncated, NumericType::unsigned(bit_size));

        self.check_signed_overflow(
            truncated,
            lhs,
            rhs,
            BinaryOp::Add { unchecked: false },
            "add",
            bit_size,
        );
        self.insert_cast(truncated, self.context.dfg.type_of_value(lhs).unwrap_numeric())
    }

    fn insert_sub(&mut self, lhs: ValueId, rhs: ValueId) -> ValueId {
        let bit_size = self.context.dfg.type_of_value(lhs).bit_size();
        let unchecked_result = self.insert_binary(lhs, BinaryOp::Sub { unchecked: true }, rhs);
        let truncated = self.insert_truncate(unchecked_result, bit_size, bit_size + 1);
        let truncated = self.insert_safe_cast(truncated, NumericType::unsigned(bit_size));

        self.check_signed_overflow(
            truncated,
            lhs,
            rhs,
            BinaryOp::Sub { unchecked: false },
            "subtract",
            bit_size,
        );
        self.insert_cast(truncated, self.context.dfg.type_of_value(lhs).unwrap_numeric())
    }

    fn insert_mul(&mut self, lhs: ValueId, rhs: ValueId) -> ValueId {
        let bit_size = self.context.dfg.type_of_value(lhs).bit_size();
        let unchecked_result = self.insert_binary(lhs, BinaryOp::Mul { unchecked: true }, rhs);
        let unchecked_result_field =
            self.insert_cast(unchecked_result, NumericType::unsigned(2 * bit_size));
        let truncated = self.insert_truncate(unchecked_result_field, bit_size, 2 * bit_size);

        self.check_signed_overflow(
            truncated,
            lhs,
            rhs,
            BinaryOp::Mul { unchecked: false },
            "multiply",
            bit_size,
        );
        self.insert_cast(truncated, self.context.dfg.type_of_value(lhs).unwrap_numeric())
    }

    /// Insert constraints ensuring that the operation does not overflow the bit size of the result
    /// We assume that:
    /// lhs and rhs are signed integers of bit size bit_size
    /// result is the result of the operation, casted into an unsigned integer and not reduced
    ///
    /// overflow check for signed integer is less straightforward than for unsigned integers.
    /// We first compute the sign of the operands, and then we use the following rules:
    /// addition:   positive operands => result must be positive (i.e less than half the bit size)
    ///             negative operands => result must be negative (i.e not positive)
    ///             different sign => no overflow
    /// multiplication:     we check that the product of the operands' absolute values does not overflow the bit size
    ///                     then we check that the result has the proper sign, using the rule of signs
    fn check_signed_overflow(
        &mut self,
        result: ValueId,
        lhs: ValueId,
        rhs: ValueId,
        operator: BinaryOp,
        operation: &str,
        bit_size: u32,
    ) {
        let half_width = self.numeric_constant(
            FieldElement::from(2_i128.pow(bit_size - 1)),
            NumericType::unsigned(bit_size),
        );
        // We compute the sign of the operands. The overflow checks for signed integers depends on these signs
        let lhs_as_unsigned = self.insert_safe_cast(lhs, NumericType::unsigned(bit_size));
        let rhs_as_unsigned = self.insert_safe_cast(rhs, NumericType::unsigned(bit_size));
        let lhs_sign = self.insert_binary(lhs_as_unsigned, BinaryOp::Lt, half_width);
        let mut rhs_sign = self.insert_binary(rhs_as_unsigned, BinaryOp::Lt, half_width);
        if matches!(operator, BinaryOp::Sub { .. }) {
            // lhs - rhs = lhs + (-rhs)
            rhs_sign = self.insert_not(rhs_sign);
        }

        let message = format!("attempt to {operation} with overflow");

        // same_sign is true if both operands have the same sign
        let same_sign = self.insert_binary(lhs_sign, BinaryOp::Eq, rhs_sign);
        match operator {
            BinaryOp::Add { .. } | BinaryOp::Sub { .. } => {
                //Check the result has the same sign as its inputs
                let result_sign = self.insert_binary(result, BinaryOp::Lt, half_width);
                let sign_diff = self.insert_binary(result_sign, BinaryOp::Eq, lhs_sign);
                // Unchecked multiplication because boolean inputs
                let sign_diff_with_predicate =
                    self.insert_binary(sign_diff, BinaryOp::Mul { unchecked: true }, same_sign);
                let overflow_check = Instruction::Constrain(
                    sign_diff_with_predicate,
                    same_sign,
                    Some(message.into()),
                );
                self.context.insert_instruction(overflow_check, None);
            }
            BinaryOp::Mul { .. } => {
                // Overflow check for the multiplication:
                // First we compute the absolute value of operands, and their product
                let lhs_abs = self.absolute_value_helper(lhs, lhs_sign, bit_size);
                let rhs_abs = self.absolute_value_helper(rhs, rhs_sign, bit_size);
                // Unchecked mul because these are fields
                let product_field =
                    self.insert_binary(lhs_abs, BinaryOp::Mul { unchecked: true }, rhs_abs);
                // It must not already overflow the bit_size
                self.insert_range_check(product_field, bit_size, Some(message.clone()));
                let product = self.insert_safe_cast(product_field, NumericType::unsigned(bit_size));

                // Then we check the signed product fits in a signed integer of bit_size-bits
                let not_same = self.insert_not(same_sign);
                let not_same_sign_field =
                    self.insert_safe_cast(not_same, NumericType::unsigned(bit_size));
                // Unchecked add because adding 1 to half_width can't overflow
                let positive_maximum_with_offset = self.insert_binary(
                    half_width,
                    BinaryOp::Add { unchecked: true },
                    not_same_sign_field,
                );
                let product_overflow_check =
                    self.insert_binary(product, BinaryOp::Lt, positive_maximum_with_offset);

                let one = self.numeric_constant(FieldElement::one(), NumericType::bool());
                self.insert_constrain(product_overflow_check, one, Some(message.into()));
            }
            _ => unreachable!("operator {} should not overflow", operator),
        }
    }

    /// helper function which add instructions to the block computing the absolute value of the
    /// given signed integer input. When the input is negative, we return its two complement, and itself when it is positive.
    fn absolute_value_helper(&mut self, input: ValueId, sign: ValueId, bit_size: u32) -> ValueId {
        assert_eq!(self.context.dfg.type_of_value(sign), Type::bool());

        // We compute the absolute value of lhs
        let bit_width = FieldElement::from(2_i128.pow(bit_size));
        let bit_width = self.numeric_constant(bit_width, NumericType::NativeField);
        let sign_not = self.insert_not(sign);

        // We use unsafe casts here, this is fine as we're casting to a `field` type.
        let as_field = self.insert_safe_cast(input, NumericType::NativeField);
        let sign_field = self.insert_safe_cast(sign, NumericType::NativeField);

        // All of these operations are unchecked because they deal with fields
        let positive_predicate =
            self.insert_binary(sign_field, BinaryOp::Mul { unchecked: true }, as_field);
        let two_complement =
            self.insert_binary(bit_width, BinaryOp::Sub { unchecked: true }, as_field);
        let sign_not_field = self.insert_safe_cast(sign_not, NumericType::NativeField);
        let negative_predicate =
            self.insert_binary(sign_not_field, BinaryOp::Mul { unchecked: true }, two_complement);
        // Unchecked addition because either `positive_predicate` or `negative_predicate` will be 0
        self.insert_binary(
            positive_predicate,
            BinaryOp::Add { unchecked: true },
            negative_predicate,
        )
    }

    /// Inserts a cast instruction at the end of the current block and returns the results
    /// of the cast.
    ///
    /// Compared to `self.insert_cast`, this version will automatically truncate `value` to be a valid `typ`.
    pub(super) fn insert_safe_cast(&mut self, mut value: ValueId, typ: NumericType) -> ValueId {
        let incoming_type = self.context.dfg.type_of_value(value);

        let result = match (&incoming_type, typ) {
            // Casting to field is safe
            (_, NumericType::NativeField) => value,
            (
                Type::Numeric(NumericType::Signed { bit_size: incoming_type_size }),
                NumericType::Signed { bit_size: target_type_size },
            ) => {
                match target_type_size.cmp(incoming_type_size) {
                    std::cmp::Ordering::Less => {
                        // If target size is smaller, we do a truncation
                        self.insert_truncate(value, target_type_size, *incoming_type_size)
                    }
                    std::cmp::Ordering::Equal => value,
                    std::cmp::Ordering::Greater => {
                        // If target size is bigger, we do a sign extension:
                        // When the value is negative, it is represented in 2-complement form; `2^s-v`, where `s` is the incoming bit size and `v` is the absolute value
                        // Sign extension in this case will give `2^t-v`, where `t` is the target bit size
                        // So we simply convert `2^s-v` into `2^t-v` by adding `2^t-2^s` to the value when the value is negative.
                        // Casting s-bits signed v0 to t-bits will add the following instructions:
                        // v1 = cast v0 to 's-bits unsigned'
                        // v2 = lt v1, 2**(s-1)
                        // v3 = not(v1)
                        // v4 = cast v3 to 't-bits unsigned'
                        // v5 = v3 * (2**t - 2**s)
                        // v6 = cast v1 to 't-bits unsigned'
                        // return v6 + v5
                        let value_as_unsigned = self
                            .insert_safe_cast(value, NumericType::unsigned(*incoming_type_size));
                        let half_width = self.numeric_constant(
                            FieldElement::from(2_u128.pow(incoming_type_size - 1)),
                            NumericType::unsigned(*incoming_type_size),
                        );
                        // value_sign is 1 if the value is positive, 0 otherwise
                        let value_sign =
                            self.insert_binary(value_as_unsigned, BinaryOp::Lt, half_width);
                        let max_for_incoming_type_size = if *incoming_type_size == 128 {
                            u128::MAX
                        } else {
                            2_u128.pow(*incoming_type_size) - 1
                        };
                        let max_for_target_type_size = if target_type_size == 128 {
                            u128::MAX
                        } else {
                            2_u128.pow(target_type_size) - 1
                        };
                        let patch = self.numeric_constant(
                            FieldElement::from(
                                max_for_target_type_size - max_for_incoming_type_size,
                            ),
                            NumericType::unsigned(target_type_size),
                        );
                        let mut is_negative_predicate = self.insert_not(value_sign);
                        is_negative_predicate = self.insert_safe_cast(
                            is_negative_predicate,
                            NumericType::unsigned(target_type_size),
                        );
                        // multiplication by a boolean cannot overflow
                        let patch_with_sign_predicate = self.insert_binary(
                            patch,
                            BinaryOp::Mul { unchecked: true },
                            is_negative_predicate,
                        );
                        let value_as_unsigned = self.insert_cast(
                            value_as_unsigned,
                            NumericType::unsigned(target_type_size),
                        );
                        // Patch the bit sign, which gives a `target_type_size` bit size value, so it does not overflow.
                        self.insert_binary(
                            patch_with_sign_predicate,
                            BinaryOp::Add { unchecked: true },
                            value_as_unsigned,
                        )
                    }
                }
            }
            (
                Type::Numeric(NumericType::Unsigned { bit_size: incoming_type_size }),
                NumericType::Unsigned { bit_size: target_type_size },
            ) => {
                // If target size is smaller, we do a truncation
                if target_type_size < *incoming_type_size {
                    value = self.insert_truncate(value, target_type_size, *incoming_type_size);
                }
                value
            }
            // When casting a signed value to u1 we can truncate then cast
            (
                Type::Numeric(NumericType::Signed { bit_size: incoming_type_size }),
                NumericType::Unsigned { bit_size: 1 },
            ) => self.insert_truncate(value, 1, *incoming_type_size),
            // For mixed sign to unsigned or unsigned to sign;
            // 1. we cast to the required type using the same signedness
            // 2. then we switch the signedness
            (
                Type::Numeric(NumericType::Signed { bit_size: incoming_type_size }),
                NumericType::Unsigned { bit_size: target_type_size },
            ) => {
                if *incoming_type_size != target_type_size {
                    value = self.insert_safe_cast(value, NumericType::signed(target_type_size));
                }
                value
            }
            (
                Type::Numeric(NumericType::Unsigned { bit_size: incoming_type_size }),
                NumericType::Signed { bit_size: target_type_size },
            ) => {
                if *incoming_type_size != target_type_size {
                    value = self.insert_safe_cast(value, NumericType::unsigned(target_type_size));
                }
                value
            }
            (
                Type::Numeric(NumericType::NativeField),
                NumericType::Unsigned { bit_size: target_type_size },
            )
            | (
                Type::Numeric(NumericType::NativeField),
                NumericType::Signed { bit_size: target_type_size },
            ) => self.insert_truncate(value, target_type_size, FieldElement::max_num_bits()),
            _ => unreachable!("Invalid cast from {} to {}", incoming_type, typ),
        };
        self.insert_cast(result, typ)
    }

    /// Insert a numeric constant into the current function
    fn numeric_constant(&mut self, value: impl Into<FieldElement>, typ: NumericType) -> ValueId {
        self.context.dfg.make_constant(value.into(), typ)
    }

    /// Insert a binary instruction at the end of the current block.
    /// Returns the result of the binary instruction.
    fn insert_binary(&mut self, lhs: ValueId, operator: BinaryOp, rhs: ValueId) -> ValueId {
        let instruction = Instruction::Binary(Binary { lhs, rhs, operator });
        self.context.insert_instruction(instruction, None).first()
    }

    /// Insert a not instruction at the end of the current block.
    /// Returns the result of the instruction.
    fn insert_not(&mut self, rhs: ValueId) -> ValueId {
        self.context.insert_instruction(Instruction::Not(rhs), None).first()
    }

    /// Insert a truncate instruction at the end of the current block.
    /// Returns the result of the truncate instruction.
    fn insert_truncate(&mut self, value: ValueId, bit_size: u32, max_bit_size: u32) -> ValueId {
        self.context
            .insert_instruction(Instruction::Truncate { value, bit_size, max_bit_size }, None)
            .first()
    }

    /// Insert a cast instruction at the end of the current block.
    /// Returns the result of the cast instruction.
    fn insert_cast(&mut self, value: ValueId, typ: NumericType) -> ValueId {
        self.context.insert_instruction(Instruction::Cast(value, typ), None).first()
    }

    /// Insert a [`Instruction::RangeCheck`] instruction at the end of the current block.
    fn insert_range_check(
        &mut self,
        value: ValueId,
        max_bit_size: u32,
        assert_message: Option<String>,
    ) {
        self.context.insert_instruction(
            Instruction::RangeCheck { value, max_bit_size, assert_message },
            None,
        );
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

/// Post-check condition for [Function::expand_signed_checks].
///
/// Succeeds if:
///   - `func` does not contain any checked signed ops
///
/// Otherwise panics.
#[cfg(debug_assertions)]
fn expand_signed_checks_post_check(func: &Function) {
    for block_id in func.reachable_blocks() {
        let instruction_ids = func.dfg[block_id].instructions();
        for instruction_id in instruction_ids {
            if let Instruction::Binary(binary) = &func.dfg[*instruction_id] {
                if func.dfg.type_of_value(binary.lhs).is_signed() {
                    match binary.operator {
                        BinaryOp::Add { unchecked: false }
                        | BinaryOp::Sub { unchecked: false }
                        | BinaryOp::Mul { unchecked: false } => {
                            panic!("Checked signed binary operation has not been removed")
                        }
                        _ => (),
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_ssa_snapshot,
        ssa::{opt::assert_ssa_does_not_change, ssa_gen::Ssa},
    };

    #[test]
    fn expands_checked_add_instruction() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: i32, v1: i32):
            v2 = add v0, v1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.expand_signed_checks();
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0(v0: i32, v1: i32):
            v2 = unchecked_add v0, v1
            v3 = truncate v2 to 32 bits, max_bit_size: 33
            v4 = cast v3 as u32
            v5 = cast v0 as u32
            v6 = cast v1 as u32
            v8 = lt v5, u32 2147483648
            v9 = lt v6, u32 2147483648
            v10 = eq v8, v9
            v11 = lt v4, u32 2147483648
            v12 = eq v11, v8
            v13 = unchecked_mul v12, v10
            constrain v13 == v10, "attempt to add with overflow"
            return v3
        }
        "#);
    }

    #[test]
    fn expands_checked_sub_instruction() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: i32, v1: i32):
            v2 = sub v0, v1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.expand_signed_checks();
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0(v0: i32, v1: i32):
            v2 = unchecked_sub v0, v1
            v3 = truncate v2 to 32 bits, max_bit_size: 33
            v4 = cast v3 as u32
            v5 = cast v0 as u32
            v6 = cast v1 as u32
            v8 = lt v5, u32 2147483648
            v9 = lt v6, u32 2147483648
            v10 = not v9
            v11 = eq v8, v10
            v12 = lt v4, u32 2147483648
            v13 = eq v12, v8
            v14 = unchecked_mul v13, v11
            constrain v14 == v11, "attempt to subtract with overflow"
            return v3
        }
        "#);
    }

    #[test]
    fn expands_checked_mul_instruction() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: i32, v1: i32):
            v2 = mul v0, v1
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.expand_signed_checks();
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0(v0: i32, v1: i32):
            v2 = unchecked_mul v0, v1
            v3 = cast v2 as u64
            v4 = truncate v3 to 32 bits, max_bit_size: 64
            v5 = cast v0 as u32
            v6 = cast v1 as u32
            v8 = lt v5, u32 2147483648
            v9 = lt v6, u32 2147483648
            v10 = eq v8, v9
            v11 = not v8
            v12 = cast v0 as Field
            v13 = cast v8 as Field
            v14 = mul v13, v12
            v16 = sub Field 4294967296, v12
            v17 = cast v11 as Field
            v18 = mul v17, v16
            v19 = add v14, v18
            v20 = not v9
            v21 = cast v1 as Field
            v22 = cast v9 as Field
            v23 = mul v22, v21
            v24 = sub Field 4294967296, v21
            v25 = cast v20 as Field
            v26 = mul v25, v24
            v27 = add v23, v26
            v28 = mul v19, v27
            range_check v28 to 32 bits, "attempt to multiply with overflow"
            v29 = truncate v28 to 32 bits, max_bit_size: 254
            v30 = cast v29 as u32
            v31 = not v10
            v32 = cast v31 as u32
            v33 = unchecked_add u32 2147483648, v32
            v34 = lt v30, v33
            constrain v34 == u1 1, "attempt to multiply with overflow"
            v36 = cast v4 as i32
            return v36
        }
        "#);
    }

    #[test]
    fn ignores_unchecked_add() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: i32, v1: i32):
            v2 = unchecked_add v0, v1
            return v2
        }
        ";
        assert_ssa_does_not_change(src, Ssa::expand_signed_checks);
    }

    #[test]
    fn ignores_unchecked_sub() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: i32, v1: i32):
            v2 = unchecked_sub v0, v1
            return v2
        }
        ";
        assert_ssa_does_not_change(src, Ssa::expand_signed_checks);
    }

    #[test]
    fn ignores_unchecked_mul() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: i32, v1: i32):
            v2 = unchecked_mul v0, v1
            return v2
        }
        ";
        assert_ssa_does_not_change(src, Ssa::expand_signed_checks);
    }
}
