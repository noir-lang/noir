//! This SSA pass replaces Shl and Shr instructions in ACIR functions with more primitive
//! arithmetic instructions since ACIR doesn't directly support bit shifts.
//!
//! In all cases, if the shift amount is equal to or exceeds the operand's number of bits,
//! the result will be a constrain failure (attempt to bit-shift with overflow).
//!
//! ## Unsigned shift-right
//!
//! Shifting an unsigned integer to the right by N is the same as dividing by 2^N:
//!
//! ```ssa
//! // this:
//! v2 = shr v1, 3
//!
//! // is replaced with:
//! v2 = div v1, 8
//! ```
//!
//! If the shift amount is not a constant, 2^N is read from a constant array holding
//! `[2^0, 2^1, .., 2^(bit_size - 1)]`, indexed by the shift amount:
//!
//! ```ssa
//! // this, assuming v1 and v2 are u8:
//! v3 = shr v1, v2
//!
//! // is replaced with (overflow check omitted):
//! v4 = make_array [Field 1, Field 2, Field 4, Field 8, Field 16, Field 32, Field 64, Field 128] : [Field; 8]
//! v5 = cast v2 as u32
//! v6 = array_get v4, index v5 -> Field
//! v7 = cast v6 as u8
//! v8 = div v1, v7
//! ```
//!
//! Every shift by the same type within a function reads the same array, so its setup cost
//! is paid once per function and each shift then costs a single memory read.
//!
//! ## Unsigned shift-left
//!
//! Shifting an unsigned integer to the left by N is the same as multiplying by 2^N.
//! However, since that can overflow the target bit size, the operation is done using
//! Field, then truncated to the target bit size.
//!
//! ```ssa
//! // this, assuming v1 is a u8:
//! v2 = shl v1, 3
//!
//! // is replaced with:
//! v2 = cast v1 as Field
//! v3 = mul v2, 8
//! v4 = truncate v3 to 8 bits, max_bit_size: 11
//! v5 = cast v4 as u8
//! ```
//!
//! Like in the previous case, if the shift amount is not a constant 2^N is read from
//! a constant array of powers of two.
//!
//! ## Signed shift-right
//!
//! This case is similar to unsigned shift-right except that for negative numbers we
//! slightly adjust the value to shift, then adjust it again after performing the division,
//! so the results are the expected ones.
//!
//! ## Signed shift-left
//!
//! This case is similar to unsigned shift-left.
use std::{borrow::Cow, sync::Arc};

use acvm::{
    FieldElement,
    acir::{AcirField, brillig::lengths::SemanticLength},
};

use crate::ssa::{
    ir::{
        function::Function,
        instruction::{Binary, BinaryOp, Endian, Instruction, Intrinsic},
        types::{NumericType, Type},
        value::ValueId,
    },
    ssa_gen::Ssa,
};

use super::simple_optimization::SimpleOptimizationContext;

impl Ssa {
    /// Go through every ACIR function replacing bit shifts with more primitive arithmetic operations,
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn remove_bit_shifts(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.remove_bit_shifts();
        }
        self
    }
}

impl Function {
    /// If this is an ACIR function, go through every instruction, replacing bit shifts with
    /// more primitive arithmetic operations,
    fn remove_bit_shifts(&mut self) {
        if !self.runtime().is_acir() {
            return;
        }

        self.simple_optimization(|context| {
            let instruction_id = context.instruction_id;
            let instruction = context.instruction();

            let Instruction::Binary(Binary { lhs, rhs, operator }) = instruction else {
                return;
            };

            if !matches!(operator, BinaryOp::Shl | BinaryOp::Shr) {
                return;
            }

            let lhs = *lhs;
            let rhs = *rhs;
            let operator = *operator;

            context.remove_current_instruction();

            let [old_result] = context.dfg.instruction_result(instruction_id);

            let mut bitshift_context = Context { context };
            bitshift_context.enforce_bitshift_rhs_lt_bit_size(rhs);

            let new_result = if operator == BinaryOp::Shl {
                bitshift_context.insert_wrapping_shift_left(lhs, rhs)
            } else {
                bitshift_context.insert_shift_right(lhs, rhs)
            };

            context.replace_value(old_result, new_result);
        });

        #[cfg(debug_assertions)]
        remove_bit_shifts_post_check(self);
    }
}

struct Context<'m, 'dfg, 'mapping> {
    context: &'m mut SimpleOptimizationContext<'dfg, 'mapping>,
}

impl Context<'_, '_, '_> {
    /// Insert SSA instructions which computes lhs << rhs by doing lhs*2^rhs
    /// and truncate the result to `bit_size`.
    fn insert_wrapping_shift_left(&mut self, lhs: ValueId, rhs: ValueId) -> ValueId {
        let typ = self.context.dfg.type_of_value(lhs).unwrap_numeric();
        let max_lhs_bits = self.context.dfg.get_value_max_num_bits(lhs);
        let max_bit_shift_size = self.context.dfg.get_numeric_constant(rhs).map_or_else(
            || {
                // If we don't know `rhs`'s value then it could be anything up to the number
                // of bits in the type, e.g. u32 means shifting by up to 32 bits as otherwise we get overflow.
                // get_value_max_num_bits might indicate that the underlying type actually has less
                // bits than the RHS; e.g. because it was upcast from a u1 to a u32.
                // The maximum value we can get would be `2^bits - 1`, but `u1` is the only interesting case,
                // because even for u8 the max value is 255, larger than anything we get based on type.
                if self.context.dfg.get_value_max_num_bits(rhs) == 1 {
                    1
                } else {
                    self.context.dfg.type_of_value(rhs).bit_size()
                }
            },
            |rhs_constant| {
                // Happy case is that we know precisely by how many bits we're shifting by.
                rhs_constant.to_u128() as u32
            },
        );

        // We cap the maximum number of bits here to ensure that we don't try and truncate using a
        // `max_bit_size` greater than what's allowable by the underlying `FieldElement` as this is meaningless.
        //
        // If `max_lhs_bits + max_bit_shift_size` were ever to exceed `FieldElement::max_num_bits()`,
        // then the constraint on `rhs` in `self.two_pow` should be broken.
        let max_bit = std::cmp::min(
            max_lhs_bits.checked_add(max_bit_shift_size).unwrap_or(FieldElement::max_num_bits()),
            FieldElement::max_num_bits(),
        );
        // For signed types the high bit is the sign bit, so a value whose unsigned-bit-count
        // equals `bit_size` does not fit in the positive signed range and would cause the
        // subsequent `unchecked_mul` to overflow the target type.
        let happy_path_max_bit = match typ {
            NumericType::Signed { bit_size } => bit_size - 1,
            _ => typ.bit_size::<FieldElement>(),
        };
        if max_bit <= happy_path_max_bit {
            // If the result is guaranteed to fit in the target type we can simply multiply
            let pow = self.two_pow(rhs);
            let pow = self.insert_cast(pow, typ);
            // Unchecked mul as it can't overflow
            self.insert_binary(lhs, BinaryOp::Mul { unchecked: true }, pow)
        } else if max_bit < FieldElement::max_num_bits() {
            // If the result fits in a FieldElement we can multiply in Field, then truncate
            let pow = self.two_pow(rhs);
            let lhs_field = self.insert_cast(lhs, NumericType::NativeField);
            // Unchecked mul as this is a wrapping operation that we later truncate
            let result = self.insert_binary(lhs_field, BinaryOp::Mul { unchecked: true }, pow);
            let result = self.insert_truncate(result, typ.bit_size::<FieldElement>(), max_bit);
            self.insert_cast(result, typ)
        } else {
            // Otherwise, the result might not fit in a FieldElement.
            // For this, if we have to do `lhs << rhs` we can first shift by half of `rhs`, truncate,
            // then shift by `rhs - half_of_rhs` and truncate again.
            assert!(typ.bit_size::<FieldElement>() <= 128);

            let two = self.numeric_constant(FieldElement::from(2_u32), typ);

            // rhs_divided_by_two = rhs / 2
            let rhs_divided_by_two = self.insert_binary(rhs, BinaryOp::Div, two);

            // rhs_remainder = rhs - rhs_divided_by_two
            let rhs_remainder =
                self.insert_binary(rhs, BinaryOp::Sub { unchecked: true }, rhs_divided_by_two);

            // pow1 = 2^rhs_divided_by_two
            // pow2 = 2^rhs_remainder
            let pow1 = self.two_pow(rhs_divided_by_two);
            let pow2 = self.two_pow(rhs_remainder);

            // result = lhs * pow1 * pow2 = lhs * 2^rhs_divided_by_two * 2^rhs_remainder
            //        = lhs * 2^(rhs_divided_by_two + rhs_remainder) = lhs * 2^rhs
            let lhs_field = self.insert_cast(lhs, NumericType::NativeField);
            let result = self.insert_binary(lhs_field, BinaryOp::Mul { unchecked: true }, pow1);
            let result = self.insert_truncate(result, typ.bit_size::<FieldElement>(), max_bit);
            let result = self.insert_binary(result, BinaryOp::Mul { unchecked: true }, pow2);
            let result = self.insert_truncate(result, typ.bit_size::<FieldElement>(), max_bit);
            self.insert_cast(result, typ)
        }
    }

    /// Insert SSA instructions which computes lhs >> rhs by doing lhs/2^rhs
    ///
    /// For negative signed integers, we do the shifting using a technique based on how dividing a
    /// 2-complement value can be done by converting to the 1-complement representation of lhs,
    /// shifting, then converting back the result to the 2-complement representation.
    ///
    /// To understand the algorithm, take a look at how division works on pages 7-8 of
    /// <https://dspace.mit.edu/bitstream/handle/1721.1/6090/AIM-378.pdf>
    ///
    /// Division for a negative number represented as a 2-complement is implemented by the following steps:
    /// 1. Convert to 1-complement by subtracting 1 from the value
    /// 2. Shift right by the number of bits corresponding to the divisor
    /// 3. Convert back to 2-complement by adding 1 to the result
    ///
    /// That's division in terms of shifting; we need shifting in terms of division. The following steps show how:
    /// * `DIV(a) = SHR(a-1)+1`
    /// * `SHR(a-1) = DIV(a)-1`
    /// * `SHR(a) = DIV(a+1)-1`
    ///
    /// Hence we handle negative values in shifting by:
    /// 1. Adding 1 to the value
    /// 2. Dividing by 2^rhs
    /// 3. Subtracting 1 from the result
    fn insert_shift_right(&mut self, lhs: ValueId, rhs: ValueId) -> ValueId {
        let lhs_typ = self.context.dfg.type_of_value(lhs).unwrap_numeric();

        let pow = self.two_pow(rhs);
        let pow = self.insert_cast(pow, lhs_typ);

        match lhs_typ {
            NumericType::Unsigned { .. } => {
                // unsigned right bit shift is just a normal division
                self.insert_binary(lhs, BinaryOp::Div, pow)
            }
            NumericType::Signed { bit_size } => {
                // Get the sign of the operand; positive signed operand will just do a division as well
                let unsigned_typ = NumericType::unsigned(bit_size);
                let lhs_as_unsigned = self.insert_cast(lhs, unsigned_typ);

                // The sign will be 0 for positive numbers and 1 for negatives, so it covers both cases.
                // To compute this we check if the value, as a Field, is greater or equal than the maximum
                // value that is considered positive, that is, 2^(bit_size-1)-1: 2^(bit_size-1)-1 < lhs_as_field
                let max_positive = (1_u128 << (bit_size - 1)) - 1;
                let max_positive = self.numeric_constant(max_positive, unsigned_typ);
                let lhs_sign = self.insert_binary(max_positive, BinaryOp::Lt, lhs_as_unsigned);
                let lhs_sign_as_field = self.insert_cast(lhs_sign, NumericType::NativeField);
                let lhs_as_field = self.insert_cast(lhs, NumericType::NativeField);
                // For negative numbers, we prepare for the division using a wrapping addition of a + 1. Unchecked add as these are fields.
                let add = BinaryOp::Add { unchecked: true };
                let div_complement = self.insert_binary(lhs_sign_as_field, add, lhs_as_field);
                let div_complement = self.insert_truncate(div_complement, bit_size, bit_size + 1);
                let div_complement = self.insert_cast(div_complement, lhs_typ);
                // Performs the division on the adjusted complement (or the operand if positive)
                let shifted_complement = self.insert_binary(div_complement, BinaryOp::Div, pow);
                // For negative numbers, convert back to 2-complement by subtracting 1.

                // Cast to Field and add 2^bit_size to handle underflow; the subsequent truncate
                // to bit_size bits corrects the extra 2^bit_size.
                let shifted_complement_field =
                    self.insert_cast(shifted_complement, NumericType::NativeField);
                let modulus = self.field_constant(
                    FieldElement::from(2u32).pow(&FieldElement::from(u128::from(bit_size))),
                );
                let shifted = self.insert_binary(
                    shifted_complement_field,
                    BinaryOp::Add { unchecked: true },
                    modulus,
                );
                let shifted = self.insert_binary(
                    shifted,
                    BinaryOp::Sub { unchecked: true },
                    lhs_sign_as_field,
                );
                let truncated = self.insert_truncate(shifted, bit_size, bit_size + 1);
                self.insert_cast(truncated, lhs_typ)
            }

            NumericType::NativeField => unreachable!("Bit shifts are disallowed on `Field` type"),
        }
    }

    /// Computes 2^exponent as a Field.
    ///
    /// A non-constant exponent indexes a constant array of the powers of two below the exponent
    /// type's bit size. An exponent known to be 0 or 1 (e.g. upcast from a `u1`) is instead
    /// computed via square&multiply on its single bit, which avoids the array's setup cost.
    /// Pseudo-code of square&multiply:
    /// ```text
    /// let mut r = 1;
    /// let exponent_bits = to_bits(exponent);
    /// for i in 1 .. bit_size + 1 {
    ///     let r_squared = r * r;
    ///     let b = exponent_bits[bit_size - i];
    ///     r = if b { 2 * r_squared } else { r_squared };
    /// }
    /// ```
    fn two_pow(&mut self, exponent: ValueId) -> ValueId {
        // Require that exponent < bit_size, ensuring that `pow` returns a value consistent with `lhs`'s type.
        let max_bit_size = self.context.dfg.type_of_value(exponent).bit_size();

        if let Some(exponent_const) = self.context.dfg.get_numeric_constant(exponent) {
            let exponent_const_as_u32 = exponent_const.try_to_u32();
            let pow = if exponent_const_as_u32.is_none_or(|v| v >= max_bit_size) {
                // If the exponent is guaranteed to overflow the value returned here doesn't matter as
                // `enforce_bitshift_rhs_lt_bit_size` will trigger a constrain failure. We don't want to return
                // `2^exponent` here as that value is later cast to the target type and it would be an invalid cast.
                FieldElement::zero()
            } else {
                FieldElement::from(2u32).pow(&exponent_const)
            };
            return self.field_constant(pow);
        }

        // When shifting, for instance, `u32` values the maximum allowed value is 31, one less than the bit size.
        // Representing the maximum value requires 5 bits, which is log2(32), so any `u32` exponent will require
        // at most 5 bits. Similarly, `u64` values will require at most 6 bits, etc.
        // Using `get_value_max_num_bits` here could work, though in practice:
        // - constant exponents are handled in the `if` above
        // - if a smaller type was upcasted, for example `u8` to `u32`, an `u8` can hold values up to 256
        //   which is even larger than the largest unsigned type u128, so nothing better can be done here
        // - the exception would be casting a `u1` to a larger type, where we know the exponent can be
        //   either zero or one, which we special-case here
        let max_exponent_bits = if self.context.dfg.get_value_max_num_bits(exponent) == 1 {
            1
        } else {
            let exponent_bit_size = self.context.dfg.type_of_value(exponent).bit_size();
            assert!(
                exponent_bit_size.is_power_of_two(),
                "ICE: exponent type bit size is expected to be a power of two"
            );
            exponent_bit_size.ilog2()
        };
        if max_exponent_bits > 1 {
            return self.two_pow_from_table(exponent);
        }

        let result_types =
            vec![Type::Array(Arc::new(vec![Type::bool()]), SemanticLength(max_exponent_bits))];

        // A call to ToBits can only be done with a field argument (exponent is always u8 here)
        let exponent_as_field = self.insert_cast(exponent, NumericType::NativeField);
        let to_bits = self.context.dfg.import_intrinsic(Intrinsic::ToBits(Endian::Little));
        let exponent_bits = self.insert_call(to_bits, vec![exponent_as_field], result_types);

        let exponent_bits = exponent_bits[0];
        let one = self.field_constant(FieldElement::one());
        let two = self.field_constant(FieldElement::from(2u32));
        let mut r = one;
        // All operations are unchecked as we're acting on Field types (which are always unchecked)
        for i in 1..max_exponent_bits + 1 {
            let idx = self.numeric_constant(
                FieldElement::from(i128::from(max_exponent_bits - i)),
                NumericType::length_type(),
            );
            let b = self.insert_array_get(exponent_bits, idx, Type::bool());
            let not_b = self.insert_not(b);
            let b = self.insert_cast(b, NumericType::NativeField);
            let not_b = self.insert_cast(not_b, NumericType::NativeField);

            let r_squared = self.insert_binary(r, BinaryOp::Mul { unchecked: true }, r);
            let r1 = self.insert_binary(r_squared, BinaryOp::Mul { unchecked: true }, not_b);
            let a = self.insert_binary(r_squared, BinaryOp::Mul { unchecked: true }, two);
            let r2 = self.insert_binary(a, BinaryOp::Mul { unchecked: true }, b);
            r = self.insert_binary(r1, BinaryOp::Add { unchecked: true }, r2);
        }

        assert!(
            matches!(self.context.dfg.type_of_value(r).unwrap_numeric(), NumericType::NativeField),
            "ICE: pow is expected to always return a NativeField"
        );

        r
    }

    /// Reads 2^exponent from the constant array `[2^0, 2^1, .., 2^(bit_size - 1)]`, where `bit_size`
    /// is the bit size of the exponent's type.
    ///
    /// The read fails for an exponent of `bit_size` or more, which is exactly the overflow condition
    /// enforced by `enforce_bitshift_rhs_lt_bit_size`. That check's range constraint is then
    /// redundant in ACIR, where the optimizer moves its message onto the read.
    fn two_pow_from_table(&mut self, exponent: ValueId) -> ValueId {
        let exponent_type = self.context.dfg.type_of_value(exponent).unwrap_numeric();
        let bit_size = exponent_type.bit_size::<FieldElement>();

        let two = FieldElement::from(2u32);
        let elements = (0..bit_size)
            .map(|i| self.field_constant(two.pow(&FieldElement::from(u128::from(i)))))
            .collect();
        let table_type = Type::Array(Arc::new(vec![Type::field()]), SemanticLength(bit_size));
        let table = self
            .context
            .insert_instruction(
                Instruction::MakeArray { elements, typ: table_type.clone() },
                Some(vec![table_type]),
            )
            .first();

        let index_type = NumericType::length_type();
        let mut index = self.insert_cast(exponent, NumericType::unsigned(bit_size));
        if bit_size > index_type.bit_size::<FieldElement>() {
            // Narrowing the exponent to the index type requires it to be known to fit. The
            // table read imposes the same bound, so this range check is free in ACIR.
            self.context.insert_instruction(
                Instruction::RangeCheck {
                    value: index,
                    max_bit_size: bit_size.ilog2(),
                    assert_message: None,
                },
                None,
            );
        }
        index = self.insert_cast(index, index_type);
        self.insert_array_get(table, index, Type::field())
    }

    /// Insert constraints ensuring that the right-hand side of a bit-shift operation
    /// is less than the bit size of the left-hand side.
    /// We check against rhs bit size because this function does not see lhs,
    /// but this is effectively lhs bit size because lhs and rhs have the same type.
    ///
    /// Bit sizes are powers of two, so `rhs < bit_size` is exactly `rhs` fitting in
    /// `log2(bit_size)` bits. This is enforced with a range check, which is much cheaper
    /// than a comparison at the full width of the type.
    ///
    /// The power of two computed by `two_pow` decomposes its exponent into `log2(bit_size)`
    /// bits, which by itself rejects an out-of-range `rhs`, but this check is still required:
    /// - it fails with "attempt to bit-shift with overflow" rather than a bit decomposition failure.
    /// - a left shift whose result may not fit in a field calls `two_pow` on `rhs / 2` and
    ///   `rhs - rhs / 2`, so those decompositions do not bound `rhs` itself.
    fn enforce_bitshift_rhs_lt_bit_size(&mut self, rhs: ValueId) {
        let rhs_type = self.context.dfg.type_of_value(rhs);
        let bit_size = rhs_type.bit_size();
        assert!(
            bit_size.is_power_of_two(),
            "ICE: bit-shift rhs type bit size is expected to be a power of two"
        );

        let unsigned_typ = NumericType::unsigned(bit_size);
        let rhs = self.insert_cast(rhs, unsigned_typ);
        let assert_message = "attempt to bit-shift with overflow".to_owned();
        self.insert_range_check(rhs, bit_size.ilog2(), assert_message);
    }

    fn field_constant(&mut self, constant: FieldElement) -> ValueId {
        self.context.dfg.make_constant(constant, NumericType::NativeField)
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

    /// Insert a range check instruction at the end of the current block.
    fn insert_range_check(&mut self, value: ValueId, max_bit_size: u32, assert_message: String) {
        let instruction =
            Instruction::RangeCheck { value, max_bit_size, assert_message: Some(assert_message) };
        self.context.insert_instruction(instruction, None);
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

    /// Insert a call instruction at the end of the current block and return
    /// the results of the call.
    fn insert_call(
        &mut self,
        func: ValueId,
        arguments: Vec<ValueId>,
        result_types: Vec<Type>,
    ) -> Cow<[ValueId]> {
        self.context
            .insert_instruction(Instruction::Call { func, arguments }, Some(result_types))
            .results()
    }

    /// Insert an instruction to extract an element from an array
    fn insert_array_get(&mut self, array: ValueId, index: ValueId, element_type: Type) -> ValueId {
        let element_type = Some(vec![element_type]);
        let instruction = Instruction::ArrayGet { array, index };
        self.context.insert_instruction(instruction, element_type).first()
    }
}

/// Post-check condition for [`Function::remove_bit_shifts`].
///
/// Panics if:
///   - Any ACIR function contains bitshift instructions.
#[cfg(debug_assertions)]
fn remove_bit_shifts_post_check(func: &Function) {
    if func.runtime().is_acir() {
        // All bit shifts should be removed in ACIR functions
        super::checks::for_each_instruction(func, |instruction, _dfg| {
            super::checks::assert_not_bit_shift(instruction);
        });
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use crate::{
        assert_ssa_snapshot,
        ssa::{
            interpreter::{errors::InterpreterError, value::Value},
            opt::assert_pass_does_not_affect_execution,
            ssa_gen::{Ssa, validate_ssa},
        },
    };

    mod unsigned {
        use super::*;

        #[test]
        fn removes_shl_with_constant_rhs() {
            let src = "
            acir(inline) fn main f0 {
              b0(v0: u32):
                v2 = shl v0, u32 2
                return v2
            }
            ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();
            assert_ssa_snapshot!(ssa, @r"
            acir(inline) fn main f0 {
              b0(v0: u32):
                v1 = cast v0 as Field
                v3 = mul v1, Field 4
                v4 = truncate v3 to 32 bits, max_bit_size: 34
                v5 = cast v4 as u32
                return v5
            }
            "
            );
        }

        #[test]
        fn removes_shl_with_non_constant_rhs() {
            let src = "
            acir(inline) fn main f0 {
              b0(v0: u32, v1: u32):
                v2 = shl v0, v1
                return v2
            }
            ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();

            assert_ssa_snapshot!(ssa, @r#"
            acir(inline) fn main f0 {
              b0(v0: u32, v1: u32):
                range_check v1 to 5 bits, "attempt to bit-shift with overflow"
                v34 = make_array [Field 1, Field 2, Field 4, Field 8, Field 16, Field 32, Field 64, Field 128, Field 256, Field 512, Field 1024, Field 2048, Field 4096, Field 8192, Field 16384, Field 32768, Field 65536, Field 131072, Field 262144, Field 524288, Field 1048576, Field 2097152, Field 4194304, Field 8388608, Field 16777216, Field 33554432, Field 67108864, Field 134217728, Field 268435456, Field 536870912, Field 1073741824, Field 2147483648] : [Field; 32]
                v35 = array_get v34, index v1 -> Field
                v36 = cast v0 as Field
                v37 = mul v36, v35
                v38 = truncate v37 to 32 bits, max_bit_size: 64
                v39 = cast v38 as u32
                return v39
            }
            "#);
        }

        #[test]
        fn removes_shl_with_non_constant_rhs_casted_from_smaller_type() {
            let src = "
            acir(inline) fn main f0 {
              b0(v0: u32, v1: u8):
                v2 = cast v1 as u32
                v3 = shl v0, v2
                return v3
            }
            ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();

            // `max_bit_size` in `truncate` has to be 64, because even though `u8` is just 8 bits,
            // the maximum value can be 255, which would clearly overflow, as anything over 32 would.
            assert_ssa_snapshot!(ssa, @r#"
            acir(inline) fn main f0 {
              b0(v0: u32, v1: u8):
                v2 = cast v1 as u32
                range_check v2 to 5 bits, "attempt to bit-shift with overflow"
                v35 = make_array [Field 1, Field 2, Field 4, Field 8, Field 16, Field 32, Field 64, Field 128, Field 256, Field 512, Field 1024, Field 2048, Field 4096, Field 8192, Field 16384, Field 32768, Field 65536, Field 131072, Field 262144, Field 524288, Field 1048576, Field 2097152, Field 4194304, Field 8388608, Field 16777216, Field 33554432, Field 67108864, Field 134217728, Field 268435456, Field 536870912, Field 1073741824, Field 2147483648] : [Field; 32]
                v36 = array_get v35, index v2 -> Field
                v37 = cast v0 as Field
                v38 = mul v37, v36
                v39 = truncate v38 to 32 bits, max_bit_size: 64
                v40 = cast v39 as u32
                return v40
            }
            "#);
        }

        #[test]
        fn removes_shl_with_non_constant_rhs_casted_from_u1() {
            let src = "
            acir(inline) fn main f0 {
              b0(v0: u64, v1: u1):
                v2 = cast v1 as u8
                v3 = cast v2 as u32
                v4 = cast v3 as u64
                v5 = shl v0, v4
                return v5
            }
            ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();

            // What we are casting with can originally be only 1 bit,
            // so in the truncate we expect to shift with less than
            // if only considered the 64 bits based on the type.
            assert_ssa_snapshot!(ssa, @"
            acir(inline) fn main f0 {
              b0(v0: u64, v1: u1):
                v2 = cast v1 as u8
                v3 = cast v2 as u32
                v4 = cast v3 as u64
                v5 = cast v1 as Field
                v7 = call to_le_bits(v5) -> [u1; 1]
                v9 = array_get v7, index u32 0 -> u1
                v10 = not v9
                v11 = cast v9 as Field
                v12 = cast v10 as Field
                v14 = mul Field 2, v11
                v15 = add v12, v14
                v16 = cast v0 as Field
                v17 = mul v16, v15
                v18 = truncate v17 to 64 bits, max_bit_size: 65
                v19 = cast v18 as u64
                return v19
            }
            ");
        }

        #[test]
        fn does_not_generate_invalid_truncation_on_overflowing_bitshift() {
            // We want to ensure that the `max_bit_size` of the truncation does not exceed the field size.
            let src = "
            acir(inline) fn main f0 {
              b0(v0: u32):
                v2 = shl v0, u32 255
                return v2
            }
            ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();
            assert_ssa_snapshot!(ssa, @r#"
            acir(inline) fn main f0 {
              b0(v0: u32):
                constrain u1 0 == u1 1, "attempt to bit-shift with overflow"
                v3 = cast v0 as Field
                return u32 0
            }
            "#);
        }

        #[test]
        fn removes_shr_with_constant_rhs() {
            let src = "
            acir(inline) fn main f0 {
              b0(v0: u32):
                v2 = shr v0, u32 2
                return v2
            }
            ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();
            assert_ssa_snapshot!(ssa, @r"
            acir(inline) fn main f0 {
              b0(v0: u32):
                v2 = div v0, u32 4
                return v2
            }
            ");
        }

        #[test]
        fn removes_shr_with_non_constant_rhs() {
            let src = "
            acir(inline) fn main f0 {
              b0(v0: u32, v1: u32):
                v2 = shr v0, v1
                return v2
            }
            ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();

            assert_ssa_snapshot!(ssa, @r#"
            acir(inline) fn main f0 {
              b0(v0: u32, v1: u32):
                range_check v1 to 5 bits, "attempt to bit-shift with overflow"
                v34 = make_array [Field 1, Field 2, Field 4, Field 8, Field 16, Field 32, Field 64, Field 128, Field 256, Field 512, Field 1024, Field 2048, Field 4096, Field 8192, Field 16384, Field 32768, Field 65536, Field 131072, Field 262144, Field 524288, Field 1048576, Field 2097152, Field 4194304, Field 8388608, Field 16777216, Field 33554432, Field 67108864, Field 134217728, Field 268435456, Field 536870912, Field 1073741824, Field 2147483648] : [Field; 32]
                v35 = array_get v34, index v1 -> Field
                v36 = cast v35 as u32
                v37 = div v0, v36
                return v37
            }
            "#);
        }
    }

    mod signed {
        use super::*;

        // Regression test for fuzzer seed 0x17e3645200100000.
        //
        // For a positive signed constant whose unsigned bit-width plus the shift amount
        // equals the type's bit size, the result is in the "negative" half of the unsigned
        // representation, so it does not fit in the positive signed range. The pass'
        // happy-path multiplication therefore overflows the target type and the SSA
        // interpreter later panics with `unfit 'lt': fit types should have been restored
        // already` when the value reaches `lt`/bitwise operators that require fit operands.
        #[test]
        fn removes_shl_with_constant_lhs_overflowing_signed_range() {
            let src = "
            acir(inline) fn main f0 {
              b0(v0: i8):
                v3 = shl i8 77, i8 1
                v4 = lt v3, v0
                constrain v4 == u1 1
                return
            }
            ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();
            assert_ssa_snapshot!(ssa, @"
            acir(inline) fn main f0 {
              b0(v0: i8):
                v2 = lt i8 -102, v0
                constrain v2 == u1 1
                return
            }
            ");
        }

        // Constant-folded boundary case that does *not* go through the buggy
        // `unchecked_mul i8 _, i8 _` path even on the unfixed code: when the shift amount
        // alone is `bit_size - 1` the cast of `2^(bit_size - 1)` to the signed type wraps
        // to `iN::MIN`, so the subsequent multiplication by a small positive lhs
        // (`i8 1 << 7 == -128`) evaluates without overflow inside `checked_mul`.
        #[test]
        fn removes_shl_with_constant_lhs_one_and_max_shift() {
            let src = "
            acir(inline) fn main f0 {
              b0(v0: i8):
                v3 = shl i8 1, i8 7
                v4 = lt v3, v0
                constrain v4 == u1 1
                return
            }
            ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();
            assert_ssa_snapshot!(ssa, @"
            acir(inline) fn main f0 {
              b0(v0: i8):
                v2 = lt i8 -128, v0
                constrain v2 == u1 1
                return
            }
            ");
        }

        // Non-constant boundary case: an `i16` value reconstructed by a chain of casts
        // from a `u1`. `get_value_max_num_bits` walks the casts and reports a max width of
        // 1, so with a shift of 15 the pass picks the same boundary that triggered the
        // fuzzer for `i8`. Unlike the constant-only cases above, there is no constant
        // folding to obscure the structural change in the emitted SSA.
        #[test]
        fn removes_shl_with_lhs_cast_from_u1_to_i16_at_signed_boundary() {
            let src = "
            acir(inline) fn main f0 {
              b0(v0: u1):
                v1 = cast v0 as i16
                v2 = shl v1, i16 15
                return v2
            }
            ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();
            assert_ssa_snapshot!(ssa, @"
            acir(inline) fn main f0 {
              b0(v0: u1):
                v1 = cast v0 as i16
                v2 = cast v0 as Field
                v4 = mul v2, Field 32768
                v5 = cast v4 as i16
                return v5
            }
            ");
        }

        #[test]
        fn removes_shl_with_constant_rhs() {
            let src = "
            acir(inline) fn main f0 {
              b0(v0: i32):
                v2 = shl v0, i32 2
                return v2
            }
            ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();
            assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: i32):
            v1 = cast v0 as Field
            v3 = mul v1, Field 4
            v4 = truncate v3 to 32 bits, max_bit_size: 34
            v5 = cast v4 as i32
            return v5
        }
        ");
        }

        #[test]
        fn removes_shl_with_non_constant_rhs() {
            let src = "
            acir(inline) fn main f0 {
              b0(v0: i32, v1: i32):
                v2 = shl v0, v1
                return v2
            }
            ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();

            assert_ssa_snapshot!(ssa, @r#"
            acir(inline) fn main f0 {
              b0(v0: i32, v1: i32):
                v2 = cast v1 as u32
                range_check v2 to 5 bits, "attempt to bit-shift with overflow"
                v35 = make_array [Field 1, Field 2, Field 4, Field 8, Field 16, Field 32, Field 64, Field 128, Field 256, Field 512, Field 1024, Field 2048, Field 4096, Field 8192, Field 16384, Field 32768, Field 65536, Field 131072, Field 262144, Field 524288, Field 1048576, Field 2097152, Field 4194304, Field 8388608, Field 16777216, Field 33554432, Field 67108864, Field 134217728, Field 268435456, Field 536870912, Field 1073741824, Field 2147483648] : [Field; 32]
                v36 = cast v1 as u32
                v37 = array_get v35, index v36 -> Field
                v38 = cast v0 as Field
                v39 = mul v38, v37
                v40 = truncate v39 to 32 bits, max_bit_size: 64
                v41 = cast v40 as i32
                return v41
            }
            "#);
        }

        #[test]
        fn removes_shr_with_constant_rhs() {
            let src = "
            acir(inline) fn main f0 {
              b0(v0: i32):
                v2 = shr v0, i32 2
                return v2
            }
        ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();
            assert_ssa_snapshot!(ssa, @r"
            acir(inline) fn main f0 {
              b0(v0: i32):
                v1 = cast v0 as u32
                v3 = lt u32 2147483647, v1
                v4 = cast v3 as Field
                v5 = cast v0 as Field
                v6 = add v4, v5
                v7 = truncate v6 to 32 bits, max_bit_size: 33
                v8 = cast v7 as i32
                v10 = div v8, i32 4
                v11 = cast v10 as Field
                v13 = add v11, Field 4294967296
                v14 = sub v13, v4
                v15 = truncate v14 to 32 bits, max_bit_size: 33
                v16 = cast v15 as i32
                return v16
            }
            ");
        }

        #[test]
        fn removes_shr_with_non_constant_rhs() {
            let src = "
            acir(inline) fn main f0 {
              b0(v0: i32, v1: i32):
                v2 = shr v0, v1
                return v2
            }
            ";
            let ssa = Ssa::from_str(src).unwrap();
            let ssa = ssa.remove_bit_shifts();

            assert_ssa_snapshot!(ssa, @r#"
            acir(inline) fn main f0 {
              b0(v0: i32, v1: i32):
                v2 = cast v1 as u32
                range_check v2 to 5 bits, "attempt to bit-shift with overflow"
                v35 = make_array [Field 1, Field 2, Field 4, Field 8, Field 16, Field 32, Field 64, Field 128, Field 256, Field 512, Field 1024, Field 2048, Field 4096, Field 8192, Field 16384, Field 32768, Field 65536, Field 131072, Field 262144, Field 524288, Field 1048576, Field 2097152, Field 4194304, Field 8388608, Field 16777216, Field 33554432, Field 67108864, Field 134217728, Field 268435456, Field 536870912, Field 1073741824, Field 2147483648] : [Field; 32]
                v36 = cast v1 as u32
                v37 = array_get v35, index v36 -> Field
                v38 = cast v37 as i32
                v39 = cast v0 as u32
                v41 = lt u32 2147483647, v39
                v42 = cast v41 as Field
                v43 = cast v0 as Field
                v44 = add v42, v43
                v45 = truncate v44 to 32 bits, max_bit_size: 33
                v46 = cast v45 as i32
                v47 = div v46, v38
                v48 = cast v47 as Field
                v50 = add v48, Field 4294967296
                v51 = sub v50, v42
                v52 = truncate v51 to 32 bits, max_bit_size: 33
                v53 = cast v52 as i32
                return v53
            }
            "#);
        }
    }

    #[test]
    fn follows_canonical_block_ordering() {
        let src = r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            v4 = shr u8 1, u8 98
            v6 = eq v4, u8 0
            jmpif v6 then: b7(), else: b8()
          b1():
            jmp b3()
          b2():
            jmp b3()
          b3():
            v11 = eq v9, u8 1
            jmpif v11 then: b4(), else: b5()
          b4():
            jmp b6()
          b5():
            jmp b6()
          b6():
            return
          b7():
            jmp b9()
          b8():
            jmp b9()
          b9():
            v7 = eq v4, u8 1
            jmpif v7 then: b10(), else: b11()
          b10():
            jmp b12()
          b11():
            jmp b12()
          b12():
            v9 = shr u8 1, u8 99
            v10 = eq v9, u8 0
            jmpif v10 then: b1(), else: b2()
        }
        "#;
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_bit_shifts();

        // We expect v9 in b3 to be resolved to `u8 0`. Even though b12 has a higher value,
        // it comes before b3 in the block ordering.
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) predicate_pure fn main f0 {
          b0():
            constrain u1 0 == u1 1, "attempt to bit-shift with overflow"
            v4 = div u8 1, u8 0
            v5 = eq v4, u8 0
            jmpif v5 then: b7(), else: b8()
          b1():
            jmp b3()
          b2():
            jmp b3()
          b3():
            v9 = eq v7, u8 1
            jmpif v9 then: b4(), else: b5()
          b4():
            jmp b6()
          b5():
            jmp b6()
          b6():
            return
          b7():
            jmp b9()
          b8():
            jmp b9()
          b9():
            v6 = eq v4, u8 1
            jmpif v6 then: b10(), else: b11()
          b10():
            jmp b12()
          b11():
            jmp b12()
          b12():
            constrain u1 0 == u1 1, "attempt to bit-shift with overflow"
            v7 = div u8 1, u8 0
            v8 = eq v7, u8 0
            jmpif v8 then: b1(), else: b2()
        }
        "#);
    }

    #[test]
    fn left_bit_shift_u128_overflow_field() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u128):
            v2 = shl v0, u128 127
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_bit_shifts();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u128):
            v1 = cast v0 as Field
            v3 = mul v1, Field 9223372036854775808
            v4 = truncate v3 to 128 bits, max_bit_size: 254
            v6 = mul v4, Field 18446744073709551616
            v7 = truncate v6 to 128 bits, max_bit_size: 254
            v8 = cast v7 as u128
            return v8
        }
        ");
    }

    // A shift by a constant at least as large as the bit size fails the
    // "attempt to bit-shift with overflow" constraint, and the power of two computed for it
    // must still be castable to the shifted type: the output passes full validation.
    #[test_case("shr", "i8", 8)]
    #[test_case("shr", "i8", 9)]
    #[test_case("shr", "i16", 16)]
    #[test_case("shr", "i32", 32)]
    #[test_case("shr", "i64", 64)]
    #[test_case("shr", "u8", 8)]
    #[test_case("shr", "u32", 32)]
    #[test_case("shl", "i8", 8)]
    #[test_case("shl", "i32", 32)]
    #[test_case("shl", "u8", 8)]
    fn shift_by_constant_not_less_than_bit_size_passes_full_validation(
        op: &str,
        typ: &str,
        rhs: u32,
    ) {
        let src = format!(
            "
        acir(inline) fn main f0 {{
          b0(v0: {typ}):
            v2 = {op} v0, {typ} {rhs}
            return v2
        }}
        "
        );
        let ssa = Ssa::from_str(&src).unwrap();
        let ssa = ssa.remove_bit_shifts();
        assert!(
            ssa.to_string()
                .contains("constrain u1 0 == u1 1, \"attempt to bit-shift with overflow\""),
            "missing overflow constraint:\n{ssa}"
        );
        validate_ssa(&ssa, true);
    }

    #[test]
    fn signed_shr_by_constant_bit_size() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: i8):
            v2 = shr v0, i8 8
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_bit_shifts();
        assert_ssa_snapshot!(ssa, @r#"
        acir(inline) fn main f0 {
          b0(v0: i8):
            constrain u1 0 == u1 1, "attempt to bit-shift with overflow"
            v3 = cast v0 as u8
            v5 = lt u8 127, v3
            v6 = cast v5 as Field
            v7 = cast v0 as Field
            v8 = add v6, v7
            v9 = truncate v8 to 8 bits, max_bit_size: 9
            v10 = cast v9 as i8
            v12 = div v10, i8 0
            v13 = cast v12 as Field
            v15 = add v13, Field 256
            v16 = sub v15, v6
            v17 = truncate v16 to 8 bits, max_bit_size: 9
            v18 = cast v17 as i8
            return v18
        }
        "#);
    }

    // The largest valid shift amount, one less than the bit size, keeps its power of two.
    #[test_case(Value::i8(-3), Value::i8(-1); "minus_3")]
    #[test_case(Value::i8(-128), Value::i8(-1); "minus_128")]
    #[test_case(Value::i8(-1), Value::i8(-1); "minus_1")]
    #[test_case(Value::i8(0), Value::i8(0); "zero")]
    #[test_case(Value::i8(127), Value::i8(0); "max")]
    fn signed_shr_by_constant_bit_size_minus_one(input: Value, expected: Value) {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: i8):
            v2 = shr v0, i8 7
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let (ssa, result) =
            assert_pass_does_not_affect_execution(ssa, vec![input], |ssa| ssa.remove_bit_shifts());
        assert_eq!(result, Ok(vec![expected]));
        validate_ssa(&ssa, true);
    }

    // Every shift amount just below the bit size, including the `u128` left shift whose power of
    // two is computed from the two halves of the shift amount, is accepted by the overflow check.
    #[test_case("u8", "shl", Value::u8(u8::MAX), Value::u8(7), Value::u8(128); "u8_shl")]
    #[test_case("u8", "shr", Value::u8(u8::MAX), Value::u8(7), Value::u8(1); "u8_shr")]
    #[test_case("i8", "shr", Value::i8(i8::MIN), Value::i8(7), Value::i8(-1); "i8_shr")]
    #[test_case("u128", "shl", Value::u128(u128::MAX), Value::u128(127), Value::u128(1 << 127); "u128_shl")]
    #[test_case("u128", "shr", Value::u128(u128::MAX), Value::u128(127), Value::u128(1); "u128_shr")]
    fn shift_by_non_constant_rhs_one_less_than_bit_size(
        typ: &str,
        op: &str,
        lhs: Value,
        rhs: Value,
        expected: Value,
    ) {
        let src = format!(
            "
        acir(inline) fn main f0 {{
          b0(v0: {typ}, v1: {typ}):
            v2 = {op} v0, v1
            return v2
        }}
        "
        );
        let ssa = Ssa::from_str(&src).unwrap();
        let (_, result) = assert_pass_does_not_affect_execution(ssa, vec![lhs, rhs], |ssa| {
            ssa.remove_bit_shifts()
        });
        assert_eq!(result, Ok(vec![expected]));
    }

    // A shift amount not less than the bit size fails with the bit-shift overflow message. The
    // `u128` left shift computes its power of two from halves of the shift amount, which are
    // in range for a shift amount of 128, so only the overflow check rejects it.
    #[test_case("u8", "shl", Value::u8(1), Value::u8(8); "u8_shl")]
    #[test_case("u8", "shr", Value::u8(1), Value::u8(u8::MAX); "u8_shr")]
    #[test_case("i8", "shr", Value::i8(1), Value::i8(-1); "i8_shr_negative_amount")]
    #[test_case("u128", "shl", Value::u128(1), Value::u128(128); "u128_shl")]
    #[test_case("u128", "shr", Value::u128(1), Value::u128(128); "u128_shr")]
    fn shift_by_non_constant_rhs_not_less_than_bit_size_fails(
        typ: &str,
        op: &str,
        lhs: Value,
        rhs: Value,
    ) {
        let src = format!(
            "
        acir(inline) fn main f0 {{
          b0(v0: {typ}, v1: {typ}):
            v2 = {op} v0, v1
            return v2
        }}
        "
        );
        let ssa = Ssa::from_str(&src).unwrap().remove_bit_shifts();
        let result = ssa.interpret(vec![lhs, rhs]);
        let Err(InterpreterError::RangeCheckFailed { msg, .. }) = result else {
            panic!("expected the shift amount to fail a range check, got {result:?}");
        };
        assert_eq!(msg.as_deref(), Some("attempt to bit-shift with overflow"));
    }
}
