use std::{borrow::Cow, rc::Rc};

use acvm::{acir::AcirField, FieldElement};

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        dfg::{CallStack, InsertInstructionResult},
        function::{Function, RuntimeType},
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
            remove_bit_shifts(function);
        }
        self
    }
}

/// The structure of this pass is simple:
/// Go through each block and re-insert all instructions.
fn remove_bit_shifts(function: &mut Function) {
    if let RuntimeType::Brillig = function.runtime() {
        return;
    }

    let block = function.entry_block();
    let mut context =
        Context { function, new_instructions: Vec::new(), block, call_stack: CallStack::default() };

    context.remove_bit_shifts();
}

struct Context<'f> {
    function: &'f mut Function,
    new_instructions: Vec<InstructionId>,

    block: BasicBlockId,
    call_stack: CallStack,
}

impl Context<'_> {
    fn remove_bit_shifts(&mut self) {
        let instructions = self.function.dfg[self.block].take_instructions();

        for instruction_id in instructions {
            match self.function.dfg[instruction_id] {
                Instruction::Binary(Binary { lhs, rhs, operator })
                    if matches!(operator, BinaryOp::Shl | BinaryOp::Shr) =>
                {
                    self.call_stack = self.function.dfg.get_call_stack(instruction_id).clone();
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
        let typ = self.function.dfg.type_of_value(lhs);
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
            let pow = self.numeric_constant(FieldElement::from(rhs_bit_size_pow_2), typ.clone());

            let max_lhs_bits = self.function.dfg.get_value_max_num_bits(lhs);

            (max_lhs_bits + bit_shift_size, pow)
        } else {
            // we use a predicate to nullify the result in case of overflow
            let bit_size_var =
                self.numeric_constant(FieldElement::from(bit_size as u128), Type::unsigned(8));
            let overflow = self.insert_binary(rhs, BinaryOp::Lt, bit_size_var);
            let predicate = self.insert_cast(overflow, typ.clone());
            // we can safely cast to unsigned because overflow_checks prevent bit-shift with a negative value
            let rhs_unsigned = self.insert_cast(rhs, Type::unsigned(bit_size));
            let pow = self.pow(base, rhs_unsigned);
            let pow = self.insert_cast(pow, typ.clone());
            (FieldElement::max_num_bits(), self.insert_binary(predicate, BinaryOp::Mul, pow))
        };

        if max_bit <= bit_size {
            self.insert_binary(lhs, BinaryOp::Mul, pow)
        } else {
            let lhs_field = self.insert_cast(lhs, Type::field());
            let pow_field = self.insert_cast(pow, Type::field());
            let result = self.insert_binary(lhs_field, BinaryOp::Mul, pow_field);
            let result = self.insert_truncate(result, bit_size, max_bit);
            self.insert_cast(result, typ)
        }
    }

    /// Insert ssa instructions which computes lhs >> rhs by doing lhs/2^rhs
    pub(crate) fn insert_shift_right(
        &mut self,
        lhs: ValueId,
        rhs: ValueId,
        bit_size: u32,
    ) -> ValueId {
        let lhs_typ = self.function.dfg.type_of_value(lhs);
        let base = self.field_constant(FieldElement::from(2_u128));
        // we can safely cast to unsigned because overflow_checks prevent bit-shift with a negative value
        let rhs_unsigned = self.insert_cast(rhs, Type::unsigned(bit_size));
        let pow = self.pow(base, rhs_unsigned);
        // We need at least one more bit for the case where rhs == bit_size
        let div_type = Type::unsigned(bit_size + 1);
        let casted_lhs = self.insert_cast(lhs, div_type.clone());
        let casted_pow = self.insert_cast(pow, div_type);
        let div_result = self.insert_binary(casted_lhs, BinaryOp::Div, casted_pow);
        // We have to cast back to the original type
        self.insert_cast(div_result, lhs_typ)
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
            let length = self.field_constant(FieldElement::from(bit_size as i128));
            let result_types =
                vec![Type::field(), Type::Array(Rc::new(vec![Type::bool()]), bit_size as usize)];
            let rhs_bits = self.insert_call(to_bits, vec![rhs, length], result_types);

            let rhs_bits = rhs_bits[1];
            let one = self.field_constant(FieldElement::one());
            let mut r = one;
            for i in 1..bit_size + 1 {
                let r_squared = self.insert_binary(r, BinaryOp::Mul, r);
                let a = self.insert_binary(r_squared, BinaryOp::Mul, lhs);
                let idx = self.field_constant(FieldElement::from((bit_size - i) as i128));
                let b = self.insert_array_get(rhs_bits, idx, Type::bool());
                let not_b = self.insert_not(b);
                let b = self.insert_cast(b, Type::field());
                let not_b = self.insert_cast(not_b, Type::field());
                let r1 = self.insert_binary(a, BinaryOp::Mul, b);
                let r2 = self.insert_binary(r_squared, BinaryOp::Mul, not_b);
                r = self.insert_binary(r1, BinaryOp::Add, r2);
            }
            r
        } else {
            unreachable!("Value must be unsigned in power operation");
        }
    }

    pub(crate) fn field_constant(&mut self, constant: FieldElement) -> ValueId {
        self.function.dfg.make_constant(constant, Type::field())
    }

    /// Insert a numeric constant into the current function
    pub(crate) fn numeric_constant(
        &mut self,
        value: impl Into<FieldElement>,
        typ: Type,
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
    pub(crate) fn insert_cast(&mut self, value: ValueId, typ: Type) -> ValueId {
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
            self.call_stack.clone(),
        );

        if let InsertInstructionResult::Results(instruction_id, _) = result {
            self.new_instructions.push(instruction_id);
        }

        result
    }
}
