use acvm::brillig_vm::brillig::{BinaryIntOp, HeapVector, RegisterIndex, RegisterOrMemory};

use crate::brillig::brillig_ir::BrilligContext;

pub(crate) fn slice_push_back_operation(
    brillig_context: &mut BrilligContext,
    target_vector: HeapVector,
    source_vector: HeapVector,
    item_to_insert: RegisterIndex,
) {
    // First we need to allocate the target vector incrementing the size by 1
    brillig_context.usize_op(source_vector.size, target_vector.size, BinaryIntOp::Add, 1);
    brillig_context.allocate_array_instruction(target_vector.pointer, target_vector.size);

    // Now we copy the source vector into the target vector
    brillig_context.copy_array_instruction(
        source_vector.pointer,
        target_vector.pointer,
        source_vector.size,
    );

    brillig_context.array_set(target_vector.pointer, source_vector.size, item_to_insert);
}

pub(crate) fn slice_push_front_operation(
    brillig_context: &mut BrilligContext,
    target_vector: HeapVector,
    source_vector: HeapVector,
    item_to_insert: RegisterIndex,
) {
    // First we need to allocate the target vector incrementing the size by 1
    brillig_context.usize_op(source_vector.size, target_vector.size, BinaryIntOp::Add, 1);
    brillig_context.allocate_array_instruction(target_vector.pointer, target_vector.size);

    // Now we offset the target pointer by one
    let destination_copy_pointer = brillig_context.allocate_register();
    brillig_context.usize_op(target_vector.pointer, destination_copy_pointer, BinaryIntOp::Add, 1);

    // Now we copy the source vector into the target vector starting at index 1
    brillig_context.copy_array_instruction(
        source_vector.pointer,
        destination_copy_pointer,
        source_vector.size,
    );

    // Then we write the item to insert at index 0
    let zero = brillig_context.make_constant(0_u128.into());
    brillig_context.array_set(target_vector.pointer, zero, item_to_insert);
    brillig_context.deallocate_register(zero);
}

pub(crate) fn slice_pop_front_operation(
    brillig_context: &mut BrilligContext,
    target_vector: HeapVector,
    source_vector: HeapVector,
    removed_item: RegisterIndex,
) {
    // First we need to allocate the target vector decrementing the size by 1
    brillig_context.usize_op(source_vector.size, target_vector.size, BinaryIntOp::Sub, 1);
    brillig_context.allocate_array_instruction(target_vector.pointer, target_vector.size);

    // Now we offset the source pointer by one
    let source_copy_pointer = brillig_context.allocate_register();
    brillig_context.usize_op(source_vector.pointer, source_copy_pointer, BinaryIntOp::Add, 1);

    // Now we copy the source vector starting at index 1 into the target vector
    brillig_context.copy_array_instruction(
        source_copy_pointer,
        target_vector.pointer,
        source_vector.size,
    );

    let zero = brillig_context.make_constant(0_u128.into());
    brillig_context.array_get(source_vector.pointer, zero, removed_item);
    brillig_context.deallocate_register(zero);
}

pub(crate) fn slice_pop_back_operation(
    brillig_context: &mut BrilligContext,
    target_vector: HeapVector,
    source_vector: HeapVector,
    removed_item: RegisterIndex,
) {
    // First we need to allocate the target vector decrementing the size by 1
    brillig_context.usize_op(source_vector.size, target_vector.size, BinaryIntOp::Sub, 1);
    brillig_context.allocate_array_instruction(target_vector.pointer, target_vector.size);

    // Now we copy all elements but the last into the target vector
    brillig_context.copy_array_instruction(
        source_vector.pointer,
        target_vector.pointer,
        target_vector.size,
    );

    brillig_context.array_get(source_vector.pointer, target_vector.size, removed_item);
}

pub(crate) fn slice_insert_operation(
    brillig_context: &mut BrilligContext,
    target_vector: HeapVector,
    source_vector: HeapVector,
    index: RegisterIndex,
    item: RegisterIndex,
) {
    // First we need to allocate the target vector incrementing the size by 1
    brillig_context.usize_op(source_vector.size, target_vector.size, BinaryIntOp::Add, 1);
    brillig_context.allocate_array_instruction(target_vector.pointer, target_vector.size);

    // Copy the elements to the left of the index
    brillig_context.copy_array_instruction(source_vector.pointer, target_vector.pointer, index);

    // Compute the source pointer just at the index
    let source_pointer_at_index = brillig_context.allocate_register();
    brillig_context.memory_op(
        source_vector.pointer,
        index,
        source_pointer_at_index,
        BinaryIntOp::Add,
    );

    // Compute the target pointer after the index
    let target_pointer_after_index = brillig_context.allocate_register();
    brillig_context.memory_op(
        target_vector.pointer,
        index,
        target_pointer_after_index,
        BinaryIntOp::Add,
    );
    brillig_context.usize_op_in_place(target_pointer_after_index, BinaryIntOp::Add, 1);

    // Compute the number of elements to the right of the index
    let item_count = brillig_context.allocate_register();
    brillig_context.memory_op(source_vector.size, index, item_count, BinaryIntOp::Sub);

    // Copy the elements to the right of the index
    brillig_context.copy_array_instruction(
        source_pointer_at_index,
        target_pointer_after_index,
        item_count,
    );

    // Write the item to insert at the index
    brillig_context.array_set(target_vector.pointer, index, item);
}

pub(crate) fn slice_remove_operation(
    brillig_context: &mut BrilligContext,
    target_vector: HeapVector,
    source_vector: HeapVector,
    index: RegisterIndex,
    removed_item: RegisterIndex,
) {
    // First we need to allocate the target vector decrementing the size by 1
    brillig_context.usize_op(source_vector.size, target_vector.size, BinaryIntOp::Sub, 1);
    brillig_context.allocate_array_instruction(target_vector.pointer, target_vector.size);

    // Copy the elements to the left of the index
    brillig_context.copy_array_instruction(source_vector.pointer, target_vector.pointer, index);

    // Compute the source pointer after the index
    let source_pointer_after_index = brillig_context.allocate_register();
    brillig_context.memory_op(
        source_vector.pointer,
        index,
        source_pointer_after_index,
        BinaryIntOp::Add,
    );
    brillig_context.usize_op_in_place(source_pointer_after_index, BinaryIntOp::Add, 1);

    // Compute the target pointer at the index
    let target_pointer_at_index = brillig_context.allocate_register();
    brillig_context.memory_op(
        target_vector.pointer,
        index,
        target_pointer_at_index,
        BinaryIntOp::Add,
    );

    // Compute the number of elements to the right of the index
    let item_count = brillig_context.allocate_register();
    brillig_context.memory_op(source_vector.size, index, item_count, BinaryIntOp::Sub);
    brillig_context.usize_op_in_place(item_count, BinaryIntOp::Sub, 1);

    // Copy the elements to the right of the index
    brillig_context.copy_array_instruction(
        source_pointer_after_index,
        target_pointer_at_index,
        item_count,
    );

    // Get the item at the index
    brillig_context.array_get(source_vector.pointer, index, removed_item);
}

pub(crate) fn convert_array_or_vector_to_vector(
    brillig_context: &mut BrilligContext,
    source_variable: RegisterOrMemory,
) -> HeapVector {
    match source_variable {
        RegisterOrMemory::HeapVector(source_vector) => source_vector,
        RegisterOrMemory::HeapArray(source_array) => brillig_context.array_to_vector(&source_array),
        _ => unreachable!("ICE: unsupported slice push back source {:?}", source_variable),
    }
}
