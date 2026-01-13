use acvm::AcirField;

use super::ProcedureId;
use crate::brillig::brillig_ir::{
    BrilligBinaryOp, BrilligContext,
    brillig_variable::{BrilligVector, SingleAddrVariable},
    debug_show::DebugToString,
    registers::{RegisterAllocator, ScratchSpace},
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Copy arguments to [ScratchSpace] and call [ProcedureId::VectorRemove].
    ///
    /// Removes `item_count` items from the `source_vector` at `index`, returning the new `destination_vector`.
    /// Modifies the `source_vector` if the reference count is 1.
    ///
    /// The procedure assumes that we have constraints on the index being within the semantic length of the vector.
    pub(crate) fn call_vector_remove_procedure(
        &mut self,
        source_vector: BrilligVector,
        destination_vector: BrilligVector,
        index: SingleAddrVariable,
        item_count: usize,
    ) {
        let [
            source_vector_pointer_arg,
            index_arg,
            item_count_arg,
            destination_vector_pointer_return,
        ] = self.make_scratch_registers();

        self.mov_instruction(source_vector_pointer_arg, source_vector.pointer);
        self.mov_instruction(index_arg, index.address);
        self.usize_const_instruction(item_count_arg, item_count.into());

        self.add_procedure_call_instruction(ProcedureId::VectorRemove);

        self.mov_instruction(destination_vector.pointer, destination_vector_pointer_return);
    }
}

/// Compile [ProcedureId::VectorRemove].
pub(super) fn compile_vector_remove_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    let [source_vector_pointer_arg, index_arg, item_count_arg, destination_vector_pointer_return] =
        brillig_context.allocate_scratch_registers();

    let source_vector = BrilligVector { pointer: source_vector_pointer_arg };
    let target_vector = BrilligVector { pointer: destination_vector_pointer_return };
    let index = SingleAddrVariable::new_usize(index_arg);

    // Reallocate if necessary
    let source_size = brillig_context.codegen_read_vector_size(source_vector);

    // We don't have to worry about the semantic length of merged vectors here, because we are removing an item,
    // rather than appending it, so it doesn't matter if the semantic length was shorter than the vector size.
    let target_size = brillig_context.allocate_single_addr_usize();
    brillig_context.memory_op_instruction(
        source_size.address,
        item_count_arg,
        target_size.address,
        BrilligBinaryOp::Sub,
    );

    let rc = brillig_context.codegen_read_vector_rc(source_vector);

    let is_rc_one = brillig_context.codegen_usize_equals_one(*rc);

    let source_vector_items_pointer =
        brillig_context.codegen_make_vector_items_pointer(source_vector);

    // The items pointer in either the source vector of a new one, depending on reuse.
    let target_vector_items_pointer = brillig_context.allocate_register();

    // Set up the target vector up to the index.
    brillig_context.codegen_branch(is_rc_one.address, |brillig_context, is_rc_one| {
        if is_rc_one {
            // We can reuse the source vector: update its length and set the items pointer to be the source.
            brillig_context.mov_instruction(target_vector.pointer, source_vector.pointer);
            brillig_context.codegen_update_vector_size(target_vector, *target_size);
            brillig_context
                .codegen_vector_items_pointer(target_vector, *target_vector_items_pointer);
        } else {
            // We need to copy the vector; allocate a new one with the target size.
            brillig_context.codegen_initialize_vector(target_vector, *target_size, None);

            // Get the items pointer for the new vector.
            brillig_context
                .codegen_vector_items_pointer(target_vector, *target_vector_items_pointer);

            // Copy the elements to the left of the index.
            brillig_context.codegen_mem_copy(
                *source_vector_items_pointer,
                *target_vector_items_pointer,
                index,
            );

            // We don't modify the RC of the original, otherwise removing items repeatedly
            // from the original (immutable) handle could bring its RC down to 1.
        }
    });

    // Compute the source pointer after the removed items: source_after = source + index + item_count.
    let source_pointer_after_index = brillig_context.allocate_register();
    brillig_context.memory_op_instruction(
        *source_vector_items_pointer,
        index.address,
        *source_pointer_after_index,
        BrilligBinaryOp::Add,
    );
    brillig_context.memory_op_instruction(
        *source_pointer_after_index,
        item_count_arg,
        *source_pointer_after_index,
        BrilligBinaryOp::Add,
    );

    // Compute the target pointer at the index: target_at = target + index
    let target_pointer_at_index = brillig_context.allocate_register();
    brillig_context.memory_op_instruction(
        *target_vector_items_pointer,
        index.address,
        *target_pointer_at_index,
        BrilligBinaryOp::Add,
    );

    // Compute the number of elements to the right of the index: item_count_right = source_size - index - item_count.
    let item_count_right = brillig_context.allocate_register();
    brillig_context.memory_op_instruction(
        source_size.address,
        index.address,
        *item_count_right,
        BrilligBinaryOp::Sub,
    );
    brillig_context.memory_op_instruction(
        *item_count_right,
        item_count_arg,
        *item_count_right,
        BrilligBinaryOp::Sub,
    );

    // Copy the elements to the right of the index
    brillig_context.codegen_mem_copy(
        *source_pointer_after_index,
        *target_pointer_at_index,
        SingleAddrVariable::new_usize(*item_count_right),
    );
}
