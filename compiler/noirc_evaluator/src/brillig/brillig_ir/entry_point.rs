use crate::ssa::ir::function::FunctionId;

use super::{
    artifact::{BrilligArtifact, BrilligParameter},
    brillig_variable::{BrilligArray, BrilligVariable, BrilligVector, SingleAddrVariable},
    debug_show::DebugToString,
    registers::Stack,
    BrilligBinaryOp, BrilligContext, ReservedRegisters,
};
use acvm::acir::{
    brillig::{HeapVector, MemoryAddress},
    AcirField,
};

pub(crate) const MAX_STACK_SIZE: usize = 16 * MAX_STACK_FRAME_SIZE;
pub(crate) const MAX_STACK_FRAME_SIZE: usize = 2048;
pub(crate) const MAX_SCRATCH_SPACE: usize = 64;
pub(crate) const MAX_GLOBAL_SPACE: usize = 16384;

impl<F: AcirField + DebugToString> BrilligContext<F, Stack> {
    /// Creates an entry point artifact that will jump to the function label provided.
    pub(crate) fn new_entry_point_artifact(
        arguments: Vec<BrilligParameter>,
        return_parameters: Vec<BrilligParameter>,
        target_function: FunctionId,
        globals_init: bool,
    ) -> BrilligArtifact<F> {
        let mut context = BrilligContext::new(false);

        context.codegen_entry_point(&arguments, &return_parameters);

        if globals_init {
            context.add_globals_init_instruction();
        }

        context.add_external_call_instruction(target_function);

        context.codegen_exit_point(&arguments, &return_parameters);
        context.artifact()
    }

    fn calldata_start_offset() -> usize {
        ReservedRegisters::len() + MAX_STACK_SIZE + MAX_SCRATCH_SPACE + MAX_GLOBAL_SPACE
    }

    fn return_data_start_offset(calldata_size: usize) -> usize {
        Self::calldata_start_offset() + calldata_size
    }

    /// Adds the instructions needed to handle entry point parameters
    /// The runtime will leave the parameters in calldata.
    /// Arrays will be passed flattened.
    fn codegen_entry_point(
        &mut self,
        arguments: &[BrilligParameter],
        return_parameters: &[BrilligParameter],
    ) {
        // We need to allocate the variable for every argument first so any register allocation doesn't mangle the expected order.
        let mut argument_variables = self.allocate_function_arguments(arguments);

        let calldata_size = Self::flattened_tuple_size(arguments);
        let return_data_size = Self::flattened_tuple_size(return_parameters);

        // Set reserved registers constants
        self.const_instruction(
            SingleAddrVariable::new_usize(ReservedRegisters::usize_one()),
            1_usize.into(),
        );

        // Set initial value of free memory pointer: calldata_start_offset + calldata_size + return_data_size
        self.const_instruction(
            SingleAddrVariable::new_usize(ReservedRegisters::free_memory_pointer()),
            (Self::calldata_start_offset() + calldata_size + return_data_size).into(),
        );

        // Set initial value of stack pointer: ReservedRegisters.len()
        self.const_instruction(
            SingleAddrVariable::new_usize(ReservedRegisters::stack_pointer()),
            ReservedRegisters::len().into(),
        );

        // Copy calldata
        self.copy_and_cast_calldata(arguments);

        let mut current_calldata_pointer = Self::calldata_start_offset();

        // Initialize the variables with the calldata
        for (argument_variable, argument) in argument_variables.iter_mut().zip(arguments) {
            match (argument_variable, argument) {
                (BrilligVariable::SingleAddr(single_address), BrilligParameter::SingleAddr(_)) => {
                    self.mov_instruction(
                        single_address.address,
                        MemoryAddress::direct(current_calldata_pointer),
                    );
                    current_calldata_pointer += 1;
                }
                (
                    BrilligVariable::BrilligArray(array),
                    BrilligParameter::Array(item_type, item_count),
                ) => {
                    let flattened_size = array.size;
                    self.usize_const_instruction(array.pointer, current_calldata_pointer.into());

                    let deflattened_address =
                        self.deflatten_array(item_type, *item_count, array.pointer, false);
                    self.mov_instruction(array.pointer, deflattened_address);
                    array.size = item_type.len() * item_count;
                    self.deallocate_register(deflattened_address);

                    current_calldata_pointer += flattened_size;
                }
                (
                    BrilligVariable::BrilligVector(vector),
                    BrilligParameter::Slice(item_type, item_count),
                ) => {
                    let flattened_size = Self::flattened_size(argument);
                    self.usize_const_instruction(vector.pointer, current_calldata_pointer.into());
                    let deflattened_address =
                        self.deflatten_array(item_type, *item_count, vector.pointer, true);
                    self.mov_instruction(vector.pointer, deflattened_address);

                    self.deallocate_register(deflattened_address);

                    current_calldata_pointer += flattened_size;
                }
                _ => unreachable!("ICE: cannot match variables against arguments"),
            }
        }
    }

    fn allocate_function_arguments(
        &mut self,
        arguments: &[BrilligParameter],
    ) -> Vec<BrilligVariable> {
        arguments
            .iter()
            .map(|argument| match argument {
                BrilligParameter::SingleAddr(bit_size) => {
                    BrilligVariable::SingleAddr(SingleAddrVariable {
                        address: self.allocate_register(),
                        bit_size: *bit_size,
                    })
                }
                BrilligParameter::Array(_, _) => {
                    let flattened_size = Self::flattened_size(argument);
                    BrilligVariable::BrilligArray(BrilligArray {
                        pointer: self.allocate_register(),
                        size: flattened_size,
                    })
                }
                BrilligParameter::Slice(_, _) => BrilligVariable::BrilligVector(BrilligVector {
                    pointer: self.allocate_register(),
                }),
            })
            .collect()
    }

    fn copy_and_cast_calldata(&mut self, arguments: &[BrilligParameter]) {
        let calldata_size = Self::flattened_tuple_size(arguments);
        self.calldata_copy_instruction(
            MemoryAddress::direct(Self::calldata_start_offset()),
            calldata_size,
            0,
        );

        fn flat_bit_sizes(param: &BrilligParameter) -> Box<dyn Iterator<Item = u32> + '_> {
            match param {
                BrilligParameter::SingleAddr(bit_size) => Box::new(std::iter::once(*bit_size)),
                BrilligParameter::Array(item_types, item_count)
                | BrilligParameter::Slice(item_types, item_count) => Box::new(
                    (0..*item_count).flat_map(move |_| item_types.iter().flat_map(flat_bit_sizes)),
                ),
            }
        }

        for (i, bit_size) in arguments.iter().flat_map(flat_bit_sizes).enumerate() {
            // Calldatacopy tags everything with field type, so when downcast when necessary
            if bit_size < F::max_num_bits() {
                self.cast_instruction(
                    SingleAddrVariable::new(
                        MemoryAddress::direct(Self::calldata_start_offset() + i),
                        bit_size,
                    ),
                    SingleAddrVariable::new_field(MemoryAddress::direct(
                        Self::calldata_start_offset() + i,
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
        item_count: usize,
        flattened_array_pointer: MemoryAddress,
        is_vector: bool,
    ) -> MemoryAddress {
        let deflattened_array_pointer = self.allocate_register();
        let deflattened_size_variable =
            self.make_usize_constant_instruction((item_count * item_type.len()).into());

        let deflattened_items_pointer = if is_vector {
            let vector = BrilligVector { pointer: deflattened_array_pointer };

            self.codegen_initialize_vector(vector, deflattened_size_variable, None);

            self.codegen_make_vector_items_pointer(vector)
        } else {
            let arr = BrilligArray {
                pointer: deflattened_array_pointer,
                size: item_count * item_type.len(),
            };
            self.codegen_initialize_array(arr);
            self.codegen_make_array_items_pointer(arr)
        };

        if Self::has_nested_arrays(item_type) {
            let movement_register = self.allocate_register();

            let target_item_size = item_type.len();
            let source_item_size = Self::flattened_tuple_size(item_type);

            for item_index in 0..item_count {
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
                                source_index,
                                movement_register,
                            );
                            self.codegen_store_with_offset(
                                deflattened_items_pointer,
                                target_index,
                                movement_register,
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
                                nested_array_pointer,
                                BrilligBinaryOp::Add,
                            );
                            let deflattened_nested_array_pointer = self.deflatten_array(
                                nested_array_item_type,
                                *nested_array_item_count,
                                nested_array_pointer,
                                false,
                            );
                            self.codegen_store_with_offset(
                                deflattened_items_pointer,
                                target_index,
                                deflattened_nested_array_pointer,
                            );

                            self.deallocate_register(nested_array_pointer);
                            self.deallocate_register(deflattened_nested_array_pointer);

                            source_offset += Self::flattened_size(subitem);
                        }
                        BrilligParameter::Slice(..) => unreachable!("ICE: Cannot deflatten slices"),
                    }

                    self.deallocate_single_addr(source_index);
                    self.deallocate_single_addr(target_index);
                }
            }
            self.deallocate_register(movement_register);
        } else {
            self.codegen_mem_copy(
                flattened_array_pointer,
                deflattened_items_pointer,
                deflattened_size_variable,
            );
        }

        self.deallocate_single_addr(deflattened_size_variable);
        self.deallocate_register(deflattened_items_pointer);
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
                    BrilligVariable::SingleAddr(SingleAddrVariable {
                        address: self.allocate_register(),
                        bit_size: *bit_size,
                    })
                }
                BrilligParameter::Array(item_types, item_count) => {
                    BrilligVariable::BrilligArray(BrilligArray {
                        pointer: self.allocate_register(),
                        size: item_types.len() * item_count,
                    })
                }
                BrilligParameter::Slice(..) => unreachable!("ICE: Cannot return slices"),
            })
            .collect();

        // Now, we deflatten the return data
        let calldata_size = Self::flattened_tuple_size(arguments);
        let return_data_size = Self::flattened_tuple_size(return_parameters);

        // Return data has a reserved space after calldata
        let return_data_offset = Self::return_data_start_offset(calldata_size);
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
                        deflattened_items_pointer,
                    );

                    self.deallocate_single_addr(pointer_to_return_data);
                    self.deallocate_register(deflattened_items_pointer);

                    return_data_index += Self::flattened_size(return_param);
                }
                BrilligParameter::Slice(..) => {
                    unreachable!("ICE: Cannot return slices from brillig entrypoints")
                }
            }
        }

        let return_data = HeapVector {
            pointer: self.make_usize_constant_instruction(return_data_offset.into()).address,
            size: self.make_usize_constant_instruction(return_data_size.into()).address,
        };

        self.stop_instruction(return_data);
    }
}

#[cfg(test)]
mod tests {

    use acvm::FieldElement;

    use crate::{
        brillig::brillig_ir::{
            brillig_variable::BrilligArray,
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
                BrilligParameter::Array(vec![BrilligParameter::SingleAddr(8)], 2),
                BrilligParameter::SingleAddr(8),
            ],
            2,
        )];
        let returns = vec![BrilligParameter::SingleAddr(8)];

        let mut context = create_context(FunctionId::test_new(0));

        // Allocate the parameter
        let array_pointer = context.allocate_register();
        let array_value = context.allocate_register();

        let items_pointer = context
            .codegen_make_array_items_pointer(BrilligArray { pointer: array_pointer, size: 2 });

        // Load the nested array
        context.load_instruction(array_pointer, items_pointer);
        let items_pointer = context
            .codegen_make_array_items_pointer(BrilligArray { pointer: array_pointer, size: 2 });
        // Load the first item of the nested array.
        context.load_instruction(array_value, items_pointer);

        context.codegen_return(&[array_value]);

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
                BrilligParameter::Array(vec![BrilligParameter::SingleAddr(8)], 2),
                BrilligParameter::SingleAddr(8),
            ],
            2,
        );
        let arguments = vec![array_param.clone()];
        let returns = vec![array_param];

        let mut context = create_context(FunctionId::test_new(0));

        // Allocate the parameter
        let brillig_array = BrilligArray { pointer: context.allocate_register(), size: 2 };

        context.codegen_return(&[brillig_array.pointer]);

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
