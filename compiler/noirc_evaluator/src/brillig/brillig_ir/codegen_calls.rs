use acvm::acir::brillig::MemoryAddress;

use super::{
    brillig_variable::BrilligVariable, BrilligBinaryOp, BrilligContext, ReservedRegisters,
};

impl BrilligContext {
    /// Saves all of the registers that have been used up until this point.
    fn codegen_save_registers_of_vars(&mut self, vars: &[BrilligVariable]) -> Vec<MemoryAddress> {
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
            self.store_instruction(ReservedRegisters::free_memory_pointer(), *register);
            // Add one to our stack pointer
            self.codegen_usize_op_in_place(
                ReservedRegisters::free_memory_pointer(),
                BrilligBinaryOp::Add,
                1,
            );
        }

        // Store the location of our registers in the previous stack pointer
        self.mov_instruction(
            ReservedRegisters::previous_stack_pointer(),
            ReservedRegisters::free_memory_pointer(),
        );
        used_registers
    }

    /// Loads all of the registers that have been save by save_all_used_registers.
    fn codegen_load_all_saved_registers(&mut self, used_registers: &[MemoryAddress]) {
        // Load all of the used registers that we saved.
        // We do all the reverse operations of save_all_used_registers.
        // Iterate our registers in reverse
        let iterator_register = self.allocate_register();
        self.mov_instruction(iterator_register, ReservedRegisters::previous_stack_pointer());

        for register in used_registers.iter().rev() {
            // Subtract one from our stack pointer
            self.codegen_usize_op_in_place(iterator_register, BrilligBinaryOp::Sub, 1);
            self.load_instruction(*register, iterator_register);
        }
    }

    // Used before a call instruction.
    // Save all the registers we have used to the stack.
    // Move argument values to the front of the register indices.
    pub(crate) fn codegen_pre_call_save_registers_prep_args(
        &mut self,
        arguments: &[MemoryAddress],
        variables_to_save: &[BrilligVariable],
    ) -> Vec<MemoryAddress> {
        // Save all the registers we have used to the stack.
        let saved_registers = self.codegen_save_registers_of_vars(variables_to_save);

        // Move argument values to the front of the registers
        //
        // This means that the arguments will be in the first `n` registers after
        // the number of reserved registers.
        let (sources, destinations): (Vec<_>, Vec<_>) =
            arguments.iter().enumerate().map(|(i, argument)| (*argument, self.register(i))).unzip();
        destinations
            .iter()
            .for_each(|destination| self.registers.ensure_register_is_allocated(*destination));
        self.codegen_mov_registers_to_registers(sources, destinations);
        saved_registers
    }

    // Used after a call instruction.
    // Move return values to the front of the register indices.
    // Load all the registers we have previous saved in save_registers_prep_args.
    pub(crate) fn codegen_post_call_prep_returns_load_registers(
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
        self.codegen_mov_registers_to_registers(sources, destinations);

        // Restore all the same registers we have, in exact reverse order.
        // Note that we have allocated some registers above, which we will not be handling here,
        // only restoring registers that were used prior to the call finishing.
        // After the call instruction, the stack frame pointer should be back to where we left off,
        // so we do our instructions in reverse order.
        self.codegen_load_all_saved_registers(saved_registers);
    }
}
