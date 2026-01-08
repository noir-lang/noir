use crate::{
    brillig::{BrilligOptions, brillig_ir::registers::Allocated},
    ssa::ir::function::FunctionId,
};

use super::{
    BrilligBinaryOp, BrilligContext, ReservedRegisters,
    artifact::{BrilligArtifact, BrilligParameter},
    brillig_variable::{BrilligArray, BrilligVariable, BrilligVector, SingleAddrVariable},
    debug_show::DebugToString,
    registers::Stack,
};
use acvm::acir::{
    AcirField,
    brillig::{
        HeapVector, MemoryAddress,
        lengths::{ElementsLength, SemanticLength, SemiFlattenedLength},
    },
};

impl<F: AcirField + DebugToString> BrilligContext<F, Stack> {
    /// Creates an entry point artifact that will jump to the function label provided.
    pub(crate) fn new_entry_point_artifact(
        arguments: Vec<BrilligParameter>,
        return_parameters: Vec<BrilligParameter>,
        target_function: FunctionId,
        globals_init: bool,
        globals_memory_size: usize,
        name: &str,
        options: &BrilligOptions,
    ) -> (BrilligArtifact<F>, usize) {
        let mut context = BrilligContext::new(name, options);

        context.set_globals_memory_size(Some(globals_memory_size));

        let stack_start = context.codegen_entry_point(&arguments, &return_parameters);

        if globals_init {
            context.add_globals_init_instruction(target_function);
        }

        context.add_external_call_instruction(target_function);

        context.codegen_exit_point(&arguments, &return_parameters);
        (context.into_artifact(), stack_start)
    }

    fn calldata_start_offset(&self) -> usize {
        let globals_size =
            self.globals_memory_size.expect("The memory size of globals should be set");
        self.layout().entry_point_start(globals_size)
    }

    fn return_data_start_offset(&self, calldata_size: usize) -> usize {
        let globals_size =
            self.globals_memory_size.expect("The memory size of globals should be set");
        self.layout().return_data_start(globals_size, calldata_size)
    }

    /// Adds the instructions needed to handle entry point parameters
    /// The runtime will leave the parameters in calldata.
    /// Arrays will be passed flattened.
    ///
    /// Memory layout for entry points:
    /// {reserved} {scratch} {globals} {entry point (call data + return data)} {stack} {heap}
    ///
    /// # Returns
    /// The start of the stack memory region. The start of the stack is determined by the globals compiled as well as
    /// the amount of call data and return data. We return this information so that the [max stack depth check][super::ProcedureId::CheckMaxStackDepth]
    /// can check against the appropriate constant represented the max stack pointer we can have in memory.
    fn codegen_entry_point(
        &mut self,
        arguments: &[BrilligParameter],
        return_parameters: &[BrilligParameter],
    ) -> usize {
        // We need to allocate the variable for every argument first so any register allocation doesn't mangle the expected order.
        let mut argument_variables = self.allocate_function_arguments(arguments);

        let calldata_size = Self::flattened_tuple_size(arguments);
        let return_data_size = Self::flattened_tuple_size(return_parameters);

        // Set reserved registers constants
        self.const_instruction(
            SingleAddrVariable::new_usize(ReservedRegisters::usize_one()),
            1_usize.into(),
        );

        let return_data_start = self.return_data_start_offset(calldata_size);

        // The heap begins after the end of the stack.
        // Set initial value of free memory pointer: `return_data_start + return_data_size + self.layout.max_stack_size()`
        self.const_instruction(
            SingleAddrVariable::new_usize(ReservedRegisters::free_memory_pointer()),
            (return_data_start + return_data_size + self.layout().max_stack_size()).into(),
        );

        // The stack begins after the calldata region (calldata + return data)
        // Set initial value of the stack pointer: `return_data_start + return_data_size`
        let stack_start = return_data_start + return_data_size;
        self.const_instruction(
            SingleAddrVariable::new_usize(ReservedRegisters::stack_pointer()),
            stack_start.into(),
        );

        // Copy calldata
        self.copy_and_cast_calldata(arguments);

        let mut current_calldata_pointer = self.calldata_start_offset();

        // Initialize the variables with the calldata
        for (argument_variable, argument) in argument_variables.iter_mut().zip(arguments) {
            match (**argument_variable, argument) {
                (BrilligVariable::SingleAddr(single_address), BrilligParameter::SingleAddr(_)) => {
                    self.mov_instruction(
                        single_address.address,
                        MemoryAddress::direct(current_calldata_pointer),
                    );
                    current_calldata_pointer += 1;
                }
                (
                    BrilligVariable::BrilligArray(mut array),
                    BrilligParameter::Array(item_type, item_count),
                ) => {
                    let semi_flattened_size = array.size;
                    self.usize_const_instruction(array.pointer, current_calldata_pointer.into());

                    let deflattened_address =
                        self.deflatten_array(item_type, *item_count, array.pointer, false);
                    self.mov_instruction(array.pointer, *deflattened_address);

                    // After deflattening, we have to adjust the size of the array.
                    array.size = ElementsLength(item_type.len()) * *item_count;

                    current_calldata_pointer += semi_flattened_size.0;
                }
                (
                    BrilligVariable::BrilligVector(vector),
                    BrilligParameter::Vector(item_type, item_count),
                ) => {
                    let flattened_size = Self::flattened_size(argument);
                    self.usize_const_instruction(vector.pointer, current_calldata_pointer.into());
                    let deflattened_address =
                        self.deflatten_array(item_type, *item_count, vector.pointer, true);
                    self.mov_instruction(vector.pointer, *deflattened_address);

                    current_calldata_pointer += flattened_size;
                }
                _ => unreachable!("ICE: cannot match variables against arguments"),
            }
        }

        stack_start
    }

    fn allocate_function_arguments(
        &mut self,
        arguments: &[BrilligParameter],
    ) -> Vec<Allocated<BrilligVariable, Stack>> {
        arguments
            .iter()
            .map(|argument| match argument {
                BrilligParameter::SingleAddr(bit_size) => {
                    self.allocate_single_addr(*bit_size).map(BrilligVariable::from)
                }
                BrilligParameter::Array(_, _) => {
                    let flattened_size = Self::flattened_size(argument);
                    let flattened_size = SemiFlattenedLength(flattened_size);
                    self.allocate_brillig_array(flattened_size).map(BrilligVariable::from)
                }
                BrilligParameter::Vector(_, _) => {
                    self.allocate_brillig_vector().map(BrilligVariable::from)
                }
            })
            .collect()
    }

    fn copy_and_cast_calldata(&mut self, arguments: &[BrilligParameter]) {
        let calldata_size = Self::flattened_tuple_size(arguments);
        self.calldata_copy_instruction(
            MemoryAddress::direct(self.calldata_start_offset()),
            calldata_size,
            0,
        );

        fn flat_bit_sizes(param: &BrilligParameter) -> Box<dyn Iterator<Item = u32> + '_> {
            match param {
                BrilligParameter::SingleAddr(bit_size) => Box::new(std::iter::once(*bit_size)),
                BrilligParameter::Array(item_types, item_count)
                | BrilligParameter::Vector(item_types, item_count) => Box::new(
                    (0..item_count.0).flat_map(move |_| item_types.iter().flat_map(flat_bit_sizes)),
                ),
            }
        }

        for (i, bit_size) in arguments.iter().flat_map(flat_bit_sizes).enumerate() {
            // cSpell:disable-next-line
            // Calldatacopy tags everything with field type, so when downcast when necessary
            if bit_size < F::max_num_bits() {
                self.cast_instruction(
                    SingleAddrVariable::new(
                        MemoryAddress::direct(self.calldata_start_offset() + i),
                        bit_size,
                    ),
                    SingleAddrVariable::new_field(MemoryAddress::direct(
                        self.calldata_start_offset() + i,
                    )),
                );
            }
        }
    }

    /// Deflatten an array by recursively allocating nested arrays and copying the plain values.
    /// Returns the pointer to the deflattened items.
    fn deflatten_array(
        &mut self,
        item_type: &[BrilligParameter],
        item_count: SemanticLength,
        flattened_array_pointer: MemoryAddress,
        is_vector: bool,
    ) -> Allocated<MemoryAddress, Stack> {
        let semi_flattened_size: SemiFlattenedLength = item_count * ElementsLength(item_type.len());

        let deflattened_array_pointer = self.allocate_register();
        let deflattened_size_variable =
            self.make_usize_constant_instruction(semi_flattened_size.0.into());

        let deflattened_items_pointer = if is_vector {
            let vector = BrilligVector { pointer: *deflattened_array_pointer };
            self.codegen_initialize_vector(vector, *deflattened_size_variable, None);
            self.codegen_make_vector_items_pointer(vector)
        } else {
            let arr =
                BrilligArray { pointer: *deflattened_array_pointer, size: semi_flattened_size };
            self.codegen_initialize_array(arr);
            self.codegen_make_array_items_pointer(arr)
        };

        if Self::has_nested_arrays(item_type) {
            let movement_register = self.allocate_register();

            let target_item_size = item_type.len();
            let source_item_size = Self::flattened_tuple_size(item_type);

            for item_index in 0..item_count.0 {
                let source_item_base_index = item_index * source_item_size;
                let target_item_base_index = item_index * target_item_size;

                let mut source_offset = 0;

                for (subitem_index, subitem) in item_type.iter().enumerate() {
                    let source_index = self.make_usize_constant_instruction(
                        (source_item_base_index + source_offset).into(),
                    );

                    let target_index = self.make_usize_constant_instruction(
                        (target_item_base_index + subitem_index).into(),
                    );

                    match subitem {
                        BrilligParameter::SingleAddr(_) => {
                            self.codegen_load_with_offset(
                                flattened_array_pointer,
                                *source_index,
                                *movement_register,
                            );
                            self.codegen_store_with_offset(
                                *deflattened_items_pointer,
                                *target_index,
                                *movement_register,
                            );
                            source_offset += 1;
                        }
                        BrilligParameter::Array(
                            nested_array_item_type,
                            nested_array_item_count,
                        ) => {
                            let nested_array_pointer = self.allocate_register();
                            self.memory_op_instruction(
                                flattened_array_pointer,
                                source_index.address,
                                *nested_array_pointer,
                                BrilligBinaryOp::Add,
                            );
                            let deflattened_nested_array_pointer = self.deflatten_array(
                                nested_array_item_type,
                                *nested_array_item_count,
                                *nested_array_pointer,
                                false,
                            );
                            self.codegen_store_with_offset(
                                *deflattened_items_pointer,
                                *target_index,
                                *deflattened_nested_array_pointer,
                            );

                            source_offset += Self::flattened_size(subitem);
                        }
                        BrilligParameter::Vector(..) => {
                            unreachable!("ICE: Cannot deflatten vectors")
                        }
                    }
                }
            }
        } else {
            self.codegen_mem_copy(
                flattened_array_pointer,
                *deflattened_items_pointer,
                *deflattened_size_variable,
            );
        }

        deflattened_array_pointer
    }

    /// Adds the instructions needed to handle return parameters
    /// The runtime expects the results in a contiguous memory region.
    /// Arrays are expected to be returned with all the nested arrays flattened.
    /// However, the function called returns variables (that have extra data) and the returned arrays are deflattened.
    fn codegen_exit_point(
        &mut self,
        arguments: &[BrilligParameter],
        return_parameters: &[BrilligParameter],
    ) {
        // First, we allocate the registers that hold the returned variables from the function call.
        self.set_allocated_registers(vec![]);

        let returned_variables: Vec<_> = return_parameters
            .iter()
            .map(|return_parameter| match return_parameter {
                BrilligParameter::SingleAddr(bit_size) => {
                    self.allocate_single_addr(*bit_size).map(BrilligVariable::from)
                }
                BrilligParameter::Array(item_types, item_count) => {
                    let semi_flattened_size: SemiFlattenedLength =
                        ElementsLength(item_types.len()) * *item_count;
                    self.allocate_brillig_array(semi_flattened_size).map(BrilligVariable::from)
                }
                BrilligParameter::Vector(..) => unreachable!("ICE: Cannot return vectors"),
            })
            .collect();

        // Now, we deflatten the return data
        let calldata_size = Self::flattened_tuple_size(arguments);
        let return_data_size = Self::flattened_tuple_size(return_parameters);

        // Return data has a reserved space after calldata
        let return_data_offset = self.return_data_start_offset(calldata_size);
        let mut return_data_index = return_data_offset;

        for (return_param, returned_variable) in return_parameters.iter().zip(&returned_variables) {
            match return_param {
                BrilligParameter::SingleAddr(_) => {
                    self.mov_instruction(
                        MemoryAddress::direct(return_data_index),
                        returned_variable.extract_single_addr().address,
                    );
                    return_data_index += 1;
                }
                BrilligParameter::Array(item_type, item_count) => {
                    let deflattened_items_pointer =
                        self.codegen_make_array_items_pointer(returned_variable.extract_array());
                    let pointer_to_return_data =
                        self.make_usize_constant_instruction(return_data_index.into());

                    self.flatten_array(
                        item_type,
                        *item_count,
                        pointer_to_return_data.address,
                        *deflattened_items_pointer,
                    );

                    return_data_index += Self::flattened_size(return_param);
                }
                BrilligParameter::Vector(..) => {
                    unreachable!("ICE: Cannot return vectors from brillig entrypoints")
                }
            }
        }

        let return_pointer = self.make_usize_constant_instruction(return_data_offset.into());
        let return_size = self.make_usize_constant_instruction(return_data_size.into());
        let return_data = HeapVector { pointer: return_pointer.address, size: return_size.address };

        self.stop_instruction(return_data);
    }
}

#[cfg(test)]
mod tests {

    use acvm::{
        FieldElement,
        acir::brillig::lengths::{SemanticLength, SemiFlattenedLength},
    };

    use crate::{
        brillig::brillig_ir::{
            brillig_variable::{BrilligArray, BrilligVariable, SingleAddrVariable},
            entry_point::BrilligParameter,
            tests::{create_and_run_vm, create_context, create_entry_point_bytecode},
        },
        ssa::ir::function::FunctionId,
    };

    #[test]
    fn entry_point_with_nested_array_parameter() {
        let calldata = vec![
            FieldElement::from(1_usize),
            FieldElement::from(2_usize),
            FieldElement::from(3_usize),
            FieldElement::from(4_usize),
            FieldElement::from(5_usize),
            FieldElement::from(6_usize),
        ];
        let arguments = vec![BrilligParameter::Array(
            vec![
                BrilligParameter::Array(vec![BrilligParameter::SingleAddr(8)], SemanticLength(2)),
                BrilligParameter::SingleAddr(8),
            ],
            SemanticLength(2),
        )];
        let returns = vec![BrilligParameter::SingleAddr(8)];

        let mut context = create_context(FunctionId::test_new(0));

        // Allocate the parameter
        let array_pointer = context.allocate_register();
        let array_value = context.allocate_register();

        let items_pointer = context.codegen_make_array_items_pointer(BrilligArray {
            pointer: *array_pointer,
            size: SemiFlattenedLength(2),
        });

        // Load the nested array
        context.load_instruction(*array_pointer, *items_pointer);
        let items_pointer = context.codegen_make_array_items_pointer(BrilligArray {
            pointer: *array_pointer,
            size: SemiFlattenedLength(2),
        });
        // Load the first item of the nested array.
        context.load_instruction(*array_value, *items_pointer);

        let return_value = BrilligVariable::from(SingleAddrVariable::new_usize(*array_value));
        context.codegen_return(&[return_value]);

        let bytecode = create_entry_point_bytecode(context, arguments, returns).byte_code;
        let (vm, return_data_offset, return_data_size) =
            create_and_run_vm(calldata.clone(), &bytecode);
        assert_eq!(return_data_size, 1, "Return data size is incorrect");
        assert_eq!(vm.get_memory()[return_data_offset].to_field(), FieldElement::from(1_usize));
    }

    #[test]
    fn entry_point_with_nested_array_return() {
        let flattened_array = vec![
            FieldElement::from(1_usize),
            FieldElement::from(2_usize),
            FieldElement::from(3_usize),
            FieldElement::from(4_usize),
            FieldElement::from(5_usize),
            FieldElement::from(6_usize),
        ];
        let array_param = BrilligParameter::Array(
            vec![
                BrilligParameter::Array(vec![BrilligParameter::SingleAddr(8)], SemanticLength(2)),
                BrilligParameter::SingleAddr(8),
            ],
            SemanticLength(2),
        );
        let arguments = vec![array_param.clone()];
        let returns = vec![array_param];

        let mut context = create_context(FunctionId::test_new(0));

        // Allocate the parameter
        let return_register = context.allocate_brillig_array(SemiFlattenedLength(2));

        context.codegen_return(&[return_register.to_var()]);

        let bytecode = create_entry_point_bytecode(context, arguments, returns).byte_code;
        let (vm, return_data_pointer, return_data_size) =
            create_and_run_vm(flattened_array.clone(), &bytecode);
        let memory = vm.get_memory();

        assert_eq!(
            memory[return_data_pointer..(return_data_pointer + flattened_array.len())]
                .iter()
                .map(|mem_val| mem_val.to_field())
                .collect::<Vec<_>>(),
            flattened_array
        );
        assert_eq!(return_data_size, flattened_array.len());
    }
}
