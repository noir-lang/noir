use std::vec;

use acvm::{AcirField, acir::brillig::MemoryAddress};

use super::ProcedureId;
use crate::{
    brillig::brillig_ir::{
        BrilligBinaryOp, BrilligContext,
        brillig_variable::{BrilligVector, SingleAddrVariable},
        debug_show::DebugToString,
        registers::{RegisterAllocator, ScratchSpace},
    },
    set_allocated_registers,
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Pops items from the back of a vector, returning the new vector and the pointer to the popped items in read_pointer.
    pub(crate) fn call_vector_pop_back_procedure(
        &mut self,
        source_len: SingleAddrVariable,
        source_vector: BrilligVector,
        destination_vector: BrilligVector,
        read_pointer: MemoryAddress,
        item_pop_count: usize,
    ) {
        let scratch_start = ScratchSpace::start();
        let source_vector_length_arg = MemoryAddress::direct(scratch_start);
        let source_vector_pointer_arg = MemoryAddress::direct(scratch_start + 1);
        let item_pop_count_arg = MemoryAddress::direct(scratch_start + 2);
        let new_vector_pointer_return = MemoryAddress::direct(scratch_start + 3);
        let read_pointer_return = MemoryAddress::direct(scratch_start + 4);

        self.mov_instruction(source_vector_length_arg, source_len.address);
        self.mov_instruction(source_vector_pointer_arg, source_vector.pointer);
        self.usize_const_instruction(item_pop_count_arg, item_pop_count.into());

        self.add_procedure_call_instruction(ProcedureId::VectorPopBack);

        self.mov_instruction(destination_vector.pointer, new_vector_pointer_return);
        self.mov_instruction(read_pointer, read_pointer_return);
    }
}

pub(super) fn compile_vector_pop_back_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    let scratch_start = brillig_context.registers().start();

    set_allocated_registers!(brillig_context, {
        let source_vector_length_arg = MemoryAddress::direct(scratch_start);
        let source_vector_pointer_arg = MemoryAddress::direct(scratch_start + 1);
        let item_pop_count_arg = MemoryAddress::direct(scratch_start + 2);
        let new_vector_pointer_return = MemoryAddress::direct(scratch_start + 3);
        let read_pointer_return = MemoryAddress::direct(scratch_start + 4);
    });

    let source_vector = BrilligVector { pointer: source_vector_pointer_arg };
    let target_vector = BrilligVector { pointer: new_vector_pointer_return };

    // First we need to allocate the target vector decrementing the size by removed_items.len()
    // We use the semantic length, rather than load the vector size from the meta-data.
    let source_size = brillig_context.allocate_single_addr_usize();
    brillig_context.codegen_vector_flattened_size(
        source_size.address,
        source_vector_length_arg,
        item_pop_count_arg,
    );

    let target_size = brillig_context.allocate_single_addr_usize();
    brillig_context.memory_op_instruction(
        source_size.address,
        item_pop_count_arg,
        target_size.address,
        BrilligBinaryOp::Sub,
    );

    let rc = brillig_context.allocate_register();
    brillig_context.load_instruction(*rc, source_vector.pointer);

    let is_rc_one = brillig_context.allocate_register();
    brillig_context.codegen_usize_op(*rc, *is_rc_one, BrilligBinaryOp::Equals, 1_usize);

    let source_vector_items_pointer =
        brillig_context.codegen_make_vector_items_pointer(source_vector);

    brillig_context.codegen_branch(*is_rc_one, |brillig_context, is_rc_one| {
        if is_rc_one {
            // We can reuse the source vector updating its length
            brillig_context.mov_instruction(target_vector.pointer, source_vector.pointer);
            brillig_context.codegen_update_vector_length(target_vector, *target_size);
        } else {
            // We need to clone the source vector
            brillig_context.codegen_initialize_vector(target_vector, *target_size, None);

            let target_vector_items_pointer =
                brillig_context.codegen_make_vector_items_pointer(target_vector);

            // Now we copy the source vector starting at index 0 into the target vector but with the reduced length
            brillig_context.codegen_mem_copy(
                *source_vector_items_pointer,
                *target_vector_items_pointer,
                *target_size,
            );
        }
    });

    brillig_context.memory_op_instruction(
        *source_vector_items_pointer,
        target_size.address,
        read_pointer_return,
        BrilligBinaryOp::Add,
    );
}
