use acvm::{acir::brillig::MemoryAddress, AcirField};

use super::{
    debug_show::DebugToString, instructions::BrilligBinaryOp, registers::RegisterAllocator,
    BrilligContext, ReservedRegisters,
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Utility method to perform a binary instruction with a constant value in place
    pub(crate) fn codegen_usize_op_in_place(
        &mut self,
        destination: MemoryAddress,
        op: BrilligBinaryOp,
        constant: usize,
    ) {
        self.codegen_usize_op(destination, destination, op, constant);
    }

    /// Utility method to perform a binary instruction with a constant value
    pub(crate) fn codegen_usize_op(
        &mut self,
        operand: MemoryAddress,
        destination: MemoryAddress,
        op: BrilligBinaryOp,
        constant: usize,
    ) {
        if constant == 1 {
            self.memory_op_instruction(operand, ReservedRegisters::usize_one(), destination, op);
        } else {
            let const_register = self.make_usize_constant_instruction(F::from(constant));
            if constant == 3 {
                dbg!(const_register);
            }
            self.memory_op_instruction(operand, const_register.address, destination, op);
            // Mark as no longer used for this purpose, frees for reuse
            self.deallocate_single_addr(const_register);
        }
    }
}
