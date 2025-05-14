use std::{borrow::Cow, sync::Arc};

use acvm::{FieldElement, acir::AcirField};
use noirc_errors::call_stack::CallStackId;

use crate::ssa::{
    ir::{
        dfg::InsertInstructionResult,
        function::Function,
        instruction::{Binary, BinaryOp, Endian, Instruction, Intrinsic},
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
        // Nullify rhs in case of overflow, to ensure that pow returns a value compatible with lhs
        let rhs = self.insert_binary(
            rhs_is_less_than_bit_size_with_rhs_typ,
            BinaryOp::Mul { unchecked: true },
            rhs,
        );
        let pow = self.pow(base, rhs);
        let pow = self.insert_cast(pow, lhs_typ);
        let result = if lhs_typ.is_unsigned() {
            // unsigned right bit shift is just a normal division
            self.insert_binary(lhs, BinaryOp::Div, pow)
        } else {
            // Get the sign of the operand; positive signed operand will just do a division as well
            let zero = self.numeric_constant(FieldElement::zero(), NumericType::signed(bit_size));
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
            self.insert_truncate(shifted, bit_size, bit_size + 1)
        };
        // Returns 0 in case of overflow
        let rhs_is_less_than_bit_size_with_lhs_typ =
            self.insert_cast(rhs_is_less_than_bit_size, lhs_typ);
        self.insert_binary(
            rhs_is_less_than_bit_size_with_lhs_typ,
            BinaryOp::Mul { unchecked: true },
            result,
        )
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
        if let Type::Numeric(NumericType::Unsigned { bit_size }) = typ {
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
                let idx = self.field_constant(FieldElement::from((bit_size - i) as i128));
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
            r
        } else {
            unreachable!("Value must be unsigned in power operation");
        }
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
        self.insert_instruction(Instruction::ArrayGet { array, index }, element_type).first()
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
            v9 = array_get v7, index Field 7 -> u1
            v10 = not v9
            v11 = cast v9 as Field
            v12 = cast v10 as Field
            v14 = mul Field 2, v11
            v15 = add v12, v14
            v17 = array_get v7, index Field 6 -> u1
            v18 = not v17
            v19 = cast v17 as Field
            v20 = cast v18 as Field
            v21 = mul v15, v15
            v22 = mul v21, v20
            v23 = mul v21, Field 2
            v24 = mul v23, v19
            v25 = add v22, v24
            v27 = array_get v7, index Field 5 -> u1
            v28 = not v27
            v29 = cast v27 as Field
            v30 = cast v28 as Field
            v31 = mul v25, v25
            v32 = mul v31, v30
            v33 = mul v31, Field 2
            v34 = mul v33, v29
            v35 = add v32, v34
            v37 = array_get v7, index Field 4 -> u1
            v38 = not v37
            v39 = cast v37 as Field
            v40 = cast v38 as Field
            v41 = mul v35, v35
            v42 = mul v41, v40
            v43 = mul v41, Field 2
            v44 = mul v43, v39
            v45 = add v42, v44
            v47 = array_get v7, index Field 3 -> u1
            v48 = not v47
            v49 = cast v47 as Field
            v50 = cast v48 as Field
            v51 = mul v45, v45
            v52 = mul v51, v50
            v53 = mul v51, Field 2
            v54 = mul v53, v49
            v55 = add v52, v54
            v56 = array_get v7, index Field 2 -> u1
            v57 = not v56
            v58 = cast v56 as Field
            v59 = cast v57 as Field
            v60 = mul v55, v55
            v61 = mul v60, v59
            v62 = mul v60, Field 2
            v63 = mul v62, v58
            v64 = add v61, v63
            v66 = array_get v7, index Field 1 -> u1
            v67 = not v66
            v68 = cast v66 as Field
            v69 = cast v67 as Field
            v70 = mul v64, v64
            v71 = mul v70, v69
            v72 = mul v70, Field 2
            v73 = mul v72, v68
            v74 = add v71, v73
            v76 = array_get v7, index Field 0 -> u1
            v77 = not v76
            v78 = cast v76 as Field
            v79 = cast v77 as Field
            v80 = mul v74, v74
            v81 = mul v80, v79
            v82 = mul v80, Field 2
            v83 = mul v82, v78
            v84 = add v81, v83
            v85 = cast v84 as u32
            v86 = unchecked_mul v4, v85
            v87 = cast v0 as Field
            v88 = cast v86 as Field
            v89 = mul v87, v88
            v90 = truncate v89 to 32 bits, max_bit_size: 254
            v91 = cast v90 as u32
            v92 = truncate v91 to 32 bits, max_bit_size: 33
            return v91
        }
        ");
    }
}
