use std::vec;

use acvm::{acir::brillig::MemoryAddress, AcirField};

use super::ProcedureId;
use crate::brillig::brillig_ir::{
    brillig_variable::{BrilligArray, SingleAddrVariable},
    debug_show::DebugToString,
    registers::{RegisterAllocator, ScratchSpace},
    BrilligBinaryOp, BrilligContext, BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Conditionally copies a source array to a destination array.
    /// If the reference count of the source array is 1, then we can directly copy the pointer of the source array to the destination array.
    pub(crate) fn call_array_copy_procedure(
        &mut self,
        source_array: BrilligArray,
        destination_array: BrilligArray,
    ) {
        let source_array_pointer_arg = MemoryAddress::direct(ScratchSpace::start());
        let source_array_memory_size_arg = MemoryAddress::direct(ScratchSpace::start() + 1);
        let new_array_pointer_return = MemoryAddress::direct(ScratchSpace::start() + 2);

        self.mov_instruction(source_array_pointer_arg, source_array.pointer);
        self.usize_const_instruction(source_array_memory_size_arg, (source_array.size + 1).into());

        self.add_procedure_call_instruction(ProcedureId::ArrayCopy);

        self.mov_instruction(destination_array.pointer, new_array_pointer_return);
    }
}

pub(super) fn compile_array_copy_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    let source_array_pointer_arg = MemoryAddress::direct(ScratchSpace::start());
    let source_array_memory_size_arg = MemoryAddress::direct(ScratchSpace::start() + 1);
    let new_array_pointer_return = MemoryAddress::direct(ScratchSpace::start() + 2);

    brillig_context.set_allocated_registers(vec![
        source_array_pointer_arg,
        source_array_memory_size_arg,
        new_array_pointer_return,
    ]);

    let rc = SingleAddrVariable::new_usize(brillig_context.allocate_register());
    brillig_context.load_instruction(rc.address, source_array_pointer_arg);

    let is_rc_one = SingleAddrVariable::new(brillig_context.allocate_register(), 1);
    brillig_context.codegen_usize_op(rc.address, is_rc_one.address, BrilligBinaryOp::Equals, 1);

    brillig_context.codegen_branch(is_rc_one.address, |ctx, cond| {
        if cond {
            // Reference count is 1, we can mutate the array directly
            ctx.mov_instruction(new_array_pointer_return, source_array_pointer_arg);
        } else {
            // First issue a array copy to the destination
            ctx.codegen_allocate_mem(new_array_pointer_return, source_array_memory_size_arg);

            ctx.codegen_mem_copy(
                source_array_pointer_arg,
                new_array_pointer_return,
                SingleAddrVariable::new_usize(source_array_memory_size_arg),
            );
            // Then set the new rc to 1
            ctx.indirect_const_instruction(
                new_array_pointer_return,
                BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
                1_usize.into(),
            );
            // Decrease the original ref count now that this copy is no longer pointing to it
            ctx.codegen_usize_op(rc.address, rc.address, BrilligBinaryOp::Sub, 1);
        }
    });
}
