use std::sync::Arc;

use acvm::{FieldElement, acir::AcirField};

use crate::ssa::{
    ir::{
        function::Function,
        instruction::{ArrayOffset, Binary, BinaryOp, ConstrainError, Instruction},
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

            let mut bitshift_context = Context { context };
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
    /// Insert ssa instructions which computes lhs << rhs by doing lhs*2^rhs
    /// and truncate the result to bit_size
    pub(crate) fn insert_wrapping_shift_left(&mut self, lhs: ValueId, rhs: ValueId) -> ValueId {
        let typ = self.context.dfg.type_of_value(lhs).unwrap_numeric();
        let max_lhs_bits = self.context.dfg.get_value_max_num_bits(lhs);
        let max_bit_shift_size = self.context.dfg.get_numeric_constant(rhs).map_or_else(
            || {
                // If we don't know `rhs`'s value then it could be anything up to the number
                // of bits in the type, e.g. u32 means shifting by up to 32 bits as otherwise we get overflow.
                self.context.dfg.get_value_max_num_bits(rhs)
            },
            |rhs_constant| {
                // Happy case is that we know precisely by how many bits we're shifting by.
                rhs_constant.to_u128() as u32
            },
        );

        let pow = self.two_pow(rhs);

        // We cap the maximum number of bits here to ensure that we don't try and truncate using a
        // `max_bit_size` greater than what's allowable by the underlying `FieldElement` as this is meaningless.
        //
        // If `max_lhs_bits + max_bit_shift_size` were ever to exceed `FieldElement::max_num_bits()`,
        // then the constraint on `rhs` in `self.two_pow` should be broken.
        let max_bit = std::cmp::min(
            max_lhs_bits.checked_add(max_bit_shift_size).unwrap_or(FieldElement::max_num_bits()),
            FieldElement::max_num_bits(),
        );
        if max_bit <= typ.bit_size() {
            let pow = self.insert_cast(pow, typ);

            // Unchecked mul as it can't overflow
            self.insert_binary(lhs, BinaryOp::Mul { unchecked: true }, pow)
        } else {
            let lhs_field = self.insert_cast(lhs, NumericType::NativeField);
            let pow = self.insert_cast(pow, NumericType::NativeField);
            // Unchecked mul as this is a wrapping operation that we later truncate
            let result = self.insert_binary(lhs_field, BinaryOp::Mul { unchecked: true }, pow);
            let result = self.insert_truncate(result, typ.bit_size(), max_bit);
            self.insert_cast(result, typ)
        }
    }

    /// Insert ssa instructions which computes lhs >> rhs by doing lhs/2^rhs
    /// For negative signed integers, we do the division on the 1-complement representation of lhs,
    /// before converting back the result to the 2-complement representation.
    pub(crate) fn insert_shift_right(&mut self, lhs: ValueId, rhs: ValueId) -> ValueId {
        let lhs_typ = self.context.dfg.type_of_value(lhs).unwrap_numeric();

        let pow = self.two_pow(rhs);

        match lhs_typ {
            NumericType::Unsigned { .. } => {
                // unsigned right bit shift is just a normal division
                self.insert_binary(lhs, BinaryOp::Div, pow)
            }
            NumericType::Signed { bit_size } => {
                let pow = self.insert_cast(pow, lhs_typ);

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
    fn two_pow(&mut self, exponent: ValueId) -> ValueId {
        // Require that exponent < bit_size, ensuring that `pow` returns a value consistent with `lhs`'s type.
        // This isn't strictly necessary but it emits a more user-friendly error message.
        self.enforce_bitshift_rhs_lt_bit_size(exponent);

        let typ = self.context.dfg.type_of_value(exponent);
        let max_bit_size = typ.bit_size();

        if let Some(exponent_const) = self.context.dfg.get_numeric_constant(exponent) {
            if exponent_const < FieldElement::from(max_bit_size) {
                let pow = FieldElement::from(2u32).pow(&exponent_const);
                return self.numeric_constant(pow, typ.unwrap_numeric());
            }
        }

        // We store the powers of 2 as an unsigned type to avoid initializing duplicate arrays for signed and unsigned types.
        let equivalent_unsigned_type = Type::unsigned(max_bit_size);
        let lookup_array = (0..max_bit_size)
            .map(|i| {
                self.context.dfg.make_constant(
                    FieldElement::from(1u128 << i),
                    NumericType::unsigned(max_bit_size),
                )
            })
            .collect::<im::Vector<_>>();
        let lookup_array = Instruction::MakeArray {
            elements: lookup_array,
            typ: Type::Array(Arc::new(vec![equivalent_unsigned_type.clone()]), max_bit_size),
        };
        let bitshift_lookup = self
            .context
            .insert_instruction(
                lookup_array,
                Some(vec![Type::Array(
                    Arc::new(vec![equivalent_unsigned_type.clone()]),
                    max_bit_size,
                )]),
            )
            .first();

        let index = self.insert_cast(exponent, NumericType::length_type());
        self.insert_array_get(bitshift_lookup, index, equivalent_unsigned_type.clone())
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
                constrain v3 == u1 1, "attempt to bit-shift with overflow"
                v36 = make_array [u32 1, u32 2, u32 4, u32 8, u32 16, u32 32, u32 64, u32 128, u32 256, u32 512, u32 1024, u32 2048, u32 4096, u32 8192, u32 16384, u32 32768, u32 65536, u32 131072, u32 262144, u32 524288, u32 1048576, u32 2097152, u32 4194304, u32 8388608, u32 16777216, u32 33554432, u32 67108864, u32 134217728, u32 268435456, u32 536870912, u32 1073741824, u32 2147483648] : [u32; 32]
                v37 = array_get v36, index v1 -> u32
                v38 = cast v0 as Field
                v39 = cast v37 as Field
                v40 = mul v38, v39
                v41 = truncate v40 to 32 bits, max_bit_size: 64
                v42 = cast v41 as u32
                return v42
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
                v35 = make_array [u32 1, u32 2, u32 4, u32 8, u32 16, u32 32, u32 64, u32 128, u32 256, u32 512, u32 1024, u32 2048, u32 4096, u32 8192, u32 16384, u32 32768, u32 65536, u32 131072, u32 262144, u32 524288, u32 1048576, u32 2097152, u32 4194304, u32 8388608, u32 16777216, u32 33554432, u32 67108864, u32 134217728, u32 268435456, u32 536870912, u32 1073741824, u32 2147483648] : [u32; 32]
                v37 = array_get v35, index u32 255 -> u32
                v38 = cast v0 as Field
                v39 = cast v37 as Field
                v40 = mul v38, v39
                v41 = truncate v40 to 32 bits, max_bit_size: 254
                v42 = cast v41 as u32
                return v42
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
                v36 = make_array [u32 1, u32 2, u32 4, u32 8, u32 16, u32 32, u32 64, u32 128, u32 256, u32 512, u32 1024, u32 2048, u32 4096, u32 8192, u32 16384, u32 32768, u32 65536, u32 131072, u32 262144, u32 524288, u32 1048576, u32 2097152, u32 4194304, u32 8388608, u32 16777216, u32 33554432, u32 67108864, u32 134217728, u32 268435456, u32 536870912, u32 1073741824, u32 2147483648] : [u32; 32]
                v37 = array_get v36, index v1 -> u32
                v38 = div v0, v37
                return v38
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
                v4 = lt v2, u32 31
                constrain v4 == u1 1, "attempt to bit-shift with overflow"
                v38 = make_array [u32 1, u32 2, u32 4, u32 8, u32 16, u32 32, u32 64, u32 128, u32 256, u32 512, u32 1024, u32 2048, u32 4096, u32 8192, u32 16384, u32 32768, u32 65536, u32 131072, u32 262144, u32 524288, u32 1048576, u32 2097152, u32 4194304, u32 8388608, u32 16777216, u32 33554432, u32 67108864, u32 134217728, u32 268435456, u32 536870912, u32 1073741824, u32 2147483648] : [u32; 32]
                v39 = cast v1 as u32
                v40 = array_get v38, index v39 -> u32
                v41 = cast v0 as Field
                v42 = cast v40 as Field
                v43 = mul v41, v42
                v44 = truncate v43 to 32 bits, max_bit_size: 64
                v45 = cast v44 as i32
                return v45
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
                v38 = make_array [u32 1, u32 2, u32 4, u32 8, u32 16, u32 32, u32 64, u32 128, u32 256, u32 512, u32 1024, u32 2048, u32 4096, u32 8192, u32 16384, u32 32768, u32 65536, u32 131072, u32 262144, u32 524288, u32 1048576, u32 2097152, u32 4194304, u32 8388608, u32 16777216, u32 33554432, u32 67108864, u32 134217728, u32 268435456, u32 536870912, u32 1073741824, u32 2147483648] : [u32; 32]
                v39 = cast v1 as u32
                v40 = array_get v38, index v39 -> u32
                v41 = cast v40 as i32
                v43 = lt v0, i32 0
                v44 = cast v43 as Field
                v45 = cast v0 as Field
                v46 = add v44, v45
                v47 = truncate v46 to 32 bits, max_bit_size: 33
                v48 = cast v47 as i32
                v49 = div v48, v41
                v50 = cast v43 as i32
                v51 = unchecked_sub v49, v50
                v52 = truncate v51 to 32 bits, max_bit_size: 33
                return v52
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
            v10 = make_array [u8 1, u8 2, u8 4, u8 8, u8 16, u8 32, u8 64, u8 128] : [u8; 8]
            v12 = array_get v10, index u32 98 -> u8
            v13 = div u8 1, v12
            v15 = eq v13, u8 0
            jmpif v15 then: b7, else: b8
          b1():
            jmp b3()
          b2():
            jmp b3()
          b3():
            v22 = eq v20, u8 1
            jmpif v22 then: b4, else: b5
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
            v16 = eq v13, u8 1
            jmpif v16 then: b10, else: b11
          b10():
            jmp b12()
          b11():
            jmp b12()
          b12():
            constrain u1 0 == u1 1, "attempt to bit-shift with overflow"
            v17 = make_array [u8 1, u8 2, u8 4, u8 8, u8 16, u8 32, u8 64, u8 128] : [u8; 8]
            v19 = array_get v17, index u32 99 -> u8
            v20 = div u8 1, v19
            v21 = eq v20, u8 0
            jmpif v21 then: b1, else: b2
        }
        "#);
    }
}
