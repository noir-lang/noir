use std::vec;

use acvm::{acir::brillig::MemoryAddress, AcirField};

use super::ProcedureId;
use crate::brillig::brillig_ir::{
    brillig_variable::{BrilligVector, SingleAddrVariable},
    debug_show::DebugToString,
    registers::{RegisterAllocator, ScratchSpace},
    BrilligBinaryOp, BrilligContext,
};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Prepares a vector for a push operation, allocating a larger vector and copying the source vector into the destination vector.
    /// It returns the write pointer to where to put the new items.
    pub(crate) fn call_prepare_vector_push_procedure(
        &mut self,
        source_vector: BrilligVector,
        destination_vector: BrilligVector,
        write_pointer: MemoryAddress,
        item_push_count: usize,
        back: bool,
    ) {
        let source_vector_pointer_arg = MemoryAddress::direct(ScratchSpace::start());
        let item_push_count_arg = MemoryAddress::direct(ScratchSpace::start() + 1);
        let new_vector_pointer_return = MemoryAddress::direct(ScratchSpace::start() + 2);
        let write_pointer_return = MemoryAddress::direct(ScratchSpace::start() + 3);

        self.mov_instruction(source_vector_pointer_arg, source_vector.pointer);
        self.usize_const_instruction(item_push_count_arg, item_push_count.into());

        self.add_procedure_call_instruction(ProcedureId::PrepareVectorPush(back));

        self.mov_instruction(destination_vector.pointer, new_vector_pointer_return);
        self.mov_instruction(write_pointer, write_pointer_return);
    }
}

pub(super) fn compile_prepare_vector_push_procedure<F: AcirField + DebugToString>(
    brillig_context: &mut BrilligContext<F, ScratchSpace>,
    push_back: bool,
) {
    let source_vector_pointer_arg = MemoryAddress::direct(ScratchSpace::start());
    let item_push_count_arg = MemoryAddress::direct(ScratchSpace::start() + 1);
    let new_vector_pointer_return = MemoryAddress::direct(ScratchSpace::start() + 2);
    let write_pointer_return = MemoryAddress::direct(ScratchSpace::start() + 3);

    brillig_context.set_allocated_registers(vec![
        source_vector_pointer_arg,
        item_push_count_arg,
        new_vector_pointer_return,
        write_pointer_return,
    ]);

    let source_vector = BrilligVector { pointer: source_vector_pointer_arg };
    let target_vector = BrilligVector { pointer: new_vector_pointer_return };

    let source_rc = SingleAddrVariable::new_usize(brillig_context.allocate_register());
    let source_size = SingleAddrVariable::new_usize(brillig_context.allocate_register());
    let source_capacity = SingleAddrVariable::new_usize(brillig_context.allocate_register());
    let source_items_pointer = SingleAddrVariable::new_usize(brillig_context.allocate_register());
    brillig_context.codegen_read_vector_metadata(
        source_vector,
        source_rc,
        source_size,
        source_capacity,
        source_items_pointer,
    );

    let target_size = SingleAddrVariable::new_usize(brillig_context.allocate_register());
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
        source_rc,
        source_capacity,
        target_vector,
        target_size,
    );

    let target_vector_items_pointer =
        brillig_context.codegen_make_vector_items_pointer(target_vector);

    if push_back {
        let was_reused = SingleAddrVariable::new(brillig_context.allocate_register(), 1);
        brillig_context.memory_op_instruction(
            source_vector.pointer,
            target_vector.pointer,
            was_reused.address,
            BrilligBinaryOp::Equals,
        );

        brillig_context.codegen_if_not(was_reused.address, |brillig_context| {
            brillig_context.codegen_mem_copy(
                source_items_pointer.address,
                target_vector_items_pointer,
                source_size,
            );
        });
        // Target vector is ready for push back at this point
        brillig_context.memory_op_instruction(
            target_vector_items_pointer,
            source_size.address,
            write_pointer_return,
            BrilligBinaryOp::Add,
        );

        brillig_context.deallocate_single_addr(was_reused);
    } else {
        // If push front we need to shift the items independently of it being reused or not
        let target_start = brillig_context.allocate_register();
        brillig_context.memory_op_instruction(
            target_vector_items_pointer,
            item_push_count_arg,
            target_start,
            BrilligBinaryOp::Add,
        );
        brillig_context.codegen_mem_copy_from_the_end(
            source_items_pointer.address,
            target_start,
            source_size,
        );
        brillig_context.deallocate_register(target_start);
        brillig_context.mov_instruction(write_pointer_return, target_vector_items_pointer);
    }

    brillig_context.deallocate_single_addr(source_rc);
    brillig_context.deallocate_single_addr(source_size);
    brillig_context.deallocate_single_addr(source_capacity);
    brillig_context.deallocate_single_addr(source_items_pointer);
    brillig_context.deallocate_single_addr(target_size);
    brillig_context.deallocate_register(target_vector_items_pointer);
}

/// Reallocates the target vector for insertion, skipping reallocation if the source vector can be reused.
/// If it doesn't fit capacity we will reallocate the vector to double the capacity.
/// Does not copy the items, only reallocates the vector.
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
    let does_capacity_fit = SingleAddrVariable::new(brillig_context.allocate_register(), 1);
    brillig_context.memory_op_instruction(
        target_size.address,
        source_capacity.address,
        does_capacity_fit.address,
        BrilligBinaryOp::LessThanEquals,
    );

    let is_rc_one = SingleAddrVariable::new(brillig_context.allocate_register(), 1);
    brillig_context.codegen_usize_op(
        source_rc.address,
        is_rc_one.address,
        BrilligBinaryOp::Equals,
        1,
    );

    // Reallocate target vector for insertion
    brillig_context.codegen_branch(
        does_capacity_fit.address,
        |brillig_context, does_capacity_fit| {
            if does_capacity_fit {
                brillig_context.codegen_branch(is_rc_one.address, |brillig_context, is_rc_one| {
                    if is_rc_one {
                        // We can insert in place, so we can just move the source pointer to the destination pointer and update the length
                        brillig_context
                            .mov_instruction(target_vector.pointer, source_vector.pointer);
                        brillig_context.codegen_update_vector_length(target_vector, target_size);
                    } else {
                        brillig_context.codegen_initialize_vector(
                            target_vector,
                            target_size,
                            Some(source_capacity),
                        );
                    }
                });
            } else {
                let double_size =
                    SingleAddrVariable::new_usize(brillig_context.allocate_register());
                brillig_context.codegen_usize_op(
                    target_size.address,
                    double_size.address,
                    BrilligBinaryOp::Mul,
                    2_usize,
                );
                brillig_context.codegen_initialize_vector(
                    target_vector,
                    target_size,
                    Some(double_size),
                );
                brillig_context.deallocate_single_addr(double_size);
            }
        },
    );

    brillig_context.deallocate_single_addr(is_rc_one);
    brillig_context.deallocate_single_addr(does_capacity_fit);
}
