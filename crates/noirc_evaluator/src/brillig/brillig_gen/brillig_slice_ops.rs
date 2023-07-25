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
    brillig_context.deallocate_register(destination_copy_pointer);

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
        target_vector.size,
    );
    brillig_context.deallocate_register(source_copy_pointer);

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

    brillig_context.deallocate_register(source_pointer_at_index);
    brillig_context.deallocate_register(target_pointer_after_index);
    brillig_context.deallocate_register(item_count);

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

    brillig_context.deallocate_register(source_pointer_after_index);
    brillig_context.deallocate_register(target_pointer_at_index);
    brillig_context.deallocate_register(item_count);

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

#[cfg(test)]
mod tests {
    use std::vec;

    use acvm::acir::brillig::{HeapVector, Value};
    use acvm::brillig_vm::brillig::{Opcode, RegisterIndex};
    use acvm::brillig_vm::{Registers, VMStatus, VM};
    use acvm::{BlackBoxFunctionSolver, BlackBoxResolutionError, FieldElement};

    use crate::brillig::brillig_gen::brillig_slice_ops::{
        slice_insert_operation, slice_pop_back_operation, slice_pop_front_operation,
        slice_push_back_operation, slice_push_front_operation, slice_remove_operation,
    };
    use crate::brillig::brillig_ir::artifact::{BrilligArtifact, BrilligParameter};
    use crate::brillig::brillig_ir::BrilligContext;

    struct DummyBlackBoxSolver;

    impl BlackBoxFunctionSolver for DummyBlackBoxSolver {
        fn schnorr_verify(
            &self,
            _public_key_x: &FieldElement,
            _public_key_y: &FieldElement,
            _signature: &[u8],
            _message: &[u8],
        ) -> Result<bool, BlackBoxResolutionError> {
            Ok(true)
        }
        fn pedersen(
            &self,
            _inputs: &[FieldElement],
            _domain_separator: u32,
        ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
            Ok((2_u128.into(), 3_u128.into()))
        }
        fn fixed_base_scalar_mul(
            &self,
            _input: &FieldElement,
        ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
            Ok((4_u128.into(), 5_u128.into()))
        }
    }

    fn create_context(
        arguments: Vec<BrilligParameter>,
        returns: Vec<BrilligParameter>,
    ) -> BrilligContext {
        let mut context = BrilligContext::new(arguments, returns, true);
        context.enter_context("test");
        context
    }

    fn create_entry_point_bytecode(
        context: BrilligContext,
        arguments: Vec<BrilligParameter>,
        returns: Vec<BrilligParameter>,
    ) -> Vec<Opcode> {
        let artifact = context.artifact();
        let mut entry_point_artifact =
            BrilligArtifact::new_entry_point_artifact(arguments, returns, "test".to_string());
        entry_point_artifact.link_with(&artifact);
        entry_point_artifact.finish()
    }

    fn create_and_run_vm(
        memory: Vec<Value>,
        param_registers: Vec<Value>,
        context: BrilligContext,
        arguments: Vec<BrilligParameter>,
        returns: Vec<BrilligParameter>,
    ) -> VM<'static, DummyBlackBoxSolver> {
        let mut vm = VM::new(
            Registers { inner: param_registers },
            memory,
            create_entry_point_bytecode(context, arguments, returns),
            vec![],
            &DummyBlackBoxSolver,
        );

        let status = vm.process_opcodes();
        assert_eq!(status, VMStatus::Finished);
        vm
    }

    #[test]
    fn test_slice_push_operation() {
        fn test_case_push(
            push_back: bool,
            array: Vec<Value>,
            expected_mem: Vec<Value>,
            item_to_push: Value,
        ) {
            let arguments =
                vec![BrilligParameter::HeapArray(array.len()), BrilligParameter::Register];
            let returns =
                vec![BrilligParameter::HeapArray(array.len() + 1), BrilligParameter::Register];

            let mut context = create_context(arguments.clone(), returns.clone());

            // Allocate the parameters
            let array_pointer = context.allocate_register();
            let item_to_insert = context.allocate_register();

            // Cast the source array to a vector
            let array_size = context.make_constant(array.len().into());

            // Allocate the results
            let copied_array_pointer = context.allocate_register();
            let copied_array_size = context.allocate_register();

            if push_back {
                slice_push_back_operation(
                    &mut context,
                    HeapVector { pointer: copied_array_pointer, size: copied_array_size },
                    HeapVector { pointer: array_pointer, size: array_size },
                    item_to_insert,
                );
            } else {
                slice_push_front_operation(
                    &mut context,
                    HeapVector { pointer: copied_array_pointer, size: copied_array_size },
                    HeapVector { pointer: array_pointer, size: array_size },
                    item_to_insert,
                );
            }

            context.return_instruction(&[copied_array_pointer, copied_array_size]);

            let vm = create_and_run_vm(
                array.clone(),
                vec![Value::from(0_usize), item_to_push],
                context,
                arguments,
                returns,
            );

            assert_eq!(vm.get_memory(), &expected_mem);

            assert_eq!(vm.get_registers().get(RegisterIndex(0)), Value::from(array.len()));
            assert_eq!(vm.get_registers().get(RegisterIndex(1)), Value::from(array.len() + 1));
        }

        test_case_push(
            true,
            vec![Value::from(1_usize), Value::from(2_usize), Value::from(3_usize)],
            vec![
                Value::from(1_usize),
                Value::from(2_usize),
                Value::from(3_usize),
                Value::from(1_usize),
                Value::from(2_usize),
                Value::from(3_usize),
                Value::from(27_usize),
            ],
            Value::from(27_usize),
        );
        test_case_push(true, vec![], vec![Value::from(27_usize)], Value::from(27_usize));
        test_case_push(
            false,
            vec![Value::from(1_usize), Value::from(2_usize), Value::from(3_usize)],
            vec![
                Value::from(1_usize),
                Value::from(2_usize),
                Value::from(3_usize),
                Value::from(27_usize),
                Value::from(1_usize),
                Value::from(2_usize),
                Value::from(3_usize),
            ],
            Value::from(27_usize),
        );
        test_case_push(false, vec![], vec![Value::from(27_usize)], Value::from(27_usize));
    }

    #[test]
    fn test_slice_pop_back_operation() {
        fn test_case_pop(
            pop_back: bool,
            array: Vec<Value>,
            expected_mem: Vec<Value>,
            expected_removed_item: Value,
        ) {
            let arguments = vec![BrilligParameter::HeapArray(array.len())];
            let returns = vec![
                BrilligParameter::HeapArray(array.len() - 1),
                BrilligParameter::Register,
                BrilligParameter::Register,
            ];

            let mut context = create_context(arguments.clone(), returns.clone());

            // Allocate the parameters
            let array_pointer = context.allocate_register();

            // Cast the source array to a vector
            let array_size = context.make_constant(array.len().into());

            // Allocate the results
            let copied_array_pointer = context.allocate_register();
            let removed_item = context.allocate_register();

            let copied_array_size = context.allocate_register();

            if pop_back {
                slice_pop_back_operation(
                    &mut context,
                    HeapVector { pointer: copied_array_pointer, size: copied_array_size },
                    HeapVector { pointer: array_pointer, size: array_size },
                    removed_item,
                );
            } else {
                slice_pop_front_operation(
                    &mut context,
                    HeapVector { pointer: copied_array_pointer, size: copied_array_size },
                    HeapVector { pointer: array_pointer, size: array_size },
                    removed_item,
                );
            }

            context.return_instruction(&[copied_array_pointer, copied_array_size, removed_item]);

            let vm = create_and_run_vm(
                array.clone(),
                vec![Value::from(0_usize)],
                context,
                arguments,
                returns,
            );

            assert_eq!(vm.get_memory(), &expected_mem);

            assert_eq!(vm.get_registers().get(RegisterIndex(0)), Value::from(array.len()));
            assert_eq!(vm.get_registers().get(RegisterIndex(1)), Value::from(array.len() - 1));
            assert_eq!(vm.get_registers().get(RegisterIndex(2)), expected_removed_item);
        }

        test_case_pop(
            true,
            vec![Value::from(1_usize), Value::from(2_usize), Value::from(3_usize)],
            vec![
                Value::from(1_usize),
                Value::from(2_usize),
                Value::from(3_usize),
                Value::from(1_usize),
                Value::from(2_usize),
            ],
            Value::from(3_usize),
        );
        test_case_pop(
            true,
            vec![Value::from(1_usize)],
            vec![Value::from(1_usize)],
            Value::from(1_usize),
        );
        test_case_pop(
            false,
            vec![Value::from(1_usize), Value::from(2_usize), Value::from(3_usize)],
            vec![
                Value::from(1_usize),
                Value::from(2_usize),
                Value::from(3_usize),
                Value::from(2_usize),
                Value::from(3_usize),
            ],
            Value::from(1_usize),
        );
    }

    #[test]
    fn test_slice_insert_operation() {
        fn test_case_insert(
            array: Vec<Value>,
            expected_mem: Vec<Value>,
            item: Value,
            index: Value,
        ) {
            let arguments = vec![
                BrilligParameter::HeapArray(array.len()),
                BrilligParameter::Register,
                BrilligParameter::Register,
            ];
            let returns =
                vec![BrilligParameter::HeapArray(array.len() + 1), BrilligParameter::Register];

            let mut context = create_context(arguments.clone(), returns.clone());

            // Allocate the parameters
            let array_pointer = context.allocate_register();
            let item_to_insert = context.allocate_register();
            let index_to_insert = context.allocate_register();

            // Cast the source array to a vector
            let array_size = context.make_constant(array.len().into());

            // Allocate the results
            let copied_array_pointer = context.allocate_register();

            let copied_array_size = context.allocate_register();

            slice_insert_operation(
                &mut context,
                HeapVector { pointer: copied_array_pointer, size: copied_array_size },
                HeapVector { pointer: array_pointer, size: array_size },
                index_to_insert,
                item_to_insert,
            );

            context.return_instruction(&[copied_array_pointer, copied_array_size]);

            let vm = create_and_run_vm(
                array.clone(),
                vec![Value::from(0_usize), item, index],
                context,
                arguments,
                returns,
            );

            assert_eq!(vm.get_memory(), &expected_mem);

            assert_eq!(vm.get_registers().get(RegisterIndex(0)), Value::from(array.len()));
            assert_eq!(vm.get_registers().get(RegisterIndex(1)), Value::from(array.len() + 1));
        }

        test_case_insert(
            vec![Value::from(1_usize), Value::from(2_usize), Value::from(3_usize)],
            vec![
                Value::from(1_usize),
                Value::from(2_usize),
                Value::from(3_usize),
                Value::from(1_usize),
                Value::from(27_usize),
                Value::from(2_usize),
                Value::from(3_usize),
            ],
            Value::from(27_usize),
            Value::from(1_usize),
        );

        test_case_insert(
            vec![Value::from(1_usize), Value::from(2_usize), Value::from(3_usize)],
            vec![
                Value::from(1_usize),
                Value::from(2_usize),
                Value::from(3_usize),
                Value::from(27_usize),
                Value::from(1_usize),
                Value::from(2_usize),
                Value::from(3_usize),
            ],
            Value::from(27_usize),
            Value::from(0_usize),
        );
        test_case_insert(
            vec![Value::from(1_usize), Value::from(2_usize), Value::from(3_usize)],
            vec![
                Value::from(1_usize),
                Value::from(2_usize),
                Value::from(3_usize),
                Value::from(1_usize),
                Value::from(2_usize),
                Value::from(27_usize),
                Value::from(3_usize),
            ],
            Value::from(27_usize),
            Value::from(2_usize),
        );
        test_case_insert(
            vec![Value::from(1_usize), Value::from(2_usize), Value::from(3_usize)],
            vec![
                Value::from(1_usize),
                Value::from(2_usize),
                Value::from(3_usize),
                Value::from(1_usize),
                Value::from(2_usize),
                Value::from(3_usize),
                Value::from(27_usize),
            ],
            Value::from(27_usize),
            Value::from(3_usize),
        );
        test_case_insert(
            vec![],
            vec![Value::from(27_usize)],
            Value::from(27_usize),
            Value::from(0_usize),
        );
    }

    #[test]
    fn test_slice_remove_operation() {
        fn test_case_remove(
            array: Vec<Value>,
            expected_mem: Vec<Value>,
            index: Value,
            expected_removed_item: Value,
        ) {
            let arguments =
                vec![BrilligParameter::HeapArray(array.len()), BrilligParameter::Register];
            let returns = vec![
                BrilligParameter::HeapArray(array.len() - 1),
                BrilligParameter::Register,
                BrilligParameter::Register,
            ];

            let mut context = create_context(arguments.clone(), returns.clone());

            // Allocate the parameters
            let array_pointer = context.allocate_register();
            let index_to_insert = context.allocate_register();

            // Cast the source array to a vector
            let array_size = context.make_constant(array.len().into());

            // Allocate the results
            let copied_array_pointer = context.allocate_register();
            let removed_item = context.allocate_register();

            let copied_array_size = context.allocate_register();

            slice_remove_operation(
                &mut context,
                HeapVector { pointer: copied_array_pointer, size: copied_array_size },
                HeapVector { pointer: array_pointer, size: array_size },
                index_to_insert,
                removed_item,
            );

            context.return_instruction(&[copied_array_pointer, copied_array_size, removed_item]);

            let vm = create_and_run_vm(
                array.clone(),
                vec![Value::from(0_usize), index],
                context,
                arguments,
                returns,
            );

            assert_eq!(vm.get_memory(), &expected_mem);

            assert_eq!(vm.get_registers().get(RegisterIndex(0)), Value::from(array.len()));
            assert_eq!(vm.get_registers().get(RegisterIndex(1)), Value::from(array.len() - 1));
            assert_eq!(vm.get_registers().get(RegisterIndex(2)), expected_removed_item);
        }

        test_case_remove(
            vec![Value::from(1_usize), Value::from(2_usize), Value::from(3_usize)],
            vec![
                Value::from(1_usize),
                Value::from(2_usize),
                Value::from(3_usize),
                Value::from(2_usize),
                Value::from(3_usize),
            ],
            Value::from(0_usize),
            Value::from(1_usize),
        );

        test_case_remove(
            vec![Value::from(1_usize), Value::from(2_usize), Value::from(3_usize)],
            vec![
                Value::from(1_usize),
                Value::from(2_usize),
                Value::from(3_usize),
                Value::from(1_usize),
                Value::from(3_usize),
            ],
            Value::from(1_usize),
            Value::from(2_usize),
        );

        test_case_remove(
            vec![Value::from(1_usize), Value::from(2_usize), Value::from(3_usize)],
            vec![
                Value::from(1_usize),
                Value::from(2_usize),
                Value::from(3_usize),
                Value::from(1_usize),
                Value::from(2_usize),
            ],
            Value::from(2_usize),
            Value::from(3_usize),
        );
        test_case_remove(
            vec![Value::from(1_usize)],
            vec![Value::from(1_usize)],
            Value::from(0_usize),
            Value::from(1_usize),
        );
    }
}
