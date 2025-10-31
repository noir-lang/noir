use acvm::{AcirField, acir::brillig::MemoryAddress};

use super::ProcedureId;
use crate::brillig::brillig_ir::{
    BrilligBinaryOp, BrilligContext,
    brillig_variable::{BrilligVector, SingleAddrVariable},
    debug_show::DebugToString,
    registers::{RegisterAllocator, ScratchSpace},
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Copy arguments to [ScratchSpace] and call [ProcedureId::VectorPopBack].
    ///
    /// Pops `item_pop_count` items from the back of a `source_vector`, returning the new `destination_vector`
    /// and the pointer to the popped items in `read_pointer`.
    ///
    /// The `item_pop_count` equals the number of variables to pop into in SSA.
    /// The `source_len` is the semantic (not flattened) length of the vector.
    pub(crate) fn call_vector_pop_back_procedure(
        &mut self,
        source_len: SingleAddrVariable,
        source_vector: BrilligVector,
        destination_vector: BrilligVector,
        read_pointer: MemoryAddress,
        item_pop_count: usize,
    ) {
        let [
            source_vector_length_arg,
            source_vector_pointer_arg,
            item_pop_count_arg,
            destination_vector_pointer_return,
            read_pointer_return,
        ] = self.make_scratch_registers();

        self.mov_instruction(source_vector_length_arg, source_len.address);
        self.mov_instruction(source_vector_pointer_arg, source_vector.pointer);
        self.usize_const_instruction(item_pop_count_arg, item_pop_count.into());

        self.add_procedure_call_instruction(ProcedureId::VectorPopBack);

        self.mov_instruction(destination_vector.pointer, destination_vector_pointer_return);
        self.mov_instruction(read_pointer, read_pointer_return);
    }
}

/// Compile [ProcedureId::VectorPopBack].
pub(super) fn compile_vector_pop_back_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    let [
        source_vector_length_arg,
        source_vector_pointer_arg,
        item_pop_count_arg,
        destination_vector_pointer_return,
        read_pointer_return,
    ] = brillig_context.allocate_scratch_registers();

    let source_vector = BrilligVector { pointer: source_vector_pointer_arg };
    let target_vector = BrilligVector { pointer: destination_vector_pointer_return };

    // First we need to allocate the target vector decrementing the size by `item_pop_count`.
    // We use the semantic length, rather than load the vector size from the meta-data,
    // because after merging vectors of different lengths, the semantic length can be different
    // then the number of items in the vector, which is the longer of the merged values.
    let source_size = brillig_context.allocate_single_addr_usize();
    brillig_context.codegen_vector_flattened_size(
        source_size.address,
        source_vector_length_arg,
        item_pop_count_arg,
    );

    // We will pop a number of (flattened) items; targets_size = source_size - item_pop_count
    let target_size = brillig_context.allocate_single_addr_usize();
    brillig_context.memory_op_instruction(
        source_size.address,
        item_pop_count_arg,
        target_size.address,
        BrilligBinaryOp::Sub,
    );

    let rc = brillig_context.codegen_read_vector_rc(source_vector);

    let is_rc_one = brillig_context.codegen_usize_equals_one(*rc);

    let source_vector_items_pointer =
        brillig_context.codegen_make_vector_items_pointer(source_vector);

    brillig_context.codegen_branch(is_rc_one.address, |brillig_context, is_rc_one| {
        if is_rc_one {
            // We can reuse the source vector, updating its length
            brillig_context.mov_instruction(target_vector.pointer, source_vector.pointer);
            brillig_context.codegen_update_vector_size(target_vector, *target_size);
        } else {
            // We need to clone the source vector; allocate memory for it.
            brillig_context.codegen_initialize_vector(target_vector, *target_size, None);

            let target_vector_items_pointer =
                brillig_context.codegen_make_vector_items_pointer(target_vector);

            // Now we copy the source vector starting at index 0 into the target vector but with the reduced length.
            brillig_context.codegen_mem_copy(
                *source_vector_items_pointer,
                *target_vector_items_pointer,
                *target_size,
            );

            // We don't decrease the RC of the source vector, otherwise repeatedly popping the same item
            // from the original (immutable) handle would bring its RC down to 1.
        }
    });

    // Finally set the pointer where the popped items can be read from to the source vector.
    brillig_context.memory_op_instruction(
        *source_vector_items_pointer,
        target_size.address,
        read_pointer_return,
        BrilligBinaryOp::Add,
    );
}
