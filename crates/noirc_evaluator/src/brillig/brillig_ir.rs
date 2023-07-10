//! This module is an abstraction layer over `Brillig`
//! To allow for separation of concerns, it knows nothing
//! about SSA types, and can therefore be tested independently.
//! `brillig_gen` is therefore the module which combines both
//! ssa types and types in this module.
//! A similar paradigm can be seen with the `acir_ir` module.
pub(crate) mod artifact;
pub(crate) mod debug_show;
pub(crate) mod registers;

use self::{
    artifact::{BrilligArtifact, BrilligParameter, UnresolvedJumpLocation},
    registers::BrilligRegistersContext,
};
use acvm::{
    acir::brillig_vm::{
        BinaryFieldOp, BinaryIntOp, BlackBoxOp, HeapArray, HeapVector, Opcode as BrilligOpcode,
        RegisterIndex, RegisterOrMemory, Value,
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
/// The Brillig VM does not apply a limit to the memory address space,
/// As a convention, we take use 64 bits. This means that we assume that
/// memory has 2^64 memory slots.
pub(crate) const BRILLIG_MEMORY_ADDRESSING_BIT_SIZE: u32 = 64;

// Registers reserved in runtime for special purposes.
pub(crate) enum ReservedRegisters {
    /// This register stores the stack pointer. Allocations must be done after this pointer.
    StackPointer = 0,
    /// This register stores the previous stack pointer. The registers of the caller are stored here.
    PreviousStackPointer = 1,
}

impl ReservedRegisters {
    /// The number of reserved registers.
    ///
    /// This is used to offset the general registers
    /// which should not overwrite the special register
    const NUM_RESERVED_REGISTERS: usize = 2;

    /// Returns the length of the reserved registers
    pub(crate) fn len() -> usize {
        Self::NUM_RESERVED_REGISTERS
    }

    /// Returns the stack pointer register. This will get used to allocate memory in runtime.
    pub(crate) fn stack_pointer() -> RegisterIndex {
        RegisterIndex::from(ReservedRegisters::StackPointer as usize)
    }

    /// Returns the previous stack pointer register. This will be used to restore the registers after a fn call.
    pub(crate) fn previous_stack_pointer() -> RegisterIndex {
        RegisterIndex::from(ReservedRegisters::PreviousStackPointer as usize)
    }

    /// Returns a user defined (non-reserved) register index.
    fn user_register_index(index: usize) -> RegisterIndex {
        RegisterIndex::from(index + ReservedRegisters::len())
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
    pub(crate) fn new(
        arguments: Vec<BrilligParameter>,
        return_parameters: Vec<BrilligParameter>,
    ) -> BrilligContext {
        BrilligContext {
            obj: BrilligArtifact::new(arguments, return_parameters),
            registers: BrilligRegistersContext::new(),
            context_label: String::default(),
            section_label: 0,
        }
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
        // debug_show handled by allocate_array_instruction
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
        debug_show::allocate_array_instruction(pointer_register, size_register);
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

    /// Allocates a single value and stores the
    /// pointer to the array in `pointer_register`
    pub(crate) fn allocate_instruction(&mut self, pointer_register: RegisterIndex) {
        debug_show::allocate_instruction(pointer_register);
        let size_register = self.make_constant(1_u128.into());
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
        debug_show::array_get(array_ptr, index, result);
        // Computes array_ptr + index, ie array[index]
        let index_of_element_in_memory = self.allocate_register();
        self.binary_instruction(
            array_ptr,
            index,
            index_of_element_in_memory,
            BrilligBinaryOp::Field { op: BinaryFieldOp::Add },
        );

        self.load_instruction(result, index_of_element_in_memory);
        // Free up temporary register
        self.deallocate_register(index_of_element_in_memory);
    }

    /// Sets the item in the array at index `index` to `value`
    pub(crate) fn array_set(
        &mut self,
        array_ptr: RegisterIndex,
        index: RegisterIndex,
        value: RegisterIndex,
    ) {
        debug_show::array_set(array_ptr, index, value);
        // Computes array_ptr + index, ie array[index]
        let index_of_element_in_memory = self.allocate_register();
        self.binary_instruction(
            array_ptr,
            index,
            index_of_element_in_memory,
            BrilligBinaryOp::Field { op: BinaryFieldOp::Add },
        );

        self.store_instruction(index_of_element_in_memory, value);
        // Free up temporary register
        self.deallocate_register(index_of_element_in_memory);
    }

    /// Copies the values of an array pointed by source with length stored in `num_elements_register`
    /// Into the array pointed by destination
    pub(crate) fn copy_array_instruction(
        &mut self,
        source: RegisterIndex,
        destination: RegisterIndex,
        num_elements_register: RegisterIndex,
    ) {
        debug_show::copy_array_instruction(source, destination, num_elements_register);
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

        self.not_instruction(index_less_than_array_len, 1, index_less_than_array_len);
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
        self.deallocate_register(index_less_than_array_len);
        self.deallocate_register(value_register);
        self.deallocate_register(one_register);
        self.deallocate_register(index_register);
    }

    /// Adds a label to the next opcode
    pub(crate) fn enter_context<T: ToString>(&mut self, label: T) {
        debug_show::enter_context(label.to_string());
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
        debug_show::jump_instruction(target_label.to_string());
        self.add_unresolved_jump(BrilligOpcode::Jump { location: 0 }, target_label.to_string());
    }

    /// Adds a unresolved `JumpIf` instruction to the bytecode.
    pub(crate) fn jump_if_instruction<T: ToString>(
        &mut self,
        condition: RegisterIndex,
        target_label: T,
    ) {
        debug_show::jump_if_instruction(condition, target_label.to_string());
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

    /// Allocates an unused register.
    pub(crate) fn allocate_register(&mut self) -> RegisterIndex {
        self.registers.allocate_register()
    }

    /// Push a register to the deallocation list, ready for reuse.
    /// TODO(AD): currently, register deallocation is only done with immediate values.
    /// TODO(AD): See https://github.com/noir-lang/noir/issues/1720
    pub(crate) fn deallocate_register(&mut self, register_index: RegisterIndex) {
        self.registers.deallocate_register(register_index);
    }
}

impl BrilligContext {
    /// Emits brillig bytecode to jump to a trap condition if `condition`
    /// is false.
    pub(crate) fn constrain_instruction(&mut self, condition: RegisterIndex) {
        debug_show::constrain_instruction(condition);
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
        debug_show::return_instruction(return_registers);
        let mut sources = Vec::with_capacity(return_registers.len());
        let mut destinations = Vec::with_capacity(return_registers.len());

        for (destination_index, return_register) in return_registers.iter().enumerate() {
            // In case we have fewer return registers than indices to write to, ensure we've allocated this register
            let destination_register = ReservedRegisters::user_register_index(destination_index);
            self.registers.ensure_register_is_allocated(destination_register);
            sources.push(*return_register);
            destinations.push(destination_register);
        }
        self.mov_registers_to_registers_instruction(sources, destinations);
        self.stop_instruction();
    }

    /// This function moves values from a set of registers to another set of registers.
    /// It first moves all sources to new allocated registers to avoid overwriting.
    pub(crate) fn mov_registers_to_registers_instruction(
        &mut self,
        sources: Vec<RegisterIndex>,
        destinations: Vec<RegisterIndex>,
    ) {
        let new_sources: Vec<_> = sources
            .iter()
            .map(|source| {
                let new_source = self.allocate_register();
                self.mov_instruction(new_source, *source);
                new_source
            })
            .collect();
        for (new_source, destination) in new_sources.iter().zip(destinations.iter()) {
            self.mov_instruction(*destination, *new_source);
        }
    }

    /// Emits a `mov` instruction.
    ///
    /// Copies the value at `source` into `destination`
    pub(crate) fn mov_instruction(&mut self, destination: RegisterIndex, source: RegisterIndex) {
        debug_show::mov_instruction(destination, source);
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
        debug_show::binary_instruction(lhs, rhs, result, operation.clone());
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
        debug_show::const_instruction(result, constant);
        self.push_opcode(BrilligOpcode::Const { destination: result, value: constant });
    }

    /// Processes a not instruction.
    ///
    /// Not is computed using a subtraction operation as there is no native not instruction
    /// in Brillig.
    pub(crate) fn not_instruction(
        &mut self,
        input: RegisterIndex,
        bit_size: u32,
        result: RegisterIndex,
    ) {
        debug_show::not_instruction(input, bit_size, result);
        // Compile !x as ((-1) - x)
        let u_max = FieldElement::from(2_i128).pow(&FieldElement::from(bit_size as i128))
            - FieldElement::one();
        let max = self.make_constant(Value::from(u_max));
        let opcode = BrilligOpcode::BinaryIntOp {
            destination: result,
            op: BinaryIntOp::Sub,
            bit_size,
            lhs: max,
            rhs: input,
        };
        self.push_opcode(opcode);
        self.deallocate_register(max);
    }

    /// Processes a foreign call instruction.
    ///
    /// Note: the function being called is external and will
    /// not be linked during brillig generation.
    pub(crate) fn foreign_call_instruction(
        &mut self,
        func_name: String,
        inputs: &[RegisterOrMemory],
        outputs: &[RegisterOrMemory],
    ) {
        debug_show::foreign_call_instruction(func_name.clone(), inputs, outputs);
        let opcode = BrilligOpcode::ForeignCall {
            function: func_name,
            destinations: outputs.to_vec(),
            inputs: inputs.to_vec(),
        };
        self.push_opcode(opcode);
    }

    /// Emits a load instruction
    pub(crate) fn load_instruction(
        &mut self,
        destination: RegisterIndex,
        source_pointer: RegisterIndex,
    ) {
        debug_show::load_instruction(destination, source_pointer);
        self.push_opcode(BrilligOpcode::Load { destination, source_pointer });
    }

    /// Emits a store instruction
    pub(crate) fn store_instruction(
        &mut self,
        destination_pointer: RegisterIndex,
        source: RegisterIndex,
    ) {
        debug_show::store_instruction(destination_pointer, source);
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
        debug_show::stop_instruction();
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
        // no debug_show, shown in binary instruction
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
        // Free scratch registers
        self.deallocate_register(scratch_register_i);
        self.deallocate_register(scratch_register_j);
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
        debug_show::cast_instruction(destination, source, target_bit_size);
        assert!(
            target_bit_size <= BRILLIG_INTEGER_ARITHMETIC_BIT_SIZE,
            "tried to cast to a bit size greater than allowed {target_bit_size}"
        );

        // The brillig VM performs all arithmetic operations modulo 2**bit_size
        // So to cast any value to a target bit size we can just issue a no-op arithmetic operation
        // With bit size equal to target_bit_size
        let zero_register = self.make_constant(Value::from(FieldElement::zero()));
        self.binary_instruction(
            source,
            zero_register,
            destination,
            BrilligBinaryOp::Integer { op: BinaryIntOp::Add, bit_size: target_bit_size },
        );
        self.deallocate_register(zero_register);
    }

    /// Adds a unresolved external `Call` instruction to the bytecode.
    /// This calls into another function compiled into this brillig artifact.
    pub(crate) fn add_external_call_instruction<T: ToString>(&mut self, func_label: T) {
        debug_show::add_external_call_instruction(func_label.to_string());
        self.obj.add_unresolved_external_call(
            BrilligOpcode::Call { location: 0 },
            func_label.to_string(),
        );
    }

    /// Returns the i'th register after the reserved ones
    pub(crate) fn register(&self, i: usize) -> RegisterIndex {
        RegisterIndex::from(ReservedRegisters::NUM_RESERVED_REGISTERS + i)
    }

    /// Saves all of the registers that have been used up until this point.
    fn save_all_used_registers(&mut self) -> Vec<RegisterIndex> {
        // Save all of the used registers at this point in memory
        // because the function call will/may overwrite them.
        //
        // Note that here it is important that the stack pointer register is at register 0,
        // as after the first register save we add to the pointer.
        let mut used_registers: Vec<_> = self.registers.used_registers_iter().collect();

        // Also dump the previous stack pointer
        used_registers.push(ReservedRegisters::previous_stack_pointer());
        for register in used_registers.iter() {
            self.store_instruction(ReservedRegisters::stack_pointer(), *register);
            // Add one to our stack pointer
            self.usize_op(ReservedRegisters::stack_pointer(), BinaryIntOp::Add, 1);
        }

        // Store the location of our registers in the previous stack pointer
        self.mov_instruction(
            ReservedRegisters::previous_stack_pointer(),
            ReservedRegisters::stack_pointer(),
        );
        used_registers
    }

    /// Loads all of the registers that have been save by save_all_used_registers.
    fn load_all_saved_registers(&mut self, used_registers: &[RegisterIndex]) {
        // Load all of the used registers that we saved.
        // We do all the reverse operations of save_all_used_registers.
        // Iterate our registers in reverse
        let iterator_register = self.allocate_register();
        self.mov_instruction(iterator_register, ReservedRegisters::previous_stack_pointer());

        for register in used_registers.iter().rev() {
            // Subtract one from our stack pointer
            self.usize_op(iterator_register, BinaryIntOp::Sub, 1);
            self.load_instruction(*register, iterator_register);
        }
    }

    /// Utility method to perform a binary instruction with a constant value
    pub(crate) fn usize_op(
        &mut self,
        destination: RegisterIndex,
        op: BinaryIntOp,
        constant: usize,
    ) {
        let const_register = self.make_constant(Value::from(constant));
        self.binary_instruction(
            destination,
            const_register,
            destination,
            BrilligBinaryOp::Integer { op, bit_size: BRILLIG_MEMORY_ADDRESSING_BIT_SIZE },
        );
        // Mark as no longer used for this purpose, frees for reuse
        self.deallocate_register(const_register);
    }

    // Used before a call instruction.
    // Save all the registers we have used to the stack.
    // Move argument values to the front of the register indices.
    pub(crate) fn pre_call_save_registers_prep_args(
        &mut self,
        arguments: &[RegisterIndex],
    ) -> Vec<RegisterIndex> {
        // Save all the registers we have used to the stack.
        let saved_registers = self.save_all_used_registers();

        // Move argument values to the front of the registers
        //
        // This means that the arguments will be in the first `n` registers after
        // the number of reserved registers.
        let (sources, destinations) =
            arguments.iter().enumerate().map(|(i, argument)| (*argument, self.register(i))).unzip();
        self.mov_registers_to_registers_instruction(sources, destinations);
        saved_registers
    }

    // Used after a call instruction.
    // Move return values to the front of the register indices.
    // Load all the registers we have previous saved in save_registers_prep_args.
    pub(crate) fn post_call_prep_returns_load_registers(
        &mut self,
        result_registers: &[RegisterIndex],
        saved_registers: &[RegisterIndex],
    ) {
        // Allocate our result registers and write into them
        // We assume the return values of our call are held in 0..num results register indices
        for (i, result_register) in result_registers.iter().enumerate() {
            self.mov_instruction(*result_register, self.register(i));
        }

        // Restore all the same registers we have, in exact reverse order.
        // Note that we have allocated some registers above, which we will not be handling here,
        // only restoring registers that were used prior to the call finishing.
        // After the call instruction, the stack frame pointer should be back to where we left off,
        // so we do our instructions in reverse order.
        self.load_all_saved_registers(saved_registers);
    }

    /// Utility method to transform a HeapArray to a HeapVector by making a runtime constant with the size.
    pub(crate) fn array_to_vector(&mut self, array: &HeapArray) -> HeapVector {
        let size_register = self.make_constant(array.size.into());
        HeapVector { size: size_register, pointer: array.pointer }
    }

    /// Issues a blackbox operation.
    pub(crate) fn black_box_op_instruction(&mut self, op: BlackBoxOp) {
        debug_show::black_box_op_instruction(op);
        self.push_opcode(BrilligOpcode::BlackBox(op));
    }
}

/// Type to encapsulate the binary operation types in Brillig
#[derive(Clone)]
pub(crate) enum BrilligBinaryOp {
    Field { op: BinaryFieldOp },
    Integer { op: BinaryIntOp, bit_size: u32 },
    // Modulo operation requires more than one opcode
    // Brillig.
    Modulo { is_signed_integer: bool, bit_size: u32 },
}

#[cfg(test)]
mod tests {
    use std::vec;

    use acvm::acir::brillig_vm::{
        BinaryIntOp, ForeignCallOutput, ForeignCallResult, HeapVector, RegisterIndex,
        RegisterOrMemory, Registers, VMStatus, Value, VM,
    };

    use crate::brillig::brillig_ir::{BrilligContext, BRILLIG_MEMORY_ADDRESSING_BIT_SIZE};

    use super::{BrilligBinaryOp, BrilligOpcode, ReservedRegisters};

    /// Test a Brillig foreign call returning a vector
    #[test]
    fn test_brillig_ir_foreign_call_return_vector() {
        // pseudo-noir:
        //
        // #[oracle(make_number_sequence)]
        // unconstrained fn make_number_sequence(size: u32) -> Vec<u32> {
        // }
        //
        // unconstrained fn main() -> Vec<u32> {
        //   let the_sequence = make_number_sequence(12);
        //   assert(the_sequence.len() == 12);
        // }
        let mut context = BrilligContext::new(vec![], vec![]);
        let r_stack = ReservedRegisters::stack_pointer();
        // Start stack pointer at 0
        context.const_instruction(r_stack, Value::from(0_usize));
        let r_input_size = RegisterIndex::from(ReservedRegisters::len());
        let r_array_ptr = RegisterIndex::from(ReservedRegisters::len() + 1);
        let r_output_size = RegisterIndex::from(ReservedRegisters::len() + 2);
        let r_equality = RegisterIndex::from(ReservedRegisters::len() + 3);
        context.const_instruction(r_input_size, Value::from(12_usize));
        // copy our stack frame to r_array_ptr
        context.mov_instruction(r_array_ptr, r_stack);
        context.foreign_call_instruction(
            "make_number_sequence".into(),
            &[RegisterOrMemory::RegisterIndex(r_input_size)],
            &[RegisterOrMemory::HeapVector(HeapVector { pointer: r_stack, size: r_output_size })],
        );
        // push stack frame by r_returned_size
        context.binary_instruction(
            r_stack,
            r_output_size,
            r_stack,
            BrilligBinaryOp::Integer {
                op: BinaryIntOp::Add,
                bit_size: BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
            },
        );
        // check r_input_size == r_output_size
        context.binary_instruction(
            r_input_size,
            r_output_size,
            r_equality,
            BrilligBinaryOp::Integer {
                op: BinaryIntOp::Equals,
                bit_size: BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
            },
        );
        // We push a JumpIf and Trap opcode directly as the constrain instruction
        // uses unresolved jumps which requires a block to be constructed in SSA and
        // we don't need this for Brillig IR tests
        context.push_opcode(BrilligOpcode::JumpIf { condition: r_equality, location: 8 });
        context.push_opcode(BrilligOpcode::Trap);

        context.stop_instruction();

        let bytecode = context.artifact().byte_code;
        let number_sequence: Vec<Value> = (0_usize..12_usize).map(Value::from).collect();
        let mut vm = VM::new(
            Registers { inner: vec![] },
            vec![],
            bytecode,
            vec![ForeignCallResult { values: vec![ForeignCallOutput::Array(number_sequence)] }],
        );
        let status = vm.process_opcodes();
        assert_eq!(status, VMStatus::Finished);
    }
}
