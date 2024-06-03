use super::{
    artifact::{BrilligArtifact, BrilligParameter},
    brillig_variable::{BrilligArray, BrilligVariable, BrilligVector, SingleAddrVariable},
    debug_show::DebugShow,
    registers::BrilligRegistersContext,
    BrilligBinaryOp, BrilligContext, ReservedRegisters,
};
use acvm::{acir::brillig::MemoryAddress, acir::AcirField, FieldElement};

pub(crate) const MAX_STACK_SIZE: usize = 2048;

impl BrilligContext {
    /// Creates an entry point artifact that will jump to the function label provided.
    pub(crate) fn new_entry_point_artifact<T: ToString>(
        arguments: Vec<BrilligParameter>,
        return_parameters: Vec<BrilligParameter>,
        target_function: T,
    ) -> BrilligArtifact {
        let mut context = BrilligContext {
            obj: BrilligArtifact::default(),
            registers: BrilligRegistersContext::new(),
            context_label: String::default(),
            section_label: 0,
            next_section: 1,
            debug_show: DebugShow::new(false),
            bigint_new_id: 0,
        };

        context.codegen_entry_point(&arguments, &return_parameters);

        context.add_external_call_instruction(target_function);

        context.codegen_exit_point(&arguments, &return_parameters);
        context.artifact()
    }

    /// Adds the instructions needed to handle entry point parameters
    /// The runtime will leave the parameters in calldata.
    /// Arrays will be passed flattened.
    fn codegen_entry_point(
        &mut self,
        arguments: &[BrilligParameter],
        return_parameters: &[BrilligParameter],
    ) {
        let calldata_size = BrilligContext::flattened_tuple_size(arguments);
        let return_data_size = BrilligContext::flattened_tuple_size(return_parameters);

        // Set initial value of stack pointer: MAX_STACK_SIZE + calldata_size + return_data_size
        self.const_instruction(
            SingleAddrVariable::new_usize(ReservedRegisters::free_memory_pointer()),
            (MAX_STACK_SIZE + calldata_size + return_data_size).into(),
        );

        // Copy calldata
        self.copy_and_cast_calldata(arguments);

        // Allocate the variables for every argument:
        let mut current_calldata_pointer = MAX_STACK_SIZE;

        let mut argument_variables: Vec<_> = arguments
            .iter()
            .map(|argument| match argument {
                BrilligParameter::SingleAddr(bit_size) => {
                    let single_address = self.allocate_register();
                    let var = BrilligVariable::SingleAddr(SingleAddrVariable {
                        address: single_address,
                        bit_size: *bit_size,
                    });
                    self.mov_instruction(single_address, MemoryAddress(current_calldata_pointer));
                    current_calldata_pointer += 1;
                    var
                }
                BrilligParameter::Array(_, _) => {
                    let pointer_to_the_array_in_calldata =
                        self.make_usize_constant_instruction(current_calldata_pointer.into());
                    let rc_register = self.make_usize_constant_instruction(1_usize.into());
                    let flattened_size = BrilligContext::flattened_size(argument);
                    let var = BrilligVariable::BrilligArray(BrilligArray {
                        pointer: pointer_to_the_array_in_calldata.address,
                        size: flattened_size,
                        rc: rc_register.address,
                    });

                    current_calldata_pointer += flattened_size;
                    var
                }
                BrilligParameter::Slice(_, _) => {
                    let pointer_to_the_array_in_calldata =
                        self.make_usize_constant_instruction(current_calldata_pointer.into());

                    let flattened_size = BrilligContext::flattened_size(argument);
                    let size_register = self.make_usize_constant_instruction(flattened_size.into());
                    let rc_register = self.make_usize_constant_instruction(1_usize.into());

                    let var = BrilligVariable::BrilligVector(BrilligVector {
                        pointer: pointer_to_the_array_in_calldata.address,
                        size: size_register.address,
                        rc: rc_register.address,
                    });

                    current_calldata_pointer += flattened_size;
                    var
                }
            })
            .collect();

        // Deflatten arrays
        for (argument_variable, argument) in argument_variables.iter_mut().zip(arguments) {
            match (argument_variable, argument) {
                (
                    BrilligVariable::BrilligArray(array),
                    BrilligParameter::Array(item_type, item_count),
                ) => {
                    let deflattened_address =
                        self.deflatten_array(item_type, array.size, array.pointer);
                    self.mov_instruction(array.pointer, deflattened_address);
                    array.size = item_type.len() * item_count;
                    self.deallocate_register(deflattened_address);
                }
                (
                    BrilligVariable::BrilligVector(vector),
                    BrilligParameter::Slice(item_type, item_count),
                ) => {
                    let flattened_size = BrilligContext::flattened_size(argument);

                    let deflattened_address =
                        self.deflatten_array(item_type, flattened_size, vector.pointer);
                    self.mov_instruction(vector.pointer, deflattened_address);
                    self.usize_const_instruction(
                        vector.size,
                        (item_type.len() * item_count).into(),
                    );

                    self.deallocate_register(deflattened_address);
                }
                _ => {}
            }
        }
    }

    fn copy_and_cast_calldata(&mut self, arguments: &[BrilligParameter]) {
        let calldata_size = BrilligContext::flattened_tuple_size(arguments);
        self.calldata_copy_instruction(MemoryAddress(MAX_STACK_SIZE), calldata_size, 0);

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
            if bit_size < FieldElement::max_num_bits() {
                self.cast_instruction(
                    SingleAddrVariable::new(MemoryAddress(MAX_STACK_SIZE + i), bit_size),
                    SingleAddrVariable::new_field(MemoryAddress(MAX_STACK_SIZE + i)),
                );
            }
        }
    }

    /// Computes the size of a parameter if it was flattened
    pub(super) fn flattened_size(param: &BrilligParameter) -> usize {
        match param {
            BrilligParameter::SingleAddr(_) => 1,
            BrilligParameter::Array(item_types, item_count)
            | BrilligParameter::Slice(item_types, item_count) => {
                let item_size: usize = item_types.iter().map(BrilligContext::flattened_size).sum();
                item_count * item_size
            }
        }
    }

    /// Computes the size of a parameter if it was flattened
    pub(super) fn flattened_tuple_size(tuple: &[BrilligParameter]) -> usize {
        tuple.iter().map(BrilligContext::flattened_size).sum()
    }

    /// Computes the size of a parameter if it was flattened
    fn has_nested_arrays(tuple: &[BrilligParameter]) -> bool {
        tuple.iter().any(|param| !matches!(param, BrilligParameter::SingleAddr(_)))
    }

    /// Deflatten an array by recursively allocating nested arrays and copying the plain values.
    /// Returns the pointer to the deflattened items.
    fn deflatten_array(
        &mut self,
        item_type: &[BrilligParameter],
        item_count: usize,
        flattened_array_pointer: MemoryAddress,
    ) -> MemoryAddress {
        if BrilligContext::has_nested_arrays(item_type) {
            let movement_register = self.allocate_register();
            let deflattened_array_pointer = self.allocate_register();

            let target_item_size = item_type.len();
            let source_item_size = BrilligContext::flattened_tuple_size(item_type);

            self.codegen_allocate_fixed_length_array(
                deflattened_array_pointer,
                item_count * target_item_size,
            );

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
                            self.codegen_array_get(
                                flattened_array_pointer,
                                source_index,
                                movement_register,
                            );
                            self.codegen_array_set(
                                deflattened_array_pointer,
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
                            self.mov_instruction(nested_array_pointer, flattened_array_pointer);
                            self.memory_op_instruction(
                                nested_array_pointer,
                                source_index.address,
                                nested_array_pointer,
                                BrilligBinaryOp::Add,
                            );
                            let deflattened_nested_array_pointer = self.deflatten_array(
                                nested_array_item_type,
                                *nested_array_item_count,
                                nested_array_pointer,
                            );
                            let reference = self.allocate_register();
                            let rc = self.allocate_register();
                            self.usize_const_instruction(rc, 1_usize.into());

                            self.codegen_allocate_array_reference(reference);
                            let array_variable = BrilligVariable::BrilligArray(BrilligArray {
                                pointer: deflattened_nested_array_pointer,
                                size: nested_array_item_type.len() * nested_array_item_count,
                                rc,
                            });
                            self.codegen_store_variable(reference, array_variable);

                            self.codegen_array_set(
                                deflattened_array_pointer,
                                target_index,
                                reference,
                            );

                            self.deallocate_register(nested_array_pointer);
                            self.deallocate_register(reference);
                            array_variable
                                .extract_registers()
                                .into_iter()
                                .for_each(|register| self.deallocate_register(register));

                            source_offset += BrilligContext::flattened_size(subitem);
                        }
                        BrilligParameter::Slice(..) => unreachable!("ICE: Cannot deflatten slices"),
                    }

                    self.deallocate_single_addr(source_index);
                    self.deallocate_single_addr(target_index);
                }
            }

            self.deallocate_register(movement_register);

            deflattened_array_pointer
        } else {
            let deflattened_array_pointer = self.allocate_register();
            self.mov_instruction(deflattened_array_pointer, flattened_array_pointer);
            deflattened_array_pointer
        }
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
                        rc: self.allocate_register(),
                    })
                }
                BrilligParameter::Slice(..) => unreachable!("ICE: Cannot return slices"),
            })
            .collect();

        // Now, we deflatten the return data
        let calldata_size = BrilligContext::flattened_tuple_size(arguments);
        let return_data_size = BrilligContext::flattened_tuple_size(return_parameters);

        // Return data has a reserved space after calldata
        let return_data_offset = MAX_STACK_SIZE + calldata_size;
        let mut return_data_index = return_data_offset;

        for (return_param, returned_variable) in return_parameters.iter().zip(&returned_variables) {
            match return_param {
                BrilligParameter::SingleAddr(_) => {
                    self.mov_instruction(
                        MemoryAddress(return_data_index),
                        returned_variable.extract_single_addr().address,
                    );
                    return_data_index += 1;
                }
                BrilligParameter::Array(item_type, item_count) => {
                    let returned_pointer = returned_variable.extract_array().pointer;
                    let pointer_to_return_data =
                        self.make_usize_constant_instruction(return_data_index.into());

                    self.flatten_array(
                        item_type,
                        *item_count,
                        pointer_to_return_data.address,
                        returned_pointer,
                    );

                    self.deallocate_single_addr(pointer_to_return_data);
                    return_data_index += BrilligContext::flattened_size(return_param);
                }
                BrilligParameter::Slice(..) => {
                    unreachable!("ICE: Cannot return slices from brillig entrypoints")
                }
            }
        }

        self.external_stop_instruction(return_data_offset, return_data_size);
    }

    // Flattens an array by recursively copying nested arrays and regular items.
    pub(super) fn flatten_array(
        &mut self,
        item_type: &[BrilligParameter],
        item_count: usize,
        flattened_array_pointer: MemoryAddress,
        deflattened_array_pointer: MemoryAddress,
    ) {
        if BrilligContext::has_nested_arrays(item_type) {
            let movement_register = self.allocate_register();

            let source_item_size = item_type.len();
            let target_item_size: usize =
                item_type.iter().map(BrilligContext::flattened_size).sum();

            for item_index in 0..item_count {
                let source_item_base_index = item_index * source_item_size;
                let target_item_base_index = item_index * target_item_size;

                let mut target_offset = 0;

                for (subitem_index, subitem) in item_type.iter().enumerate() {
                    let source_index = self.make_usize_constant_instruction(
                        (source_item_base_index + subitem_index).into(),
                    );
                    let target_index = self.make_usize_constant_instruction(
                        (target_item_base_index + target_offset).into(),
                    );

                    match subitem {
                        BrilligParameter::SingleAddr(_) => {
                            self.codegen_array_get(
                                deflattened_array_pointer,
                                source_index,
                                movement_register,
                            );
                            self.codegen_array_set(
                                flattened_array_pointer,
                                target_index,
                                movement_register,
                            );
                            target_offset += 1;
                        }
                        BrilligParameter::Array(
                            nested_array_item_type,
                            nested_array_item_count,
                        ) => {
                            let nested_array_reference = self.allocate_register();
                            self.codegen_array_get(
                                deflattened_array_pointer,
                                source_index,
                                nested_array_reference,
                            );

                            let nested_array_variable =
                                BrilligVariable::BrilligArray(BrilligArray {
                                    pointer: self.allocate_register(),
                                    size: nested_array_item_type.len() * nested_array_item_count,
                                    rc: self.allocate_register(),
                                });

                            self.codegen_load_variable(
                                nested_array_variable,
                                nested_array_reference,
                            );

                            let flattened_nested_array_pointer = self.allocate_register();

                            self.mov_instruction(
                                flattened_nested_array_pointer,
                                flattened_array_pointer,
                            );

                            self.memory_op_instruction(
                                flattened_nested_array_pointer,
                                target_index.address,
                                flattened_nested_array_pointer,
                                BrilligBinaryOp::Add,
                            );

                            self.flatten_array(
                                nested_array_item_type,
                                *nested_array_item_count,
                                flattened_nested_array_pointer,
                                nested_array_variable.extract_array().pointer,
                            );

                            self.deallocate_register(nested_array_reference);
                            self.deallocate_register(flattened_nested_array_pointer);
                            nested_array_variable
                                .extract_registers()
                                .into_iter()
                                .for_each(|register| self.deallocate_register(register));

                            target_offset += BrilligContext::flattened_size(subitem);
                        }
                        BrilligParameter::Slice(..) => unreachable!("ICE: Cannot flatten slices"),
                    }

                    self.deallocate_single_addr(source_index);
                    self.deallocate_single_addr(target_index);
                }
            }

            self.deallocate_register(movement_register);
        } else {
            let item_count =
                self.make_usize_constant_instruction((item_count * item_type.len()).into());
            self.codegen_copy_array(deflattened_array_pointer, flattened_array_pointer, item_count);
            self.deallocate_single_addr(item_count);
        }
    }
}

#[cfg(test)]
mod tests {

    use acvm::FieldElement;

    use crate::brillig::brillig_ir::{
        brillig_variable::BrilligArray,
        entry_point::BrilligParameter,
        tests::{create_and_run_vm, create_context, create_entry_point_bytecode},
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

        let mut context = create_context();

        // Allocate the parameter
        let array_pointer = context.allocate_register();
        let array_value = context.allocate_register();

        context.load_instruction(array_pointer, array_pointer);
        context.load_instruction(array_pointer, array_pointer);
        context.load_instruction(array_value, array_pointer);

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

        let mut context = create_context();

        // Allocate the parameter
        let brillig_array = BrilligArray {
            pointer: context.allocate_register(),
            size: 2,
            rc: context.allocate_register(),
        };

        context.codegen_return(&brillig_array.extract_registers());

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
