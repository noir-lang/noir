use acvm::{AcirField, brillig_vm::offsets};

use super::ProcedureId;
use crate::brillig::brillig_ir::{
    BrilligBinaryOp, BrilligContext,
    brillig_variable::BrilligVector,
    debug_show::DebugToString,
    registers::{RegisterAllocator, ScratchSpace},
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

    let rc = brillig_context.codegen_read_vector_rc(source_vector);

    let is_rc_one = brillig_context.codegen_usize_equals_one(*rc);

    brillig_context.codegen_branch(is_rc_one.address, |ctx, cond| {
        if cond {
            // Reference count is 1, we can mutate the vector directly; just the the destination to equal the source.
            ctx.mov_instruction(target_vector.pointer, source_vector.pointer);
        } else {
            // Allocate the memory for the new vector.
            let allocation_size = ctx.codegen_read_vector_capacity(source_vector);
            ctx.codegen_usize_op_in_place(
                allocation_size.address,
                BrilligBinaryOp::Add,
                usize::try_from(offsets::VECTOR_META_COUNT)
                    .expect("Failed conversion from u32 to usize"),
            );
            ctx.codegen_allocate_mem(target_vector.pointer, allocation_size.address);

            // Copy the entire source vector, including metadata and items.
            ctx.codegen_mem_copy(source_vector.pointer, target_vector.pointer, *allocation_size);

            // Then reset the new RC to 1.
            ctx.codegen_initialize_rc(target_vector.pointer, 1);

            // Decrease the original ref count now that this copy is no longer pointing to it.
            // Copying a vector this way is an implicit side effect of setting an item by index through a mutable variable;
            // unlike with pop and push, we won't end up with a new vector handle, so we can split the RC between the old and the new.
            ctx.codegen_decrement_rc(source_vector.pointer, rc.address);
        }
    });
}
