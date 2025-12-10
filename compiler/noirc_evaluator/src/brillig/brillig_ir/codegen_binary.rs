use acvm::{AcirField, acir::brillig::MemoryAddress};

use crate::brillig::brillig_ir::{brillig_variable::SingleAddrVariable, registers::Allocated};

use super::{
    BrilligContext, ReservedRegisters, debug_show::DebugToString, instructions::BrilligBinaryOp,
    registers::RegisterAllocator,
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
            self.memory_op_instruction(operand, const_register.address, destination, op);
        }
    }

    pub(crate) fn codegen_increment_array_copy_counter(&mut self) {
        let array_copy_counter = self.array_copy_counter_address();
        self.codegen_usize_op(array_copy_counter, array_copy_counter, BrilligBinaryOp::Add, 1);
    }

    /// Utility method to check if the value at a memory address equals one.
    pub(crate) fn codegen_usize_equals_one(
        &mut self,
        operand: SingleAddrVariable,
    ) -> Allocated<SingleAddrVariable, Registers> {
        let is_one = self.allocate_single_addr_bool();
        self.codegen_usize_op(operand.address, is_one.address, BrilligBinaryOp::Equals, 1);
        is_one
    }
}
