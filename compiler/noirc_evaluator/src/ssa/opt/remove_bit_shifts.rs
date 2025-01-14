use std::{borrow::Cow, sync::Arc};

use acvm::{acir::AcirField, FieldElement};

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        call_stack::CallStackId,
        dfg::InsertInstructionResult,
        function::Function,
        instruction::{Binary, BinaryOp, Endian, Instruction, InstructionId, Intrinsic},
        types::{NumericType, Type},
        value::ValueId,
    },
    ssa_gen::Ssa,
};

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
        let mut context = Context {
            function: self,
            new_instructions: Vec::new(),
            block,
            call_stack: CallStackId::root(),
        };

        context.remove_bit_shifts();
    }
}

struct Context<'f> {
    function: &'f mut Function,
    new_instructions: Vec<InstructionId>,

    block: BasicBlockId,
    call_stack: CallStackId,
}

impl Context<'_> {
    fn remove_bit_shifts(&mut self) {
        let instructions = self.function.dfg[self.block].take_instructions();

        for instruction_id in instructions {
            match self.function.dfg[instruction_id] {
                Instruction::Binary(Binary { lhs, rhs, operator })
                    if matches!(operator, BinaryOp::Shl | BinaryOp::Shr) =>
                {
                    self.call_stack =
                        self.function.dfg.get_instruction_call_stack_id(instruction_id);
                    let old_result =
                        *self.function.dfg.instruction_results(instruction_id).first().unwrap();

                    let bit_size = match self.function.dfg.type_of_value(lhs) {
                        Type::Numeric(NumericType::Signed { bit_size })
                        | Type::Numeric(NumericType::Unsigned { bit_size }) => bit_size,
                        _ => unreachable!("ICE: right-shift attempted on non-integer"),
                    };
                    let new_result = if operator == BinaryOp::Shl {
                        self.insert_wrapping_shift_left(lhs, rhs, bit_size)
                    } else {
                        self.insert_shift_right(lhs, rhs, bit_size)
                    };

                    self.function.dfg.set_value_from_id(old_result, new_result);
                }
                _ => {
                    self.new_instructions.push(instruction_id);
                }
            };
        }

        *self.function.dfg[self.block].instructions_mut() =
            std::mem::take(&mut self.new_instructions);
    }

    /// Insert ssa instructions which computes lhs << rhs by doing lhs*2^rhs
    /// and truncate the result to bit_size
    pub(crate) fn insert_wrapping_shift_left(
        &mut self,
        lhs: ValueId,
        rhs: ValueId,
        bit_size: u32,
    ) -> ValueId {
        let base = self.field_constant(FieldElement::from(2_u128));
        let typ = self.function.dfg.type_of_value(lhs).unwrap_numeric();
        let (max_bit, pow) = if let Some(rhs_constant) = self.function.dfg.get_numeric_constant(rhs)
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

            let max_lhs_bits = self.function.dfg.get_value_max_num_bits(lhs);
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
        let lhs_typ = self.function.dfg.type_of_value(lhs).unwrap_numeric();
        let base = self.field_constant(FieldElement::from(2_u128));
        let pow = self.pow(base, rhs);
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
        let typ = self.function.dfg.type_of_value(rhs);
        if let Type::Numeric(NumericType::Unsigned { bit_size }) = typ {
            let to_bits = self.function.dfg.import_intrinsic(Intrinsic::ToBits(Endian::Little));
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
        self.function.dfg.make_constant(constant, NumericType::NativeField)
    }

    /// Insert a numeric constant into the current function
    pub(crate) fn numeric_constant(
        &mut self,
        value: impl Into<FieldElement>,
        typ: NumericType,
    ) -> ValueId {
        self.function.dfg.make_constant(value.into(), typ)
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
        let result = self.function.dfg.insert_instruction_and_results(
            instruction,
            self.block,
            ctrl_typevars,
            self.call_stack,
        );

        if let InsertInstructionResult::Results(instruction_id, _) = result {
            self.new_instructions.push(instruction_id);
        }

        result
    }
}
