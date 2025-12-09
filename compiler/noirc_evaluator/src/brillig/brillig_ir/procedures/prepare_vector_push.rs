use acvm::{AcirField, acir::brillig::MemoryAddress};

use super::ProcedureId;
use crate::brillig::brillig_ir::{
    BrilligBinaryOp, BrilligContext,
    brillig_variable::{BrilligVector, SingleAddrVariable},
    codegen_memory::VectorMetaData,
    debug_show::DebugToString,
    registers::{RegisterAllocator, ScratchSpace},
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Copy arguments to the [ScratchSpace] and call [ProcedureId::PrepareVectorPush].
    ///
    /// Prepares a vector for a push operation, allocating a larger vector and copying the source vector into the destination vector if necessary.
    /// It returns the destination vector and the write pointer to where to put the new items.
    ///
    /// Brillig vectors have an inherent size and capacity, but their semantic length can be different,
    /// and it's passed as a separate variable in SSA. When ACIR flattens vectors of unequal length,
    /// it can be different from the size and capacity in the vector data structure itself.
    pub(crate) fn call_prepare_vector_push_procedure(
        &mut self,
        source_len: SingleAddrVariable,
        source_vector: BrilligVector,
        destination_vector: BrilligVector,
        write_pointer: MemoryAddress,
        item_push_count: usize,
        back: bool,
    ) {
        let [
            source_vector_length_arg,
            source_vector_pointer_arg,
            item_push_count_arg,
            destination_vector_pointer_return,
            write_pointer_return,
        ] = self.make_scratch_registers();

        self.mov_instruction(source_vector_length_arg, source_len.address);
        self.mov_instruction(source_vector_pointer_arg, source_vector.pointer);
        self.usize_const_instruction(item_push_count_arg, item_push_count.into());

        self.add_procedure_call_instruction(ProcedureId::PrepareVectorPush(back));

        self.mov_instruction(destination_vector.pointer, destination_vector_pointer_return);
        self.mov_instruction(write_pointer, write_pointer_return);
    }
}

/// Compile [ProcedureId::PrepareVectorPush].
pub(super) fn compile_prepare_vector_push_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
    push_back: bool,
) {
    let [
        source_vector_length_arg,
        source_vector_pointer_arg,
        item_push_count_arg,
        destination_vector_pointer_return,
        write_pointer_return,
    ] = brillig_context.allocate_scratch_registers();

    let source_vector = BrilligVector { pointer: source_vector_pointer_arg };
    let target_vector = BrilligVector { pointer: destination_vector_pointer_return };

    let VectorMetaData {
        rc: source_rc,
        size: source_size,
        capacity: source_capacity,
        items_pointer: source_items_pointer,
    } = brillig_context.codegen_read_vector_metadata(
        source_vector,
        Some((source_vector_length_arg, item_push_count_arg)),
    );

    // The target size is the source size plus the number of items we are pushing.
    let target_size = brillig_context.allocate_single_addr_usize();
    brillig_context.memory_op_instruction(
        source_size.address,
        item_push_count_arg,
        target_size.address,
        BrilligBinaryOp::Add,
    );

    // The strategy is to reallocate first and then depending if it's push back or not, copy the items or not.
    reallocate_vector_for_insertion(
        brillig_context,
        source_vector,
        *source_rc,
        *source_capacity,
        target_vector,
        *target_size,
    );

    // Get the pointer to the start of the items in the target vector.
    // This is adjusted below based on whether we push to the front or the back.
    let target_vector_items_pointer =
        brillig_context.codegen_make_vector_items_pointer(target_vector);

    if push_back {
        // If we are pushing to the back, we could be reusing the source vector if the RC was 1 and it had excess capacity.
        let was_reused = brillig_context.allocate_single_addr_bool();
        brillig_context.memory_op_instruction(
            source_vector.pointer,
            target_vector.pointer,
            was_reused.address,
            BrilligBinaryOp::Equals,
        );
        // If we are not reusing the source, then we need to copy its items to the target.
        brillig_context.codegen_if_not(was_reused.address, |brillig_context| {
            brillig_context.codegen_mem_copy(
                source_items_pointer.address,
                *target_vector_items_pointer,
                *source_size,
            );
        });
        // Target vector is ready for push back at this point.
        // The write pointer returned points after source-length number of items in the target vector.
        brillig_context.memory_op_instruction(
            *target_vector_items_pointer,
            source_size.address,
            write_pointer_return,
            BrilligBinaryOp::Add,
        );
    } else {
        // If pushing to the front we need to shift the items independently of it being reused or not.
        let target_start = brillig_context.allocate_register();
        // Shift items by the number of items we want to push to the front.
        brillig_context.memory_op_instruction(
            *target_vector_items_pointer,
            item_push_count_arg,
            *target_start,
            BrilligBinaryOp::Add,
        );
        brillig_context.codegen_mem_copy_from_the_end(
            source_items_pointer.address,
            *target_start,
            *source_size,
        );
        // The write pointer returned is the the first (now free) item in the target vector.
        brillig_context.mov_instruction(write_pointer_return, *target_vector_items_pointer);
    }
}

/// Reallocates the target vector for insertion, skipping reallocation if the source vector can be reused:
/// * if the capacity accommodates the target size:
///   * if the RC is 1, we can increase the size of the source vector as reuse it as the destination
///   * if the RC is not 1, we allocate a new destination vector with the source capacity and target size
/// * if the capacity is too small for the target size, we allocate the destination vector with a capacity that is double the target size.
///
/// Does not copy the items, only reallocates the vector.
///
/// Whether a copy is necessary can be gleaned by comparing the target address to the source address.
pub(crate) fn reallocate_vector_for_insertion<
    F: AcirField + DebugToString,
    Registers: RegisterAllocator,
>(
    brillig_context: &mut BrilligContext<F, Registers>,
    source_vector: BrilligVector,
    source_rc: SingleAddrVariable,
    source_capacity: SingleAddrVariable,
    target_vector: BrilligVector,
    target_size: SingleAddrVariable,
) {
    // If the source capacity is at least as large than the target size, we can potentially reuse the source vector to write the new items.
    let does_capacity_fit = brillig_context.allocate_single_addr_bool();
    brillig_context.memory_op_instruction(
        target_size.address,
        source_capacity.address,
        does_capacity_fit.address,
        BrilligBinaryOp::LessThanEquals,
    );

    // Reallocate target vector for insertion
    brillig_context.codegen_branch(
        does_capacity_fit.address,
        |brillig_context, does_capacity_fit| {
            if does_capacity_fit {
                // We can only reuse the source vector if the ref-count is 1.
                let is_rc_one = brillig_context.codegen_usize_equals_one(source_rc);

                brillig_context.codegen_branch(is_rc_one.address, |brillig_context, is_rc_one| {
                    if is_rc_one {
                        // We can insert in place, so we can just move the source pointer to the destination pointer and update the length
                        brillig_context
                            .mov_instruction(target_vector.pointer, source_vector.pointer);
                        brillig_context.codegen_update_vector_size(target_vector, target_size);
                    } else {
                        // Increase our array copy counter if that flag is set
                        if brillig_context.count_arrays_copied {
                            brillig_context.codegen_increment_array_copy_counter();
                        }
                        // We could not reuse the source vector, because there are other references to it.
                        // Allocate a new vector with the target size and source capacity.
                        brillig_context.codegen_initialize_vector(
                            target_vector,
                            target_size,
                            Some(source_capacity),
                        );
                    }
                });
            } else {
                let double_size = brillig_context.allocate_single_addr_usize();
                brillig_context.codegen_usize_op(
                    target_size.address,
                    double_size.address,
                    BrilligBinaryOp::Mul,
                    2_usize,
                );

                // We will have to copy the vector to the new expanded memory region.
                brillig_context.codegen_initialize_vector(
                    target_vector,
                    target_size,
                    Some(*double_size),
                );

                // Increase our array copy counter if that flag is set
                if brillig_context.count_arrays_copied {
                    brillig_context.codegen_increment_array_copy_counter();
                }
            }
        },
    );
}
