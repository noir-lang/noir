use acvm::brillig_vm::brillig::{BinaryIntOp, HeapVector, RegisterIndex, RegisterOrMemory};

use super::brillig_block::BrilligBlock;

impl<'block> BrilligBlock<'block> {
    pub(crate) fn slice_push_back_operation(
        &mut self,
        target_vector: HeapVector,
        source_vector: HeapVector,
        variables_to_insert: &[RegisterOrMemory],
    ) {
        // First we need to allocate the target vector incrementing the size by variables_to_insert.len()
        self.brillig_context.usize_op(
            source_vector.size,
            target_vector.size,
            BinaryIntOp::Add,
            variables_to_insert.len(),
        );
        self.brillig_context.allocate_array_instruction(target_vector.pointer, target_vector.size);

        // Now we copy the source vector into the target vector
        self.brillig_context.copy_array_instruction(
            source_vector.pointer,
            target_vector.pointer,
            source_vector.size,
        );

        for (index, variable) in variables_to_insert.iter().enumerate() {
            let target_index = self.brillig_context.make_constant(index.into());
            self.brillig_context.memory_op(
                target_index,
                source_vector.size,
                target_index,
                BinaryIntOp::Add,
            );
            self.store_variable_in_array(target_vector.pointer, target_index, *variable);
            self.brillig_context.deallocate_register(target_index);
        }
    }

    pub(crate) fn slice_push_front_operation(
        &mut self,
        target_vector: HeapVector,
        source_vector: HeapVector,
        variables_to_insert: &[RegisterOrMemory],
    ) {
        // First we need to allocate the target vector incrementing the size by variables_to_insert.len()
        self.brillig_context.usize_op(
            source_vector.size,
            target_vector.size,
            BinaryIntOp::Add,
            variables_to_insert.len(),
        );
        self.brillig_context.allocate_array_instruction(target_vector.pointer, target_vector.size);

        // Now we offset the target pointer by variables_to_insert.len()
        let destination_copy_pointer = self.brillig_context.allocate_register();
        self.brillig_context.usize_op(
            target_vector.pointer,
            destination_copy_pointer,
            BinaryIntOp::Add,
            variables_to_insert.len(),
        );

        // Now we copy the source vector into the target vector starting at index variables_to_insert.len()
        self.brillig_context.copy_array_instruction(
            source_vector.pointer,
            destination_copy_pointer,
            source_vector.size,
        );

        // Then we write the items to insert at the start
        for (index, variable) in variables_to_insert.iter().enumerate() {
            let target_index = self.brillig_context.make_constant(index.into());
            self.store_variable_in_array(target_vector.pointer, target_index, *variable);
            self.brillig_context.deallocate_register(target_index);
        }

        self.brillig_context.deallocate_register(destination_copy_pointer);
    }

    pub(crate) fn slice_pop_front_operation(
        &mut self,
        target_vector: HeapVector,
        source_vector: HeapVector,
        removed_items: &[RegisterOrMemory],
    ) {
        // First we need to allocate the target vector decrementing the size by removed_items.len()
        self.brillig_context.usize_op(
            source_vector.size,
            target_vector.size,
            BinaryIntOp::Sub,
            removed_items.len(),
        );
        self.brillig_context.allocate_array_instruction(target_vector.pointer, target_vector.size);

        // Now we offset the source pointer by removed_items.len()
        let source_copy_pointer = self.brillig_context.allocate_register();
        self.brillig_context.usize_op(
            source_vector.pointer,
            source_copy_pointer,
            BinaryIntOp::Add,
            removed_items.len(),
        );

        // Now we copy the source vector starting at index removed_items.len() into the target vector
        self.brillig_context.copy_array_instruction(
            source_copy_pointer,
            target_vector.pointer,
            target_vector.size,
        );

        for (index, variable) in removed_items.iter().enumerate() {
            let target_index = self.brillig_context.make_constant(index.into());
            self.retrieve_variable_from_array(source_vector.pointer, target_index, *variable);
            self.brillig_context.deallocate_register(target_index);
        }

        self.brillig_context.deallocate_register(source_copy_pointer);
    }

    pub(crate) fn slice_pop_back_operation(
        &mut self,
        target_vector: HeapVector,
        source_vector: HeapVector,
        removed_items: &[RegisterOrMemory],
    ) {
        // First we need to allocate the target vector decrementing the size by removed_items.len()
        self.brillig_context.usize_op(
            source_vector.size,
            target_vector.size,
            BinaryIntOp::Sub,
            removed_items.len(),
        );
        self.brillig_context.allocate_array_instruction(target_vector.pointer, target_vector.size);

        // Now we copy all elements except the last items into the target vector
        self.brillig_context.copy_array_instruction(
            source_vector.pointer,
            target_vector.pointer,
            target_vector.size,
        );

        for (index, variable) in removed_items.iter().enumerate() {
            let target_index = self.brillig_context.make_constant(index.into());
            self.brillig_context.memory_op(
                target_index,
                target_vector.size,
                target_index,
                BinaryIntOp::Add,
            );
            self.retrieve_variable_from_array(source_vector.pointer, target_index, *variable);
            self.brillig_context.deallocate_register(target_index);
        }
    }

    pub(crate) fn slice_insert_operation(
        &mut self,
        target_vector: HeapVector,
        source_vector: HeapVector,
        index: RegisterIndex,
        items: &[RegisterOrMemory],
    ) {
        // First we need to allocate the target vector incrementing the size by items.len()
        self.brillig_context.usize_op(
            source_vector.size,
            target_vector.size,
            BinaryIntOp::Add,
            items.len(),
        );
        self.brillig_context.allocate_array_instruction(target_vector.pointer, target_vector.size);

        // Copy the elements to the left of the index
        self.brillig_context.copy_array_instruction(
            source_vector.pointer,
            target_vector.pointer,
            index,
        );

        // Compute the source pointer just at the index
        let source_pointer_at_index = self.brillig_context.allocate_register();
        self.brillig_context.memory_op(
            source_vector.pointer,
            index,
            source_pointer_at_index,
            BinaryIntOp::Add,
        );

        // Compute the target pointer after the inserted elements
        let target_pointer_after_index = self.brillig_context.allocate_register();
        self.brillig_context.memory_op(
            target_vector.pointer,
            index,
            target_pointer_after_index,
            BinaryIntOp::Add,
        );
        self.brillig_context.usize_op_in_place(
            target_pointer_after_index,
            BinaryIntOp::Add,
            items.len(),
        );

        // Compute the number of elements to the right of the index
        let item_count = self.brillig_context.allocate_register();
        self.brillig_context.memory_op(source_vector.size, index, item_count, BinaryIntOp::Sub);

        // Copy the elements to the right of the index
        self.brillig_context.copy_array_instruction(
            source_pointer_at_index,
            target_pointer_after_index,
            item_count,
        );

        // Write the items to insert starting at the index
        for (subitem_index, variable) in items.iter().enumerate() {
            let target_index = self.brillig_context.make_constant(subitem_index.into());
            self.brillig_context.memory_op(target_index, index, target_index, BinaryIntOp::Add);
            self.store_variable_in_array(target_vector.pointer, target_index, *variable);
            self.brillig_context.deallocate_register(target_index);
        }

        self.brillig_context.deallocate_register(source_pointer_at_index);
        self.brillig_context.deallocate_register(target_pointer_after_index);
        self.brillig_context.deallocate_register(item_count);
    }

    pub(crate) fn slice_remove_operation(
        &mut self,
        target_vector: HeapVector,
        source_vector: HeapVector,
        index: RegisterIndex,
        removed_items: &[RegisterOrMemory],
    ) {
        // First we need to allocate the target vector decrementing the size by removed_items.len()
        self.brillig_context.usize_op(
            source_vector.size,
            target_vector.size,
            BinaryIntOp::Sub,
            removed_items.len(),
        );
        self.brillig_context.allocate_array_instruction(target_vector.pointer, target_vector.size);

        // Copy the elements to the left of the index
        self.brillig_context.copy_array_instruction(
            source_vector.pointer,
            target_vector.pointer,
            index,
        );

        // Compute the source pointer after the removed items
        let source_pointer_after_index = self.brillig_context.allocate_register();
        self.brillig_context.memory_op(
            source_vector.pointer,
            index,
            source_pointer_after_index,
            BinaryIntOp::Add,
        );
        self.brillig_context.usize_op_in_place(
            source_pointer_after_index,
            BinaryIntOp::Add,
            removed_items.len(),
        );

        // Compute the target pointer at the index
        let target_pointer_at_index = self.brillig_context.allocate_register();
        self.brillig_context.memory_op(
            target_vector.pointer,
            index,
            target_pointer_at_index,
            BinaryIntOp::Add,
        );

        // Compute the number of elements to the right of the index
        let item_count = self.brillig_context.allocate_register();
        self.brillig_context.memory_op(source_vector.size, index, item_count, BinaryIntOp::Sub);
        self.brillig_context.usize_op_in_place(item_count, BinaryIntOp::Sub, removed_items.len());

        // Copy the elements to the right of the index
        self.brillig_context.copy_array_instruction(
            source_pointer_after_index,
            target_pointer_at_index,
            item_count,
        );

        // Get the removed items
        for (subitem_index, variable) in removed_items.iter().enumerate() {
            let target_index = self.brillig_context.make_constant(subitem_index.into());
            self.brillig_context.memory_op(target_index, index, target_index, BinaryIntOp::Add);
            self.retrieve_variable_from_array(source_vector.pointer, target_index, *variable);
            self.brillig_context.deallocate_register(target_index);
        }

        self.brillig_context.deallocate_register(source_pointer_after_index);
        self.brillig_context.deallocate_register(target_pointer_at_index);
        self.brillig_context.deallocate_register(item_count);
    }

    pub(crate) fn convert_array_or_vector_to_vector(
        &mut self,
        source_variable: RegisterOrMemory,
    ) -> HeapVector {
        match source_variable {
            RegisterOrMemory::HeapVector(source_vector) => source_vector,
            RegisterOrMemory::HeapArray(source_array) => {
                self.brillig_context.array_to_vector(&source_array)
            }
            _ => unreachable!("ICE: unsupported slice push back source {:?}", source_variable),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::vec;

    use acvm::acir::brillig::{HeapVector, Value};
    use acvm::brillig_vm::brillig::{RegisterIndex, RegisterOrMemory};

    use crate::brillig::brillig_gen::brillig_block::BrilligBlock;
    use crate::brillig::brillig_gen::brillig_fn::FunctionContext;
    use crate::brillig::brillig_ir::artifact::BrilligParameter;
    use crate::brillig::brillig_ir::tests::{create_and_run_vm, create_context};
    use crate::brillig::brillig_ir::BrilligContext;
    use crate::ssa::function_builder::FunctionBuilder;
    use crate::ssa::ir::function::RuntimeType;
    use crate::ssa::ir::map::Id;
    use crate::ssa::ssa_gen::Ssa;

    fn create_test_environment() -> (Ssa, FunctionContext, BrilligContext) {
        let builder =
            FunctionBuilder::new("main".to_string(), Id::test_new(0), RuntimeType::Brillig);
        let ssa = builder.finish();
        let function_context = FunctionContext::new(ssa.main());
        let brillig_context = create_context();
        (ssa, function_context, brillig_context)
    }

    fn create_brillig_block<'a>(
        function_context: &'a mut FunctionContext,
        brillig_context: &'a mut BrilligContext,
    ) -> BrilligBlock<'a> {
        BrilligBlock {
            function_context,
            block_id: Id::test_new(0),
            brillig_context,
            available_variables: HashSet::new(),
        }
    }

    #[test]
    fn test_slice_push_operation() {
        fn test_case_push(
            push_back: bool,
            array: Vec<Value>,
            expected_mem: Vec<Value>,
            item_to_push: Value,
        ) {
            let arguments = vec![
                BrilligParameter::Array(vec![BrilligParameter::Simple], array.len()),
                BrilligParameter::Simple,
            ];
            let returns = vec![
                BrilligParameter::Array(vec![BrilligParameter::Simple], array.len() + 1),
                BrilligParameter::Simple,
            ];

            let (_, mut function_context, mut context) = create_test_environment();

            // Allocate the parameters
            let array_pointer = context.allocate_register();
            let item_to_insert = context.allocate_register();

            // Cast the source array to a vector
            let array_size = context.make_constant(array.len().into());

            // Allocate the results
            let copied_array_pointer = context.allocate_register();
            let copied_array_size = context.allocate_register();

            let mut block = create_brillig_block(&mut function_context, &mut context);

            if push_back {
                block.slice_push_back_operation(
                    HeapVector { pointer: copied_array_pointer, size: copied_array_size },
                    HeapVector { pointer: array_pointer, size: array_size },
                    &[RegisterOrMemory::RegisterIndex(item_to_insert)],
                );
            } else {
                block.slice_push_front_operation(
                    HeapVector { pointer: copied_array_pointer, size: copied_array_size },
                    HeapVector { pointer: array_pointer, size: array_size },
                    &[RegisterOrMemory::RegisterIndex(item_to_insert)],
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
            let arguments =
                vec![BrilligParameter::Array(vec![BrilligParameter::Simple], array.len())];
            let returns = vec![
                BrilligParameter::Array(vec![BrilligParameter::Simple], array.len() - 1),
                BrilligParameter::Simple,
                BrilligParameter::Simple,
            ];

            let (_, mut function_context, mut context) = create_test_environment();

            // Allocate the parameters
            let array_pointer = context.allocate_register();

            // Cast the source array to a vector
            let array_size = context.make_constant(array.len().into());

            // Allocate the results
            let copied_array_pointer = context.allocate_register();
            let removed_item = context.allocate_register();

            let copied_array_size = context.allocate_register();

            let mut block = create_brillig_block(&mut function_context, &mut context);

            if pop_back {
                block.slice_pop_back_operation(
                    HeapVector { pointer: copied_array_pointer, size: copied_array_size },
                    HeapVector { pointer: array_pointer, size: array_size },
                    &[RegisterOrMemory::RegisterIndex(removed_item)],
                );
            } else {
                block.slice_pop_front_operation(
                    HeapVector { pointer: copied_array_pointer, size: copied_array_size },
                    HeapVector { pointer: array_pointer, size: array_size },
                    &[RegisterOrMemory::RegisterIndex(removed_item)],
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
                BrilligParameter::Array(vec![BrilligParameter::Simple], array.len()),
                BrilligParameter::Simple,
                BrilligParameter::Simple,
            ];
            let returns = vec![
                BrilligParameter::Array(vec![BrilligParameter::Simple], array.len() + 1),
                BrilligParameter::Simple,
            ];

            let (_, mut function_context, mut context) = create_test_environment();

            // Allocate the parameters
            let array_pointer = context.allocate_register();
            let item_to_insert = context.allocate_register();
            let index_to_insert = context.allocate_register();

            // Cast the source array to a vector
            let array_size = context.make_constant(array.len().into());

            // Allocate the results
            let copied_array_pointer = context.allocate_register();

            let copied_array_size = context.allocate_register();

            let mut block = create_brillig_block(&mut function_context, &mut context);

            block.slice_insert_operation(
                HeapVector { pointer: copied_array_pointer, size: copied_array_size },
                HeapVector { pointer: array_pointer, size: array_size },
                index_to_insert,
                &[RegisterOrMemory::RegisterIndex(item_to_insert)],
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
            let arguments = vec![
                BrilligParameter::Array(vec![BrilligParameter::Simple], array.len()),
                BrilligParameter::Simple,
            ];
            let returns = vec![
                BrilligParameter::Array(vec![BrilligParameter::Simple], array.len() - 1),
                BrilligParameter::Simple,
                BrilligParameter::Simple,
            ];

            let (_, mut function_context, mut context) = create_test_environment();

            // Allocate the parameters
            let array_pointer = context.allocate_register();
            let index_to_insert = context.allocate_register();

            // Cast the source array to a vector
            let array_size = context.make_constant(array.len().into());

            // Allocate the results
            let copied_array_pointer = context.allocate_register();
            let removed_item = context.allocate_register();

            let copied_array_size = context.allocate_register();

            let mut block = create_brillig_block(&mut function_context, &mut context);

            block.slice_remove_operation(
                HeapVector { pointer: copied_array_pointer, size: copied_array_size },
                HeapVector { pointer: array_pointer, size: array_size },
                index_to_insert,
                &[RegisterOrMemory::RegisterIndex(removed_item)],
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
