use acvm::{
    acir::brillig::{
        BinaryFieldOp, BinaryIntOp, BlackBoxOp, HeapArray, HeapValueType, MemoryAddress,
        Opcode as BrilligOpcode, ValueOrArray,
    },
    acir::AcirField,
    FieldElement,
};

use super::{
    artifact::UnresolvedJumpLocation,
    brillig_variable::{BrilligArray, BrilligVector, SingleAddrVariable},
    BrilligContext, ReservedRegisters, BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
};

/// Low level instructions of the brillig IR, used by the brillig ir codegens and brillig_gen
/// Printed using debug_slow
impl BrilligContext {
    /// Processes a binary instruction according `operation`.
    ///
    /// This method will compute lhs <operation> rhs
    /// and store the result in the `result` register.
    pub(crate) fn binary_instruction(
        &mut self,
        lhs: SingleAddrVariable,
        rhs: SingleAddrVariable,
        result: SingleAddrVariable,
        operation: BrilligBinaryOp,
    ) {
        self.debug_show.binary_instruction(lhs.address, rhs.address, result.address, operation);
        self.binary(lhs, rhs, result, operation);
    }

    /// Processes a not instruction.
    ///
    /// Not is computed using a subtraction operation as there is no native not instruction
    /// in Brillig.
    pub(crate) fn not_instruction(
        &mut self,
        input: SingleAddrVariable,
        result: SingleAddrVariable,
    ) {
        self.debug_show.not_instruction(input.address, input.bit_size, result.address);
        // Compile !x as ((-1) - x)
        let u_max = FieldElement::from(2_i128).pow(&FieldElement::from(input.bit_size as i128))
            - FieldElement::one();
        let max = self.make_constant(u_max, input.bit_size);

        self.binary(max, input, result, BrilligBinaryOp::Sub);
        self.deallocate_single_addr(max);
    }

    /// Utility method to perform a binary instruction with a memory address
    pub(crate) fn memory_op_instruction(
        &mut self,
        lhs: MemoryAddress,
        rhs: MemoryAddress,
        destination: MemoryAddress,
        op: BrilligBinaryOp,
    ) {
        self.binary_instruction(
            SingleAddrVariable::new_usize(lhs),
            SingleAddrVariable::new_usize(rhs),
            SingleAddrVariable::new(
                destination,
                BrilligContext::binary_result_bit_size(op, BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
            ),
            op,
        );
    }

    fn binary(
        &mut self,
        lhs: SingleAddrVariable,
        rhs: SingleAddrVariable,
        result: SingleAddrVariable,
        operation: BrilligBinaryOp,
    ) {
        let is_field_op = lhs.bit_size == FieldElement::max_num_bits();
        let expected_result_bit_size =
            BrilligContext::binary_result_bit_size(operation, lhs.bit_size);
        assert!(
            result.bit_size == expected_result_bit_size,
            "Expected result bit size to be {}, got {} for operation {:?}",
            expected_result_bit_size,
            result.bit_size,
            operation
        );

        if let BrilligBinaryOp::Modulo = operation {
            self.modulo(result, lhs, rhs);
        } else if is_field_op {
            self.push_opcode(BrilligOpcode::BinaryFieldOp {
                op: operation.into(),
                destination: result.address,
                lhs: lhs.address,
                rhs: rhs.address,
            });
        } else {
            self.push_opcode(BrilligOpcode::BinaryIntOp {
                op: operation.into(),
                destination: result.address,
                bit_size: lhs.bit_size,
                lhs: lhs.address,
                rhs: rhs.address,
            });
        }
    }

    /// Computes left % right by emitting the necessary Brillig opcodes.
    ///
    /// This is done by using the following formula:
    ///
    /// a % b = a - (b * (a / b))
    ///
    /// Brillig does not have an explicit modulo operation,
    /// so we must emit multiple opcodes and process it differently
    /// to other binary instructions.
    fn modulo(
        &mut self,
        result: SingleAddrVariable,
        left: SingleAddrVariable,
        right: SingleAddrVariable,
    ) {
        assert!(
            left.bit_size == right.bit_size,
            "Not equal bitsize: lhs {}, rhs {}",
            left.bit_size,
            right.bit_size
        );
        let bit_size = left.bit_size;

        let scratch_var_i = SingleAddrVariable::new(self.allocate_register(), bit_size);
        let scratch_var_j = SingleAddrVariable::new(self.allocate_register(), bit_size);

        // i = left / right
        self.binary(left, right, scratch_var_i, BrilligBinaryOp::UnsignedDiv);

        // j = i * right
        self.binary(scratch_var_i, right, scratch_var_j, BrilligBinaryOp::Mul);

        // result_register = left - j
        self.binary(left, scratch_var_j, result, BrilligBinaryOp::Sub);
        // Free scratch registers
        self.deallocate_single_addr(scratch_var_i);
        self.deallocate_single_addr(scratch_var_j);
    }

    fn binary_result_bit_size(operation: BrilligBinaryOp, arguments_bit_size: u32) -> u32 {
        match operation {
            BrilligBinaryOp::Equals
            | BrilligBinaryOp::LessThan
            | BrilligBinaryOp::LessThanEquals => 1,
            _ => arguments_bit_size,
        }
    }

    /// Processes a foreign call instruction.
    ///
    /// Note: the function being called is external and will
    /// not be linked during brillig generation.
    pub(crate) fn foreign_call_instruction(
        &mut self,
        func_name: String,
        inputs: &[ValueOrArray],
        input_value_types: &[HeapValueType],
        outputs: &[ValueOrArray],
        output_value_types: &[HeapValueType],
    ) {
        self.debug_show.foreign_call_instruction(func_name.clone(), inputs, outputs);

        assert!(inputs.len() == input_value_types.len());
        assert!(outputs.len() == output_value_types.len());

        self.push_opcode(BrilligOpcode::ForeignCall {
            function: func_name,
            destinations: outputs.to_vec(),
            destination_value_types: output_value_types.to_vec(),
            inputs: inputs.to_vec(),
            input_value_types: input_value_types.to_vec(),
        });
    }

    /// Adds a unresolved external `Call` instruction to the bytecode.
    /// This calls into another function compiled into this brillig artifact.
    pub(crate) fn add_external_call_instruction<T: ToString>(&mut self, func_label: T) {
        self.debug_show.add_external_call_instruction(func_label.to_string());
        self.obj.add_unresolved_external_call(
            BrilligOpcode::Call { location: 0 },
            func_label.to_string(),
        );
    }

    /// Adds a unresolved `Jump` instruction to the bytecode.
    pub(crate) fn jump_instruction<T: ToString>(&mut self, target_label: T) {
        self.debug_show.jump_instruction(target_label.to_string());
        self.add_unresolved_jump(BrilligOpcode::Jump { location: 0 }, target_label.to_string());
    }

    /// Adds a unresolved `JumpIf` instruction to the bytecode.
    pub(crate) fn jump_if_instruction<T: ToString>(
        &mut self,
        condition: MemoryAddress,
        target_label: T,
    ) {
        self.debug_show.jump_if_instruction(condition, target_label.to_string());
        self.add_unresolved_jump(
            BrilligOpcode::JumpIf { condition, location: 0 },
            target_label.to_string(),
        );
    }

    /// Adds a unresolved `Jump` to the bytecode.
    fn add_unresolved_jump(
        &mut self,
        jmp_instruction: BrilligOpcode<FieldElement>,
        destination: UnresolvedJumpLocation,
    ) {
        self.obj.add_unresolved_jump(jmp_instruction, destination);
    }

    /// Adds a label to the next opcode
    pub(crate) fn enter_context<T: ToString>(&mut self, label: T) {
        self.debug_show.enter_context(label.to_string());
        self.context_label = label.to_string();
        self.section_label = 0;
        // Add a context label to the next opcode
        self.obj.add_label_at_position(label.to_string(), self.obj.index_of_next_opcode());
        // Add a section label to the next opcode
        self.obj
            .add_label_at_position(self.current_section_label(), self.obj.index_of_next_opcode());
    }

    /// Enter the given section
    pub(super) fn enter_section(&mut self, section: usize) {
        self.section_label = section;
        self.obj
            .add_label_at_position(self.current_section_label(), self.obj.index_of_next_opcode());
    }

    /// Create, reserve, and return a new section label.
    pub(super) fn reserve_next_section_label(&mut self) -> (usize, String) {
        let section = self.next_section;
        self.next_section += 1;
        (section, self.compute_section_label(section))
    }

    /// Internal function used to compute the section labels
    fn compute_section_label(&self, section: usize) -> String {
        format!("{}-{}", self.context_label, section)
    }

    /// Returns the current section label
    fn current_section_label(&self) -> String {
        self.compute_section_label(self.section_label)
    }

    /// Emits a stop instruction
    pub(crate) fn stop_instruction(&mut self) {
        self.debug_show.stop_instruction();
        self.push_opcode(BrilligOpcode::Stop { return_data_offset: 0, return_data_size: 0 });
    }

    /// Emits a external stop instruction (returns data)
    pub(crate) fn external_stop_instruction(
        &mut self,
        return_data_offset: usize,
        return_data_size: usize,
    ) {
        self.debug_show.external_stop_instruction(return_data_offset, return_data_size);
        self.push_opcode(BrilligOpcode::Stop { return_data_offset, return_data_size });
    }

    /// Issues a blackbox operation.
    pub(crate) fn black_box_op_instruction(&mut self, op: BlackBoxOp) {
        self.debug_show.black_box_op_instruction(&op);
        self.push_opcode(BrilligOpcode::BlackBox(op));
    }

    pub(crate) fn load_free_memory_pointer_instruction(&mut self, pointer_register: MemoryAddress) {
        self.debug_show.mov_instruction(pointer_register, ReservedRegisters::free_memory_pointer());
        self.push_opcode(BrilligOpcode::Mov {
            destination: pointer_register,
            source: ReservedRegisters::free_memory_pointer(),
        });
    }

    pub(crate) fn increase_free_memory_pointer_instruction(
        &mut self,
        size_register: MemoryAddress,
    ) {
        self.memory_op_instruction(
            ReservedRegisters::free_memory_pointer(),
            size_register,
            ReservedRegisters::free_memory_pointer(),
            BrilligBinaryOp::Add,
        );
    }

    /// Emits a store instruction
    pub(crate) fn store_instruction(
        &mut self,
        destination_pointer: MemoryAddress,
        source: MemoryAddress,
    ) {
        self.debug_show.store_instruction(destination_pointer, source);
        self.push_opcode(BrilligOpcode::Store { destination_pointer, source });
    }

    /// Utility method to transform a HeapArray to a HeapVector by making a runtime constant with the size.
    pub(crate) fn array_to_vector_instruction(&mut self, array: &BrilligArray) -> BrilligVector {
        let size_register = self.make_usize_constant_instruction(array.size.into());
        BrilligVector { size: size_register.address, pointer: array.pointer, rc: array.rc }
    }

    /// Emits a load instruction
    pub(crate) fn load_instruction(
        &mut self,
        destination: MemoryAddress,
        source_pointer: MemoryAddress,
    ) {
        self.debug_show.load_instruction(destination, source_pointer);
        self.push_opcode(BrilligOpcode::Load { destination, source_pointer });
    }

    /// Emits a `mov` instruction.
    ///
    /// Copies the value at `source` into `destination`
    pub(crate) fn mov_instruction(&mut self, destination: MemoryAddress, source: MemoryAddress) {
        self.debug_show.mov_instruction(destination, source);
        self.push_opcode(BrilligOpcode::Mov { destination, source });
    }

    /// Emits a conditional `mov` instruction.
    ///
    /// Copies the value at `source` into `destination`
    pub(crate) fn conditional_mov_instruction(
        &mut self,
        destination: MemoryAddress,
        condition: MemoryAddress,
        source_a: MemoryAddress,
        source_b: MemoryAddress,
    ) {
        self.debug_show.conditional_mov_instruction(destination, condition, source_a, source_b);
        self.push_opcode(BrilligOpcode::ConditionalMov {
            destination,
            source_a,
            source_b,
            condition,
        });
    }

    /// Cast truncates the value to the given bit size and converts the type of the value in memory to that bit size.
    pub(crate) fn cast_instruction(
        &mut self,
        destination: SingleAddrVariable,
        source: SingleAddrVariable,
    ) {
        self.debug_show.cast_instruction(destination.address, source.address, destination.bit_size);
        self.cast(destination, source);
    }

    pub(crate) fn cast(&mut self, destination: SingleAddrVariable, source: SingleAddrVariable) {
        self.push_opcode(BrilligOpcode::Cast {
            destination: destination.address,
            source: source.address,
            bit_size: destination.bit_size,
        });
    }

    /// Stores the value of `constant` in the `result` register
    pub(crate) fn const_instruction(&mut self, result: SingleAddrVariable, constant: FieldElement) {
        self.debug_show.const_instruction(result.address, constant);
        self.constant(result, constant);
    }

    fn constant(&mut self, result: SingleAddrVariable, constant: FieldElement) {
        assert!(
            result.bit_size >= constant.num_bits(),
            "Constant {} does not fit in bit size {}",
            constant,
            result.bit_size
        );
        if result.bit_size > 128 && constant.num_bits() > 128 {
            let high = FieldElement::from_be_bytes_reduce(
                constant.to_be_bytes().get(0..16).expect("FieldElement::to_be_bytes() too short!"),
            );
            let low = FieldElement::from(constant.to_u128());
            let high_register = SingleAddrVariable::new(self.allocate_register(), 254);
            let low_register = SingleAddrVariable::new(self.allocate_register(), 254);
            let intermediate_register = SingleAddrVariable::new(self.allocate_register(), 254);
            self.constant(high_register, high);
            self.constant(low_register, low);
            // I want to multiply high by 2^128, but I can't get that big constant in.
            // So I'll multiply by 2^64 twice.
            self.constant(intermediate_register, FieldElement::from(1_u128 << 64));
            self.binary(high_register, intermediate_register, high_register, BrilligBinaryOp::Mul);
            self.binary(high_register, intermediate_register, high_register, BrilligBinaryOp::Mul);
            // Now we can add.
            self.binary(high_register, low_register, intermediate_register, BrilligBinaryOp::Add);
            self.cast(result, intermediate_register);
            self.deallocate_single_addr(high_register);
            self.deallocate_single_addr(low_register);
            self.deallocate_single_addr(intermediate_register);
        } else {
            self.push_opcode(BrilligOpcode::Const {
                destination: result.address,
                value: constant,
                bit_size: result.bit_size,
            });
        }
    }

    pub(crate) fn usize_const_instruction(
        &mut self,
        result: MemoryAddress,
        constant: FieldElement,
    ) {
        self.const_instruction(SingleAddrVariable::new_usize(result), constant);
    }

    /// Returns a register which holds the value of a constant
    pub(crate) fn make_constant_instruction(
        &mut self,
        constant: FieldElement,
        bit_size: u32,
    ) -> SingleAddrVariable {
        let var = SingleAddrVariable::new(self.allocate_register(), bit_size);
        self.const_instruction(var, constant);
        var
    }

    fn make_constant(&mut self, constant: FieldElement, bit_size: u32) -> SingleAddrVariable {
        let var = SingleAddrVariable::new(self.allocate_register(), bit_size);
        self.constant(var, constant);
        var
    }

    /// Returns a register which holds the value of an usize constant
    pub(crate) fn make_usize_constant_instruction(
        &mut self,
        constant: FieldElement,
    ) -> SingleAddrVariable {
        let register = self.allocate_register();
        self.usize_const_instruction(register, constant);
        SingleAddrVariable::new_usize(register)
    }

    pub(super) fn calldata_copy_instruction(
        &mut self,
        destination: MemoryAddress,
        calldata_size: usize,
        offset: usize,
    ) {
        self.debug_show.calldata_copy_instruction(destination, calldata_size, offset);

        self.push_opcode(BrilligOpcode::CalldataCopy {
            destination_address: destination,
            size: calldata_size,
            offset,
        });
    }

    pub(super) fn trap_instruction(&mut self, revert_data: HeapArray) {
        self.debug_show.trap_instruction(revert_data);

        self.push_opcode(BrilligOpcode::Trap { revert_data });
    }
}

/// Type to encapsulate the binary operation types in Brillig
#[derive(Clone, Copy, Debug)]
pub(crate) enum BrilligBinaryOp {
    Add,
    Sub,
    Mul,
    FieldDiv,
    UnsignedDiv,
    Equals,
    LessThan,
    LessThanEquals,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    // Modulo operation requires more than one brillig opcode
    Modulo,
}

impl From<BrilligBinaryOp> for BinaryFieldOp {
    fn from(operation: BrilligBinaryOp) -> BinaryFieldOp {
        match operation {
            BrilligBinaryOp::Add => BinaryFieldOp::Add,
            BrilligBinaryOp::Sub => BinaryFieldOp::Sub,
            BrilligBinaryOp::Mul => BinaryFieldOp::Mul,
            BrilligBinaryOp::FieldDiv => BinaryFieldOp::Div,
            BrilligBinaryOp::UnsignedDiv => BinaryFieldOp::IntegerDiv,
            BrilligBinaryOp::Equals => BinaryFieldOp::Equals,
            BrilligBinaryOp::LessThan => BinaryFieldOp::LessThan,
            BrilligBinaryOp::LessThanEquals => BinaryFieldOp::LessThanEquals,
            _ => panic!("Unsupported operation: {:?} on a field", operation),
        }
    }
}

impl From<BrilligBinaryOp> for BinaryIntOp {
    fn from(operation: BrilligBinaryOp) -> BinaryIntOp {
        match operation {
            BrilligBinaryOp::Add => BinaryIntOp::Add,
            BrilligBinaryOp::Sub => BinaryIntOp::Sub,
            BrilligBinaryOp::Mul => BinaryIntOp::Mul,
            BrilligBinaryOp::UnsignedDiv => BinaryIntOp::Div,
            BrilligBinaryOp::Equals => BinaryIntOp::Equals,
            BrilligBinaryOp::LessThan => BinaryIntOp::LessThan,
            BrilligBinaryOp::LessThanEquals => BinaryIntOp::LessThanEquals,
            BrilligBinaryOp::And => BinaryIntOp::And,
            BrilligBinaryOp::Or => BinaryIntOp::Or,
            BrilligBinaryOp::Xor => BinaryIntOp::Xor,
            BrilligBinaryOp::Shl => BinaryIntOp::Shl,
            BrilligBinaryOp::Shr => BinaryIntOp::Shr,
            _ => panic!("Unsupported operation: {:?} on an integer", operation),
        }
    }
}
