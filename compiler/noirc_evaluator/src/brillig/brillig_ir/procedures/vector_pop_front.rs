use acvm::AcirField;

use super::ProcedureId;
use crate::brillig::brillig_ir::{
    BrilligBinaryOp, BrilligContext,
    brillig_variable::{BrilligVector, SingleAddrVariable},
    debug_show::DebugToString,
    registers::{RegisterAllocator, ScratchSpace},
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Pops items from the front of a vector, returning the new vector
    pub(crate) fn call_vector_pop_front_procedure(
        &mut self,
        source_len: SingleAddrVariable,
        source_vector: BrilligVector,
        destination_vector: BrilligVector,
        item_pop_count: usize,
    ) {
        let [
            source_vector_length_arg,
            source_vector_pointer_arg,
            item_pop_count_arg,
            new_vector_pointer_return,
        ] = self.make_scratch_registers();

        self.mov_instruction(source_vector_length_arg, source_len.address);
        self.mov_instruction(source_vector_pointer_arg, source_vector.pointer);
        self.usize_const_instruction(item_pop_count_arg, item_pop_count.into());

        self.add_procedure_call_instruction(ProcedureId::VectorPopFront);

        self.mov_instruction(destination_vector.pointer, new_vector_pointer_return);
    }
}

pub(super) fn compile_vector_pop_front_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    let [
        source_vector_length_arg,
        source_vector_pointer_arg,
        item_pop_count_arg,
        new_vector_pointer_return,
    ] = brillig_context.allocate_scratch_registers();

    let source_vector = BrilligVector { pointer: source_vector_pointer_arg };
    let target_vector = BrilligVector { pointer: new_vector_pointer_return };

    let source_rc = brillig_context.allocate_single_addr_usize();
    let source_size = brillig_context.allocate_single_addr_usize();
    let source_capacity = brillig_context.allocate_single_addr_usize();
    let source_items_pointer = brillig_context.allocate_single_addr_usize();
    brillig_context.codegen_read_vector_metadata(
        source_vector,
        *source_rc,
        *source_size,
        *source_capacity,
        *source_items_pointer,
        Some((source_vector_length_arg, item_pop_count_arg)),
    );

    // target_size = source_size - item_pop_count
    let target_size = brillig_context.allocate_single_addr_usize();
    brillig_context.memory_op_instruction(
        source_size.address,
        item_pop_count_arg,
        target_size.address,
        BrilligBinaryOp::Sub,
    );

    let is_rc_one = brillig_context.allocate_register();
    brillig_context.codegen_usize_op(
        source_rc.address,
        *is_rc_one,
        BrilligBinaryOp::Equals,
        1_usize,
    );

    brillig_context.codegen_branch(*is_rc_one, |brillig_context, is_rc_one| {
        if is_rc_one {
            // We reuse the source vector, moving the metadata to the right (decreasing capacity)
            // Set the target vector pointer to be the source plus the number of popped items.
            brillig_context.memory_op_instruction(
                source_vector.pointer,
                item_pop_count_arg,
                target_vector.pointer,
                BrilligBinaryOp::Add,
            );
            // Decrease the source/target capacity by the number of popped items.
            brillig_context.memory_op_instruction(
                source_capacity.address,
                item_pop_count_arg,
                source_capacity.address,
                BrilligBinaryOp::Sub,
            );
            // Re-initialize the metadata at the shifted position.
            brillig_context.codegen_initialize_vector_metadata(
                target_vector,
                *target_size,
                *source_capacity,
            );
        } else {
            // We can't reuse the source vector, so allocate a new one.
            brillig_context.codegen_initialize_vector(target_vector, *target_size, None);

            let target_vector_items_pointer =
                brillig_context.codegen_make_vector_items_pointer(target_vector);

            // Set the source pointer to copy from to the source items start plus the number of popped items.
            let source_copy_pointer = brillig_context.allocate_register();
            brillig_context.memory_op_instruction(
                source_items_pointer.address,
                item_pop_count_arg,
                *source_copy_pointer,
                BrilligBinaryOp::Add,
            );
            // Now we copy the source vector starting at index removed_items.len() into the target vector
            brillig_context.codegen_mem_copy(
                *source_copy_pointer,
                *target_vector_items_pointer,
                *target_size,
            );
        }
    });
}
