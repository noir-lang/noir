use std::{borrow::Cow, sync::Arc};

use acvm::{FieldElement, acir::AcirField};

use crate::ssa::{
    ir::{
        call_stack::CallStackId,
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
        let pow = self.pow(base, rhs);
        let pow = self.pow_or_max_for_bit_size(pow, rhs, bit_size, lhs_typ);
        let pow = self.insert_cast(pow, lhs_typ);
        if lhs_typ.is_unsigned() {
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
        }
    }

    /// Returns `pow` or the maximum value allowed for `typ` if 2^rhs is guaranteed to exceed that maximum.
    fn pow_or_max_for_bit_size(
        &mut self,
        pow: ValueId,
        rhs: ValueId,
        bit_size: u32,
        typ: NumericType,
    ) -> ValueId {
        let max = if typ.is_unsigned() {
            if bit_size == 128 { u128::MAX } else { (1_u128 << bit_size) - 1 }
        } else {
            1_u128 << (bit_size - 1)
        };
        let max = self.field_constant(FieldElement::from(max));

        // Here we check whether rhs is less than the bit_size: if it's not then it will overflow.
        // Then we do:
        //
        // rhs_is_less_than_bit_size = lt rhs, bit_size
        // rhs_is_not_less_than_bit_size = not rhs_is_less_than_bit_size
        // pow_when_is_less_than_bit_size = rhs_is_less_than_bit_size * pow
        // pow_when_is_not_less_than_bit_size = rhs_is_not_less_than_bit_size * max
        // pow = add pow_when_is_less_than_bit_size, pow_when_is_not_less_than_bit_size
        //
        // All operations here are unchecked because they work on field types.
        let rhs_typ = self.context.dfg.type_of_value(rhs).unwrap_numeric();
        let bit_size = self.numeric_constant(bit_size as u128, rhs_typ);
        let rhs_is_less_than_bit_size = self.insert_binary(rhs, BinaryOp::Lt, bit_size);
        let rhs_is_not_less_than_bit_size = self.insert_not(rhs_is_less_than_bit_size);
        let rhs_is_less_than_bit_size =
            self.insert_cast(rhs_is_less_than_bit_size, NumericType::NativeField);
        let rhs_is_not_less_than_bit_size =
            self.insert_cast(rhs_is_not_less_than_bit_size, NumericType::NativeField);
        let pow_when_is_less_than_bit_size =
            self.insert_binary(rhs_is_less_than_bit_size, BinaryOp::Mul { unchecked: true }, pow);
        let pow_when_is_not_less_than_bit_size = self.insert_binary(
            rhs_is_not_less_than_bit_size,
            BinaryOp::Mul { unchecked: true },
            max,
        );
        self.insert_binary(
            pow_when_is_less_than_bit_size,
            BinaryOp::Add { unchecked: true },
            pow_when_is_not_less_than_bit_size,
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
            let rhs_bits = self.insert_call(to_bits, vec![rhs], result_types);

            let rhs_bits = rhs_bits[0];
            let one = self.field_constant(FieldElement::one());
            let mut r = one;
            // All operations are unchecked as we're acting on Field types (which are always unchecked)
            for i in 1..bit_size + 1 {
                let r_squared = self.insert_binary(r, BinaryOp::Mul { unchecked: true }, r);
                let a = self.insert_binary(r_squared, BinaryOp::Mul { unchecked: true }, lhs);
                let idx = self.field_constant(FieldElement::from((bit_size - i) as i128));
                let b = self.insert_array_get(rhs_bits, idx, Type::bool());
                let not_b = self.insert_not(b);
                let b = self.insert_cast(b, NumericType::NativeField);
                let not_b = self.insert_cast(not_b, NumericType::NativeField);
                let r1 = self.insert_binary(a, BinaryOp::Mul { unchecked: true }, b);
                let r2 = self.insert_binary(r_squared, BinaryOp::Mul { unchecked: true }, not_b);
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
    fn removes_shl() {
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
}
