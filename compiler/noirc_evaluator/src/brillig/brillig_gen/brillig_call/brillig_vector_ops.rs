//! Codegen for converting SSA vector intrinsic functions to Brillig bytecode.
use acvm::acir::brillig::MemoryAddress;

use crate::brillig::brillig_ir::{
    BrilligBinaryOp,
    brillig_variable::{BrilligVariable, BrilligVector, SingleAddrVariable},
    registers::RegisterAllocator,
};

use super::super::brillig_block::BrilligBlock;

impl<Registers: RegisterAllocator> BrilligBlock<'_, Registers> {
    /// Take a list of [BrilligVariable] and copy the memory they point at to the `write_pointer`,
    /// increasing the address by 1 between each variable.
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

    /// Prepare a vector for pushing a number of new (flattened) items to the back,
    /// then write those variables to the returned write pointer.
    pub(crate) fn vector_push_back_operation(
        &mut self,
        target_vector: BrilligVector,
        source_len: SingleAddrVariable,
        source_vector: BrilligVector,
        variables_to_insert: &[BrilligVariable],
    ) {
        let write_pointer = self.brillig_context.allocate_register();
        self.brillig_context.call_prepare_vector_push_procedure(
            source_len,
            source_vector,
            target_vector,
            *write_pointer,
            variables_to_insert.len(),
            true,
        );

        self.write_variables(*write_pointer, variables_to_insert);
    }

    /// Prepare a vector for pushing a number of new (flattened) items to the front,
    /// then write those variables to the returned write pointer.
    pub(crate) fn vector_push_front_operation(
        &mut self,
        target_vector: BrilligVector,
        source_len: SingleAddrVariable,
        source_vector: BrilligVector,
        variables_to_insert: &[BrilligVariable],
    ) {
        let write_pointer = self.brillig_context.allocate_register();
        self.brillig_context.call_prepare_vector_push_procedure(
            source_len,
            source_vector,
            target_vector,
            *write_pointer,
            variables_to_insert.len(),
            false,
        );

        self.write_variables(*write_pointer, variables_to_insert);
    }

    /// Read the memory starting at the pointer into successive variables.
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

    /// Read the popped number of variables from the front of the source vector,
    /// then create the target vector from the source by skipping the number of popped items.
    pub(crate) fn vector_pop_front_operation(
        &mut self,
        target_vector: BrilligVector,
        source_len: SingleAddrVariable,
        source_vector: BrilligVector,
        removed_items: &[BrilligVariable],
    ) {
        let read_pointer = self.brillig_context.codegen_make_vector_items_pointer(source_vector);
        self.read_variables(*read_pointer, removed_items);

        self.brillig_context.call_vector_pop_front_procedure(
            source_len,
            source_vector,
            target_vector,
            removed_items.len(),
        );
    }

    /// Create new vector with a number of (flattened) items popped from the back,
    /// then read the popped items into the variables representing the removed items.
    pub(crate) fn vector_pop_back_operation(
        &mut self,
        target_vector: BrilligVector,
        source_len: SingleAddrVariable,
        source_vector: BrilligVector,
        removed_items: &[BrilligVariable],
    ) {
        let read_pointer = self.brillig_context.allocate_register();
        self.brillig_context.call_vector_pop_back_procedure(
            source_len,
            source_vector,
            target_vector,
            *read_pointer,
            removed_items.len(),
        );

        self.read_variables(*read_pointer, removed_items);
    }

    /// Prepare a vector for inserting a number of (flattened) items at a specific index
    /// by making a hole for them, then write the variables to the returned write pointer.
    pub(crate) fn vector_insert_operation(
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
            *write_pointer,
            items.len(),
        );

        self.write_variables(*write_pointer, items);
    }

    /// Read a number of (flattened) items at a specific index of a vector into the variables
    /// representing the removed items, then create a new vector with the same number of
    /// items removed and subsequent items shifted to the left.
    pub(crate) fn vector_remove_operation(
        &mut self,
        target_vector: BrilligVector,
        source_vector: BrilligVector,
        index: SingleAddrVariable,
        removed_items: &[BrilligVariable],
    ) {
        let read_pointer = self.brillig_context.codegen_make_vector_items_pointer(source_vector);
        self.brillig_context.memory_op_instruction(
            *read_pointer,
            index.address,
            *read_pointer,
            BrilligBinaryOp::Add,
        );
        self.read_variables(*read_pointer, removed_items);

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
    use acvm::acir::brillig::lengths::SemanticLength;
    use noirc_frontend::monomorphization::ast::InlineType;
    use rustc_hash::FxHashMap as HashMap;

    use crate::brillig::ValueId;
    use crate::brillig::brillig_gen::brillig_block::BrilligBlock;
    use crate::brillig::brillig_gen::brillig_block_variables::BlockVariables;
    use crate::brillig::brillig_gen::brillig_fn::FunctionContext;
    use crate::brillig::brillig_ir::artifact::{BrilligParameter, Label};
    use crate::brillig::brillig_ir::brillig_variable::BrilligVariable;
    use crate::brillig::brillig_ir::registers::Stack;
    use crate::brillig::brillig_ir::tests::{
        create_and_run_vm, create_context, create_entry_point_bytecode,
    };
    use crate::brillig::brillig_ir::{BRILLIG_MEMORY_ADDRESSING_BIT_SIZE, BrilligContext};
    use crate::ssa::function_builder::FunctionBuilder;
    use crate::ssa::ir::function::RuntimeType;
    use crate::ssa::ir::map::Id;
    use crate::ssa::ir::types::NumericType;
    use crate::ssa::ssa_gen::Ssa;

    fn create_test_environment() -> (Ssa, FunctionContext, BrilligContext<FieldElement, Stack>) {
        let mut builder = FunctionBuilder::new("main".to_string(), Id::test_new(0));
        builder.set_runtime(RuntimeType::Brillig(InlineType::default()));
        builder.terminate_with_return(vec![]);
        let ssa = builder.finish();
        let mut brillig_context = create_context(ssa.main_id);
        brillig_context.enter_context(Label::block(ssa.main_id, Id::test_new(0)));

        let function_context = FunctionContext::new(ssa.main(), true);
        (ssa, function_context, brillig_context)
    }

    fn create_brillig_block<'a>(
        function_context: &'a mut FunctionContext,
        brillig_context: &'a mut BrilligContext<FieldElement, Stack>,
        globals: &'a HashMap<ValueId, BrilligVariable>,
        hoisted_global_constants: &'a HashMap<(FieldElement, NumericType), BrilligVariable>,
    ) -> BrilligBlock<'a, Stack> {
        let variables = BlockVariables::default();
        BrilligBlock {
            function_context,
            block_id: Id::test_new(0),
            brillig_context,
            variables,
            last_uses: Default::default(),
            globals,
            hoisted_global_constants,
            building_globals: false,
        }
    }

    #[test]
    fn test_vector_push_operation() {
        fn test_case_push(
            push_back: bool,
            source_len: usize,
            source: Vec<FieldElement>,
            item_to_push: FieldElement,
            expected_return: Vec<FieldElement>,
        ) {
            // Types of arguments passed to the entry point.
            let arguments = vec![
                // The input size semantic length.
                BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
                // The input vector of the array items, with a known size/capacity.
                BrilligParameter::Vector(
                    vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                    SemanticLength(source.len() as u32),
                ),
                // The item to push.
                BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
            ];
            // We expect 1 item to be pushed. The capacity might be more, depending on reuse.
            let result_length = source_len + 1;
            assert_eq!(
                result_length,
                expected_return.len(),
                "expected return data should be 1 longer than the input"
            );
            // Returned data expected to be leading with length and capacity, followed by the items.
            let result_length_with_metadata = result_length + 2;

            // Entry points don't support returning vectors, so we implicitly cast the vector to an array
            // With the metadata at the start.
            let returns = vec![BrilligParameter::Array(
                vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                SemanticLength(result_length_with_metadata as u32),
            )];

            let (_, mut function_context, mut context) = create_test_environment();

            // Allocate the parameters
            let source_len_var = context.allocate_single_addr_mem();
            let source_vector = context.allocate_brillig_vector();
            let item_to_insert = context.allocate_single_addr_mem();

            // Allocate the results
            let target_vector = context.allocate_brillig_vector();

            let brillig_globals = HashMap::default();
            let hoisted_globals = HashMap::default();
            let mut block = create_brillig_block(
                &mut function_context,
                &mut context,
                &brillig_globals,
                &hoisted_globals,
            );

            if push_back {
                block.vector_push_back_operation(
                    *target_vector,
                    *source_len_var,
                    *source_vector,
                    &[item_to_insert.to_var()],
                );
            } else {
                block.vector_push_front_operation(
                    *target_vector,
                    *source_len_var,
                    *source_vector,
                    &[item_to_insert.to_var()],
                );
            }

            context.codegen_return(&[target_vector.to_var()]);

            // Compile to byte code.
            let bytecode = create_entry_point_bytecode(context, arguments, returns).byte_code;

            // Prepare flattened inputs.
            let inputs = [vec![source_len.into()], source, vec![item_to_push]].concat();

            // Execute the byte code.
            let (vm, return_data_offset, return_data_size) = create_and_run_vm(inputs, &bytecode);

            assert_eq!(
                return_data_size, result_length_with_metadata,
                "expect result length to be data+meta"
            );

            let mut returned_vector: Vec<FieldElement> = vm.get_memory()
                [return_data_offset..(return_data_offset + result_length_with_metadata)]
                .iter()
                .map(|mem_val| mem_val.to_field())
                .collect();

            let returned_size = returned_vector.remove(0);
            assert_eq!(returned_size, result_length.into(), "expect size to be input+1");

            let _returned_capacity = returned_vector.remove(0);
            assert_eq!(
                returned_vector, expected_return,
                "expect items after size and capacity to be input+new"
            );
        }

        test_case_push(
            true,
            3,
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
            2,
            vec![
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(3_usize),
            ],
            FieldElement::from(27_usize),
            vec![
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(27_usize),
            ],
        );
        test_case_push(
            true,
            1,
            vec![
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(3_usize),
            ],
            FieldElement::from(27_usize),
            vec![FieldElement::from(1_usize), FieldElement::from(27_usize)],
        );
        test_case_push(
            true,
            0,
            vec![],
            FieldElement::from(27_usize),
            vec![FieldElement::from(27_usize)],
        );
        test_case_push(
            false,
            3,
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
            1,
            vec![
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(3_usize),
            ],
            FieldElement::from(27_usize),
            vec![FieldElement::from(27_usize), FieldElement::from(1_usize)],
        );
        test_case_push(
            false,
            0,
            vec![],
            FieldElement::from(27_usize),
            vec![FieldElement::from(27_usize)],
        );
    }

    #[test]
    fn test_vector_pop_operation() {
        fn test_case_pop(
            pop_back: bool,
            source_len: usize,
            source: Vec<FieldElement>,
            expected_return_array: Vec<FieldElement>,
            expected_return_item: FieldElement,
        ) {
            let arguments = vec![
                BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
                BrilligParameter::Vector(
                    vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                    SemanticLength(source.len() as u32),
                ),
            ];
            let result_length = source_len - 1;
            assert_eq!(
                result_length,
                expected_return_array.len(),
                "expect return length to be 1 less than input"
            );
            let result_length_with_metadata = result_length + 2; // Leading length and capacity

            // Entry points don't support returning vectors, so we implicitly cast the vector to an array
            // With the metadata at the start.
            let returns = vec![
                BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
                BrilligParameter::Array(
                    vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                    SemanticLength(result_length_with_metadata as u32),
                ),
            ];

            let (_, mut function_context, mut context) = create_test_environment();

            // Allocate the parameters
            let source_len_var = context.allocate_single_addr_mem();
            let source_vector = context.allocate_brillig_vector();

            // Allocate the results
            let target_vector = context.allocate_brillig_vector();
            let removed_item = context.allocate_single_addr_mem();

            let brillig_globals = HashMap::default();
            let hoisted_globals = HashMap::default();
            let mut block = create_brillig_block(
                &mut function_context,
                &mut context,
                &brillig_globals,
                &hoisted_globals,
            );

            if pop_back {
                block.vector_pop_back_operation(
                    *target_vector,
                    *source_len_var,
                    *source_vector,
                    &[removed_item.to_var()],
                );
            } else {
                block.vector_pop_front_operation(
                    *target_vector,
                    *source_len_var,
                    *source_vector,
                    &[removed_item.to_var()],
                );
            }

            context.codegen_return(&[removed_item.to_var(), target_vector.to_var()]);

            let bytecode = create_entry_point_bytecode(context, arguments, returns).byte_code;

            let inputs = [vec![source_len.into()], source].concat();

            let (vm, return_data_offset, return_data_size) = create_and_run_vm(inputs, &bytecode);
            // vector + removed item
            assert_eq!(
                return_data_size,
                result_length_with_metadata + 1,
                "expect return data size to be the metadata, the remaining items, plus 1 popped item"
            );

            let mut return_data: Vec<FieldElement> = vm.get_memory()
                [return_data_offset..(return_data_offset + return_data_size)]
                .iter()
                .map(|mem_val| mem_val.to_field())
                .collect();

            let returned_item = return_data.remove(0);
            assert_eq!(returned_item, expected_return_item, "expect the popped item to match");

            let returned_size = return_data.remove(0);
            assert_eq!(
                returned_size,
                result_length.into(),
                "expect size to be 1 less than the input size"
            );

            let _returned_capacity = return_data.remove(0);

            assert_eq!(return_data, expected_return_array, "expect the returned items to match");
        }

        test_case_pop(
            true,
            3,
            vec![
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(3_usize),
            ],
            vec![FieldElement::from(1_usize), FieldElement::from(2_usize)],
            FieldElement::from(3_usize),
        );
        test_case_pop(
            true,
            2,
            vec![
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(3_usize),
            ],
            vec![FieldElement::from(1_usize)],
            FieldElement::from(2_usize),
        );
        test_case_pop(
            true,
            1,
            vec![FieldElement::from(1_usize)],
            vec![],
            FieldElement::from(1_usize),
        );
        test_case_pop(
            false,
            3,
            vec![
                FieldElement::from(10_usize),
                FieldElement::from(20_usize),
                FieldElement::from(30_usize),
            ],
            vec![FieldElement::from(20_usize), FieldElement::from(30_usize)],
            FieldElement::from(10_usize),
        );
        test_case_pop(
            false,
            2,
            vec![
                FieldElement::from(1_usize),
                FieldElement::from(2_usize),
                FieldElement::from(3_usize),
            ],
            vec![FieldElement::from(2_usize)],
            FieldElement::from(1_usize),
        );
    }

    #[test]
    fn test_vector_insert_operation() {
        fn test_case_insert(
            array: Vec<FieldElement>,
            item: FieldElement,
            index: FieldElement,
            expected_return: Vec<FieldElement>,
        ) {
            let arguments = vec![
                BrilligParameter::Vector(
                    vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                    SemanticLength(array.len() as u32),
                ),
                BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
                BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE),
            ];
            let result_length = array.len() + 1;
            assert_eq!(result_length, expected_return.len());
            let result_length_with_metadata = result_length + 2; // Leading length and capacity

            // Entry points don't support returning vectors, so we implicitly cast the vector to an array
            // With the metadata at the start.
            let returns = vec![BrilligParameter::Array(
                vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                SemanticLength(result_length_with_metadata as u32),
            )];

            let (_, mut function_context, mut context) = create_test_environment();

            // Allocate the parameters
            let source_vector = context.allocate_brillig_vector();
            let item_to_insert = context.allocate_single_addr_mem();
            let index_to_insert = context.allocate_single_addr_mem();

            // Allocate the results
            let target_vector = context.allocate_brillig_vector();

            let brillig_globals = HashMap::default();
            let hoisted_globals = HashMap::default();
            let mut block = create_brillig_block(
                &mut function_context,
                &mut context,
                &brillig_globals,
                &hoisted_globals,
            );

            block.vector_insert_operation(
                *target_vector,
                *source_vector,
                *index_to_insert,
                &[item_to_insert.to_var()],
            );

            context.codegen_return(&[target_vector.to_var()]);
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
    fn test_vector_remove_operation() {
        fn test_case_remove(
            array: Vec<FieldElement>,
            index: FieldElement,
            expected_array: Vec<FieldElement>,
            expected_removed_item: FieldElement,
        ) {
            let arguments = vec![
                BrilligParameter::Vector(
                    vec![BrilligParameter::SingleAddr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)],
                    SemanticLength(array.len() as u32),
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
                    SemanticLength(result_length_with_metadata as u32),
                ),
            ];

            let (_, mut function_context, mut context) = create_test_environment();

            // Allocate the parameters
            let source_vector = context.allocate_brillig_vector();
            let index_to_insert = context.allocate_single_addr_mem();

            // Allocate the results
            let target_vector = context.allocate_brillig_vector();
            let removed_item = context.allocate_single_addr_mem();

            let brillig_globals = HashMap::default();
            let hoisted_globals = HashMap::default();
            let mut block = create_brillig_block(
                &mut function_context,
                &mut context,
                &brillig_globals,
                &hoisted_globals,
            );

            block.vector_remove_operation(
                *target_vector,
                *source_vector,
                *index_to_insert,
                &[removed_item.to_var()],
            );

            context.codegen_return(&[removed_item.to_var(), target_vector.to_var()]);

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
