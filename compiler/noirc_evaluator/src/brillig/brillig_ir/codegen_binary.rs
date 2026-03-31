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

    /// If the `count_array_copies` flag is set, registers this as a per-site copy location and
    /// emits runtime code that increments the per-site counter whenever a copy actually occurred
    /// (i.e. `source_pointer != dest_pointer` after a copy procedure call).
    pub(crate) fn codegen_count_if_copy_occurred(
        &mut self,
        source_pointer: MemoryAddress,
        dest_pointer: MemoryAddress,
    ) {
        use crate::brillig::MAX_TRACK_SITES;

        if !self.count_arrays_copied {
            return;
        }
        let Some(registry) = self.copy_site_registry.clone() else {
            return;
        };

        // Deduplicate by CallStackId: the same call site compiled more than once shares a counter.
        let call_stack_id = self.current_call_stack_id();
        let site_index = registry.register_site(call_stack_id);

        if site_index >= MAX_TRACK_SITES {
            return;
        }

        let counter_addr = self.per_site_counter_address(site_index);

        // Emit: if source_pointer != dest_pointer { counter_addr += 1 }
        // We use: did_not_copy = (source == dest); if did_not_copy => skip increment
        let did_not_copy = self.allocate_single_addr_bool();
        self.memory_op_instruction(
            source_pointer,
            dest_pointer,
            did_not_copy.address,
            BrilligBinaryOp::Equals,
        );
        self.codegen_if_not(did_not_copy.address, |ctx| {
            ctx.codegen_usize_op(counter_addr, counter_addr, BrilligBinaryOp::Add, 1);
        });
    }

    /// Like `codegen_count_if_copy_occurred` but driven by an explicit boolean flag register
    /// rather than a pointer comparison. Registers this as a per-site copy location and emits
    /// runtime code that increments the per-site counter when `flag != 0`.
    pub(crate) fn codegen_count_if_nonzero(&mut self, flag: MemoryAddress) {
        use crate::brillig::MAX_TRACK_SITES;

        if !self.count_arrays_copied {
            return;
        }
        let Some(registry) = self.copy_site_registry.clone() else {
            return;
        };

        let call_stack_id = self.current_call_stack_id();
        let site_index = registry.register_site(call_stack_id);

        if site_index >= MAX_TRACK_SITES {
            return;
        }

        let counter_addr = self.per_site_counter_address(site_index);

        // if flag != 0 { counter_addr += 1 }
        self.codegen_if(flag, |ctx| {
            ctx.codegen_usize_op(counter_addr, counter_addr, BrilligBinaryOp::Add, 1);
        });
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
