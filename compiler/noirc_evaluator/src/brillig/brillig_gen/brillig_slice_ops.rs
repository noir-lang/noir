use crate::brillig::brillig_ir::{
    brillig_variable::{BrilligVariable, BrilligVector, SingleAddrVariable},
    BrilligBinaryOp,
};

use super::brillig_block::BrilligBlock;

impl<'block> BrilligBlock<'block> {
    pub(crate) fn slice_push_back_operation(
        &mut self,
        target_vector: BrilligVector,
        source_vector: BrilligVector,
        variables_to_insert: &[BrilligVariable],
    ) {
        // First we need to allocate the target vector incrementing the size by variables_to_insert.len()
        self.brillig_context.codegen_usize_op(
            source_vector.size,
            target_vector.size,
            BrilligBinaryOp::Add,
            variables_to_insert.len(),
        );
        self.brillig_context.codegen_allocate_array(target_vector.pointer, target_vector.size);
        // We initialize the RC of the target vector to 1
        self.brillig_context.usize_const_instruction(target_vector.rc, 1_usize.into());

        // Now we copy the source vector into the target vector
        self.brillig_context.codegen_copy_array(
            source_vector.pointer,
            target_vector.pointer,
            SingleAddrVariable::new_usize(source_vector.size),
        );

        for (index, variable) in variables_to_insert.iter().enumerate() {
            let target_index = self.brillig_context.make_usize_constant_instruction(index.into());
            self.brillig_context.memory_op_instruction(
                target_index.address,
                source_vector.size,
                target_index.address,
                BrilligBinaryOp::Add,
            );
            self.store_variable_in_array(target_vector.pointer, target_index, *variable);
            self.brillig_context.deallocate_single_addr(target_index);
        }
    }

    pub(crate) fn slice_push_front_operation(
        &mut self,
        target_vector: BrilligVector,
        source_vector: BrilligVector,
        variables_to_insert: &[BrilligVariable],
    ) {
        // First we need to allocate the target vector incrementing the size by variables_to_insert.len()
        self.brillig_context.codegen_usize_op(
            source_vector.size,
            target_vector.size,
            BrilligBinaryOp::Add,
            variables_to_insert.len(),
        );
        self.brillig_context.codegen_allocate_array(target_vector.pointer, target_vector.size);
        // We initialize the RC of the target vector to 1
        self.brillig_context.usize_const_instruction(target_vector.rc, 1_usize.into());

        // Now we offset the target pointer by variables_to_insert.len()
        let destination_copy_pointer = self.brillig_context.allocate_register();
        self.brillig_context.codegen_usize_op(
            target_vector.pointer,
            destination_copy_pointer,
            BrilligBinaryOp::Add,
            variables_to_insert.len(),
        );

        // Now we copy the source vector into the target vector starting at index variables_to_insert.len()
        self.brillig_context.codegen_copy_array(
            source_vector.pointer,
            destination_copy_pointer,
            SingleAddrVariable::new_usize(source_vector.size),
        );

        // Then we write the items to insert at the start
        for (index, variable) in variables_to_insert.iter().enumerate() {
            let target_index = self.brillig_context.make_usize_constant_instruction(index.into());
            self.store_variable_in_array(target_vector.pointer, target_index, *variable);
            self.brillig_context.deallocate_single_addr(target_index);
        }

        self.brillig_context.deallocate_register(destination_copy_pointer);
    }

    pub(crate) fn slice_pop_front_operation(
        &mut self,
        target_vector: BrilligVector,
        source_vector: BrilligVector,
        removed_items: &[BrilligVariable],
    ) {
        // First we need to allocate the target vector decrementing the size by removed_items.len()
        self.brillig_context.codegen_usize_op(
            source_vector.size,
            target_vector.size,
            BrilligBinaryOp::Sub,
            removed_items.len(),
        );
        self.brillig_context.codegen_allocate_array(target_vector.pointer, target_vector.size);
        // We initialize the RC of the target vector to 1
        self.brillig_context.usize_const_instruction(target_vector.rc, 1_usize.into());

        // Now we offset the source pointer by removed_items.len()
        let source_copy_pointer = self.brillig_context.allocate_register();
        self.brillig_context.codegen_usize_op(
            source_vector.pointer,
            source_copy_pointer,
            BrilligBinaryOp::Add,
            removed_items.len(),
        );

        // Now we copy the source vector starting at index removed_items.len() into the target vector
        self.brillig_context.codegen_copy_array(
            source_copy_pointer,
            target_vector.pointer,
            SingleAddrVariable::new_usize(target_vector.size),
        );

        for (index, variable) in removed_items.iter().enumerate() {
            let target_index = self.brillig_context.make_usize_constant_instruction(index.into());
            self.retrieve_variable_from_array(source_vector.pointer, target_index, *variable);
            self.brillig_context.deallocate_single_addr(target_index);
        }

        self.brillig_context.deallocate_register(source_copy_pointer);
    }

    pub(crate) fn slice_pop_back_operation(
        &mut self,
        target_vector: BrilligVector,
        source_vector: BrilligVector,
        removed_items: &[BrilligVariable],
    ) {
        // First we need to allocate the target vector decrementing the size by removed_items.len()
        self.brillig_context.codegen_usize_op(
            source_vector.size,
            target_vector.size,
            BrilligBinaryOp::Sub,
            removed_items.len(),
        );
        self.brillig_context.codegen_allocate_array(target_vector.pointer, target_vector.size);
        // We initialize the RC of the target vector to 1
        self.brillig_context.usize_const_instruction(target_vector.rc, 1_usize.into());

        // Now we copy all elements except the last items into the target vector
        self.brillig_context.codegen_copy_array(
            source_vector.pointer,
            target_vector.pointer,
            SingleAddrVariable::new_usize(target_vector.size),
        );

        for (index, variable) in removed_items.iter().enumerate() {
            let target_index = self.brillig_context.make_usize_constant_instruction(index.into());
            self.brillig_context.memory_op_instruction(
                target_index.address,
                target_vector.size,
                target_index.address,
                BrilligBinaryOp::Add,
            );
            self.retrieve_variable_from_array(source_vector.pointer, target_index, *variable);
            self.brillig_context.deallocate_single_addr(target_index);
        }
    }

    pub(crate) fn slice_insert_operation(
        &mut self,
        target_vector: BrilligVector,
        source_vector: BrilligVector,
        index: SingleAddrVariable,
        items: &[BrilligVariable],
    ) {
        // First we need to allocate the target vector incrementing the size by items.len()
        self.brillig_context.codegen_usize_op(
            source_vector.size,
            target_vector.size,
            BrilligBinaryOp::Add,
            items.len(),
        );
        self.brillig_context.codegen_allocate_array(target_vector.pointer, target_vector.size);
        // We initialize the RC of the target vector to 1
        self.brillig_context.usize_const_instruction(target_vector.rc, 1_usize.into());

        // Copy the elements to the left of the index
        self.brillig_context.codegen_copy_array(
            source_vector.pointer,
            target_vector.pointer,
            index,
        );

        // Compute the source pointer just at the index
        let source_pointer_at_index = self.brillig_context.allocate_register();
        self.brillig_context.memory_op_instruction(
            source_vector.pointer,
            index.address,
            source_pointer_at_index,
            BrilligBinaryOp::Add,
        );

        // Compute the target pointer after the inserted elements
        let target_pointer_after_index = self.brillig_context.allocate_register();
        self.brillig_context.memory_op_instruction(
            target_vector.pointer,
            index.address,
            target_pointer_after_index,
            BrilligBinaryOp::Add,
        );
        self.brillig_context.codegen_usize_op_in_place(
            target_pointer_after_index,
            BrilligBinaryOp::Add,
            items.len(),
        );

        // Compute the number of elements to the right of the index
        let item_count = self.brillig_context.allocate_register();
        self.brillig_context.memory_op_instruction(
            source_vector.size,
            index.address,
            item_count,
            BrilligBinaryOp::Sub,
        );

        // Copy the elements to the right of the index
        self.brillig_context.codegen_copy_array(
            source_pointer_at_index,
            target_pointer_after_index,
            SingleAddrVariable::new_usize(item_count),
        );

        // Write the items to insert starting at the index
        for (subitem_index, variable) in items.iter().enumerate() {
            let target_index =
                self.brillig_context.make_usize_constant_instruction(subitem_index.into());
            self.brillig_context.memory_op_instruction(
                target_index.address,
                index.address,
                target_index.address,
                BrilligBinaryOp::Add,
            );
            self.store_variable_in_array(target_vector.pointer, target_index, *variable);
            self.brillig_context.deallocate_single_addr(target_index);
        }

        self.brillig_context.deallocate_register(source_pointer_at_index);
        self.brillig_context.deallocate_register(target_pointer_after_index);
        self.brillig_context.deallocate_register(item_count);
    }

    pub(crate) fn slice_remove_operation(
        &mut self,
        target_vector: BrilligVector,
        source_vector: BrilligVector,
        index: SingleAddrVariable,
        removed_items: &[BrilligVariable],
    ) {
        // First we need to allocate the target vector decrementing the size by removed_items.len()
        self.brillig_context.codegen_usize_op(
            source_vector.size,
            target_vector.size,
            BrilligBinaryOp::Sub,
            removed_items.len(),
        );
        self.brillig_context.codegen_allocate_array(target_vector.pointer, target_vector.size);
        // We initialize the RC of the target vector to 1
        self.brillig_context.usize_const_instruction(target_vector.rc, 1_usize.into());

        // Copy the elements to the left of the index
        self.brillig_context.codegen_copy_array(
            source_vector.pointer,
            target_vector.pointer,
            index,
        );

        // Compute the source pointer after the removed items
        let source_pointer_after_index = self.brillig_context.allocate_register();
        self.brillig_context.memory_op_instruction(
            source_vector.pointer,
            index.address,
            source_pointer_after_index,
            BrilligBinaryOp::Add,
        );
        self.brillig_context.codegen_usize_op_in_place(
            source_pointer_after_index,
            BrilligBinaryOp::Add,
            removed_items.len(),
        );

        // Compute the target pointer at the index
        let target_pointer_at_index = self.brillig_context.allocate_register();
        self.brillig_context.memory_op_instruction(
            target_vector.pointer,
            index.address,
            target_pointer_at_index,
            BrilligBinaryOp::Add,
        );

        // Compute the number of elements to the right of the index
        let item_count = self.brillig_context.allocate_register();
        self.brillig_context.memory_op_instruction(
            source_vector.size,
            index.address,
            item_count,
            BrilligBinaryOp::Sub,
        );
        self.brillig_context.codegen_usize_op_in_place(
            item_count,
            BrilligBinaryOp::Sub,
            removed_items.len(),
        );

        // Copy the elements to the right of the index
        self.brillig_context.codegen_copy_array(
            source_pointer_after_index,
            target_pointer_at_index,
            SingleAddrVariable::new_usize(item_count),
        );

        // Get the removed items
        for (subitem_index, variable) in removed_items.iter().enumerate() {
            let target_index =
                self.brillig_context.make_usize_constant_instruction(subitem_index.into());
            self.brillig_context.memory_op_instruction(
                target_index.address,
                index.address,
                target_index.address,
                BrilligBinaryOp::Add,
            );
            self.retrieve_variable_from_array(source_vector.pointer, target_index, *variable);
            self.brillig_context.deallocate_single_addr(target_index);
        }

        self.brillig_context.deallocate_register(source_pointer_after_index);
        self.brillig_context.deallocate_register(target_pointer_at_index);
        self.brillig_context.deallocate_register(item_count);
    }

    pub(crate) fn convert_array_or_vector_to_vector(
        &mut self,
        source_variable: BrilligVariable,
    ) -> BrilligVector {
        match source_variable {
            BrilligVariable::BrilligVector(source_vector) => source_vector,
            BrilligVariable::BrilligArray(source_array) => {
                self.brillig_context.array_to_vector_instruction(&source_array)
            }
            _ => unreachable!("ICE: unsupported slice push back source {:?}", source_variable),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use acvm::FieldElement;

    use crate::brillig::brillig_gen::brillig_block::BrilligBlock;
    use crate::brillig::brillig_gen::brillig_block_variables::BlockVariables;
    use crate::brillig::brillig_gen::brillig_fn::FunctionContext;
    use crate::brillig::brillig_ir::artifact::BrilligParameter;
    use crate::brillig::brillig_ir::brillig_variable::{
        BrilligArray, BrilligVariable, BrilligVector, SingleAddrVariable,
    };
    use crate::brillig::brillig_ir::tests::{
        create_and_run_vm, create_context, create_entry_point_bytecode,
    };
    use crate::brillig::brillig_ir::{BrilligContext, BRILLIG_MEMORY_ADDRESSING_BIT_SIZE};
    use crate::ssa::function_builder::FunctionBuilder;
    use crate::ssa::ir::function::RuntimeType;
    use crate::ssa::ir::map::Id;
    use crate::ssa::ssa_gen::Ssa;

    fn create_test_environment() -> (Ssa, FunctionContext, BrilligContext) {
        let mut builder = FunctionBuilder::new("main".to_string(), Id::test_new(0));
        builder.set_runtime(RuntimeType::Brillig);

        let ssa = builder.finish();
        let mut brillig_context = create_context();

        let function_context = FunctionContext::new(ssa.main(), &mut brillig_context);
        (ssa, function_context, brillig_context)
    }

    fn create_brillig_block<'a>(
        function_context: &'a mut FunctionContext,
        brillig_context: &'a mut BrilligContext,
    ) -> BrilligBlock<'a> {
        let variables = BlockVariables::default();
        BrilligBlock {
            function_context,
            block_id: Id::test_new(0),
            brillig_context,
            variables,
            last_uses: Default::default(),
        }
    }

    #[test]
    fn test_slice_push_operation() {
        fn test_case_push(
            push_back: bool,
            array: Vec<FieldElement>,
            item_to_push: FieldElement,
            expected_return: Vec<FieldElement>,
        ) {
            let arguments = vec![
                BrilligParameter::Array(
                    vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                    array.len(),
                ),
                BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
            ];
            let returns = vec![BrilligParameter::Array(
                vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                array.len() + 1,
            )];

            let (_, mut function_context, mut context) = create_test_environment();

            // Allocate the parameters
            let array_variable = BrilligArray {
                pointer: context.allocate_register(),
                size: array.len(),
                rc: context.allocate_register(),
            };
            let item_to_insert = SingleAddrVariable {
                address: context.allocate_register(),
                bit_size: BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
            };

            // Cast the source array to a vector
            let source_vector = context.array_to_vector_instruction(&array_variable);

            // Allocate the results
            let target_vector = BrilligVector {
                pointer: context.allocate_register(),
                size: context.allocate_register(),
                rc: context.allocate_register(),
            };

            let mut block = create_brillig_block(&mut function_context, &mut context);

            if push_back {
                block.slice_push_back_operation(
                    target_vector,
                    source_vector,
                    &[BrilligVariable::SingleAddr(item_to_insert)],
                );
            } else {
                block.slice_push_front_operation(
                    target_vector,
                    source_vector,
                    &[BrilligVariable::SingleAddr(item_to_insert)],
                );
            }

            context.codegen_return(&[target_vector.pointer, target_vector.rc]);

            let bytecode = create_entry_point_bytecode(context, arguments, returns).byte_code;
            let (vm, return_data_offset, return_data_size) =
                create_and_run_vm(array.into_iter().chain(vec![item_to_push]).collect(), &bytecode);
            assert_eq!(return_data_size, expected_return.len());
            assert_eq!(
                vm.get_memory()[return_data_offset..(return_data_offset + expected_return.len())]
                    .iter()
                    .map(|mem_val| mem_val.to_field())
                    .collect::<Vec<_>>(),
                expected_return
            );
        }

        test_case_push(
            true,
            vec![
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(3_usize),
            ],
            FieldElement::from(27_usize),
            vec![
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(3_usize),
                FieldElement::from(27_usize),
            ],
        );
        test_case_push(
            true,
            vec![],
            FieldElement::from(27_usize),
            vec![FieldElement::from(27_usize)],
        );
        test_case_push(
            false,
            vec![
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(3_usize),
            ],
            FieldElement::from(27_usize),
            vec![
                FieldElement::from(27_usize),
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(3_usize),
            ],
        );
        test_case_push(
            false,
            vec![],
            FieldElement::from(27_usize),
            vec![FieldElement::from(27_usize)],
        );
    }

    #[test]
    fn test_slice_pop_back_operation() {
        fn test_case_pop(
            pop_back: bool,
            array: Vec<FieldElement>,
            expected_return_array: Vec<FieldElement>,
            expected_return_item: FieldElement,
        ) {
            let arguments = vec![BrilligParameter::Array(
                vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                array.len(),
            )];
            let returns = vec![
                BrilligParameter::Array(
                    vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                    array.len() - 1,
                ),
                BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
            ];

            let (_, mut function_context, mut context) = create_test_environment();

            // Allocate the parameters
            let array_variable = BrilligArray {
                pointer: context.allocate_register(),
                size: array.len(),
                rc: context.allocate_register(),
            };

            // Cast the source array to a vector
            let source_vector = context.array_to_vector_instruction(&array_variable);

            // Allocate the results
            let target_vector = BrilligVector {
                pointer: context.allocate_register(),
                size: context.allocate_register(),
                rc: context.allocate_register(),
            };
            let removed_item = SingleAddrVariable {
                address: context.allocate_register(),
                bit_size: BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
            };

            let mut block = create_brillig_block(&mut function_context, &mut context);

            if pop_back {
                block.slice_pop_back_operation(
                    target_vector,
                    source_vector,
                    &[BrilligVariable::SingleAddr(removed_item)],
                );
            } else {
                block.slice_pop_front_operation(
                    target_vector,
                    source_vector,
                    &[BrilligVariable::SingleAddr(removed_item)],
                );
            }

            context.codegen_return(&[
                target_vector.pointer,
                target_vector.rc,
                removed_item.address,
            ]);

            let bytecode = create_entry_point_bytecode(context, arguments, returns).byte_code;
            let expected_return: Vec<_> =
                expected_return_array.into_iter().chain(vec![expected_return_item]).collect();
            let (vm, return_data_offset, return_data_size) =
                create_and_run_vm(array.clone(), &bytecode);
            assert_eq!(return_data_size, expected_return.len());

            assert_eq!(
                vm.get_memory()[return_data_offset..(return_data_offset + expected_return.len())]
                    .iter()
                    .map(|mem_val| mem_val.to_field())
                    .collect::<Vec<_>>(),
                expected_return
            );
        }

        test_case_pop(
            true,
            vec![
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(3_usize),
            ],
            vec![FieldElement::from(1_usize), FieldElement::from(2_usize)],
            FieldElement::from(3_usize),
        );
        test_case_pop(true, vec![FieldElement::from(1_usize)], vec![], FieldElement::from(1_usize));
        test_case_pop(
            false,
            vec![
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(3_usize),
            ],
            vec![FieldElement::from(2_usize), FieldElement::from(3_usize)],
            FieldElement::from(1_usize),
        );
    }

    #[test]
    fn test_slice_insert_operation() {
        fn test_case_insert(
            array: Vec<FieldElement>,
            item: FieldElement,
            index: FieldElement,
            expected_return: Vec<FieldElement>,
        ) {
            let arguments = vec![
                BrilligParameter::Array(
                    vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                    array.len(),
                ),
                BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
                BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
            ];
            let returns = vec![BrilligParameter::Array(
                vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                array.len() + 1,
            )];

            let (_, mut function_context, mut context) = create_test_environment();

            // Allocate the parameters
            let array_variable = BrilligArray {
                pointer: context.allocate_register(),
                size: array.len(),
                rc: context.allocate_register(),
            };
            let item_to_insert = SingleAddrVariable {
                address: context.allocate_register(),
                bit_size: BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
            };
            let index_to_insert = SingleAddrVariable::new(
                context.allocate_register(),
                BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
            );

            // Cast the source array to a vector
            let source_vector = context.array_to_vector_instruction(&array_variable);

            // Allocate the results
            let target_vector = BrilligVector {
                pointer: context.allocate_register(),
                size: context.allocate_register(),
                rc: context.allocate_register(),
            };

            let mut block = create_brillig_block(&mut function_context, &mut context);

            block.slice_insert_operation(
                target_vector,
                source_vector,
                index_to_insert,
                &[BrilligVariable::SingleAddr(item_to_insert)],
            );

            context.codegen_return(&[target_vector.pointer, target_vector.rc]);
            let calldata = array.into_iter().chain(vec![item]).chain(vec![index]).collect();

            let bytecode = create_entry_point_bytecode(context, arguments, returns).byte_code;
            let (vm, return_data_offset, return_data_size) = create_and_run_vm(calldata, &bytecode);
            assert_eq!(return_data_size, expected_return.len());

            assert_eq!(
                vm.get_memory()[return_data_offset..(return_data_offset + expected_return.len())]
                    .iter()
                    .map(|mem_val| mem_val.to_field())
                    .collect::<Vec<_>>(),
                expected_return
            );
        }

        test_case_insert(
            vec![
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(3_usize),
            ],
            FieldElement::from(27_usize),
            FieldElement::from(1_usize),
            vec![
                FieldElement::from(1_usize),
                FieldElement::from(27_usize),
                FieldElement::from(2_usize),
                FieldElement::from(3_usize),
            ],
        );

        test_case_insert(
            vec![
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(3_usize),
            ],
            FieldElement::from(27_usize),
            FieldElement::from(0_usize),
            vec![
                FieldElement::from(27_usize),
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(3_usize),
            ],
        );
        test_case_insert(
            vec![
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(3_usize),
            ],
            FieldElement::from(27_usize),
            FieldElement::from(2_usize),
            vec![
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(27_usize),
                FieldElement::from(3_usize),
            ],
        );
        test_case_insert(
            vec![
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(3_usize),
            ],
            FieldElement::from(27_usize),
            FieldElement::from(3_usize),
            vec![
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(3_usize),
                FieldElement::from(27_usize),
            ],
        );
        test_case_insert(
            vec![],
            FieldElement::from(27_usize),
            FieldElement::from(0_usize),
            vec![FieldElement::from(27_usize)],
        );
    }

    #[test]
    fn test_slice_remove_operation() {
        fn test_case_remove(
            array: Vec<FieldElement>,
            index: FieldElement,
            expected_array: Vec<FieldElement>,
            expected_removed_item: FieldElement,
        ) {
            let arguments = vec![
                BrilligParameter::Array(
                    vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                    array.len(),
                ),
                BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
            ];
            let returns = vec![
                BrilligParameter::Array(
                    vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                    array.len() - 1,
                ),
                BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
            ];

            let (_, mut function_context, mut context) = create_test_environment();

            // Allocate the parameters
            let array_variable = BrilligArray {
                pointer: context.allocate_register(),
                size: array.len(),
                rc: context.allocate_register(),
            };
            let index_to_insert = SingleAddrVariable::new(
                context.allocate_register(),
                BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
            );

            // Cast the source array to a vector
            let source_vector = context.array_to_vector_instruction(&array_variable);

            // Allocate the results
            let target_vector = BrilligVector {
                pointer: context.allocate_register(),
                size: context.allocate_register(),
                rc: context.allocate_register(),
            };
            let removed_item = SingleAddrVariable {
                address: context.allocate_register(),
                bit_size: BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
            };

            let mut block = create_brillig_block(&mut function_context, &mut context);

            block.slice_remove_operation(
                target_vector,
                source_vector,
                index_to_insert,
                &[BrilligVariable::SingleAddr(removed_item)],
            );

            context.codegen_return(&[
                target_vector.pointer,
                target_vector.size,
                removed_item.address,
            ]);

            let calldata: Vec<_> = array.into_iter().chain(vec![index]).collect();

            let bytecode = create_entry_point_bytecode(context, arguments, returns).byte_code;
            let (vm, return_data_offset, return_data_size) = create_and_run_vm(calldata, &bytecode);

            let expected_return: Vec<_> =
                expected_array.into_iter().chain(vec![expected_removed_item]).collect();
            assert_eq!(return_data_size, expected_return.len());

            assert_eq!(
                vm.get_memory()[return_data_offset..(return_data_offset + expected_return.len())]
                    .iter()
                    .map(|mem_val| mem_val.to_field())
                    .collect::<Vec<_>>(),
                expected_return
            );
        }

        test_case_remove(
            vec![
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(3_usize),
            ],
            FieldElement::from(0_usize),
            vec![FieldElement::from(2_usize), FieldElement::from(3_usize)],
            FieldElement::from(1_usize),
        );

        test_case_remove(
            vec![
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(3_usize),
            ],
            FieldElement::from(1_usize),
            vec![FieldElement::from(1_usize), FieldElement::from(3_usize)],
            FieldElement::from(2_usize),
        );

        test_case_remove(
            vec![
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(3_usize),
            ],
            FieldElement::from(2_usize),
            vec![FieldElement::from(1_usize), FieldElement::from(2_usize)],
            FieldElement::from(3_usize),
        );
        test_case_remove(
            vec![FieldElement::from(1_usize)],
            FieldElement::from(0_usize),
            vec![],
            FieldElement::from(1_usize),
        );
    }
}
