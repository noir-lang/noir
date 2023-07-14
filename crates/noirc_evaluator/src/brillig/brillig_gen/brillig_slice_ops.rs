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
}

pub(crate) fn slice_pop_back_operation(
    brillig_context: &mut BrilligContext,
    target_vector: HeapVector,
    source_vector: HeapVector,
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
