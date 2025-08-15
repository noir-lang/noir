use std::{borrow::Cow, sync::Arc};

use acvm::{FieldElement, acir::AcirField};

use crate::ssa::{
    ir::{
        function::Function,
        instruction::{
            ArrayOffset, Binary, BinaryOp, ConstrainError, Endian, Instruction, Intrinsic,
        },
        types::{NumericType, Type},
        value::ValueId,
    },
    ssa_gen::Ssa,
};

use super::simple_optimization::SimpleOptimizationContext;

impl Ssa {
    /// Replaces Shl and Shr instructions with more primitive arithmetic instructions
    /// since our backend doesn't directly support bit shifts.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn remove_bit_shifts(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.remove_bit_shifts();
        }
        self
    }
}

impl Function {
    /// Go through every instruction, replacing bit shifts with more primitive arithmetic
    /// operations.
    pub(crate) fn remove_bit_shifts(&mut self) {
        if self.runtime().is_brillig() {
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

            let old_result = *context.dfg.instruction_results(instruction_id).first().unwrap();

            let bit_size = match context.dfg.type_of_value(lhs) {
                Type::Numeric(NumericType::Signed { bit_size })
                | Type::Numeric(NumericType::Unsigned { bit_size }) => bit_size,
                _ => unreachable!("ICE: right-shift attempted on non-integer"),
            };

            let mut bitshift_context = Context { context };
            let new_result = if operator == BinaryOp::Shl {
                bitshift_context.insert_wrapping_shift_left(lhs, rhs, bit_size)
            } else {
                bitshift_context.insert_shift_right(lhs, rhs, bit_size)
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
    /// Insert ssa instructions which computes lhs << rhs by doing lhs*2^rhs
    /// and truncate the result to bit_size
    pub(crate) fn insert_wrapping_shift_left(
        &mut self,
        lhs: ValueId,
        rhs: ValueId,
        bit_size: u32,
    ) -> ValueId {
        let typ = self.context.dfg.type_of_value(lhs).unwrap_numeric();
        let (max_bit, pow) = if let Some(rhs_constant) = self.context.dfg.get_numeric_constant(rhs)
        {
            // Happy case is that we know precisely by how many bits the integer will
            // increase: lhs_bit_size + rhs
            let bit_shift_size = rhs_constant.to_u128() as u32;

            let pow = self.two_pow(rhs, bit_shift_size.ilog2() + 1);

            let max_lhs_bits = self.context.dfg.get_value_max_num_bits(lhs);
            let max_bit_size = max_lhs_bits + bit_shift_size;
            // There is no point trying to truncate to more than the Field size.
            // A higher `max_lhs_bits` input can come from trying to left-shift a Field.
            let max_bit_size = max_bit_size.min(NumericType::NativeField.bit_size());
            (max_bit_size, pow)
        } else {
            // we use a predicate to nullify the result in case of overflow
            let bit_size_var = self.numeric_constant(FieldElement::from(bit_size as u128), typ);
            let overflow = self.insert_binary(rhs, BinaryOp::Lt, bit_size_var);
            let predicate = self.insert_cast(overflow, NumericType::NativeField);
            let pow = self.two_pow(rhs, bit_size.ilog2() + 1);

            // Unchecked mul because `predicate` will be 1 or 0
            (
                FieldElement::max_num_bits(),
                self.insert_binary(predicate, BinaryOp::Mul { unchecked: true }, pow),
            )
        };

        if max_bit <= bit_size {
            let pow = self.insert_cast(pow, typ);
            // Unchecked mul as it can't overflow
            self.insert_binary(lhs, BinaryOp::Mul { unchecked: true }, pow)
        } else {
            let lhs_field = self.insert_cast(lhs, NumericType::NativeField);
            // Unchecked mul as this is a wrapping operation that we later truncate
            let result = self.insert_binary(lhs_field, BinaryOp::Mul { unchecked: true }, pow);
            let result = self.insert_truncate(result, bit_size, max_bit);
            self.insert_cast(result, typ)
        }
    }

    /// Insert ssa instructions which computes lhs >> rhs by doing lhs/2^rhs
    /// For negative signed integers, we do the division on the 1-complement representation of lhs,
    /// before converting back the result to the 2-complement representation.
    pub(crate) fn insert_shift_right(
        &mut self,
        lhs: ValueId,
        rhs: ValueId,
        bit_size: u32,
    ) -> ValueId {
        let lhs_typ = self.context.dfg.type_of_value(lhs).unwrap_numeric();

        let pow = self.two_pow(rhs, bit_size.ilog2() + 1);
        let pow = self.insert_cast(pow, lhs_typ);

        match lhs_typ {
            NumericType::Unsigned { .. } => {
                // unsigned right bit shift is just a normal division
                self.insert_binary(lhs, BinaryOp::Div, pow)
            }
            NumericType::Signed { bit_size } => {
                // Get the sign of the operand; positive signed operand will just do a division as well
                let zero =
                    self.numeric_constant(FieldElement::zero(), NumericType::signed(bit_size));
                let lhs_sign = self.insert_binary(lhs, BinaryOp::Lt, zero);
                let lhs_sign_as_field = self.insert_cast(lhs_sign, NumericType::NativeField);
                let lhs_as_field = self.insert_cast(lhs, NumericType::NativeField);
                // For negative numbers, convert to 1-complement using wrapping addition of a + 1
                // Unchecked add as these are fields
                let one_complement = self.insert_binary(
                    lhs_sign_as_field,
                    BinaryOp::Add { unchecked: true },
                    lhs_as_field,
                );
                let one_complement = self.insert_truncate(one_complement, bit_size, bit_size + 1);
                let one_complement =
                    self.insert_cast(one_complement, NumericType::signed(bit_size));
                // Performs the division on the 1-complement (or the operand if positive)
                let shifted_complement = self.insert_binary(one_complement, BinaryOp::Div, pow);
                // Convert back to 2-complement representation if operand is negative
                let lhs_sign_as_int = self.insert_cast(lhs_sign, lhs_typ);

                // The requirements for this to underflow are all of these:
                // - lhs < 0
                // - ones_complement(lhs) / (2^rhs) == 0
                // As the upper bit is set for the ones complement of negative numbers we'd need 2^rhs
                // to be larger than the lhs bitsize for this to overflow.
                let shifted = self.insert_binary(
                    shifted_complement,
                    BinaryOp::Sub { unchecked: true },
                    lhs_sign_as_int,
                );
                self.insert_truncate(shifted, bit_size, bit_size + 1)
            }

            NumericType::NativeField => unreachable!("Bit shifts are disallowed on `Field` type"),
        }
    }

    /// Computes 2^exponent via square&multiply, using the bits decomposition of exponent
    /// Pseudo-code of the computation:
    /// let mut r = 1;
    /// let exponent_bits = to_bits(exponent);
    /// for i in 1 .. bit_size + 1 {
    ///     let r_squared = r * r;
    ///     let b = exponent_bits[bit_size - i];
    ///     r = if b { 2 * r_squared } else { r_squared };
    /// }
    fn two_pow(&mut self, exponent: ValueId, bit_size: u32) -> ValueId {
        // Require that exponent < bit_size, ensuring that `pow` returns a value consistent with `lhs`'s type.
        self.enforce_bitshift_rhs_lt_bit_size(exponent);

        if let Some(exponent_const) = self.context.dfg.get_numeric_constant(exponent) {
            let pow = FieldElement::from(2u32).pow(&exponent_const);
            return self.numeric_constant(pow, NumericType::NativeField);
        }

        let to_bits = self.context.dfg.import_intrinsic(Intrinsic::ToBits(Endian::Little));
        let result_types = vec![Type::Array(Arc::new(vec![Type::bool()]), bit_size)];

        // A call to ToBits can only be done with a field argument (exponent is always u8 here)
        let exponent_as_field = self.insert_cast(exponent, NumericType::NativeField);
        let exponent_bits = self.insert_call(to_bits, vec![exponent_as_field], result_types);

        let exponent_bits = exponent_bits[0];
        let one = self.field_constant(FieldElement::one());
        let two = self.field_constant(FieldElement::from(2u32));
        let mut r = one;
        // All operations are unchecked as we're acting on Field types (which are always unchecked)
        for i in 1..bit_size + 1 {
            let idx = self.numeric_constant(
                FieldElement::from((bit_size - i) as i128),
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

        let (bit_size, bit_size_field) = match rhs_type {
            Type::Numeric(NumericType::Unsigned { bit_size }) => {
                (bit_size, FieldElement::from(bit_size))
            }
            Type::Numeric(NumericType::Signed { bit_size }) => {
                assert!(bit_size > 1, "ICE - i1 is not a valid type");

                (bit_size, FieldElement::from(bit_size - 1))
            }
            _ => unreachable!("check_shift_overflow called with non-numeric type"),
        };

        let unsigned_typ = NumericType::unsigned(bit_size);
        let max = self.numeric_constant(bit_size_field, unsigned_typ);
        let rhs = self.insert_cast(rhs, unsigned_typ);
        let overflow = self.insert_binary(rhs, BinaryOp::Lt, max);
        self.insert_constrain(overflow, one, assert_message.map(Into::into));
    }

    pub(crate) fn field_constant(&mut self, constant: FieldElement) -> ValueId {
        self.context.dfg.make_constant(constant, NumericType::NativeField)
    }

    /// Insert a numeric constant into the current function
    pub(crate) fn numeric_constant(
        &mut self,
        value: impl Into<FieldElement>,
        typ: NumericType,
    ) -> ValueId {
        self.context.dfg.make_constant(value.into(), typ)
    }

    /// Insert a binary instruction at the end of the current block.
    /// Returns the result of the binary instruction.
    pub(crate) fn insert_binary(
        &mut self,
        lhs: ValueId,
        operator: BinaryOp,
        rhs: ValueId,
    ) -> ValueId {
        let instruction = Instruction::Binary(Binary { lhs, rhs, operator });
        self.context.insert_instruction(instruction, None).first()
    }

    /// Insert a not instruction at the end of the current block.
    /// Returns the result of the instruction.
    pub(crate) fn insert_not(&mut self, rhs: ValueId) -> ValueId {
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
    pub(crate) fn insert_truncate(
        &mut self,
        value: ValueId,
        bit_size: u32,
        max_bit_size: u32,
    ) -> ValueId {
        self.context
            .insert_instruction(Instruction::Truncate { value, bit_size, max_bit_size }, None)
            .first()
    }

    /// Insert a cast instruction at the end of the current block.
    /// Returns the result of the cast instruction.
    pub(crate) fn insert_cast(&mut self, value: ValueId, typ: NumericType) -> ValueId {
        self.context.insert_instruction(Instruction::Cast(value, typ), None).first()
    }

    /// Insert a call instruction at the end of the current block and return
    /// the results of the call.
    pub(crate) fn insert_call(
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
    pub(crate) fn insert_array_get(
        &mut self,
        array: ValueId,
        index: ValueId,
        element_type: Type,
    ) -> ValueId {
        let element_type = Some(vec![element_type]);
        let offset = ArrayOffset::None;
        let instruction = Instruction::ArrayGet { array, index, offset };
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
        ");
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
                v4 = cast v3 as Field
                v5 = lt v1, u32 32
                constrain v5 == u1 1, "attempt to bit-shift with overflow"
                v7 = cast v1 as Field
                v9 = call to_le_bits(v7) -> [u1; 6]
                v11 = array_get v9, index u32 5 -> u1
                v12 = not v11
                v13 = cast v11 as Field
                v14 = cast v12 as Field
                v16 = mul Field 2, v13
                v17 = add v14, v16
                v19 = array_get v9, index u32 4 -> u1
                v20 = not v19
                v21 = cast v19 as Field
                v22 = cast v20 as Field
                v23 = mul v17, v17
                v24 = mul v23, v22
                v25 = mul v23, Field 2
                v26 = mul v25, v21
                v27 = add v24, v26
                v29 = array_get v9, index u32 3 -> u1
                v30 = not v29
                v31 = cast v29 as Field
                v32 = cast v30 as Field
                v33 = mul v27, v27
                v34 = mul v33, v32
                v35 = mul v33, Field 2
                v36 = mul v35, v31
                v37 = add v34, v36
                v39 = array_get v9, index u32 2 -> u1
                v40 = not v39
                v41 = cast v39 as Field
                v42 = cast v40 as Field
                v43 = mul v37, v37
                v44 = mul v43, v42
                v45 = mul v43, Field 2
                v46 = mul v45, v41
                v47 = add v44, v46
                v49 = array_get v9, index u32 1 -> u1
                v50 = not v49
                v51 = cast v49 as Field
                v52 = cast v50 as Field
                v53 = mul v47, v47
                v54 = mul v53, v52
                v55 = mul v53, Field 2
                v56 = mul v55, v51
                v57 = add v54, v56
                v59 = array_get v9, index u32 0 -> u1
                v60 = not v59
                v61 = cast v59 as Field
                v62 = cast v60 as Field
                v63 = mul v57, v57
                v64 = mul v63, v62
                v65 = mul v63, Field 2
                v66 = mul v65, v61
                v67 = add v64, v66
                v68 = mul v4, v67
                v69 = cast v0 as Field
                v70 = mul v69, v68
                v71 = truncate v70 to 32 bits, max_bit_size: 254
                v72 = cast v71 as u32
                return v72
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
                v7 = call to_le_bits(v5) -> [u1; 6]
                v9 = array_get v7, index u32 5 -> u1
                v10 = not v9
                v11 = cast v9 as Field
                v12 = cast v10 as Field
                v14 = mul Field 2, v11
                v15 = add v12, v14
                v17 = array_get v7, index u32 4 -> u1
                v18 = not v17
                v19 = cast v17 as Field
                v20 = cast v18 as Field
                v21 = mul v15, v15
                v22 = mul v21, v20
                v23 = mul v21, Field 2
                v24 = mul v23, v19
                v25 = add v22, v24
                v27 = array_get v7, index u32 3 -> u1
                v28 = not v27
                v29 = cast v27 as Field
                v30 = cast v28 as Field
                v31 = mul v25, v25
                v32 = mul v31, v30
                v33 = mul v31, Field 2
                v34 = mul v33, v29
                v35 = add v32, v34
                v37 = array_get v7, index u32 2 -> u1
                v38 = not v37
                v39 = cast v37 as Field
                v40 = cast v38 as Field
                v41 = mul v35, v35
                v42 = mul v41, v40
                v43 = mul v41, Field 2
                v44 = mul v43, v39
                v45 = add v42, v44
                v47 = array_get v7, index u32 1 -> u1
                v48 = not v47
                v49 = cast v47 as Field
                v50 = cast v48 as Field
                v51 = mul v45, v45
                v52 = mul v51, v50
                v53 = mul v51, Field 2
                v54 = mul v53, v49
                v55 = add v52, v54
                v57 = array_get v7, index u32 0 -> u1
                v58 = not v57
                v59 = cast v57 as Field
                v60 = cast v58 as Field
                v61 = mul v55, v55
                v62 = mul v61, v60
                v63 = mul v61, Field 2
                v64 = mul v63, v59
                v65 = add v62, v64
                v66 = cast v65 as u32
                v67 = div v0, v66
                return v67
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
                v3 = lt v1, i32 32
                v4 = cast v3 as Field
                v5 = cast v1 as u32
                v7 = lt v5, u32 31
                constrain v7 == u1 1, "attempt to bit-shift with overflow"
                v9 = cast v1 as Field
                v11 = call to_le_bits(v9) -> [u1; 6]
                v13 = array_get v11, index u32 5 -> u1
                v14 = not v13
                v15 = cast v13 as Field
                v16 = cast v14 as Field
                v18 = mul Field 2, v15
                v19 = add v16, v18
                v21 = array_get v11, index u32 4 -> u1
                v22 = not v21
                v23 = cast v21 as Field
                v24 = cast v22 as Field
                v25 = mul v19, v19
                v26 = mul v25, v24
                v27 = mul v25, Field 2
                v28 = mul v27, v23
                v29 = add v26, v28
                v31 = array_get v11, index u32 3 -> u1
                v32 = not v31
                v33 = cast v31 as Field
                v34 = cast v32 as Field
                v35 = mul v29, v29
                v36 = mul v35, v34
                v37 = mul v35, Field 2
                v38 = mul v37, v33
                v39 = add v36, v38
                v41 = array_get v11, index u32 2 -> u1
                v42 = not v41
                v43 = cast v41 as Field
                v44 = cast v42 as Field
                v45 = mul v39, v39
                v46 = mul v45, v44
                v47 = mul v45, Field 2
                v48 = mul v47, v43
                v49 = add v46, v48
                v51 = array_get v11, index u32 1 -> u1
                v52 = not v51
                v53 = cast v51 as Field
                v54 = cast v52 as Field
                v55 = mul v49, v49
                v56 = mul v55, v54
                v57 = mul v55, Field 2
                v58 = mul v57, v53
                v59 = add v56, v58
                v61 = array_get v11, index u32 0 -> u1
                v62 = not v61
                v63 = cast v61 as Field
                v64 = cast v62 as Field
                v65 = mul v59, v59
                v66 = mul v65, v64
                v67 = mul v65, Field 2
                v68 = mul v67, v63
                v69 = add v66, v68
                v70 = mul v4, v69
                v71 = cast v0 as Field
                v72 = mul v71, v70
                v73 = truncate v72 to 32 bits, max_bit_size: 254
                v74 = cast v73 as i32
                return v74
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
                v2 = lt v0, i32 0
                v3 = cast v2 as Field
                v4 = cast v0 as Field
                v5 = add v3, v4
                v6 = truncate v5 to 32 bits, max_bit_size: 33
                v7 = cast v6 as i32
                v9 = div v7, i32 4
                v10 = cast v2 as i32
                v11 = unchecked_sub v9, v10
                v12 = truncate v11 to 32 bits, max_bit_size: 33
                return v12
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
                v4 = lt v2, u32 31
                constrain v4 == u1 1, "attempt to bit-shift with overflow"
                v6 = cast v1 as Field
                v8 = call to_le_bits(v6) -> [u1; 6]
                v10 = array_get v8, index u32 5 -> u1
                v11 = not v10
                v12 = cast v10 as Field
                v13 = cast v11 as Field
                v15 = mul Field 2, v12
                v16 = add v13, v15
                v18 = array_get v8, index u32 4 -> u1
                v19 = not v18
                v20 = cast v18 as Field
                v21 = cast v19 as Field
                v22 = mul v16, v16
                v23 = mul v22, v21
                v24 = mul v22, Field 2
                v25 = mul v24, v20
                v26 = add v23, v25
                v28 = array_get v8, index u32 3 -> u1
                v29 = not v28
                v30 = cast v28 as Field
                v31 = cast v29 as Field
                v32 = mul v26, v26
                v33 = mul v32, v31
                v34 = mul v32, Field 2
                v35 = mul v34, v30
                v36 = add v33, v35
                v38 = array_get v8, index u32 2 -> u1
                v39 = not v38
                v40 = cast v38 as Field
                v41 = cast v39 as Field
                v42 = mul v36, v36
                v43 = mul v42, v41
                v44 = mul v42, Field 2
                v45 = mul v44, v40
                v46 = add v43, v45
                v48 = array_get v8, index u32 1 -> u1
                v49 = not v48
                v50 = cast v48 as Field
                v51 = cast v49 as Field
                v52 = mul v46, v46
                v53 = mul v52, v51
                v54 = mul v52, Field 2
                v55 = mul v54, v50
                v56 = add v53, v55
                v58 = array_get v8, index u32 0 -> u1
                v59 = not v58
                v60 = cast v58 as Field
                v61 = cast v59 as Field
                v62 = mul v56, v56
                v63 = mul v62, v61
                v64 = mul v62, Field 2
                v65 = mul v64, v60
                v66 = add v63, v65
                v67 = cast v66 as i32
                v69 = lt v0, i32 0
                v70 = cast v69 as Field
                v71 = cast v0 as Field
                v72 = add v70, v71
                v73 = truncate v72 to 32 bits, max_bit_size: 33
                v74 = cast v73 as i32
                v75 = div v74, v67
                v76 = cast v69 as i32
                v77 = unchecked_sub v75, v76
                v78 = truncate v77 to 32 bits, max_bit_size: 33
                return v78
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
}
