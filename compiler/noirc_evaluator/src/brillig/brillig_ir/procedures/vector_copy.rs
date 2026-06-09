use acvm::{AcirField, brillig_vm::offsets};

use super::ProcedureId;
use crate::brillig::{
    assert_usize,
    brillig_ir::{
        BrilligBinaryOp, BrilligContext,
        brillig_variable::BrilligVector,
        codegen_memory::RC_UNIQUE,
        debug_show::DebugToString,
        registers::{RegisterAllocator, ScratchSpace},
    },
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Copy arguments to [ScratchSpace] and call [ProcedureId::VectorCopy].
    ///
    /// Conditionally copies a source vector to a destination vector.
    /// If the reference count of the source vector is 1, then we can directly copy the pointer of the source vector to the destination vector.
    pub(crate) fn call_vector_copy_procedure(
        &mut self,
        source_vector: BrilligVector,
        destination_vector: BrilligVector,
    ) {
        let [source_vector_pointer_arg, destination_vector_pointer_return] =
            self.make_scratch_registers();

        self.mov_instruction(source_vector_pointer_arg, source_vector.pointer);

        self.add_procedure_call_instruction(ProcedureId::VectorCopy);

        self.mov_instruction(destination_vector.pointer, destination_vector_pointer_return);
    }
}

/// Compile [ProcedureId::VectorCopy].
pub(super) fn compile_vector_copy_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    let [source_vector_pointer_arg, destination_vector_pointer_return] =
        brillig_context.allocate_scratch_registers();

    let source_vector = BrilligVector { pointer: source_vector_pointer_arg };
    let target_vector = BrilligVector { pointer: destination_vector_pointer_return };

    // The reference-count slot is a "unique / shared" boolean, so we can branch on it
    // directly: unique (truthy) means we can mutate in place, shared (falsy) means copy.
    let rc = brillig_context.codegen_read_vector_rc(source_vector);

    brillig_context.codegen_branch(rc.address, |ctx, is_unique| {
        if is_unique {
            // Uniquely owned, we can mutate the vector directly; just set the destination to equal the source.
            ctx.mov_instruction(target_vector.pointer, source_vector.pointer);
        } else {
            // Allocate the memory for the new vector.
            let allocation_size = ctx.codegen_read_vector_capacity(source_vector);
            ctx.codegen_usize_op_in_place(
                allocation_size.address,
                BrilligBinaryOp::Add,
                assert_usize(offsets::VECTOR_META_COUNT),
            );
            ctx.codegen_allocate_mem(target_vector.pointer, allocation_size.address);

            // Copy the entire source vector, including metadata and items.
            ctx.codegen_mem_copy(source_vector.pointer, target_vector.pointer, *allocation_size);

            // The fresh copy is uniquely owned. The source stays marked as shared.
            ctx.codegen_initialize_rc(target_vector.pointer, RC_UNIQUE);

            // Increase our array copy counter if that flag is set
            if ctx.count_arrays_copied {
                ctx.codegen_increment_array_copy_counter();
            }
        }
    });
}
