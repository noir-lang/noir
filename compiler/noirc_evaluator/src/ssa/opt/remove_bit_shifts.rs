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
//! If the shift amount is not a constant, 2^N is computed via square&multiply,
//! using the bits decomposition of the exponent.
//!
//! Pseudo-code of the computation:
//!
//! ```text
//! let mut r = 1;
//! let exponent_bits = to_bits(exponent);
//! for i in 1 .. bit_size + 1 {
//!     let r_squared = r * r;
//!     let b = exponent_bits[bit_size - i];
//!     r = if b { 2 * r_squared } else { r_squared };
//! }
//! ```
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
//! Like in the previous case, if the shift amount is not a constant it's computed
//! via square&multiply.
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

use acvm::{FieldElement, acir::AcirField};

use crate::ssa::{
    ir::{
        function::Function,
        instruction::{Binary, BinaryOp, ConstrainError, Endian, Instruction, Intrinsic},
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
        if max_bit <= typ.bit_size::<FieldElement>() {
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
                let div_complement =
                    self.insert_cast(div_complement, NumericType::signed(bit_size));
                // Performs the division on the adjusted complement (or the operand if positive)
                let shifted_complement = self.insert_binary(div_complement, BinaryOp::Div, pow);
                // For negative numbers, convert back to 2-complement by subtracting 1.
                let lhs_sign_as_int = self.insert_cast(lhs_sign, lhs_typ);

                // The requirements for this to underflow are all of these:
                // - lhs < 0
                // - div_complement(lhs) / (2^rhs) == 0
                // As the upper bit is set for the ones complement of negative numbers we'd need 2^rhs
                // to be larger than the lhs bitsize for this to overflow.
                let sub = BinaryOp::Sub { unchecked: true };
                let shifted = self.insert_binary(shifted_complement, sub, lhs_sign_as_int);
                self.insert_truncate(shifted, bit_size, bit_size + 1)
            }

            NumericType::NativeField => unreachable!("Bit shifts are disallowed on `Field` type"),
        }
    }

    /// Computes 2^exponent via square&multiply, using the bits decomposition of exponent
    /// Pseudo-code of the computation:
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
            let pow = if exponent_const_as_u32.is_none_or(|v| v > max_bit_size) {
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
            assert!(exponent_bit_size.is_power_of_two(), "ICE: exponent type bit size is expected to be a power of two");
            exponent_bit_size.ilog2()
        };
        let result_types = vec![Type::Array(Arc::new(vec![Type::bool()]), max_exponent_bits)];

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

    /// Insert constraints ensuring that the right-hand side of a bit-shift operation
    /// is less than the bit size of the left-hand side.
    fn enforce_bitshift_rhs_lt_bit_size(&mut self, rhs: ValueId) {
        let one = self.numeric_constant(FieldElement::one(), NumericType::bool());
        let rhs_type = self.context.dfg.type_of_value(rhs);

        let assert_message = Some("attempt to bit-shift with overflow".to_owned());

        let bit_size = rhs_type.bit_size();
        let bit_size_field = FieldElement::from(bit_size);

        let unsigned_typ = NumericType::unsigned(bit_size);
        let max = self.numeric_constant(bit_size_field, unsigned_typ);
        let rhs = self.insert_cast(rhs, unsigned_typ);
        let overflow = self.insert_binary(rhs, BinaryOp::Lt, max);
        self.insert_constrain(overflow, one, assert_message.map(Into::into));
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

    /// Insert a constrain instruction at the end of the current block.
    fn insert_constrain(
        &mut self,
        lhs: ValueId,
        rhs: ValueId,
        assert_message: Option<ConstrainError>,
    ) {
        self.context.insert_instruction(Instruction::Constrain(lhs, rhs, assert_message), None);
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

/// Post-check condition for [Function::remove_bit_shifts].
///
/// Succeeds if:
///   - `func` is not an ACIR function, OR
///   - `func` does not contain any bitshift instructions.
///
/// Otherwise panics.
#[cfg(debug_assertions)]
fn remove_bit_shifts_post_check(func: &Function) {
    // Non-ACIR functions should be unaffected.
    if !func.runtime().is_acir() {
        return;
    }

    // Otherwise there should be no shift-left or shift-right instructions in any reachable block.
    for block_id in func.reachable_blocks() {
        let instruction_ids = func.dfg[block_id].instructions();
        for instruction_id in instruction_ids {
            if matches!(
                func.dfg[*instruction_id],
                Instruction::Binary(Binary { operator: BinaryOp::Shl | BinaryOp::Shr, .. })
            ) {
                panic!("Bitshift instruction still remains in ACIR function");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_ssa_snapshot, ssa::ssa_gen::Ssa};

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
                v3 = lt v1, u32 32
                constrain v3 == u1 1, "attempt to bit-shift with overflow"
                v5 = cast v1 as Field
                v7 = call to_le_bits(v5) -> [u1; 5]
                v9 = array_get v7, index u32 4 -> u1
                v10 = not v9
                v11 = cast v9 as Field
                v12 = cast v10 as Field
                v14 = mul Field 2, v11
                v15 = add v12, v14
                v17 = array_get v7, index u32 3 -> u1
                v18 = not v17
                v19 = cast v17 as Field
                v20 = cast v18 as Field
                v21 = mul v15, v15
                v22 = mul v21, v20
                v23 = mul v21, Field 2
                v24 = mul v23, v19
                v25 = add v22, v24
                v27 = array_get v7, index u32 2 -> u1
                v28 = not v27
                v29 = cast v27 as Field
                v30 = cast v28 as Field
                v31 = mul v25, v25
                v32 = mul v31, v30
                v33 = mul v31, Field 2
                v34 = mul v33, v29
                v35 = add v32, v34
                v37 = array_get v7, index u32 1 -> u1
                v38 = not v37
                v39 = cast v37 as Field
                v40 = cast v38 as Field
                v41 = mul v35, v35
                v42 = mul v41, v40
                v43 = mul v41, Field 2
                v44 = mul v43, v39
                v45 = add v42, v44
                v47 = array_get v7, index u32 0 -> u1
                v48 = not v47
                v49 = cast v47 as Field
                v50 = cast v48 as Field
                v51 = mul v45, v45
                v52 = mul v51, v50
                v53 = mul v51, Field 2
                v54 = mul v53, v49
                v55 = add v52, v54
                v56 = cast v0 as Field
                v57 = mul v56, v55
                v58 = truncate v57 to 32 bits, max_bit_size: 64
                v59 = cast v58 as u32
                return v59
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
                v4 = lt v2, u32 32
                constrain v4 == u1 1, "attempt to bit-shift with overflow"
                v6 = cast v1 as Field
                v8 = call to_le_bits(v6) -> [u1; 5]
                v10 = array_get v8, index u32 4 -> u1
                v11 = not v10
                v12 = cast v10 as Field
                v13 = cast v11 as Field
                v15 = mul Field 2, v12
                v16 = add v13, v15
                v18 = array_get v8, index u32 3 -> u1
                v19 = not v18
                v20 = cast v18 as Field
                v21 = cast v19 as Field
                v22 = mul v16, v16
                v23 = mul v22, v21
                v24 = mul v22, Field 2
                v25 = mul v24, v20
                v26 = add v23, v25
                v28 = array_get v8, index u32 2 -> u1
                v29 = not v28
                v30 = cast v28 as Field
                v31 = cast v29 as Field
                v32 = mul v26, v26
                v33 = mul v32, v31
                v34 = mul v32, Field 2
                v35 = mul v34, v30
                v36 = add v33, v35
                v38 = array_get v8, index u32 1 -> u1
                v39 = not v38
                v40 = cast v38 as Field
                v41 = cast v39 as Field
                v42 = mul v36, v36
                v43 = mul v42, v41
                v44 = mul v42, Field 2
                v45 = mul v44, v40
                v46 = add v43, v45
                v48 = array_get v8, index u32 0 -> u1
                v49 = not v48
                v50 = cast v48 as Field
                v51 = cast v49 as Field
                v52 = mul v46, v46
                v53 = mul v52, v51
                v54 = mul v52, Field 2
                v55 = mul v54, v50
                v56 = add v53, v55
                v57 = cast v0 as Field
                v58 = mul v57, v56
                v59 = truncate v58 to 32 bits, max_bit_size: 64
                v60 = cast v59 as u32
                return v60
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
            assert_ssa_snapshot!(ssa, @r#"
            acir(inline) fn main f0 {
              b0(v0: u64, v1: u1):
                v2 = cast v1 as u8
                v3 = cast v2 as u32
                v4 = cast v3 as u64
                v6 = lt v4, u64 64
                constrain v6 == u1 1, "attempt to bit-shift with overflow"
                v8 = cast v1 as Field
                v10 = call to_le_bits(v8) -> [u1; 1]
                v12 = array_get v10, index u32 0 -> u1
                v13 = not v12
                v14 = cast v12 as Field
                v15 = cast v13 as Field
                v17 = mul Field 2, v14
                v18 = add v15, v17
                v19 = cast v0 as Field
                v20 = mul v19, v18
                v21 = truncate v20 to 64 bits, max_bit_size: 65
                v22 = cast v21 as u64
                return v22
            }
            "#);
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
                v3 = lt v1, u32 32
                constrain v3 == u1 1, "attempt to bit-shift with overflow"
                v5 = cast v1 as Field
                v7 = call to_le_bits(v5) -> [u1; 5]
                v9 = array_get v7, index u32 4 -> u1
                v10 = not v9
                v11 = cast v9 as Field
                v12 = cast v10 as Field
                v14 = mul Field 2, v11
                v15 = add v12, v14
                v17 = array_get v7, index u32 3 -> u1
                v18 = not v17
                v19 = cast v17 as Field
                v20 = cast v18 as Field
                v21 = mul v15, v15
                v22 = mul v21, v20
                v23 = mul v21, Field 2
                v24 = mul v23, v19
                v25 = add v22, v24
                v27 = array_get v7, index u32 2 -> u1
                v28 = not v27
                v29 = cast v27 as Field
                v30 = cast v28 as Field
                v31 = mul v25, v25
                v32 = mul v31, v30
                v33 = mul v31, Field 2
                v34 = mul v33, v29
                v35 = add v32, v34
                v37 = array_get v7, index u32 1 -> u1
                v38 = not v37
                v39 = cast v37 as Field
                v40 = cast v38 as Field
                v41 = mul v35, v35
                v42 = mul v41, v40
                v43 = mul v41, Field 2
                v44 = mul v43, v39
                v45 = add v42, v44
                v47 = array_get v7, index u32 0 -> u1
                v48 = not v47
                v49 = cast v47 as Field
                v50 = cast v48 as Field
                v51 = mul v45, v45
                v52 = mul v51, v50
                v53 = mul v51, Field 2
                v54 = mul v53, v49
                v55 = add v52, v54
                v56 = cast v55 as u32
                v57 = div v0, v56
                return v57
            }
            "#);
        }
    }

    mod signed {
        use super::*;
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
                v4 = lt v2, u32 32
                constrain v4 == u1 1, "attempt to bit-shift with overflow"
                v6 = cast v1 as Field
                v8 = call to_le_bits(v6) -> [u1; 5]
                v10 = array_get v8, index u32 4 -> u1
                v11 = not v10
                v12 = cast v10 as Field
                v13 = cast v11 as Field
                v15 = mul Field 2, v12
                v16 = add v13, v15
                v18 = array_get v8, index u32 3 -> u1
                v19 = not v18
                v20 = cast v18 as Field
                v21 = cast v19 as Field
                v22 = mul v16, v16
                v23 = mul v22, v21
                v24 = mul v22, Field 2
                v25 = mul v24, v20
                v26 = add v23, v25
                v28 = array_get v8, index u32 2 -> u1
                v29 = not v28
                v30 = cast v28 as Field
                v31 = cast v29 as Field
                v32 = mul v26, v26
                v33 = mul v32, v31
                v34 = mul v32, Field 2
                v35 = mul v34, v30
                v36 = add v33, v35
                v38 = array_get v8, index u32 1 -> u1
                v39 = not v38
                v40 = cast v38 as Field
                v41 = cast v39 as Field
                v42 = mul v36, v36
                v43 = mul v42, v41
                v44 = mul v42, Field 2
                v45 = mul v44, v40
                v46 = add v43, v45
                v48 = array_get v8, index u32 0 -> u1
                v49 = not v48
                v50 = cast v48 as Field
                v51 = cast v49 as Field
                v52 = mul v46, v46
                v53 = mul v52, v51
                v54 = mul v52, Field 2
                v55 = mul v54, v50
                v56 = add v53, v55
                v57 = cast v0 as Field
                v58 = mul v57, v56
                v59 = truncate v58 to 32 bits, max_bit_size: 64
                v60 = cast v59 as i32
                return v60
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
                v11 = cast v3 as i32
                v12 = unchecked_sub v10, v11
                v13 = truncate v12 to 32 bits, max_bit_size: 33
                return v13
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
                v4 = lt v2, u32 32
                constrain v4 == u1 1, "attempt to bit-shift with overflow"
                v6 = cast v1 as Field
                v8 = call to_le_bits(v6) -> [u1; 5]
                v10 = array_get v8, index u32 4 -> u1
                v11 = not v10
                v12 = cast v10 as Field
                v13 = cast v11 as Field
                v15 = mul Field 2, v12
                v16 = add v13, v15
                v18 = array_get v8, index u32 3 -> u1
                v19 = not v18
                v20 = cast v18 as Field
                v21 = cast v19 as Field
                v22 = mul v16, v16
                v23 = mul v22, v21
                v24 = mul v22, Field 2
                v25 = mul v24, v20
                v26 = add v23, v25
                v28 = array_get v8, index u32 2 -> u1
                v29 = not v28
                v30 = cast v28 as Field
                v31 = cast v29 as Field
                v32 = mul v26, v26
                v33 = mul v32, v31
                v34 = mul v32, Field 2
                v35 = mul v34, v30
                v36 = add v33, v35
                v38 = array_get v8, index u32 1 -> u1
                v39 = not v38
                v40 = cast v38 as Field
                v41 = cast v39 as Field
                v42 = mul v36, v36
                v43 = mul v42, v41
                v44 = mul v42, Field 2
                v45 = mul v44, v40
                v46 = add v43, v45
                v48 = array_get v8, index u32 0 -> u1
                v49 = not v48
                v50 = cast v48 as Field
                v51 = cast v49 as Field
                v52 = mul v46, v46
                v53 = mul v52, v51
                v54 = mul v52, Field 2
                v55 = mul v54, v50
                v56 = add v53, v55
                v57 = cast v56 as i32
                v58 = cast v0 as u32
                v60 = lt u32 2147483647, v58
                v61 = cast v60 as Field
                v62 = cast v0 as Field
                v63 = add v61, v62
                v64 = truncate v63 to 32 bits, max_bit_size: 33
                v65 = cast v64 as i32
                v66 = div v65, v57
                v67 = cast v60 as i32
                v68 = unchecked_sub v66, v67
                v69 = truncate v68 to 32 bits, max_bit_size: 33
                return v69
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
            jmpif v6 then: b7, else: b8
          b1():
            jmp b3()
          b2():
            jmp b3()
          b3():
            v11 = eq v9, u8 1
            jmpif v11 then: b4, else: b5
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
            jmpif v7 then: b10, else: b11
          b10():
            jmp b12()
          b11():
            jmp b12()
          b12():
            v9 = shr u8 1, u8 99
            v10 = eq v9, u8 0
            jmpif v10 then: b1, else: b2
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
            jmpif v5 then: b7, else: b8
          b1():
            jmp b3()
          b2():
            jmp b3()
          b3():
            v9 = eq v7, u8 1
            jmpif v9 then: b4, else: b5
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
            jmpif v6 then: b10, else: b11
          b10():
            jmp b12()
          b11():
            jmp b12()
          b12():
            constrain u1 0 == u1 1, "attempt to bit-shift with overflow"
            v7 = div u8 1, u8 0
            v8 = eq v7, u8 0
            jmpif v8 then: b1, else: b2
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
}
