//! This module is an abstraction layer over `Brillig`
//! To allow for separation of concerns, it knows nothing
//! about SSA types, and can therefore be tested independently.
//! `brillig_gen` is therefore the module which combines both
//! ssa types and types in this module.
//! A similar paradigm can be seen with the `acir_ir` module.
pub(crate) mod artifact;
pub(crate) mod registers;

use self::{
    artifact::{BrilligArtifact, UnresolvedJumpLocation},
    registers::BrilligRegistersContext,
};
use acvm::{
    acir::brillig_vm::{
        BinaryFieldOp, BinaryIntOp, Opcode as BrilligOpcode, RegisterIndex, RegisterValueOrArray,
        Value,
    },
    FieldElement,
};

/// Integer arithmetic in Brillig is limited to 127 bit
/// integers.
///
/// We could lift this in the future and have Brillig
/// do big integer arithmetic when it exceeds the field size
/// or we could have users re-implement big integer arithmetic
/// in Brillig.
/// Since constrained functions do not have this property, it
/// would mean that unconstrained functions will differ from
/// constrained functions in terms of syntax compatibility.
pub(crate) const BRILLIG_INTEGER_ARITHMETIC_BIT_SIZE: u32 = 127;
pub(crate) const BRILLIG_MEMORY_ADDRESSING_BIT_SIZE: u32 = 64;

// Registers reserved in runtime for special purposes.
pub(crate) enum ReservedRegisters {
    /// This register stores the stack pointer. Allocations must be done after this pointer.
    StackPointer = 0,
    /// Number of reserved registers
    Len = 1,
}

impl ReservedRegisters {
    /// Returns the length of the reserved registers
    pub(crate) fn len() -> usize {
        ReservedRegisters::Len as usize
    }

    /// Returns the stack pointer register. This will get used to allocate memory in runtime.
    pub(crate) fn stack_pointer() -> RegisterIndex {
        RegisterIndex::from(ReservedRegisters::StackPointer as usize)
    }
}

/// Brillig context object that is used while constructing the
/// Brillig bytecode.
pub(crate) struct BrilligContext {
    obj: BrilligArtifact,
    /// Tracks register allocations
    registers: BrilligRegistersContext,
    /// Context label, must be unique with respect to the function
    /// being linked.
    context_label: String,
    /// Section label, used to separate sections of code
    section_label: usize,
}

impl BrilligContext {
    /// Initial context state
    pub(crate) fn new() -> BrilligContext {
        BrilligContext {
            obj: BrilligArtifact::default(),
            registers: BrilligRegistersContext::new(),
            context_label: String::default(),
            section_label: 0,
        }
    }

    /// Adds the instructions needed to handle entry point parameters
    /// And sets the starting value of the reserved registers
    pub(crate) fn entry_point_instruction(&mut self, num_arguments: usize) {
        // Translate the inputs by the reserved registers offset
        for i in (0..num_arguments).rev() {
            self.mov_instruction(self.user_register_index(i), RegisterIndex::from(i));
        }
        // Set the initial value of the stack pointer register
        self.const_instruction(ReservedRegisters::stack_pointer(), Value::from(0_usize));
    }

    /// Adds a brillig instruction to the brillig byte code
    pub(crate) fn push_opcode(&mut self, opcode: BrilligOpcode) {
        self.obj.byte_code.push(opcode);
    }

    /// Returns the artifact
    pub(crate) fn artifact(self) -> BrilligArtifact {
        self.obj
    }

    /// Allocates an array of size `size` and stores the pointer to the array
    /// in `pointer_register`
    pub(crate) fn allocate_fixed_length_array(
        &mut self,
        pointer_register: RegisterIndex,
        size: usize,
    ) {
        let size_register = self.make_constant(size.into());
        self.allocate_array_instruction(pointer_register, size_register);
    }

    /// Allocates an array of size contained in size_register and stores the
    /// pointer to the array in `pointer_register`
    pub(crate) fn allocate_array_instruction(
        &mut self,
        pointer_register: RegisterIndex,
        size_register: RegisterIndex,
    ) {
        self.push_opcode(BrilligOpcode::Mov {
            destination: pointer_register,
            source: ReservedRegisters::stack_pointer(),
        });
        self.push_opcode(BrilligOpcode::BinaryIntOp {
            destination: ReservedRegisters::stack_pointer(),
            op: BinaryIntOp::Add,
            bit_size: BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
            lhs: ReservedRegisters::stack_pointer(),
            rhs: size_register,
        });
    }

    /// Gets the value in the array at index `index` and stores it in `result`
    pub(crate) fn array_get(
        &mut self,
        array_ptr: RegisterIndex,
        index: RegisterIndex,
        result: RegisterIndex,
    ) {
        // Computes array_ptr + index, ie array[index]
        let index_of_element_in_memory = self.allocate_register();
        self.binary_instruction(
            array_ptr,
            index,
            index_of_element_in_memory,
            BrilligBinaryOp::Field { op: BinaryFieldOp::Add },
        );

        self.load_instruction(result, index_of_element_in_memory);
    }

    /// Sets the item in the array at index `index` to `value`
    pub(crate) fn array_set(
        &mut self,
        array_ptr: RegisterIndex,
        index: RegisterIndex,
        value: RegisterIndex,
    ) {
        // Computes array_ptr + index, ie array[index]
        let index_of_element_in_memory = self.allocate_register();
        self.binary_instruction(
            array_ptr,
            index,
            index_of_element_in_memory,
            BrilligBinaryOp::Field { op: BinaryFieldOp::Add },
        );

        self.store_instruction(index_of_element_in_memory, value);
    }

    /// Copies the values of an array pointed by source with length stored in `num_elements_register`
    /// Into the array pointed by destination
    pub(crate) fn copy_array_instruction(
        &mut self,
        source: RegisterIndex,
        destination: RegisterIndex,
        num_elements_register: RegisterIndex,
    ) {
        let index_register = self.make_constant(0_u128.into());

        let loop_label = self.next_section_label();
        self.enter_next_section();

        // Loop body

        // Check if index < num_elements
        let index_less_than_array_len = self.allocate_register();
        self.binary_instruction(
            index_register,
            num_elements_register,
            index_less_than_array_len,
            BrilligBinaryOp::Integer {
                op: BinaryIntOp::LessThan,
                bit_size: BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
            },
        );

        let exit_loop_label = self.next_section_label();

        self.not_instruction(index_less_than_array_len, index_less_than_array_len);
        self.jump_if_instruction(index_less_than_array_len, exit_loop_label);

        // Copy the element from source to destination
        let value_register = self.allocate_register();
        self.array_get(source, index_register, value_register);
        self.array_set(destination, index_register, value_register);

        // Increment the index register
        let one_register = self.make_constant(1u128.into());
        self.binary_instruction(
            index_register,
            one_register,
            index_register,
            BrilligBinaryOp::Integer {
                op: BinaryIntOp::Add,
                bit_size: BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
            },
        );

        self.jump_instruction(loop_label);

        // Exit the loop
        self.enter_next_section();
        // Deallocate our temporary registers
        self.deallocate_register(one_register);
        self.deallocate_register(index_register);
    }

    /// Adds a label to the next opcode
    pub(crate) fn enter_context<T: ToString>(&mut self, label: T) {
        self.context_label = label.to_string();
        self.section_label = 0;
        // Add a context label to the next opcode
        self.obj.add_label_at_position(label.to_string(), self.obj.index_of_next_opcode());
        // Add a section label to the next opcode
        self.obj
            .add_label_at_position(self.current_section_label(), self.obj.index_of_next_opcode());
    }

    /// Increments the section label and adds a section label to the next opcode
    fn enter_next_section(&mut self) {
        self.section_label += 1;
        self.obj
            .add_label_at_position(self.current_section_label(), self.obj.index_of_next_opcode());
    }

    /// Internal function used to compute the section labels
    fn compute_section_label(&self, section: usize) -> String {
        format!("{}-{}", self.context_label, section)
    }

    /// Returns the next section label
    fn next_section_label(&self) -> String {
        self.compute_section_label(self.section_label + 1)
    }

    /// Returns the current section label
    fn current_section_label(&self) -> String {
        self.compute_section_label(self.section_label)
    }

    /// Adds a unresolved `Jump` instruction to the bytecode.
    pub(crate) fn jump_instruction<T: ToString>(&mut self, target_label: T) {
        self.add_unresolved_jump(BrilligOpcode::Jump { location: 0 }, target_label.to_string());
    }

    /// Adds a unresolved `JumpIf` instruction to the bytecode.
    pub(crate) fn jump_if_instruction<T: ToString>(
        &mut self,
        condition: RegisterIndex,
        target_label: T,
    ) {
        self.add_unresolved_jump(
            BrilligOpcode::JumpIf { condition, location: 0 },
            target_label.to_string(),
        );
    }

    /// Adds a unresolved `Jump` instruction to the bytecode.
    fn add_unresolved_jump(
        &mut self,
        jmp_instruction: BrilligOpcode,
        destination: UnresolvedJumpLocation,
    ) {
        self.obj.add_unresolved_jump(jmp_instruction, destination);
    }

    /// Returns a user defined (non-reserved) register index.
    fn user_register_index(&self, index: usize) -> RegisterIndex {
        RegisterIndex::from(index + ReservedRegisters::len())
    }

    /// Allocates an unused register.
    pub(crate) fn allocate_register(&mut self) -> RegisterIndex {
        self.registers.allocate_register()
    }

    /// Push a register to the deallocation list, ready for reuse.
    /// TODO(AD): Currently only used for constants. Later, do lifecycle analysis.
    pub(crate) fn deallocate_register(&mut self, register_index: RegisterIndex) {
        self.registers.deallocate_register(register_index);
    }
}

impl BrilligContext {
    /// Emits brillig bytecode to jump to a trap condition if `condition`
    /// is false.
    pub(crate) fn constrain_instruction(&mut self, condition: RegisterIndex) {
        self.add_unresolved_jump(
            BrilligOpcode::JumpIf { condition, location: 0 },
            self.next_section_label(),
        );
        self.push_opcode(BrilligOpcode::Trap);
        self.enter_next_section();
    }

    /// Processes a return instruction.
    ///
    /// For Brillig, the return is implicit, since there is no explicit return instruction.
    /// The caller will take `N` values from the Register starting at register index 0.
    /// `N` indicates the number of return values expected.
    ///
    /// Brillig does not have an explicit return instruction, so this
    /// method will move all register values to the first `N` values in
    /// the VM.
    pub(crate) fn return_instruction(&mut self, return_registers: &[RegisterIndex]) {
        for (destination_index, return_register) in return_registers.iter().enumerate() {
            self.mov_instruction(destination_index.into(), *return_register);
        }
        self.stop_instruction();
    }

    /// Emits a `mov` instruction.
    ///
    /// Copies the value at `source` into `destination`
    pub(crate) fn mov_instruction(&mut self, destination: RegisterIndex, source: RegisterIndex) {
        self.push_opcode(BrilligOpcode::Mov { destination, source });
    }

    /// Processes a binary instruction according `operation`.
    ///
    /// This method will compute lhs <operation> rhs
    /// and store the result in the `result` register.
    pub(crate) fn binary_instruction(
        &mut self,
        lhs: RegisterIndex,
        rhs: RegisterIndex,
        result: RegisterIndex,
        operation: BrilligBinaryOp,
    ) {
        match operation {
            BrilligBinaryOp::Field { op } => {
                let opcode = BrilligOpcode::BinaryFieldOp { op, destination: result, lhs, rhs };
                self.push_opcode(opcode);
            }
            BrilligBinaryOp::Integer { op, bit_size } => {
                let opcode =
                    BrilligOpcode::BinaryIntOp { op, destination: result, bit_size, lhs, rhs };
                self.push_opcode(opcode);
            }
            BrilligBinaryOp::Modulo { is_signed_integer, bit_size } => {
                self.modulo_instruction(result, lhs, rhs, bit_size, is_signed_integer);
            }
        }
    }

    /// Stores the value of `constant` in the `result` register
    pub(crate) fn const_instruction(&mut self, result: RegisterIndex, constant: Value) {
        self.push_opcode(BrilligOpcode::Const { destination: result, value: constant });
    }

    /// Processes a not instruction.
    ///
    /// Not is computed using a subtraction operation as there is no native not instruction
    /// in Brillig.
    pub(crate) fn not_instruction(&mut self, condition: RegisterIndex, result: RegisterIndex) {
        let one = self.make_constant(Value::from(FieldElement::one()));

        // Compile !x as (1 - x)
        let opcode = BrilligOpcode::BinaryIntOp {
            destination: result,
            op: BinaryIntOp::Sub,
            bit_size: 1,
            lhs: one,
            rhs: condition,
        };
        self.push_opcode(opcode);
    }

    /// Processes a foreign call instruction.
    ///
    /// Note: the function being called is external and will
    /// not be linked during brillig generation.
    pub(crate) fn foreign_call_instruction(
        &mut self,
        func_name: String,
        inputs: &[RegisterValueOrArray],
        outputs: &[RegisterValueOrArray],
    ) {
        // TODO(https://github.com/noir-lang/acvm/issues/366): Enable multiple inputs and outputs to a foreign call
        let opcode = BrilligOpcode::ForeignCall {
            function: func_name,
            destination: outputs[0],
            input: inputs[0],
        };
        self.push_opcode(opcode);
    }

    /// Emits a load instruction
    pub(crate) fn load_instruction(
        &mut self,
        destination: RegisterIndex,
        source_pointer: RegisterIndex,
    ) {
        self.push_opcode(BrilligOpcode::Load { destination, source_pointer });
    }

    /// Emits a store instruction
    pub(crate) fn store_instruction(
        &mut self,
        destination_pointer: RegisterIndex,
        source: RegisterIndex,
    ) {
        self.push_opcode(BrilligOpcode::Store { destination_pointer, source });
    }

    /// Emits a truncate instruction.
    ///
    /// Note: Truncation is used as an optimization in the SSA IR
    /// for the ACIR generation pass; ACIR gen does not overflow
    /// on every integer operation since it would be in-efficient.
    /// Instead truncation instructions are emitted as to when a
    /// truncation should be done.
    /// For Brillig, all integer operations will overflow as its cheap.
    pub(crate) fn truncate_instruction(
        &mut self,
        destination_of_truncated_value: RegisterIndex,
        value_to_truncate: RegisterIndex,
    ) {
        // Effectively a no-op because brillig already has implicit truncation on integer
        // operations. We need only copy the value to it's destination.
        self.mov_instruction(destination_of_truncated_value, value_to_truncate);
    }

    /// Emits a stop instruction
    pub(crate) fn stop_instruction(&mut self) {
        self.push_opcode(BrilligOpcode::Stop);
    }

    /// Returns a register which holds the value of a constant
    pub(crate) fn make_constant(&mut self, constant: Value) -> RegisterIndex {
        let register = self.allocate_register();
        self.const_instruction(register, constant);
        register
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
    pub(crate) fn modulo_instruction(
        &mut self,
        result_register: RegisterIndex,
        left: RegisterIndex,
        right: RegisterIndex,
        bit_size: u32,
        signed: bool,
    ) {
        let scratch_register_i = self.allocate_register();
        let scratch_register_j = self.allocate_register();

        // i = left / right
        self.push_opcode(BrilligOpcode::BinaryIntOp {
            op: match signed {
                true => BinaryIntOp::SignedDiv,
                false => BinaryIntOp::UnsignedDiv,
            },
            destination: scratch_register_i,
            bit_size,
            lhs: left,
            rhs: right,
        });

        // j = i * right
        self.push_opcode(BrilligOpcode::BinaryIntOp {
            op: BinaryIntOp::Mul,
            destination: scratch_register_j,
            bit_size,
            lhs: scratch_register_i,
            rhs: right,
        });

        // result_register = left - j
        self.push_opcode(BrilligOpcode::BinaryIntOp {
            op: BinaryIntOp::Sub,
            destination: result_register,
            bit_size,
            lhs: left,
            rhs: scratch_register_j,
        });
    }

    /// Emits a modulo instruction against 2**target_bit_size
    ///
    /// Integer arithmetic in Brillig is currently constrained to 127 bit integers.
    /// We restrict the cast operation, so that integer types over 127 bits
    /// cannot be created.
    pub(crate) fn cast_instruction(
        &mut self,
        destination: RegisterIndex,
        source: RegisterIndex,
        target_bit_size: u32,
    ) {
        assert!(
            target_bit_size <= BRILLIG_INTEGER_ARITHMETIC_BIT_SIZE,
            "tried to cast to a bit size greater than allowed {target_bit_size}"
        );

        // The brillig VM performs all arithmetic operations modulo 2**bit_size
        // So to cast any value to a target bit size we can just issue a no-op arithmetic operation
        // With bit size equal to target_bit_size
        let zero = self.make_constant(Value::from(FieldElement::zero()));
        self.binary_instruction(
            source,
            zero,
            destination,
            BrilligBinaryOp::Integer { op: BinaryIntOp::Add, bit_size: target_bit_size },
        );
    }
}

/// Type to encapsulate the binary operation types in Brillig
pub(crate) enum BrilligBinaryOp {
    Field { op: BinaryFieldOp },
    Integer { op: BinaryIntOp, bit_size: u32 },
    // Modulo operation requires more than one opcode
    // Brillig.
    Modulo { is_signed_integer: bool, bit_size: u32 },
}
