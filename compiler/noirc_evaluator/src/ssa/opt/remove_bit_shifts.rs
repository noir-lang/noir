use std::{borrow::Cow, sync::Arc};

use acvm::{FieldElement, acir::AcirField};
use noirc_errors::call_stack::CallStackId;

use crate::ssa::{
    ir::{
        dfg::InsertInstructionResult,
        function::Function,
        instruction::{ArrayOffset, Binary, BinaryOp, Endian, Instruction, Intrinsic},
        types::{NumericType, Type},
        value::ValueId,
    },
    ssa_gen::Ssa,
};

use super::simple_optimization::SimpleOptimizationContext;

impl Ssa {
    /// Performs constant folding on each instruction.
    ///
    /// See [`constant_folding`][self] module for more information.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn remove_bit_shifts(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            function.remove_bit_shifts();
        }
        self
    }
}

impl Function {
    /// The structure of this pass is simple:
    /// Go through each block and re-insert all instructions.
    pub(crate) fn remove_bit_shifts(&mut self) {
        if self.runtime().is_brillig() {
            return;
        }

        let block = self.entry_block();

        // Make sure this optimization runs when there's only one block
        assert_eq!(self.dfg[block].successors().count(), 0);

        self.simple_reachable_blocks_optimization(|context| {
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

            let call_stack = context.dfg.get_instruction_call_stack_id(instruction_id);
            let old_result = *context.dfg.instruction_results(instruction_id).first().unwrap();

            let bit_size = match context.dfg.type_of_value(lhs) {
                Type::Numeric(NumericType::Signed { bit_size })
                | Type::Numeric(NumericType::Unsigned { bit_size }) => bit_size,
                _ => unreachable!("ICE: right-shift attempted on non-integer"),
            };

            let new_result = if operator == BinaryOp::Shl {
                let mut context = Context { context, call_stack };
                context.insert_wrapping_shift_left(lhs, rhs, bit_size)
            } else {
                let mut context = Context { context, call_stack };
                context.insert_shift_right(lhs, rhs, bit_size)
            };

            context.replace_value(old_result, new_result);
        });

        #[cfg(debug_assertions)]
        remove_bit_shifts_post_check(self);
    }
}

struct Context<'m, 'dfg, 'mapping> {
    context: &'m mut SimpleOptimizationContext<'dfg, 'mapping>,
    call_stack: CallStackId,
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
        let base = self.field_constant(FieldElement::from(2_u128));
        let typ = self.context.dfg.type_of_value(lhs).unwrap_numeric();
        let (max_bit, pow) = if let Some(rhs_constant) = self.context.dfg.get_numeric_constant(rhs)
        {
            // Happy case is that we know precisely by how many bits the integer will
            // increase: lhs_bit_size + rhs
            let bit_shift_size = rhs_constant.to_u128() as u32;

            let (rhs_bit_size_pow_2, overflows) = 2_u128.overflowing_pow(bit_shift_size);
            if overflows {
                assert!(bit_size < 128, "ICE - shift left with big integers are not supported");
                if bit_size < 128 {
                    let zero = self.numeric_constant(FieldElement::zero(), typ);
                    return InsertInstructionResult::SimplifiedTo(zero).first();
                }
            }
            let pow = self.numeric_constant(FieldElement::from(rhs_bit_size_pow_2), typ);

            let max_lhs_bits = self.context.dfg.get_value_max_num_bits(lhs);
            let max_bit_size = max_lhs_bits + bit_shift_size;
            // There is no point trying to truncate to more than the Field size.
            // A higher `max_lhs_bits` input can come from trying to left-shift a Field.
            let max_bit_size = max_bit_size.min(NumericType::NativeField.bit_size());
            (max_bit_size, pow)
        } else {
            // we use a predicate to nullify the result in case of overflow
            let u8_type = NumericType::unsigned(8);
            let bit_size_var = self.numeric_constant(FieldElement::from(bit_size as u128), u8_type);
            let overflow = self.insert_binary(rhs, BinaryOp::Lt, bit_size_var);
            let predicate = self.insert_cast(overflow, typ);
            let pow = self.pow(base, rhs);
            let pow = self.insert_cast(pow, typ);

            // Unchecked mul because `predicate` will be 1 or 0
            (
                FieldElement::max_num_bits(),
                self.insert_binary(predicate, BinaryOp::Mul { unchecked: true }, pow),
            )
        };

        if max_bit <= bit_size {
            // Unchecked mul as it can't overflow
            self.insert_binary(lhs, BinaryOp::Mul { unchecked: true }, pow)
        } else {
            let lhs_field = self.insert_cast(lhs, NumericType::NativeField);
            let pow_field = self.insert_cast(pow, NumericType::NativeField);
            // Unchecked mul as this is a wrapping operation that we later truncate
            let result =
                self.insert_binary(lhs_field, BinaryOp::Mul { unchecked: true }, pow_field);
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
        let base = self.field_constant(FieldElement::from(2_u128));
        let rhs_typ = self.context.dfg.type_of_value(rhs).unwrap_numeric();
        //Check whether rhs is less than the bit_size: if it's not then it will overflow and we will return 0 instead.
        let bit_size_value = self.numeric_constant(bit_size as u128, rhs_typ);
        let rhs_is_less_than_bit_size = self.insert_binary(rhs, BinaryOp::Lt, bit_size_value);
        let rhs_is_less_than_bit_size_with_rhs_typ =
            self.insert_cast(rhs_is_less_than_bit_size, rhs_typ);
        let rhs_is_less_than_bit_size_with_lhs_typ =
            self.insert_cast(rhs_is_less_than_bit_size, lhs_typ);
        // Nullify rhs in case of overflow, to ensure that pow returns a value compatible with lhs
        let rhs = self.insert_binary(
            rhs_is_less_than_bit_size_with_rhs_typ,
            BinaryOp::Mul { unchecked: true },
            rhs,
        );
        let pow = self.pow(base, rhs);
        let pow = self.insert_cast(pow, lhs_typ);

        if lhs_typ.is_unsigned() {
            // unsigned right bit shift is just a normal division
            let result = self.insert_binary(lhs, BinaryOp::Div, pow);
            // In case of overflow, pow is 1, because rhs was nullified, so we return explicitly 0.
            return self.insert_binary(
                rhs_is_less_than_bit_size_with_lhs_typ,
                BinaryOp::Mul { unchecked: true },
                result,
            );
        }
        // Get the sign of the operand; positive signed operand will just do a division as well
        let zero = self.numeric_constant(FieldElement::zero(), NumericType::signed(bit_size));
        let lhs_sign = self.insert_binary(lhs, BinaryOp::Lt, zero);
        let lhs_sign_as_field = self.insert_cast(lhs_sign, NumericType::NativeField);
        let lhs_as_field = self.insert_cast(lhs, NumericType::NativeField);
        // For negative numbers, convert to 1-complement using wrapping addition of a + 1
        // Unchecked add as these are fields
        let one_complement =
            self.insert_binary(lhs_sign_as_field, BinaryOp::Add { unchecked: true }, lhs_as_field);
        let one_complement = self.insert_truncate(one_complement, bit_size, bit_size + 1);
        let one_complement = self.insert_cast(one_complement, NumericType::signed(bit_size));
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
        let result = self.insert_truncate(shifted, bit_size, bit_size + 1);

        // Returns 0 or -1 in case of overflow:
        // In  case of overflow, and because rhs was nullified, we need to
        // return the correct value, which is 0 or -1 depending on the sign of lhs

        // Computes -1, or 0 if lhs is positive: is the expected result if there is an overflow
        let minus_one = self.numeric_constant(
            NumericType::Unsigned { bit_size }.max_value().expect("Invalid bit size"),
            lhs_typ,
        );
        let minus_one_or_zero =
            self.insert_binary(minus_one, BinaryOp::Mul { unchecked: true }, lhs_sign_as_int);
        // -1, or 0 if lhs is positive or if there is no overflow
        let one = self.numeric_constant(FieldElement::one(), lhs_typ);
        let no_overflow = self.insert_binary(
            one,
            BinaryOp::Sub { unchecked: true },
            rhs_is_less_than_bit_size_with_lhs_typ,
        );
        let minus_one_or_zero =
            self.insert_binary(minus_one_or_zero, BinaryOp::Mul { unchecked: true }, no_overflow);

        // predicated result: 0 if overflow, else: result
        let result = self.insert_binary(
            rhs_is_less_than_bit_size_with_lhs_typ,
            BinaryOp::Mul { unchecked: true },
            result,
        );

        // result + minus_one_or_zero gives the expected result in all cases
        self.insert_binary(result, BinaryOp::Add { unchecked: true }, minus_one_or_zero)
    }

    /// Computes lhs^rhs via square&multiply, using the bits decomposition of rhs
    /// Pseudo-code of the computation:
    /// let mut r = 1;
    /// let rhs_bits = to_bits(rhs);
    /// for i in 1 .. bit_size + 1 {
    ///     let r_squared = r * r;
    ///     let b = rhs_bits[bit_size - i];
    ///     r = (r_squared * lhs * b) + (1 - b) * r_squared;
    /// }
    fn pow(&mut self, lhs: ValueId, rhs: ValueId) -> ValueId {
        let typ = self.context.dfg.type_of_value(rhs);
        let Type::Numeric(NumericType::Unsigned { bit_size }) = typ else {
            unreachable!("Value must be unsigned in power operation");
        };

        let to_bits = self.context.dfg.import_intrinsic(Intrinsic::ToBits(Endian::Little));
        let result_types = vec![Type::Array(Arc::new(vec![Type::bool()]), bit_size)];

        // A call to ToBits can only be done with a field argument (rhs is always u8 here)
        let rhs_as_field = self.insert_cast(rhs, NumericType::NativeField);
        let rhs_bits = self.insert_call(to_bits, vec![rhs_as_field], result_types);

        let rhs_bits = rhs_bits[0];
        let one = self.field_constant(FieldElement::one());
        let mut r = one;
        // All operations are unchecked as we're acting on Field types (which are always unchecked)
        for i in 1..bit_size + 1 {
            let idx = self.numeric_constant(
                FieldElement::from((bit_size - i) as i128),
                NumericType::length_type(),
            );
            let b = self.insert_array_get(rhs_bits, idx, Type::bool());
            let not_b = self.insert_not(b);
            let b = self.insert_cast(b, NumericType::NativeField);
            let not_b = self.insert_cast(not_b, NumericType::NativeField);

            let r_squared = self.insert_binary(r, BinaryOp::Mul { unchecked: true }, r);
            let r1 = self.insert_binary(r_squared, BinaryOp::Mul { unchecked: true }, not_b);
            let a = self.insert_binary(r_squared, BinaryOp::Mul { unchecked: true }, lhs);
            let r2 = self.insert_binary(a, BinaryOp::Mul { unchecked: true }, b);
            r = self.insert_binary(r1, BinaryOp::Add { unchecked: true }, r2);
        }

        assert!(
            matches!(self.context.dfg.type_of_value(r).unwrap_numeric(), NumericType::NativeField),
            "ICE: pow is expected to always return a NativeField"
        );

        r
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
        self.insert_instruction(instruction, None).first()
    }

    /// Insert a not instruction at the end of the current block.
    /// Returns the result of the instruction.
    pub(crate) fn insert_not(&mut self, rhs: ValueId) -> ValueId {
        self.insert_instruction(Instruction::Not(rhs), None).first()
    }

    /// Insert a truncate instruction at the end of the current block.
    /// Returns the result of the truncate instruction.
    pub(crate) fn insert_truncate(
        &mut self,
        value: ValueId,
        bit_size: u32,
        max_bit_size: u32,
    ) -> ValueId {
        self.insert_instruction(Instruction::Truncate { value, bit_size, max_bit_size }, None)
            .first()
    }

    /// Insert a cast instruction at the end of the current block.
    /// Returns the result of the cast instruction.
    pub(crate) fn insert_cast(&mut self, value: ValueId, typ: NumericType) -> ValueId {
        self.insert_instruction(Instruction::Cast(value, typ), None).first()
    }

    /// Insert a call instruction at the end of the current block and return
    /// the results of the call.
    pub(crate) fn insert_call(
        &mut self,
        func: ValueId,
        arguments: Vec<ValueId>,
        result_types: Vec<Type>,
    ) -> Cow<[ValueId]> {
        self.insert_instruction(Instruction::Call { func, arguments }, Some(result_types)).results()
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
        self.insert_instruction(instruction, element_type).first()
    }

    pub(crate) fn insert_instruction(
        &mut self,
        instruction: Instruction,
        ctrl_typevars: Option<Vec<Type>>,
    ) -> InsertInstructionResult {
        self.context.dfg.insert_instruction_and_results(
            instruction,
            self.context.block_id,
            ctrl_typevars,
            self.call_stack,
        )
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

    #[test]
    fn removes_shl_with_constant_rhs() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32):
            v2 = shl v0, u8 2
            v3 = truncate v2 to 32 bits, max_bit_size: 33
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
            v6 = truncate v5 to 32 bits, max_bit_size: 33
            return v5
        }
        ");
    }

    #[test]
    fn removes_shl_with_non_constant_rhs() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u8):
            v2 = shl v0, v1
            v3 = truncate v2 to 32 bits, max_bit_size: 33
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.remove_bit_shifts();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u8):
            v3 = lt v1, u8 32
            v4 = cast v3 as u32
            v5 = cast v1 as Field
            v7 = call to_le_bits(v5) -> [u1; 8]
            v9 = array_get v7, index u32 7 -> u1
            v10 = not v9
            v11 = cast v9 as Field
            v12 = cast v10 as Field
            v14 = mul Field 2, v11
            v15 = add v12, v14
            v17 = array_get v7, index u32 6 -> u1
            v18 = not v17
            v19 = cast v17 as Field
            v20 = cast v18 as Field
            v21 = mul v15, v15
            v22 = mul v21, v20
            v23 = mul v21, Field 2
            v24 = mul v23, v19
            v25 = add v22, v24
            v27 = array_get v7, index u32 5 -> u1
            v28 = not v27
            v29 = cast v27 as Field
            v30 = cast v28 as Field
            v31 = mul v25, v25
            v32 = mul v31, v30
            v33 = mul v31, Field 2
            v34 = mul v33, v29
            v35 = add v32, v34
            v37 = array_get v7, index u32 4 -> u1
            v38 = not v37
            v39 = cast v37 as Field
            v40 = cast v38 as Field
            v41 = mul v35, v35
            v42 = mul v41, v40
            v43 = mul v41, Field 2
            v44 = mul v43, v39
            v45 = add v42, v44
            v47 = array_get v7, index u32 3 -> u1
            v48 = not v47
            v49 = cast v47 as Field
            v50 = cast v48 as Field
            v51 = mul v45, v45
            v52 = mul v51, v50
            v53 = mul v51, Field 2
            v54 = mul v53, v49
            v55 = add v52, v54
            v57 = array_get v7, index u32 2 -> u1
            v58 = not v57
            v59 = cast v57 as Field
            v60 = cast v58 as Field
            v61 = mul v55, v55
            v62 = mul v61, v60
            v63 = mul v61, Field 2
            v64 = mul v63, v59
            v65 = add v62, v64
            v67 = array_get v7, index u32 1 -> u1
            v68 = not v67
            v69 = cast v67 as Field
            v70 = cast v68 as Field
            v71 = mul v65, v65
            v72 = mul v71, v70
            v73 = mul v71, Field 2
            v74 = mul v73, v69
            v75 = add v72, v74
            v77 = array_get v7, index u32 0 -> u1
            v78 = not v77
            v79 = cast v77 as Field
            v80 = cast v78 as Field
            v81 = mul v75, v75
            v82 = mul v81, v80
            v83 = mul v81, Field 2
            v84 = mul v83, v79
            v85 = add v82, v84
            v86 = cast v85 as u32
            v87 = unchecked_mul v4, v86
            v88 = cast v0 as Field
            v89 = cast v87 as Field
            v90 = mul v88, v89
            v91 = truncate v90 to 32 bits, max_bit_size: 254
            v92 = cast v91 as u32
            v93 = truncate v92 to 32 bits, max_bit_size: 33
            return v92
        }
        ");
    }
}
