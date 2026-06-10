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

    /// Emit overflow check for addition: traps if `lhs > result` (i.e., overflow occurred).
    /// Assumes the addition `result = lhs + rhs` has already been performed.
    pub(crate) fn codegen_add_overflow_check(
        &mut self,
        lhs: SingleAddrVariable,
        result: SingleAddrVariable,
    ) {
        let no_overflow = self.allocate_single_addr_bool();
        // Check that lhs <= result (if overflow occurred, result wrapped and result < lhs)
        self.binary_instruction(lhs, result, *no_overflow, BrilligBinaryOp::LessThanEquals);
        self.codegen_constrain(*no_overflow, Some("attempt to add with overflow".to_string()));
    }

    /// Emit overflow check for multiplication: traps if `result / rhs != lhs` (i.e., overflow occurred).
    /// Assumes the multiplication `result = lhs * rhs` has already been performed.
    /// Skips check if `rhs == 0` to avoid division by zero.
    pub(crate) fn codegen_mul_overflow_check(
        &mut self,
        lhs: MemoryAddress,
        rhs: MemoryAddress,
        result: MemoryAddress,
    ) {
        let is_rhs_zero = self.allocate_single_addr_bool();
        self.codegen_usize_op(rhs, is_rhs_zero.address, BrilligBinaryOp::Equals, 0);

        self.codegen_if_not(is_rhs_zero.address, |ctx| {
            let quotient = ctx.allocate_single_addr_usize();
            ctx.memory_op_instruction(result, rhs, quotient.address, BrilligBinaryOp::UnsignedDiv);

            let no_overflow = ctx.allocate_single_addr_bool();
            ctx.memory_op_instruction(
                quotient.address,
                lhs,
                no_overflow.address,
                BrilligBinaryOp::Equals,
            );

            ctx.codegen_constrain(
                *no_overflow,
                Some("attempt to multiply with overflow".to_string()),
            );
        });
    }

    /// Checked usize addition: `destination = lhs + rhs`, traps on overflow.
    /// Uses 32-bit (usize) arithmetic for memory addressing operations.
    pub(crate) fn codegen_checked_add(
        &mut self,
        lhs: MemoryAddress,
        rhs: MemoryAddress,
        destination: MemoryAddress,
    ) {
        // The overflow check reads `lhs` back after the add, so it must survive being written.
        assert_ne!(destination, lhs, "codegen_checked_add: destination must not alias lhs");
        self.memory_op_instruction(lhs, rhs, destination, BrilligBinaryOp::Add);
        self.codegen_add_overflow_check(
            SingleAddrVariable::new_usize(lhs),
            SingleAddrVariable::new_usize(destination),
        );
    }

    /// Checked multiplication: `destination = lhs * rhs`, traps on overflow.
    pub(crate) fn codegen_checked_mul(
        &mut self,
        lhs: MemoryAddress,
        rhs: MemoryAddress,
        destination: MemoryAddress,
    ) {
        // The overflow check reads `lhs` and `rhs` back after the mul, so they must survive being written.
        assert_ne!(destination, lhs, "codegen_checked_mul: destination must not alias lhs");
        assert_ne!(destination, rhs, "codegen_checked_mul: destination must not alias rhs");
        self.memory_op_instruction(lhs, rhs, destination, BrilligBinaryOp::Mul);
        self.codegen_mul_overflow_check(lhs, rhs, destination);
    }

    /// Checked multiplication with a constant: `destination = operand * constant`, traps on overflow.
    pub(crate) fn codegen_checked_mul_with_constant(
        &mut self,
        operand: MemoryAddress,
        destination: MemoryAddress,
        constant: usize,
    ) {
        let const_register = self.make_usize_constant_instruction(F::from(constant));
        self.codegen_checked_mul(operand, const_register.address, destination);
    }
}

#[cfg(test)]
mod tests {
    use acvm::{FieldElement, acir::brillig::MemoryAddress};

    use crate::brillig::{
        BrilligOptions,
        brillig_ir::{BrilligContext, Stack, artifact::Label},
    };
    use crate::ssa::ir::function::FunctionId;

    fn create_context() -> BrilligContext<FieldElement, Stack> {
        let mut context = BrilligContext::new("test", &BrilligOptions::default());
        context.enter_context(Label::function(FunctionId::test_new(0)));
        context
    }

    #[test]
    #[should_panic(expected = "codegen_checked_add: destination must not alias lhs")]
    fn checked_add_rejects_destination_aliasing_operand() {
        let mut context = create_context();
        let lhs = MemoryAddress::relative(1);
        let rhs = MemoryAddress::relative(2);
        context.codegen_checked_add(lhs, rhs, lhs);
    }

    #[test]
    #[should_panic(expected = "codegen_checked_mul: destination must not alias rhs")]
    fn checked_mul_rejects_destination_aliasing_operand() {
        let mut context = create_context();
        let lhs = MemoryAddress::relative(1);
        let rhs = MemoryAddress::relative(2);
        context.codegen_checked_mul(lhs, rhs, rhs);
    }
}
