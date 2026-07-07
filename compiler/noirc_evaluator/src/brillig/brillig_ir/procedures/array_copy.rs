use acvm::{AcirField, brillig_vm::offsets};

use super::ProcedureId;
use crate::brillig::brillig_ir::{
    BrilligContext,
    brillig_variable::{BrilligArray, SingleAddrVariable},
    debug_show::DebugToString,
    registers::{RegisterAllocator, ScratchSpace},
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Call [`ProcedureId::ArrayCopy`].
    ///
    /// Conditionally copies a source array to a destination array.
    /// If the reference count of the source array is 1, then we can directly copy the pointer of the source array to the destination array.
    /// Otherwise a copy is made, and the ref-count of the original is decreased by 1.
    pub(crate) fn call_array_copy_procedure(
        &mut self,
        source_array: BrilligArray,
        destination_array: BrilligArray,
    ) {
        debug_assert_eq!(
            source_array.size, destination_array.size,
            "ICE: source and destination arrays in copy must have the same size, but got {} and {}",
            source_array.size, destination_array.size
        );
        let [
            source_array_pointer_arg,
            source_array_memory_size_arg,
            destination_array_pointer_return,
        ] = self.make_scratch_registers();

        self.mov_instruction(source_array_pointer_arg, source_array.pointer);
        self.usize_const_instruction(
            source_array_memory_size_arg,
            (source_array.size.0 + offsets::ARRAY_META_COUNT).into(),
        );

        self.add_procedure_call_instruction(ProcedureId::ArrayCopy);

        self.mov_instruction(destination_array.pointer, destination_array_pointer_return);

        self.codegen_count_if_copy_occurred(source_array.pointer, destination_array.pointer);
    }
}

/// Compile [`ProcedureId::ArrayCopy`].
pub(super) fn compile_array_copy_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
) {
    let [source_array_pointer_arg, source_array_memory_size_arg, destination_array_pointer_return] =
        brillig_context.allocate_scratch_registers();

    let rc = brillig_context.codegen_read_rc(source_array_pointer_arg);

    let is_rc_one = brillig_context.codegen_usize_equals_one(*rc);

    brillig_context.codegen_branch(is_rc_one.address, |ctx, cond| {
        if cond {
            // Reference count is 1, we can mutate the array directly
            ctx.mov_instruction(destination_array_pointer_return, source_array_pointer_arg);
        } else {
            // We need to copy the array; allocate the required space on the heap.
            ctx.codegen_allocate_mem(
                destination_array_pointer_return,
                source_array_memory_size_arg,
            );

            // First issue an array copy to the destination.
            // This copies the whole data structure, including metadata.
            ctx.codegen_mem_copy(
                source_array_pointer_arg,
                destination_array_pointer_return,
                SingleAddrVariable::new_usize(source_array_memory_size_arg),
            );
            // Then set the new RC to 1.
            ctx.codegen_initialize_rc(destination_array_pointer_return, 1);

            // Decrease the original ref count now that this copy is no longer pointing to it.
            // Copying an array is a potential implicit side effect of setting an item by index through a mutable variable;
            // we won't end up with two handles to the array, so we can split the RC between the old and the new.
            ctx.codegen_decrement_rc(source_array_pointer_arg, rc.address);

            // Increase our array copy counter if that flag is set
            if ctx.count_array_copies() {
                ctx.codegen_increment_array_copy_counter();
            }
        }
    });
}
