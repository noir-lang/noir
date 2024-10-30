use std::vec;

use acvm::{acir::brillig::MemoryAddress, AcirField};

use super::ProcedureId;
use crate::brillig::brillig_ir::{
    brillig_variable::{BrilligVector, SingleAddrVariable},
    debug_show::DebugToString,
    registers::{RegisterAllocator, ScratchSpace},
    BrilligBinaryOp, BrilligContext, BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Conditionally copies a source array to a destination array.
    /// If the reference count of the source array is 1, then we can directly copy the pointer of the source array to the destination array.
    pub(crate) fn call_vector_copy_procedure(
        &mut self,
        source_vector: BrilligVector,
        destination_vector: BrilligVector,
    ) {
        let source_vector_pointer_arg = MemoryAddress::direct(ScratchSpace::start());
        let new_vector_pointer_return = MemoryAddress::direct(ScratchSpace::start() + 1);

        self.mov_instruction(source_vector_pointer_arg, source_vector.pointer);

        self.add_procedure_call_instruction(ProcedureId::VectorCopy);

        self.mov_instruction(destination_vector.pointer, new_vector_pointer_return);
    }
}

pub(super) fn compile_vector_copy_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    let source_vector_pointer_arg = MemoryAddress::direct(ScratchSpace::start());
    let new_vector_pointer_return = MemoryAddress::direct(ScratchSpace::start() + 1);

    brillig_context
        .set_allocated_registers(vec![source_vector_pointer_arg, new_vector_pointer_return]);

    let rc = SingleAddrVariable::new_usize(brillig_context.allocate_register());
    brillig_context.load_instruction(rc.address, source_vector_pointer_arg);

    let is_rc_one = SingleAddrVariable::new(brillig_context.allocate_register(), 1);
    brillig_context.codegen_usize_op(rc.address, is_rc_one.address, BrilligBinaryOp::Equals, 1);

    brillig_context.codegen_branch(is_rc_one.address, |ctx, cond| {
        if cond {
            // Reference count is 1, we can mutate the array directly
            ctx.mov_instruction(new_vector_pointer_return, source_vector_pointer_arg);
        } else {
            let source_vector = BrilligVector { pointer: source_vector_pointer_arg };
            let result_vector = BrilligVector { pointer: new_vector_pointer_return };

            // Allocate the memory for the new vec
            let allocation_size = ctx.codegen_make_vector_capacity(source_vector);
            ctx.codegen_usize_op_in_place(allocation_size.address, BrilligBinaryOp::Add, 3_usize); // Capacity plus 3 (rc, len, cap)
            ctx.codegen_allocate_mem(result_vector.pointer, allocation_size.address);

            ctx.codegen_mem_copy(source_vector.pointer, result_vector.pointer, allocation_size);
            // Then set the new rc to 1
            ctx.indirect_const_instruction(
                result_vector.pointer,
                BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
                1_usize.into(),
            );
            ctx.deallocate_single_addr(allocation_size);
        }
    });
}
