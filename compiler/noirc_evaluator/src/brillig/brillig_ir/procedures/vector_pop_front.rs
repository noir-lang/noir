use acvm::AcirField;

use super::ProcedureId;
use crate::brillig::brillig_ir::{
    BrilligBinaryOp, BrilligContext,
    brillig_variable::{BrilligVector, SingleAddrVariable},
    codegen_memory::VectorMetaData,
    debug_show::DebugToString,
    registers::{RegisterAllocator, ScratchSpace},
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Copy arguments to [ScratchSpace] and call [ProcedureId::VectorPopFront].
    ///
    /// Pops items from the front of a vector, returning the new vector.
    /// The procedure assumes that there are constraints to prevent 0 size vectors from being popped.
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
            destination_vector_pointer_return,
        ] = self.make_scratch_registers();

        self.mov_instruction(source_vector_length_arg, source_len.address);
        self.mov_instruction(source_vector_pointer_arg, source_vector.pointer);
        self.usize_const_instruction(item_pop_count_arg, item_pop_count.into());

        self.add_procedure_call_instruction(ProcedureId::VectorPopFront);

        self.mov_instruction(destination_vector.pointer, destination_vector_pointer_return);
    }
}

/// Compile [ProcedureId::VectorPopFront].
pub(super) fn compile_vector_pop_front_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    let [
        source_vector_length_arg,
        source_vector_pointer_arg,
        item_pop_count_arg,
        destination_vector_pointer_return,
    ] = brillig_context.allocate_scratch_registers();

    let source_vector = BrilligVector { pointer: source_vector_pointer_arg };
    let target_vector = BrilligVector { pointer: destination_vector_pointer_return };

    let VectorMetaData { size: source_size, items_pointer: source_items_pointer, .. } =
        brillig_context.codegen_read_vector_metadata(
            source_vector,
            Some((source_vector_length_arg, item_pop_count_arg)),
        );

    // target_size = source_size - item_pop_count
    // Assumes constraints exist against underflow.
    // We don't have to worry about the semantic length of merged vectors here, because we are popping from the front.
    let target_size = brillig_context.allocate_single_addr_usize();
    brillig_context.memory_op_instruction(
        source_size.address,
        item_pop_count_arg,
        target_size.address,
        BrilligBinaryOp::Sub,
    );

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
    // Now we copy the source vector into the target vector, starting at index after the popped items.
    brillig_context.codegen_mem_copy(
        *source_copy_pointer,
        *target_vector_items_pointer,
        *target_size,
    );
    // We don't decrease the RC of the source vector, otherwise repeatedly popping the same item
    // from the original (immutable) handle would bring its RC down to 1.
}
