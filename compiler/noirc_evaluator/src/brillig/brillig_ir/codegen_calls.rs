use acvm::{AcirField, acir::brillig::MemoryAddress};

use crate::{brillig::brillig_ir::assert_u32, ssa::ir::function::FunctionId};

use super::{
    BrilligBinaryOp, BrilligContext, ReservedRegisters,
    brillig_variable::BrilligVariable,
    debug_show::DebugToString,
    registers::{RegisterAllocator, Stack},
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Generate Brillig opcodes to:
    /// * calculate the current stack size
    /// * copy the current stack pointer and the call arguments into a new stack frame
    /// * update the stack pointer and execute the call
    /// * restore the stack pointer and copy the results into the return variables
    pub(crate) fn codegen_call(
        &mut self,
        func_id: FunctionId,
        arguments: &[BrilligVariable],
        returns: &[BrilligVariable],
    ) {
        // Allocate a register for the stack size. With this allocation, we have our current stack before the call.
        let stack_size_register = self.allocate_single_addr_usize();

        // Find the start of free stack memory: this is our current stack size.
        let previous_stack_pointer = self.registers().empty_registers_start();
        let stack_size = previous_stack_pointer.unwrap_relative();

        // Write the current stack size to a register, so we can add it to the stack pointer.
        self.const_instruction(*stack_size_register, stack_size.into());

        // Copy the current stack pointer into the 0th slot of the next stack frame.
        // This is the previous stack pointer to return to after the call.
        self.mov_instruction(previous_stack_pointer, ReservedRegisters::stack_pointer());

        // Pass the arguments in the 1st, 2nd, ... slots of the stack.
        let mut current_argument_location = stack_size + 1;
        for item in arguments {
            // Here we are still using addresses relative to the current stack pointer.
            self.mov_instruction(
                MemoryAddress::relative(current_argument_location),
                item.extract_register(),
            );
            current_argument_location += 1;
        }

        // Increment the stack pointer for the call: stack_pointer := stack_pointer + stack_size.
        // By increasing it with the stack size before arguments where copied, it will include the
        // arguments at the intended relative addresses.
        self.memory_op_instruction(
            ReservedRegisters::stack_pointer(),
            stack_size_register.address,
            ReservedRegisters::stack_pointer(),
            BrilligBinaryOp::Add,
        );

        self.add_external_call_instruction(func_id);

        // Restore the previous stack pointer, which was copied into the 0th slot.
        self.mov_instruction(ReservedRegisters::stack_pointer(), MemoryAddress::relative(0));

        // Move the return values back. The return values are expected to overwrite the args.
        let mut current_return_location = stack_size + 1;
        for item in returns {
            self.mov_instruction(
                item.extract_register(),
                MemoryAddress::relative(current_return_location),
            );
            current_return_location += 1;
        }
    }

    /// Codegens a return from the current function.
    ///
    /// Takes the variables with the addresses of the values that need to be returned.
    /// The values are copied to the beginning of the stack space, into an equal number of slots.
    ///
    /// Any potential overlap between the source of the return variables and the final destination
    /// on the beginning of the stack is handled by [Self::codegen_mov_registers_to_registers].
    pub(crate) fn codegen_return(&mut self, return_variables: &[BrilligVariable]) {
        let mut sources = Vec::with_capacity(return_variables.len());
        let mut destinations = Vec::with_capacity(return_variables.len());

        for (destination_index, return_variable) in return_variables.iter().enumerate() {
            // In case we have fewer return registers than indices to write to, ensure we've allocated this register.
            let destination_register =
                MemoryAddress::relative(assert_u32(Stack::start() + destination_index));
            self.registers_mut().ensure_register_is_allocated(destination_register);
            destinations.push(destination_register);
            sources.push(return_variable.extract_register());
        }
        self.codegen_mov_registers_to_registers(&sources, &destinations);
        self.return_instruction();
    }
}
