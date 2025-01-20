use acvm::{acir::brillig::MemoryAddress, AcirField};

use crate::ssa::ir::function::FunctionId;

use super::{
    brillig_variable::{BrilligVariable, SingleAddrVariable},
    debug_show::DebugToString,
    registers::{RegisterAllocator, Stack},
    BrilligBinaryOp, BrilligContext, ReservedRegisters,
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    // impl<F: AcirField + DebugToString> BrilligContext<F, Stack> {
    pub(crate) fn codegen_call(
        &mut self,
        func_id: FunctionId,
        arguments: &[BrilligVariable],
        returns: &[BrilligVariable],
    ) {
        let stack_size_register = SingleAddrVariable::new_usize(self.allocate_register());
        let previous_stack_pointer = self.registers.empty_registers_start();
        let stack_size = previous_stack_pointer.unwrap_relative();
        // Write the stack size
        self.const_instruction(stack_size_register, stack_size.into());
        // Pass the previous stack pointer
        self.mov_instruction(previous_stack_pointer, ReservedRegisters::stack_pointer());
        // Pass the arguments
        let mut current_argument_location = stack_size + 1;
        for item in arguments {
            self.mov_instruction(
                MemoryAddress::relative(current_argument_location),
                item.extract_register(),
            );
            current_argument_location += 1;
        }
        // Increment the stack pointer
        self.memory_op_instruction(
            ReservedRegisters::stack_pointer(),
            stack_size_register.address,
            ReservedRegisters::stack_pointer(),
            BrilligBinaryOp::Add,
        );

        self.add_external_call_instruction(func_id);

        // Restore the stack pointer
        self.mov_instruction(ReservedRegisters::stack_pointer(), MemoryAddress::relative(0));

        // Move the return values back
        let mut current_return_location = stack_size + 1;
        for item in returns {
            self.mov_instruction(
                item.extract_register(),
                MemoryAddress::relative(current_return_location),
            );
            current_return_location += 1;
        }
        self.deallocate_single_addr(stack_size_register);
    }

    /// Codegens a return from the current function.
    pub(crate) fn codegen_return(&mut self, return_registers: &[MemoryAddress]) {
        let mut sources = Vec::with_capacity(return_registers.len());
        let mut destinations = Vec::with_capacity(return_registers.len());

        for (destination_index, return_register) in return_registers.iter().enumerate() {
            // In case we have fewer return registers than indices to write to, ensure we've allocated this register
            let destination_register = MemoryAddress::relative(Stack::start() + destination_index);
            self.registers.ensure_register_is_allocated(destination_register);
            sources.push(*return_register);
            destinations.push(destination_register);
        }
        destinations
            .iter()
            .for_each(|destination| self.registers.ensure_register_is_allocated(*destination));
        self.codegen_mov_registers_to_registers(sources, destinations);
        self.return_instruction();
    }
}
