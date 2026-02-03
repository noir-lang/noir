use acvm::{
    FieldElement,
    acir::{
        AcirField,
        brillig::{
            BinaryFieldOp, BinaryIntOp, BitSize, BlackBoxOp, HeapValueType, HeapVector,
            MemoryAddress, Opcode as BrilligOpcode, ValueOrArray,
        },
    },
};

use crate::{brillig::brillig_ir::registers::Allocated, ssa::ir::function::FunctionId};

use super::{
    BRILLIG_MEMORY_ADDRESSING_BIT_SIZE, BrilligContext, ReservedRegisters,
    artifact::{Label, UnresolvedJumpLocation},
    brillig_variable::SingleAddrVariable,
    debug_show::DebugToString,
    procedures::ProcedureId,
    registers::RegisterAllocator,
};

/// Low level instructions of the brillig IR, used by the brillig ir codegens and brillig_gen
/// Printed using debug_slow
impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Processes a binary instruction according `operation`.
    ///
    /// This method will compute lhs `<operation>` rhs
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
        assert_eq!(input.bit_size, result.bit_size, "Not operands should have the same bit size");
        self.push_opcode(BrilligOpcode::Not {
            destination: result.address,
            source: input.address,
            bit_size: input.bit_size.try_into().unwrap(),
        });
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
                Self::binary_result_bit_size(op, BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
            ),
            op,
        );
    }

    /// Increase the value at a pointer in place by 1.
    pub(crate) fn memory_op_inc_by_usize_one(&mut self, address: MemoryAddress) {
        self.memory_op_instruction(
            address,
            ReservedRegisters::usize_one(),
            address,
            BrilligBinaryOp::Add,
        );
    }

    /// Insert a conditional move instruction
    pub(crate) fn conditional_move_instruction(
        &mut self,
        condition: SingleAddrVariable,
        then_address: SingleAddrVariable,
        else_address: SingleAddrVariable,
        destination: SingleAddrVariable,
    ) {
        self.debug_show.conditional_mov_instruction(
            destination.address,
            then_address.address,
            else_address.address,
            condition.address,
        );
        self.push_opcode(BrilligOpcode::ConditionalMov {
            destination: destination.address,
            source_a: then_address.address,
            source_b: else_address.address,
            condition: condition.address,
        });
    }

    fn binary(
        &mut self,
        lhs: SingleAddrVariable,
        rhs: SingleAddrVariable,
        result: SingleAddrVariable,
        operation: BrilligBinaryOp,
    ) {
        let is_field_op = lhs.bit_size == FieldElement::max_num_bits();
        let expected_result_bit_size = Self::binary_result_bit_size(operation, lhs.bit_size);
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
                bit_size: lhs.bit_size.try_into().unwrap(),
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
        assert!(bit_size != BitSize::Field.to_u32::<F>(), "Attempt to modulo fields");

        let scratch_var_i = self.allocate_single_addr(bit_size);
        let scratch_var_j = self.allocate_single_addr(bit_size);

        // i = left / right
        self.binary(left, right, *scratch_var_i, BrilligBinaryOp::UnsignedDiv);

        // j = i * right
        self.binary(*scratch_var_i, right, *scratch_var_j, BrilligBinaryOp::Mul);

        // result_register = left - j
        self.binary(left, *scratch_var_j, result, BrilligBinaryOp::Sub);
    }

    fn binary_result_bit_size(operation: BrilligBinaryOp, arguments_bit_size: u32) -> u32 {
        match operation {
            BrilligBinaryOp::Equals
            | BrilligBinaryOp::LessThan
            | BrilligBinaryOp::LessThanEquals => 1,
            _ => arguments_bit_size,
        }
    }

    /// Push an opcode to processes a foreign call instruction.
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
    pub(crate) fn add_external_call_instruction(&mut self, func_id: FunctionId) {
        let func_label = Label::function(func_id);
        self.debug_show.add_external_call_instruction(func_label.to_string());
        self.obj.add_unresolved_external_call(BrilligOpcode::Call { location: 0 }, func_label);
    }

    pub(super) fn add_procedure_call_instruction(&mut self, procedure_id: ProcedureId) {
        let proc_label = Label::procedure(procedure_id);
        self.debug_show.add_external_call_instruction(proc_label.to_string());
        self.obj.add_unresolved_external_call(BrilligOpcode::Call { location: 0 }, proc_label);
    }

    pub(super) fn add_globals_init_instruction(&mut self, func_id: FunctionId) {
        let globals_init_label = Label::globals_init(func_id);
        self.debug_show.add_external_call_instruction(globals_init_label.to_string());
        self.obj
            .add_unresolved_external_call(BrilligOpcode::Call { location: 0 }, globals_init_label);
    }

    /// Adds a unresolved `Jump` instruction to the bytecode.
    pub(crate) fn jump_instruction(&mut self, target_label: Label) {
        self.debug_show.jump_instruction(target_label.to_string());
        self.add_unresolved_jump(BrilligOpcode::Jump { location: 0 }, target_label);
    }

    /// Adds a unresolved `JumpIf` instruction to the bytecode.
    pub(crate) fn jump_if_instruction(&mut self, condition: MemoryAddress, target_label: Label) {
        self.debug_show.jump_if_instruction(condition, target_label.to_string());
        self.add_unresolved_jump(BrilligOpcode::JumpIf { condition, location: 0 }, target_label);
    }

    /// Adds a unresolved `Jump` to the bytecode.
    fn add_unresolved_jump(
        &mut self,
        jmp_instruction: BrilligOpcode<F>,
        destination: UnresolvedJumpLocation,
    ) {
        self.obj.add_unresolved_jump(jmp_instruction, destination);
    }

    /// Adds a context and a section label to the next opcode.
    ///
    /// Entering a context resets the current section to 0 and the next section to 1.
    pub(crate) fn enter_context(&mut self, label: Label) {
        assert!(label.section.is_none(), "new context should have no section");
        self.debug_show.enter_context(label.to_string());
        self.context_label = label.clone();
        // Add a context label to the next opcode
        self.obj.add_label_at_position(label, self.obj.index_of_next_opcode());
        self.enter_section(0);
        self.next_section = 1;
    }

    /// Enter the given section within a basic block.
    pub(super) fn enter_section(&mut self, section: usize) {
        self.current_section = section;
        self.obj
            .add_label_at_position(self.current_section_label(), self.obj.index_of_next_opcode());
    }

    /// Create, reserve, and return a new section label.
    pub(super) fn reserve_next_section_label(&mut self) -> (usize, Label) {
        let section = self.next_section;
        self.next_section += 1;
        (section, self.compute_section_label(section))
    }

    /// Internal function used to compute the section labels
    fn compute_section_label(&self, section: usize) -> Label {
        self.context_label.with_section(section)
    }

    /// Returns the current section label
    fn current_section_label(&self) -> Label {
        self.compute_section_label(self.current_section)
    }

    /// Emits a return instruction
    pub(crate) fn return_instruction(&mut self) {
        self.debug_show.return_instruction();
        self.push_opcode(BrilligOpcode::Return);
    }

    /// Emits a stop instruction with return data
    pub(crate) fn stop_instruction(&mut self, return_data: HeapVector) {
        self.debug_show.stop_instruction(return_data);
        self.push_opcode(BrilligOpcode::Stop { return_data });
    }

    /// Issues a blackbox operation.
    pub(crate) fn black_box_op_instruction(&mut self, op: BlackBoxOp) {
        self.debug_show.black_box_op_instruction(&op);
        self.push_opcode(BrilligOpcode::BlackBox(op));
    }

    /// Push an opcode to load the current value of the _free memory pointer_ into the `pointer_register`.
    pub(crate) fn load_free_memory_pointer_instruction(&mut self, pointer_register: MemoryAddress) {
        self.debug_show.mov_instruction(pointer_register, ReservedRegisters::free_memory_pointer());
        self.push_opcode(BrilligOpcode::Mov {
            destination: pointer_register,
            source: ReservedRegisters::free_memory_pointer(),
        });
    }

    /// Push an opcode to increase the _free memory pointer_ by the value stored in `size_register`.
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

    /// Push an opcode to store the value at `source` address into the address under `destination_pointer`.
    pub(crate) fn store_instruction(
        &mut self,
        destination_pointer: MemoryAddress,
        source: MemoryAddress,
    ) {
        self.debug_show.store_instruction(destination_pointer, source);
        self.push_opcode(BrilligOpcode::Store { destination_pointer, source });
    }

    /// Push an opcode to load a value from the address under `source_pointer` into the `destination` address.
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
            bit_size: BitSize::try_from_u32::<F>(destination.bit_size).unwrap(),
        });
    }

    /// Stores the value of `constant` in the `result` register.
    pub(crate) fn const_instruction(&mut self, result: SingleAddrVariable, constant: F) {
        self.debug_show.const_instruction(result.address, constant);
        self.constant(result.address, result.bit_size, constant, false);
    }

    /// Stores the value of `constant` in the register pointed at by `result_pointer`.
    pub(crate) fn indirect_const_instruction(
        &mut self,
        result_pointer: MemoryAddress,
        bit_size: u32,
        constant: F,
    ) {
        self.debug_show.indirect_const_instruction(result_pointer, constant);
        self.constant(result_pointer, bit_size, constant, true);
    }

    /// Pushes a [IndirectConst][BrilligOpcode::IndirectConst] or [Const][BrilligOpcode::Const] opcode.
    fn constant(&mut self, result: MemoryAddress, bit_size: u32, constant: F, indirect: bool) {
        assert!(
            bit_size >= constant.num_bits(),
            "Constant {constant} does not fit in bit size {bit_size}"
        );
        if indirect {
            self.push_opcode(BrilligOpcode::IndirectConst {
                destination_pointer: result,
                value: constant,
                bit_size: BitSize::try_from_u32::<F>(bit_size).unwrap(),
            });
        } else {
            self.push_opcode(BrilligOpcode::Const {
                destination: result,
                value: constant,
                bit_size: BitSize::try_from_u32::<F>(bit_size).unwrap(),
            });
        }
    }

    /// Stores the value of `constant` at the `result` address.
    pub(crate) fn usize_const_instruction(&mut self, result: MemoryAddress, constant: F) {
        self.const_instruction(SingleAddrVariable::new_usize(result), constant);
    }

    /// Returns a register which holds the value of a constant.
    pub(crate) fn make_constant_instruction(
        &mut self,
        constant: F,
        bit_size: u32,
    ) -> Allocated<SingleAddrVariable, Registers> {
        let var = self.allocate_single_addr(bit_size);
        self.const_instruction(*var, constant);
        var
    }

    /// Returns a register which holds the value of an `usize` constant.
    pub(crate) fn make_usize_constant_instruction(
        &mut self,
        constant: F,
    ) -> Allocated<SingleAddrVariable, Registers> {
        let register = self.allocate_register();
        self.usize_const_instruction(*register, constant);
        register.map(SingleAddrVariable::new_usize)
    }

    /// Pushes a [CalldataCopy][BrilligOpcode::CalldataCopy] opcode to copy the calldata
    /// at a specific offset with and size to the `destination` address.
    pub(super) fn calldata_copy_instruction(
        &mut self,
        destination: MemoryAddress,
        calldata_size: usize,
        calldata_offset: usize,
    ) {
        self.debug_show.calldata_copy_instruction(destination, calldata_size, calldata_offset);

        let size_var = self.make_usize_constant_instruction(calldata_size.into());
        let offset_var = self.make_usize_constant_instruction(calldata_offset.into());
        self.push_opcode(BrilligOpcode::CalldataCopy {
            destination_address: destination,
            size_address: size_var.address,
            offset_address: offset_var.address,
        });
    }

    /// Pushes a [Trap][BrilligOpcode::Trap] opcode.
    pub(super) fn trap_instruction(&mut self, revert_data: HeapVector) {
        self.debug_show.trap_instruction(revert_data);

        self.push_opcode(BrilligOpcode::Trap { revert_data });
    }
}

/// Type to encapsulate the binary operation types in Brillig
#[derive(Clone, Copy, Debug)]
pub enum BrilligBinaryOp {
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
            _ => panic!("Unsupported operation: {operation:?} on a field"),
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
            _ => panic!("Unsupported operation: {operation:?} on an integer"),
        }
    }
}
