//! This module is an abstraction layer over `Brillig`
//! To allow for separation of concerns, it knows nothing
//! about SSA types, and can therefore be tested independently.
//! `brillig_gen` is therefore the module which combines both
//! ssa types and types in this module.
//! A similar paradigm can be seen with the `acir_ir` module.
pub(crate) mod artifact;
pub(crate) mod brillig_variable;
pub(crate) mod debug_show;
pub(crate) mod registers;

mod entry_point;

use crate::ssa::ir::dfg::CallStack;

use self::{
    artifact::{BrilligArtifact, UnresolvedJumpLocation},
    brillig_variable::{BrilligArray, BrilligVariable, BrilligVector, SingleAddrVariable},
    registers::BrilligRegistersContext,
};
use acvm::{
    acir::brillig::{
        BinaryFieldOp, BinaryIntOp, BlackBoxOp, MemoryAddress, Opcode as BrilligOpcode, Value,
        ValueOrArray,
    },
    brillig_vm::brillig::HeapValueType,
    FieldElement,
};
use debug_show::DebugShow;
use num_bigint::BigUint;

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
    pub(crate) fn stack_pointer() -> MemoryAddress {
        MemoryAddress::from(ReservedRegisters::StackPointer as usize)
    }

    /// Returns the previous stack pointer register. This will be used to restore the registers after a fn call.
    pub(crate) fn previous_stack_pointer() -> MemoryAddress {
        MemoryAddress::from(ReservedRegisters::PreviousStackPointer as usize)
    }

    /// Returns a user defined (non-reserved) register index.
    fn user_register_index(index: usize) -> MemoryAddress {
        MemoryAddress::from(index + ReservedRegisters::len())
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
    /// Stores the next available section
    next_section: usize,
    /// IR printer
    debug_show: DebugShow,
}

impl BrilligContext {
    /// Initial context state
    pub(crate) fn new(enable_debug_trace: bool) -> BrilligContext {
        BrilligContext {
            obj: BrilligArtifact::default(),
            registers: BrilligRegistersContext::new(),
            context_label: String::default(),
            section_label: 0,
            next_section: 1,
            debug_show: DebugShow::new(enable_debug_trace),
        }
    }

    pub(crate) fn set_allocated_registers(&mut self, allocated_registers: Vec<MemoryAddress>) {
        self.registers = BrilligRegistersContext::from_preallocated_registers(allocated_registers);
    }

    /// Adds a brillig instruction to the brillig byte code
    pub(crate) fn push_opcode(&mut self, opcode: BrilligOpcode) {
        self.obj.push_opcode(opcode);
    }

    /// Returns the artifact
    pub(crate) fn artifact(self) -> BrilligArtifact {
        self.obj
    }

    /// Allocates an array of size `size` and stores the pointer to the array
    /// in `pointer_register`
    pub(crate) fn allocate_fixed_length_array(
        &mut self,
        pointer_register: MemoryAddress,
        size: usize,
    ) {
        // debug_show handled by allocate_array_instruction
        let size_register = self.make_usize_constant(size.into());
        self.allocate_array_instruction(pointer_register, size_register.address);
        self.deallocate_single_addr(size_register);
    }

    /// Allocates an array of size contained in size_register and stores the
    /// pointer to the array in `pointer_register`
    pub(crate) fn allocate_array_instruction(
        &mut self,
        pointer_register: MemoryAddress,
        size_register: MemoryAddress,
    ) {
        self.debug_show.allocate_array_instruction(pointer_register, size_register);
        self.set_array_pointer(pointer_register);
        self.update_stack_pointer(size_register);
    }

    pub(crate) fn set_array_pointer(&mut self, pointer_register: MemoryAddress) {
        self.debug_show.mov_instruction(pointer_register, ReservedRegisters::stack_pointer());
        self.push_opcode(BrilligOpcode::Mov {
            destination: pointer_register,
            source: ReservedRegisters::stack_pointer(),
        });
    }

    pub(crate) fn update_stack_pointer(&mut self, size_register: MemoryAddress) {
        self.memory_op(
            ReservedRegisters::stack_pointer(),
            size_register,
            ReservedRegisters::stack_pointer(),
            BinaryIntOp::Add,
        );
    }

    /// Allocates a variable in memory and stores the
    /// pointer to the array in `pointer_register`
    fn allocate_variable_reference_instruction(
        &mut self,
        pointer_register: MemoryAddress,
        size: usize,
    ) {
        self.debug_show.allocate_instruction(pointer_register);
        // A variable can be stored in up to three values, so we reserve three values for that.
        let size_register = self.make_usize_constant(size.into());
        self.push_opcode(BrilligOpcode::Mov {
            destination: pointer_register,
            source: ReservedRegisters::stack_pointer(),
        });
        self.memory_op(
            ReservedRegisters::stack_pointer(),
            size_register.address,
            ReservedRegisters::stack_pointer(),
            BinaryIntOp::Add,
        );
        self.deallocate_single_addr(size_register);
    }

    pub(crate) fn allocate_single_addr_reference_instruction(
        &mut self,
        pointer_register: MemoryAddress,
    ) {
        self.allocate_variable_reference_instruction(pointer_register, 1);
    }

    pub(crate) fn allocate_array_reference_instruction(&mut self, pointer_register: MemoryAddress) {
        self.allocate_variable_reference_instruction(
            pointer_register,
            BrilligArray::registers_count(),
        );
    }

    pub(crate) fn allocate_vector_reference_instruction(
        &mut self,
        pointer_register: MemoryAddress,
    ) {
        self.allocate_variable_reference_instruction(
            pointer_register,
            BrilligVector::registers_count(),
        );
    }

    /// Gets the value in the array at index `index` and stores it in `result`
    pub(crate) fn array_get(
        &mut self,
        array_ptr: MemoryAddress,
        index: SingleAddrVariable,
        result: MemoryAddress,
    ) {
        assert!(index.bit_size == BRILLIG_MEMORY_ADDRESSING_BIT_SIZE);
        self.debug_show.array_get(array_ptr, index.address, result);
        // Computes array_ptr + index, ie array[index]
        let index_of_element_in_memory = self.allocate_register();
        self.memory_op(array_ptr, index.address, index_of_element_in_memory, BinaryIntOp::Add);

        self.load_instruction(result, index_of_element_in_memory);
        // Free up temporary register
        self.deallocate_register(index_of_element_in_memory);
    }

    /// Sets the item in the array at index `index` to `value`
    pub(crate) fn array_set(
        &mut self,
        array_ptr: MemoryAddress,
        index: SingleAddrVariable,
        value: MemoryAddress,
    ) {
        assert!(index.bit_size == BRILLIG_MEMORY_ADDRESSING_BIT_SIZE);
        self.debug_show.array_set(array_ptr, index.address, value);
        // Computes array_ptr + index, ie array[index]
        let index_of_element_in_memory = self.allocate_register();
        self.memory_op(array_ptr, index.address, index_of_element_in_memory, BinaryIntOp::Add);

        self.store_instruction(index_of_element_in_memory, value);
        // Free up temporary register
        self.deallocate_register(index_of_element_in_memory);
    }

    /// Copies the values of an array pointed by source with length stored in `num_elements_register`
    /// Into the array pointed by destination
    pub(crate) fn copy_array_instruction(
        &mut self,
        source_pointer: MemoryAddress,
        destination_pointer: MemoryAddress,
        num_elements_variable: SingleAddrVariable,
    ) {
        assert!(num_elements_variable.bit_size == BRILLIG_MEMORY_ADDRESSING_BIT_SIZE);
        self.debug_show.copy_array_instruction(
            source_pointer,
            destination_pointer,
            num_elements_variable.address,
        );

        let value_register = self.allocate_register();

        self.loop_instruction(num_elements_variable.address, |ctx, iterator| {
            ctx.array_get(source_pointer, iterator, value_register);
            ctx.array_set(destination_pointer, iterator, value_register);
        });

        self.deallocate_register(value_register);
    }

    /// This instruction will issue a loop that will iterate iteration_count times
    /// The body of the loop should be issued by the caller in the on_iteration closure.
    pub(crate) fn loop_instruction<F>(&mut self, iteration_count: MemoryAddress, on_iteration: F)
    where
        F: FnOnce(&mut BrilligContext, SingleAddrVariable),
    {
        let iterator_register = self.make_usize_constant(0_u128.into());

        let (loop_section, loop_label) = self.reserve_next_section_label();
        self.enter_section(loop_section);

        // Loop body

        // Check if iterator < iteration_count
        let iterator_less_than_iterations =
            SingleAddrVariable { address: self.allocate_register(), bit_size: 1 };

        self.memory_op(
            iterator_register.address,
            iteration_count,
            iterator_less_than_iterations.address,
            BinaryIntOp::LessThan,
        );

        let (exit_loop_section, exit_loop_label) = self.reserve_next_section_label();

        self.not_instruction(iterator_less_than_iterations, iterator_less_than_iterations);

        self.jump_if_instruction(iterator_less_than_iterations.address, exit_loop_label);

        // Call the on iteration function
        on_iteration(self, iterator_register);

        // Increment the iterator register
        self.usize_op_in_place(iterator_register.address, BinaryIntOp::Add, 1);

        self.jump_instruction(loop_label);

        // Exit the loop
        self.enter_section(exit_loop_section);

        // Deallocate our temporary registers
        self.deallocate_single_addr(iterator_less_than_iterations);
        self.deallocate_single_addr(iterator_register);
    }

    /// This instruction will issue an if-then branch that will check if the condition is true
    /// and if so, perform the instructions given in `f(self, true)` and otherwise perform the
    /// instructions given in `f(self, false)`. A boolean is passed instead of two separate
    /// functions to allow the given function to mutably alias its environment.
    pub(crate) fn branch_instruction(
        &mut self,
        condition: MemoryAddress,
        mut f: impl FnMut(&mut BrilligContext, bool),
    ) {
        // Reserve 3 sections
        let (then_section, then_label) = self.reserve_next_section_label();
        let (otherwise_section, otherwise_label) = self.reserve_next_section_label();
        let (end_section, end_label) = self.reserve_next_section_label();

        self.jump_if_instruction(condition, then_label.clone());
        self.jump_instruction(otherwise_label.clone());

        self.enter_section(then_section);
        f(self, true);
        self.jump_instruction(end_label.clone());

        self.enter_section(otherwise_section);
        f(self, false);
        self.jump_instruction(end_label.clone());

        self.enter_section(end_section);
    }

    /// This instruction issues a branch that jumps over the code generated by the given function if the condition is truthy
    pub(crate) fn if_not_instruction(
        &mut self,
        condition: MemoryAddress,
        f: impl FnOnce(&mut BrilligContext),
    ) {
        let (end_section, end_label) = self.reserve_next_section_label();

        self.jump_if_instruction(condition, end_label.clone());

        f(self);

        self.enter_section(end_section);
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
    fn enter_section(&mut self, section: usize) {
        self.section_label = section;
        self.obj
            .add_label_at_position(self.current_section_label(), self.obj.index_of_next_opcode());
    }

    /// Create, reserve, and return a new section label.
    fn reserve_next_section_label(&mut self) -> (usize, String) {
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

    /// Adds a unresolved `Jump` instruction to the bytecode.
    fn add_unresolved_jump(
        &mut self,
        jmp_instruction: BrilligOpcode,
        destination: UnresolvedJumpLocation,
    ) {
        self.obj.add_unresolved_jump(jmp_instruction, destination);
    }

    /// Allocates an unused register.
    pub(crate) fn allocate_register(&mut self) -> MemoryAddress {
        self.registers.allocate_register()
    }

    /// Push a register to the deallocation list, ready for reuse.
    pub(crate) fn deallocate_register(&mut self, register_index: MemoryAddress) {
        self.registers.deallocate_register(register_index);
    }

    /// Deallocates the address where the single address variable is stored
    pub(crate) fn deallocate_single_addr(&mut self, var: SingleAddrVariable) {
        self.deallocate_register(var.address);
    }
}

impl BrilligContext {
    /// Emits brillig bytecode to jump to a trap condition if `condition`
    /// is false.
    pub(crate) fn constrain_instruction(
        &mut self,
        condition: SingleAddrVariable,
        assert_message: Option<String>,
    ) {
        assert!(condition.bit_size == 1);
        self.debug_show.constrain_instruction(condition.address);
        let (next_section, next_label) = self.reserve_next_section_label();
        self.add_unresolved_jump(
            BrilligOpcode::JumpIf { condition: condition.address, location: 0 },
            next_label,
        );
        self.push_opcode(BrilligOpcode::Trap);
        if let Some(assert_message) = assert_message {
            self.obj.add_assert_message_to_last_opcode(assert_message);
        }
        self.enter_section(next_section);
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
    pub(crate) fn return_instruction(&mut self, return_registers: &[MemoryAddress]) {
        self.debug_show.return_instruction(return_registers);
        let mut sources = Vec::with_capacity(return_registers.len());
        let mut destinations = Vec::with_capacity(return_registers.len());

        for (destination_index, return_register) in return_registers.iter().enumerate() {
            // In case we have fewer return registers than indices to write to, ensure we've allocated this register
            let destination_register = ReservedRegisters::user_register_index(destination_index);
            self.registers.ensure_register_is_allocated(destination_register);
            sources.push(*return_register);
            destinations.push(destination_register);
        }
        destinations
            .iter()
            .for_each(|destination| self.registers.ensure_register_is_allocated(*destination));
        self.mov_registers_to_registers_instruction(sources, destinations);
        self.stop_instruction();
    }

    /// This function moves values from a set of registers to another set of registers.
    /// It first moves all sources to new allocated registers to avoid overwriting.
    pub(crate) fn mov_registers_to_registers_instruction(
        &mut self,
        sources: Vec<MemoryAddress>,
        destinations: Vec<MemoryAddress>,
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
            self.deallocate_register(*new_source);
        }
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
        self.push_opcode(BrilligOpcode::Cast {
            destination: destination.address,
            source: source.address,
            bit_size: destination.bit_size,
        });
    }

    fn binary_result_bit_size(operation: BrilligBinaryOp, arguments_bit_size: u32) -> u32 {
        match operation {
            BrilligBinaryOp::Field(BinaryFieldOp::Equals)
            | BrilligBinaryOp::Integer(BinaryIntOp::Equals)
            | BrilligBinaryOp::Integer(BinaryIntOp::LessThan)
            | BrilligBinaryOp::Integer(BinaryIntOp::LessThanEquals) => 1,
            _ => arguments_bit_size,
        }
    }

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
        assert!(
            lhs.bit_size == rhs.bit_size,
            "Not equal bit size for lhs and rhs: lhs {}, rhs {}",
            lhs.bit_size,
            rhs.bit_size
        );
        let expected_result_bit_size =
            BrilligContext::binary_result_bit_size(operation, lhs.bit_size);
        assert!(
            result.bit_size == expected_result_bit_size,
            "Expected result bit size to be {}, got {} for operation {:?}",
            expected_result_bit_size,
            result.bit_size,
            operation
        );
        self.debug_show.binary_instruction(lhs.address, rhs.address, result.address, operation);
        match operation {
            BrilligBinaryOp::Field(op) => {
                let opcode = BrilligOpcode::BinaryFieldOp {
                    op,
                    destination: result.address,
                    lhs: lhs.address,
                    rhs: rhs.address,
                };
                self.push_opcode(opcode);
            }
            BrilligBinaryOp::Integer(op) => {
                let opcode = BrilligOpcode::BinaryIntOp {
                    op,
                    destination: result.address,
                    bit_size: lhs.bit_size,
                    lhs: lhs.address,
                    rhs: rhs.address,
                };
                self.push_opcode(opcode);
            }
            BrilligBinaryOp::Modulo { is_signed_integer } => {
                self.modulo_instruction(result, lhs, rhs, is_signed_integer);
            }
        }
    }

    /// Stores the value of `constant` in the `result` register
    pub(crate) fn const_instruction(&mut self, result: SingleAddrVariable, constant: Value) {
        self.debug_show.const_instruction(result.address, constant);
        self.push_opcode(BrilligOpcode::Const {
            destination: result.address,
            value: constant,
            bit_size: result.bit_size,
        });
    }

    pub(crate) fn usize_const(&mut self, result: MemoryAddress, constant: Value) {
        self.const_instruction(SingleAddrVariable::new_usize(result), constant);
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
        let max = self.make_constant(Value::from(u_max), input.bit_size);

        let opcode = BrilligOpcode::BinaryIntOp {
            destination: result.address,
            op: BinaryIntOp::Sub,
            bit_size: input.bit_size,
            lhs: max.address,
            rhs: input.address,
        };
        self.push_opcode(opcode);
        self.deallocate_single_addr(max);
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
        assert!(inputs.len() == input_value_types.len());
        assert!(outputs.len() == output_value_types.len());
        self.debug_show.foreign_call_instruction(func_name.clone(), inputs, outputs);
        let opcode = BrilligOpcode::ForeignCall {
            function: func_name,
            destinations: outputs.to_vec(),
            destination_value_types: output_value_types.to_vec(),
            inputs: inputs.to_vec(),
            input_value_types: input_value_types.to_vec(),
        };
        self.push_opcode(opcode);
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

    /// Loads a variable stored previously
    pub(crate) fn load_variable_instruction(
        &mut self,
        destination: BrilligVariable,
        variable_pointer: MemoryAddress,
    ) {
        match destination {
            BrilligVariable::SingleAddr(single_addr) => {
                self.load_instruction(single_addr.address, variable_pointer);
            }
            BrilligVariable::BrilligArray(BrilligArray { pointer, size: _, rc }) => {
                self.load_instruction(pointer, variable_pointer);

                let rc_pointer = self.allocate_register();
                self.mov_instruction(rc_pointer, variable_pointer);
                self.usize_op_in_place(rc_pointer, BinaryIntOp::Add, 1_usize);

                self.load_instruction(rc, rc_pointer);
                self.deallocate_register(rc_pointer);
            }
            BrilligVariable::BrilligVector(BrilligVector { pointer, size, rc }) => {
                self.load_instruction(pointer, variable_pointer);

                let size_pointer = self.allocate_register();
                self.mov_instruction(size_pointer, variable_pointer);
                self.usize_op_in_place(size_pointer, BinaryIntOp::Add, 1_usize);

                self.load_instruction(size, size_pointer);
                self.deallocate_register(size_pointer);

                let rc_pointer = self.allocate_register();
                self.mov_instruction(rc_pointer, variable_pointer);
                self.usize_op_in_place(rc_pointer, BinaryIntOp::Add, 2_usize);

                self.load_instruction(rc, rc_pointer);
                self.deallocate_register(rc_pointer);
            }
        }
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

    /// Stores a variable by saving its registers to memory
    pub(crate) fn store_variable_instruction(
        &mut self,
        variable_pointer: MemoryAddress,
        source: BrilligVariable,
    ) {
        match source {
            BrilligVariable::SingleAddr(single_addr) => {
                self.store_instruction(variable_pointer, single_addr.address);
            }
            BrilligVariable::BrilligArray(BrilligArray { pointer, size: _, rc }) => {
                self.store_instruction(variable_pointer, pointer);

                let rc_pointer: MemoryAddress = self.allocate_register();
                self.mov_instruction(rc_pointer, variable_pointer);
                self.usize_op_in_place(rc_pointer, BinaryIntOp::Add, 1_usize);
                self.store_instruction(rc_pointer, rc);
                self.deallocate_register(rc_pointer);
            }
            BrilligVariable::BrilligVector(BrilligVector { pointer, size, rc }) => {
                self.store_instruction(variable_pointer, pointer);

                let size_pointer = self.allocate_register();
                self.mov_instruction(size_pointer, variable_pointer);
                self.usize_op_in_place(size_pointer, BinaryIntOp::Add, 1_usize);
                self.store_instruction(size_pointer, size);

                let rc_pointer: MemoryAddress = self.allocate_register();
                self.mov_instruction(rc_pointer, variable_pointer);
                self.usize_op_in_place(rc_pointer, BinaryIntOp::Add, 2_usize);
                self.store_instruction(rc_pointer, rc);

                self.deallocate_register(size_pointer);
                self.deallocate_register(rc_pointer);
            }
        }
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
        destination_of_truncated_value: SingleAddrVariable,
        value_to_truncate: SingleAddrVariable,
        bit_size: u32,
    ) {
        self.debug_show.truncate_instruction(
            destination_of_truncated_value.address,
            value_to_truncate.address,
            bit_size,
        );
        assert!(
            bit_size <= value_to_truncate.bit_size,
            "tried to truncate to a bit size {} greater than the variable size {}",
            bit_size,
            value_to_truncate.bit_size
        );

        let mask = BigUint::from(2_u32).pow(bit_size) - BigUint::from(1_u32);
        let mask_constant = self.make_constant(
            FieldElement::from_be_bytes_reduce(&mask.to_bytes_be()).into(),
            value_to_truncate.bit_size,
        );

        self.binary_instruction(
            value_to_truncate,
            mask_constant,
            destination_of_truncated_value,
            BrilligBinaryOp::Integer(BinaryIntOp::And),
        );

        self.deallocate_single_addr(mask_constant);
    }

    /// Emits a stop instruction
    pub(crate) fn stop_instruction(&mut self) {
        self.debug_show.stop_instruction();
        self.push_opcode(BrilligOpcode::Stop { return_data_offset: 0, return_data_size: 0 });
    }

    /// Returns a register which holds the value of a constant
    pub(crate) fn make_constant(&mut self, constant: Value, bit_size: u32) -> SingleAddrVariable {
        let var = SingleAddrVariable::new(self.allocate_register(), bit_size);
        self.const_instruction(var, constant);
        var
    }

    /// Returns a register which holds the value of an usize constant
    pub(crate) fn make_usize_constant(&mut self, constant: Value) -> SingleAddrVariable {
        let register = self.allocate_register();
        self.usize_const(register, constant);
        SingleAddrVariable::new_usize(register)
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
        result: SingleAddrVariable,
        left: SingleAddrVariable,
        right: SingleAddrVariable,
        signed: bool,
    ) {
        // no debug_show, shown in binary instruction
        let scratch_register_i = self.allocate_register();
        let scratch_register_j = self.allocate_register();

        assert!(
            left.bit_size == right.bit_size,
            "Not equal bitsize: lhs {}, rhs {}",
            left.bit_size,
            right.bit_size
        );
        let bit_size = left.bit_size;
        // i = left / right
        self.push_opcode(BrilligOpcode::BinaryIntOp {
            op: match signed {
                true => BinaryIntOp::SignedDiv,
                false => BinaryIntOp::UnsignedDiv,
            },
            destination: scratch_register_i,
            bit_size,
            lhs: left.address,
            rhs: right.address,
        });

        // j = i * right
        self.push_opcode(BrilligOpcode::BinaryIntOp {
            op: BinaryIntOp::Mul,
            destination: scratch_register_j,
            bit_size,
            lhs: scratch_register_i,
            rhs: right.address,
        });

        // result_register = left - j
        self.push_opcode(BrilligOpcode::BinaryIntOp {
            op: BinaryIntOp::Sub,
            destination: result.address,
            bit_size,
            lhs: left.address,
            rhs: scratch_register_j,
        });
        // Free scratch registers
        self.deallocate_register(scratch_register_i);
        self.deallocate_register(scratch_register_j);
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

    /// Returns the i'th register after the reserved ones
    pub(crate) fn register(&self, i: usize) -> MemoryAddress {
        MemoryAddress::from(ReservedRegisters::NUM_RESERVED_REGISTERS + i)
    }

    /// Saves all of the registers that have been used up until this point.
    fn save_registers_of_vars(&mut self, vars: &[BrilligVariable]) -> Vec<MemoryAddress> {
        // Save all of the used registers at this point in memory
        // because the function call will/may overwrite them.
        //
        // Note that here it is important that the stack pointer register is at register 0,
        // as after the first register save we add to the pointer.
        let mut used_registers: Vec<_> =
            vars.iter().flat_map(|var| var.extract_registers()).collect();

        // Also dump the previous stack pointer
        used_registers.push(ReservedRegisters::previous_stack_pointer());
        for register in used_registers.iter() {
            self.store_instruction(ReservedRegisters::stack_pointer(), *register);
            // Add one to our stack pointer
            self.usize_op_in_place(ReservedRegisters::stack_pointer(), BinaryIntOp::Add, 1);
        }

        // Store the location of our registers in the previous stack pointer
        self.mov_instruction(
            ReservedRegisters::previous_stack_pointer(),
            ReservedRegisters::stack_pointer(),
        );
        used_registers
    }

    /// Loads all of the registers that have been save by save_all_used_registers.
    fn load_all_saved_registers(&mut self, used_registers: &[MemoryAddress]) {
        // Load all of the used registers that we saved.
        // We do all the reverse operations of save_all_used_registers.
        // Iterate our registers in reverse
        let iterator_register = self.allocate_register();
        self.mov_instruction(iterator_register, ReservedRegisters::previous_stack_pointer());

        for register in used_registers.iter().rev() {
            // Subtract one from our stack pointer
            self.usize_op_in_place(iterator_register, BinaryIntOp::Sub, 1);
            self.load_instruction(*register, iterator_register);
        }
    }

    /// Utility method to perform a binary instruction with a constant value in place
    pub(crate) fn usize_op_in_place(
        &mut self,
        destination: MemoryAddress,
        op: BinaryIntOp,
        constant: usize,
    ) {
        self.usize_op(destination, destination, op, constant);
    }

    /// Utility method to perform a binary instruction with a constant value
    pub(crate) fn usize_op(
        &mut self,
        operand: MemoryAddress,
        destination: MemoryAddress,
        op: BinaryIntOp,
        constant: usize,
    ) {
        let const_register = self.make_usize_constant(Value::from(constant));
        self.memory_op(operand, const_register.address, destination, op);
        // Mark as no longer used for this purpose, frees for reuse
        self.deallocate_single_addr(const_register);
    }

    /// Utility method to perform a binary instruction with a memory address
    pub(crate) fn memory_op(
        &mut self,
        lhs: MemoryAddress,
        rhs: MemoryAddress,
        destination: MemoryAddress,
        op: BinaryIntOp,
    ) {
        self.binary_instruction(
            SingleAddrVariable::new_usize(lhs),
            SingleAddrVariable::new_usize(rhs),
            SingleAddrVariable::new(
                destination,
                BrilligContext::binary_result_bit_size(
                    BrilligBinaryOp::Integer(op),
                    BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
                ),
            ),
            BrilligBinaryOp::Integer(op),
        );
    }

    // Used before a call instruction.
    // Save all the registers we have used to the stack.
    // Move argument values to the front of the register indices.
    pub(crate) fn pre_call_save_registers_prep_args(
        &mut self,
        arguments: &[MemoryAddress],
        variables_to_save: &[BrilligVariable],
    ) -> Vec<MemoryAddress> {
        // Save all the registers we have used to the stack.
        let saved_registers = self.save_registers_of_vars(variables_to_save);

        // Move argument values to the front of the registers
        //
        // This means that the arguments will be in the first `n` registers after
        // the number of reserved registers.
        let (sources, destinations): (Vec<_>, Vec<_>) =
            arguments.iter().enumerate().map(|(i, argument)| (*argument, self.register(i))).unzip();
        destinations
            .iter()
            .for_each(|destination| self.registers.ensure_register_is_allocated(*destination));
        self.mov_registers_to_registers_instruction(sources, destinations);
        saved_registers
    }

    // Used after a call instruction.
    // Move return values to the front of the register indices.
    // Load all the registers we have previous saved in save_registers_prep_args.
    pub(crate) fn post_call_prep_returns_load_registers(
        &mut self,
        result_registers: &[MemoryAddress],
        saved_registers: &[MemoryAddress],
    ) {
        // Allocate our result registers and write into them
        // We assume the return values of our call are held in 0..num results register indices
        let (sources, destinations): (Vec<_>, Vec<_>) = result_registers
            .iter()
            .enumerate()
            .map(|(i, result_register)| (self.register(i), *result_register))
            .unzip();
        sources.iter().for_each(|source| self.registers.ensure_register_is_allocated(*source));
        self.mov_registers_to_registers_instruction(sources, destinations);

        // Restore all the same registers we have, in exact reverse order.
        // Note that we have allocated some registers above, which we will not be handling here,
        // only restoring registers that were used prior to the call finishing.
        // After the call instruction, the stack frame pointer should be back to where we left off,
        // so we do our instructions in reverse order.
        self.load_all_saved_registers(saved_registers);
    }

    /// Utility method to transform a HeapArray to a HeapVector by making a runtime constant with the size.
    pub(crate) fn array_to_vector(&mut self, array: &BrilligArray) -> BrilligVector {
        let size_register = self.make_usize_constant(array.size.into());
        BrilligVector { size: size_register.address, pointer: array.pointer, rc: array.rc }
    }

    /// Issues a blackbox operation.
    pub(crate) fn black_box_op_instruction(&mut self, op: BlackBoxOp) {
        self.debug_show.black_box_op_instruction(&op);
        self.push_opcode(BrilligOpcode::BlackBox(op));
    }

    /// Issues a to_radix instruction. This instruction will write the modulus of the source register
    /// And the radix register limb_count times to the target vector.
    pub(crate) fn radix_instruction(
        &mut self,
        source_field: SingleAddrVariable,
        target_vector: BrilligVector,
        radix: SingleAddrVariable,
        limb_count: SingleAddrVariable,
        big_endian: bool,
    ) {
        assert!(source_field.bit_size == FieldElement::max_num_bits());
        assert!(radix.bit_size == 32);
        assert!(limb_count.bit_size == 32);
        let radix_as_field =
            SingleAddrVariable::new(self.allocate_register(), FieldElement::max_num_bits());
        self.cast_instruction(radix_as_field, radix);

        self.cast_instruction(SingleAddrVariable::new_usize(target_vector.size), limb_count);
        self.usize_const(target_vector.rc, 1_usize.into());
        self.allocate_array_instruction(target_vector.pointer, target_vector.size);

        let shifted_field =
            SingleAddrVariable::new(self.allocate_register(), FieldElement::max_num_bits());
        self.mov_instruction(shifted_field.address, source_field.address);

        let modulus_field =
            SingleAddrVariable::new(self.allocate_register(), FieldElement::max_num_bits());

        self.loop_instruction(target_vector.size, |ctx, iterator_register| {
            // Compute the modulus
            ctx.modulo_instruction(modulus_field, shifted_field, radix_as_field, false);
            // Write it
            ctx.array_set(target_vector.pointer, iterator_register, modulus_field.address);
            // Integer div the field
            ctx.binary_instruction(
                shifted_field,
                radix_as_field,
                shifted_field,
                BrilligBinaryOp::Integer(BinaryIntOp::UnsignedDiv),
            );
        });

        // Deallocate our temporary registers
        self.deallocate_single_addr(shifted_field);
        self.deallocate_single_addr(modulus_field);
        self.deallocate_single_addr(radix_as_field);

        if big_endian {
            self.reverse_vector_in_place_instruction(target_vector);
        }
    }

    /// This instruction will reverse the order of the elements in a vector.
    pub(crate) fn reverse_vector_in_place_instruction(&mut self, vector: BrilligVector) {
        let iteration_count = self.allocate_register();
        self.usize_op(vector.size, iteration_count, BinaryIntOp::UnsignedDiv, 2);

        let start_value_register = self.allocate_register();
        let index_at_end_of_array = self.allocate_register();
        let end_value_register = self.allocate_register();

        self.loop_instruction(iteration_count, |ctx, iterator_register| {
            // Load both values
            ctx.array_get(vector.pointer, iterator_register, start_value_register);

            // The index at the end of array is size - 1 - iterator
            ctx.mov_instruction(index_at_end_of_array, vector.size);
            ctx.usize_op_in_place(index_at_end_of_array, BinaryIntOp::Sub, 1);
            ctx.memory_op(
                index_at_end_of_array,
                iterator_register.address,
                index_at_end_of_array,
                BinaryIntOp::Sub,
            );

            ctx.array_get(
                vector.pointer,
                SingleAddrVariable::new_usize(index_at_end_of_array),
                end_value_register,
            );

            // Write both values
            ctx.array_set(vector.pointer, iterator_register, end_value_register);
            ctx.array_set(
                vector.pointer,
                SingleAddrVariable::new_usize(index_at_end_of_array),
                start_value_register,
            );
        });

        self.deallocate_register(iteration_count);
        self.deallocate_register(start_value_register);
        self.deallocate_register(end_value_register);
        self.deallocate_register(index_at_end_of_array);
    }

    /// Sets a current call stack that the next pushed opcodes will be associated with.
    pub(crate) fn set_call_stack(&mut self, call_stack: CallStack) {
        self.obj.set_call_stack(call_stack);
    }
}

/// Type to encapsulate the binary operation types in Brillig
#[derive(Clone, Copy, Debug)]
pub(crate) enum BrilligBinaryOp {
    Field(BinaryFieldOp),
    Integer(BinaryIntOp),
    // Modulo operation requires more than one brillig opcode
    Modulo { is_signed_integer: bool },
}

#[cfg(test)]
pub(crate) mod tests {
    use std::vec;

    use acvm::acir::brillig::{
        BinaryIntOp, ForeignCallParam, ForeignCallResult, HeapVector, MemoryAddress, Value,
        ValueOrArray,
    };
    use acvm::brillig_vm::brillig::HeapValueType;
    use acvm::brillig_vm::{VMStatus, VM};
    use acvm::{BlackBoxFunctionSolver, BlackBoxResolutionError, FieldElement};

    use crate::brillig::brillig_ir::BrilligContext;

    use super::artifact::{BrilligParameter, GeneratedBrillig};
    use super::{BrilligOpcode, ReservedRegisters};

    pub(crate) struct DummyBlackBoxSolver;

    impl BlackBoxFunctionSolver for DummyBlackBoxSolver {
        fn schnorr_verify(
            &self,
            _public_key_x: &FieldElement,
            _public_key_y: &FieldElement,
            _signature: &[u8],
            _message: &[u8],
        ) -> Result<bool, BlackBoxResolutionError> {
            Ok(true)
        }
        fn pedersen_commitment(
            &self,
            _inputs: &[FieldElement],
            _domain_separator: u32,
        ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
            Ok((2_u128.into(), 3_u128.into()))
        }
        fn pedersen_hash(
            &self,
            _inputs: &[FieldElement],
            _domain_separator: u32,
        ) -> Result<FieldElement, BlackBoxResolutionError> {
            Ok(6_u128.into())
        }
        fn fixed_base_scalar_mul(
            &self,
            _low: &FieldElement,
            _high: &FieldElement,
        ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
            Ok((4_u128.into(), 5_u128.into()))
        }

        fn ec_add(
            &self,
            _input1_x: &FieldElement,
            _input1_y: &FieldElement,
            _input2_x: &FieldElement,
            _input2_y: &FieldElement,
        ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
            panic!("Path not trodden by this test")
        }

        fn poseidon2_permutation(
            &self,
            _inputs: &[FieldElement],
            _len: u32,
        ) -> Result<Vec<FieldElement>, BlackBoxResolutionError> {
            Ok(vec![0_u128.into(), 1_u128.into(), 2_u128.into(), 3_u128.into()])
        }
    }

    pub(crate) fn create_context() -> BrilligContext {
        let mut context = BrilligContext::new(true);
        context.enter_context("test");
        context
    }

    pub(crate) fn create_entry_point_bytecode(
        context: BrilligContext,
        arguments: Vec<BrilligParameter>,
        returns: Vec<BrilligParameter>,
    ) -> GeneratedBrillig {
        let artifact = context.artifact();
        let mut entry_point_artifact =
            BrilligContext::new_entry_point_artifact(arguments, returns, "test".to_string());
        entry_point_artifact.link_with(&artifact);
        entry_point_artifact.finish()
    }

    pub(crate) fn create_and_run_vm(
        calldata: Vec<Value>,
        bytecode: &[BrilligOpcode],
    ) -> (VM<'_, DummyBlackBoxSolver>, usize, usize) {
        let mut vm = VM::new(calldata, bytecode, vec![], &DummyBlackBoxSolver);

        let status = vm.process_opcodes();
        if let VMStatus::Finished { return_data_offset, return_data_size } = status {
            (vm, return_data_offset, return_data_size)
        } else {
            panic!("VM did not finish")
        }
    }

    /// Test a Brillig foreign call returning a vector
    #[test]
    fn test_brillig_ir_foreign_call_return_vector() {
        // pseudo-noir:
        //
        // #[oracle(get_number_sequence)]
        // unconstrained fn get_number_sequence(size: u32) -> Vec<u32> {
        // }
        //
        // unconstrained fn main() -> Vec<u32> {
        //   let the_sequence = get_number_sequence(12);
        //   assert(the_sequence.len() == 12);
        // }
        let mut context = BrilligContext::new(true);
        let r_stack = ReservedRegisters::stack_pointer();
        // Start stack pointer at 0
        context.usize_const(r_stack, Value::from(ReservedRegisters::len() + 3));
        let r_input_size = MemoryAddress::from(ReservedRegisters::len());
        let r_array_ptr = MemoryAddress::from(ReservedRegisters::len() + 1);
        let r_output_size = MemoryAddress::from(ReservedRegisters::len() + 2);
        let r_equality = MemoryAddress::from(ReservedRegisters::len() + 3);
        context.usize_const(r_input_size, Value::from(12_usize));
        // copy our stack frame to r_array_ptr
        context.mov_instruction(r_array_ptr, r_stack);
        context.foreign_call_instruction(
            "make_number_sequence".into(),
            &[ValueOrArray::MemoryAddress(r_input_size)],
            &[HeapValueType::Simple],
            &[ValueOrArray::HeapVector(HeapVector { pointer: r_stack, size: r_output_size })],
            &[HeapValueType::Vector { value_types: vec![HeapValueType::Simple] }],
        );
        // push stack frame by r_returned_size
        context.memory_op(r_stack, r_output_size, r_stack, BinaryIntOp::Add);
        // check r_input_size == r_output_size
        context.memory_op(r_input_size, r_output_size, r_equality, BinaryIntOp::Equals);
        // We push a JumpIf and Trap opcode directly as the constrain instruction
        // uses unresolved jumps which requires a block to be constructed in SSA and
        // we don't need this for Brillig IR tests
        context.push_opcode(BrilligOpcode::JumpIf { condition: r_equality, location: 8 });
        context.push_opcode(BrilligOpcode::Trap);

        context.stop_instruction();

        let bytecode = context.artifact().finish().byte_code;
        let number_sequence: Vec<Value> = (0_usize..12_usize).map(Value::from).collect();
        let mut vm = VM::new(
            vec![],
            &bytecode,
            vec![ForeignCallResult { values: vec![ForeignCallParam::Array(number_sequence)] }],
            &DummyBlackBoxSolver,
        );
        let status = vm.process_opcodes();
        assert_eq!(status, VMStatus::Finished { return_data_offset: 0, return_data_size: 0 });
    }
}
