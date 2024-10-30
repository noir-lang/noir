use acvm::acir::brillig::MemoryAddress;

use crate::brillig::brillig_ir::{
    brillig_variable::{BrilligVariable, BrilligVector, SingleAddrVariable},
    BrilligBinaryOp,
};

use super::brillig_block::BrilligBlock;

impl<'block> BrilligBlock<'block> {
    fn write_variables(&mut self, write_pointer: MemoryAddress, variables: &[BrilligVariable]) {
        for (index, variable) in variables.iter().enumerate() {
            self.brillig_context.store_instruction(write_pointer, variable.extract_register());
            if index != variables.len() - 1 {
                self.brillig_context.codegen_usize_op_in_place(
                    write_pointer,
                    BrilligBinaryOp::Add,
                    1,
                );
            }
        }
    }

    pub(crate) fn slice_push_back_operation(
        &mut self,
        target_vector: BrilligVector,
        source_vector: BrilligVector,
        variables_to_insert: &[BrilligVariable],
    ) {
        let write_pointer = self.brillig_context.allocate_register();
        self.brillig_context.call_prepare_vector_push_procedure(
            source_vector,
            target_vector,
            write_pointer,
            variables_to_insert.len(),
            true,
        );

        self.write_variables(write_pointer, variables_to_insert);

        self.brillig_context.deallocate_register(write_pointer);
    }

    pub(crate) fn slice_push_front_operation(
        &mut self,
        target_vector: BrilligVector,
        source_vector: BrilligVector,
        variables_to_insert: &[BrilligVariable],
    ) {
        let write_pointer = self.brillig_context.allocate_register();
        self.brillig_context.call_prepare_vector_push_procedure(
            source_vector,
            target_vector,
            write_pointer,
            variables_to_insert.len(),
            false,
        );

        self.write_variables(write_pointer, variables_to_insert);
        self.brillig_context.deallocate_register(write_pointer);
    }

    fn read_variables(&mut self, read_pointer: MemoryAddress, variables: &[BrilligVariable]) {
        for (index, variable) in variables.iter().enumerate() {
            self.brillig_context.load_instruction(variable.extract_register(), read_pointer);
            if index != variables.len() - 1 {
                self.brillig_context.codegen_usize_op_in_place(
                    read_pointer,
                    BrilligBinaryOp::Add,
                    1,
                );
            }
        }
    }

    pub(crate) fn slice_pop_front_operation(
        &mut self,
        target_vector: BrilligVector,
        source_vector: BrilligVector,
        removed_items: &[BrilligVariable],
    ) {
        let read_pointer = self.brillig_context.codegen_make_vector_items_pointer(source_vector);
        self.read_variables(read_pointer, removed_items);
        self.brillig_context.deallocate_register(read_pointer);

        self.brillig_context.call_vector_pop_front_procedure(
            source_vector,
            target_vector,
            removed_items.len(),
        );
    }

    pub(crate) fn slice_pop_back_operation(
        &mut self,
        target_vector: BrilligVector,
        source_vector: BrilligVector,
        removed_items: &[BrilligVariable],
    ) {
        let read_pointer = self.brillig_context.allocate_register();
        self.brillig_context.call_vector_pop_back_procedure(
            source_vector,
            target_vector,
            read_pointer,
            removed_items.len(),
        );

        self.read_variables(read_pointer, removed_items);
        self.brillig_context.deallocate_register(read_pointer);
    }

    pub(crate) fn slice_insert_operation(
        &mut self,
        target_vector: BrilligVector,
        source_vector: BrilligVector,
        index: SingleAddrVariable,
        items: &[BrilligVariable],
    ) {
        let write_pointer = self.brillig_context.allocate_register();

        self.brillig_context.call_prepare_vector_insert_procedure(
            source_vector,
            target_vector,
            index,
            write_pointer,
            items.len(),
        );

        self.write_variables(write_pointer, items);
        self.brillig_context.deallocate_register(write_pointer);
    }

    pub(crate) fn slice_remove_operation(
        &mut self,
        target_vector: BrilligVector,
        source_vector: BrilligVector,
        index: SingleAddrVariable,
        removed_items: &[BrilligVariable],
    ) {
        let read_pointer = self.brillig_context.codegen_make_vector_items_pointer(source_vector);
        self.brillig_context.memory_op_instruction(
            read_pointer,
            index.address,
            read_pointer,
            BrilligBinaryOp::Add,
        );
        self.read_variables(read_pointer, removed_items);

        self.brillig_context.call_vector_remove_procedure(
            source_vector,
            target_vector,
            index,
            removed_items.len(),
        );
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use acvm::FieldElement;
    use noirc_frontend::monomorphization::ast::InlineType;

    use crate::brillig::brillig_gen::brillig_block::BrilligBlock;
    use crate::brillig::brillig_gen::brillig_block_variables::BlockVariables;
    use crate::brillig::brillig_gen::brillig_fn::FunctionContext;
    use crate::brillig::brillig_ir::artifact::{BrilligParameter, Label};
    use crate::brillig::brillig_ir::brillig_variable::{
        BrilligVariable, BrilligVector, SingleAddrVariable,
    };
    use crate::brillig::brillig_ir::registers::Stack;
    use crate::brillig::brillig_ir::tests::{
        create_and_run_vm, create_context, create_entry_point_bytecode,
    };
    use crate::brillig::brillig_ir::{BrilligContext, BRILLIG_MEMORY_ADDRESSING_BIT_SIZE};
    use crate::ssa::function_builder::FunctionBuilder;
    use crate::ssa::ir::function::RuntimeType;
    use crate::ssa::ir::map::Id;
    use crate::ssa::ssa_gen::Ssa;

    fn create_test_environment() -> (Ssa, FunctionContext, BrilligContext<FieldElement, Stack>) {
        let mut builder = FunctionBuilder::new("main".to_string(), Id::test_new(0));
        builder.set_runtime(RuntimeType::Brillig(InlineType::default()));

        let ssa = builder.finish();
        let mut brillig_context = create_context(ssa.main_id);
        brillig_context.enter_context(Label::block(ssa.main_id, Id::test_new(0)));

        let function_context = FunctionContext::new(ssa.main());
        (ssa, function_context, brillig_context)
    }

    fn create_brillig_block<'a>(
        function_context: &'a mut FunctionContext,
        brillig_context: &'a mut BrilligContext<FieldElement, Stack>,
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
                BrilligParameter::Slice(
                    vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                    array.len(),
                ),
                BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
            ];
            let result_length = array.len() + 1;
            assert_eq!(result_length, expected_return.len());
            let result_length_with_metadata = result_length + 2; // Leading length and capacity

            // Entry points don't support returning slices, so we implicitly cast the vector to an array
            // With the metadata at the start.
            let returns = vec![BrilligParameter::Array(
                vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                result_length_with_metadata,
            )];

            let (_, mut function_context, mut context) = create_test_environment();

            // Allocate the parameters
            let source_vector = BrilligVector { pointer: context.allocate_register() };
            let item_to_insert = SingleAddrVariable {
                address: context.allocate_register(),
                bit_size: BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
            };

            // Allocate the results
            let target_vector = BrilligVector { pointer: context.allocate_register() };

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

            context.codegen_return(&[target_vector.pointer]);

            let bytecode = create_entry_point_bytecode(context, arguments, returns).byte_code;
            let (vm, return_data_offset, return_data_size) =
                create_and_run_vm(array.into_iter().chain(vec![item_to_push]).collect(), &bytecode);
            assert_eq!(return_data_size, result_length_with_metadata);
            let mut returned_vector: Vec<FieldElement> = vm.get_memory()
                [return_data_offset..(return_data_offset + result_length_with_metadata)]
                .iter()
                .map(|mem_val| mem_val.to_field())
                .collect();
            let returned_size = returned_vector.remove(0);
            assert_eq!(returned_size, result_length.into());
            let _returned_capacity = returned_vector.remove(0);

            assert_eq!(returned_vector, expected_return);
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
            let arguments = vec![BrilligParameter::Slice(
                vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                array.len(),
            )];
            let result_length = array.len() - 1;
            assert_eq!(result_length, expected_return_array.len());
            let result_length_with_metadata = result_length + 2; // Leading length and capacity

            // Entry points don't support returning slices, so we implicitly cast the vector to an array
            // With the metadata at the start.
            let returns = vec![
                BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
                BrilligParameter::Array(
                    vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                    result_length_with_metadata,
                ),
            ];

            let (_, mut function_context, mut context) = create_test_environment();

            // Allocate the parameters
            let source_vector = BrilligVector { pointer: context.allocate_register() };

            // Allocate the results
            let target_vector = BrilligVector { pointer: context.allocate_register() };
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

            context.codegen_return(&[removed_item.address, target_vector.pointer]);

            let bytecode = create_entry_point_bytecode(context, arguments, returns).byte_code;

            let (vm, return_data_offset, return_data_size) =
                create_and_run_vm(array.clone(), &bytecode);
            // vector + removed item
            assert_eq!(return_data_size, result_length_with_metadata + 1);

            let mut return_data: Vec<FieldElement> = vm.get_memory()
                [return_data_offset..(return_data_offset + return_data_size)]
                .iter()
                .map(|mem_val| mem_val.to_field())
                .collect();
            let returned_item = return_data.remove(0);
            assert_eq!(returned_item, expected_return_item);

            let returned_size = return_data.remove(0);
            assert_eq!(returned_size, result_length.into());
            let _returned_capacity = return_data.remove(0);

            assert_eq!(return_data, expected_return_array);
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
                BrilligParameter::Slice(
                    vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                    array.len(),
                ),
                BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
                BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
            ];
            let result_length = array.len() + 1;
            assert_eq!(result_length, expected_return.len());
            let result_length_with_metadata = result_length + 2; // Leading length and capacity

            // Entry points don't support returning slices, so we implicitly cast the vector to an array
            // With the metadata at the start.
            let returns = vec![BrilligParameter::Array(
                vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                result_length_with_metadata,
            )];

            let (_, mut function_context, mut context) = create_test_environment();

            // Allocate the parameters
            let source_vector = BrilligVector { pointer: context.allocate_register() };
            let item_to_insert = SingleAddrVariable {
                address: context.allocate_register(),
                bit_size: BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
            };
            let index_to_insert = SingleAddrVariable::new(
                context.allocate_register(),
                BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
            );

            // Allocate the results
            let target_vector = BrilligVector { pointer: context.allocate_register() };

            let mut block = create_brillig_block(&mut function_context, &mut context);

            block.slice_insert_operation(
                target_vector,
                source_vector,
                index_to_insert,
                &[BrilligVariable::SingleAddr(item_to_insert)],
            );

            context.codegen_return(&[target_vector.pointer]);
            let calldata = array.into_iter().chain(vec![item]).chain(vec![index]).collect();

            let bytecode = create_entry_point_bytecode(context, arguments, returns).byte_code;
            let (vm, return_data_offset, return_data_size) = create_and_run_vm(calldata, &bytecode);
            assert_eq!(return_data_size, result_length_with_metadata);

            let mut returned_vector: Vec<FieldElement> = vm.get_memory()
                [return_data_offset..(return_data_offset + result_length_with_metadata)]
                .iter()
                .map(|mem_val| mem_val.to_field())
                .collect();
            let returned_size = returned_vector.remove(0);
            assert_eq!(returned_size, result_length.into());
            let _returned_capacity = returned_vector.remove(0);

            assert_eq!(returned_vector, expected_return);
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
                BrilligParameter::Slice(
                    vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                    array.len(),
                ),
                BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
            ];
            let result_length = array.len() - 1;
            assert_eq!(result_length, expected_array.len());
            let result_length_with_metadata = result_length + 2; // Leading length and capacity

            let returns = vec![
                BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
                BrilligParameter::Array(
                    vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                    result_length_with_metadata,
                ),
            ];

            let (_, mut function_context, mut context) = create_test_environment();

            // Allocate the parameters
            let source_vector = BrilligVector { pointer: context.allocate_register() };
            let index_to_insert = SingleAddrVariable::new(
                context.allocate_register(),
                BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
            );

            // Allocate the results
            let target_vector = BrilligVector { pointer: context.allocate_register() };
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

            context.codegen_return(&[removed_item.address, target_vector.pointer]);

            let calldata: Vec<_> = array.into_iter().chain(vec![index]).collect();

            let bytecode = create_entry_point_bytecode(context, arguments, returns).byte_code;
            let (vm, return_data_offset, return_data_size) = create_and_run_vm(calldata, &bytecode);

            // vector + removed item
            assert_eq!(return_data_size, result_length_with_metadata + 1);

            let mut return_data: Vec<FieldElement> = vm.get_memory()
                [return_data_offset..(return_data_offset + return_data_size)]
                .iter()
                .map(|mem_val| mem_val.to_field())
                .collect();
            let returned_item = return_data.remove(0);
            assert_eq!(returned_item, expected_removed_item);

            let returned_size = return_data.remove(0);
            assert_eq!(returned_size, result_length.into());
            let _returned_capacity = return_data.remove(0);

            assert_eq!(return_data, expected_array);
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
