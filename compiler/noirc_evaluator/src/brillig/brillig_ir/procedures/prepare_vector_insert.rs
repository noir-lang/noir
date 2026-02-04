use acvm::{AcirField, acir::brillig::MemoryAddress};

use super::{ProcedureId, prepare_vector_push::reallocate_vector_for_insertion};
use crate::brillig::brillig_ir::{
    BrilligBinaryOp, BrilligContext,
    brillig_variable::{BrilligVector, SingleAddrVariable},
    codegen_memory::VectorMetaData,
    debug_show::DebugToString,
    registers::{RegisterAllocator, ScratchSpace},
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Copy the arguments to the [ScratchSpace] and call [ProcedureId::PrepareVectorInsert].
    ///
    /// Prepares a the `source_vector` for a insert operation by making a copy to `destination_vector`,
    /// leaving an `item_count` hole at the `index` position, which is returned as the `write_pointer`.
    pub(crate) fn call_prepare_vector_insert_procedure(
        &mut self,
        source_vector: BrilligVector,
        destination_vector: BrilligVector,
        index: SingleAddrVariable,
        write_pointer: MemoryAddress,
        item_count: usize,
    ) {
        let [
            source_vector_pointer_arg,
            index_arg,
            item_count_arg,
            destination_vector_pointer_return,
            write_pointer_return,
        ] = self.make_scratch_registers();

        self.mov_instruction(source_vector_pointer_arg, source_vector.pointer);
        self.mov_instruction(index_arg, index.address);
        self.usize_const_instruction(item_count_arg, item_count.into());

        self.add_procedure_call_instruction(ProcedureId::PrepareVectorInsert);

        self.mov_instruction(destination_vector.pointer, destination_vector_pointer_return);
        self.mov_instruction(write_pointer, write_pointer_return);
    }
}

/// Compile [ProcedureId::PrepareVectorInsert].
pub(super) fn compile_prepare_vector_insert_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    let [
        source_vector_pointer_arg,
        index_arg,
        item_count_arg,
        destination_vector_pointer_return,
        write_pointer_return,
    ] = brillig_context.allocate_scratch_registers();

    let source_vector = BrilligVector { pointer: source_vector_pointer_arg };
    let target_vector = BrilligVector { pointer: destination_vector_pointer_return };
    let index = SingleAddrVariable::new_usize(index_arg);

    let VectorMetaData {
        rc: source_rc,
        size: source_size,
        capacity: source_capacity,
        items_pointer: source_items_pointer,
    } = brillig_context.codegen_read_vector_metadata(source_vector, None);

    // Target size is source size + item_count
    let target_size = brillig_context.allocate_single_addr_usize();
    brillig_context.memory_op_instruction(
        source_size.address,
        item_count_arg,
        target_size.address,
        BrilligBinaryOp::Add,
    );

    // Reallocate the target vector if necessary.
    reallocate_vector_for_insertion(
        brillig_context,
        source_vector,
        *source_rc,
        *source_capacity,
        target_vector,
        *target_size,
    );

    let target_vector_items_pointer =
        brillig_context.codegen_make_vector_items_pointer(target_vector);

    // Check if we were able to reuse the source vector by comparing with the destination address.
    let was_reused = brillig_context.allocate_single_addr_bool();
    brillig_context.memory_op_instruction(
        source_vector.pointer,
        target_vector.pointer,
        was_reused.address,
        BrilligBinaryOp::Equals,
    );

    // If we were unable to reuse the source index, we need to copy the items, up to the index where the hole needs to be:
    // target[0..index] = source[0..index]
    brillig_context.codegen_if_not(was_reused.address, |brillig_context| {
        // Copy the elements to the left of the index
        brillig_context.codegen_mem_copy(
            source_items_pointer.address,
            *target_vector_items_pointer,
            index,
        );
    });

    was_reused.deallocate();

    // Compute the source pointer just at the index
    let source_pointer_at_index = brillig_context.allocate_register();
    brillig_context.memory_op_instruction(
        source_items_pointer.address,
        index.address,
        *source_pointer_at_index,
        BrilligBinaryOp::Add,
    );

    // Compute the target pointer where the elements need to be inserted.
    brillig_context.memory_op_instruction(
        *target_vector_items_pointer,
        index.address,
        write_pointer_return,
        BrilligBinaryOp::Add,
    );

    // Compute the target pointer pointing beyond the elements that need to be inserted.
    let target_pointer_after_index = brillig_context.allocate_register();

    brillig_context.memory_op_instruction(
        write_pointer_return,
        item_count_arg,
        *target_pointer_after_index,
        BrilligBinaryOp::Add,
    );

    // Compute the number of elements to the right of the insertion index.
    // source_size = index + element_count_to_the_right
    let element_count_to_the_right = brillig_context.allocate_register();
    brillig_context.memory_op_instruction(
        source_size.address,
        index.address,
        *element_count_to_the_right,
        BrilligBinaryOp::Sub,
    );

    // Copy the elements to the right of the index:
    // target[index+item_count .. index+item_count+elem_count_to_the_right] = source[index .. index+elem_count_to_the_right]
    brillig_context.codegen_mem_copy_from_the_end(
        *source_pointer_at_index,
        *target_pointer_after_index,
        SingleAddrVariable::new_usize(*element_count_to_the_right),
    );
}
