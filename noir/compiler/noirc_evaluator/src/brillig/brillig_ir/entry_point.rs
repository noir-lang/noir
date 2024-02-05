use super::{
    artifact::{BrilligArtifact, BrilligParameter},
    brillig_variable::{BrilligArray, BrilligVariable},
    debug_show::DebugShow,
    registers::BrilligRegistersContext,
    BrilligContext, ReservedRegisters, BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
};
use acvm::acir::brillig::{MemoryAddress, Opcode as BrilligOpcode};

pub(crate) const MAX_STACK_SIZE: usize = 1024;

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
        };

        context.entry_point_instruction(&arguments, &return_parameters);

        context.add_external_call_instruction(target_function);

        context.exit_point_instruction(&arguments, &return_parameters);
        context.artifact()
    }

    /// Adds the instructions needed to handle entry point parameters
    /// The runtime will leave the parameters in calldata.
    /// Arrays will be passed flattened.
    fn entry_point_instruction(
        &mut self,
        arguments: &[BrilligParameter],
        return_parameters: &[BrilligParameter],
    ) {
        let calldata_size = BrilligContext::flattened_tuple_size(arguments);
        let return_data_size = BrilligContext::flattened_tuple_size(return_parameters);

        // Set initial value of stack pointer: MAX_STACK_SIZE + calldata_size + return_data_size
        self.push_opcode(BrilligOpcode::Const {
            destination: ReservedRegisters::stack_pointer(),
            value: (MAX_STACK_SIZE + calldata_size + return_data_size).into(),
            bit_size: BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
        });

        // Copy calldata
        self.push_opcode(BrilligOpcode::CalldataCopy {
            destination_address: MemoryAddress(MAX_STACK_SIZE),
            size: calldata_size,
            offset: 0,
        });

        // Allocate the variables for every argument:
        let mut current_calldata_pointer = MAX_STACK_SIZE;

        let mut argument_variables: Vec<_> = arguments
            .iter()
            .map(|argument| match argument {
                BrilligParameter::Simple => {
                    let simple_address = self.allocate_register();
                    let var = BrilligVariable::Simple(simple_address);
                    self.mov_instruction(simple_address, MemoryAddress(current_calldata_pointer));
                    current_calldata_pointer += 1;
                    var
                }
                BrilligParameter::Array(_, _) => {
                    let pointer_to_the_array_in_calldata =
                        self.make_usize_constant(current_calldata_pointer.into());
                    let rc_register = self.make_usize_constant(1_usize.into());
                    let flattened_size = BrilligContext::flattened_size(argument);
                    let var = BrilligVariable::BrilligArray(BrilligArray {
                        pointer: pointer_to_the_array_in_calldata,
                        size: flattened_size,
                        rc: rc_register,
                    });

                    current_calldata_pointer += flattened_size;
                    var
                }
                BrilligParameter::Slice(_) => unimplemented!("Unsupported slices as parameter"),
            })
            .collect();

        // Deflatten arrays
        for (argument_variable, argument) in argument_variables.iter_mut().zip(arguments) {
            if let (
                BrilligVariable::BrilligArray(array),
                BrilligParameter::Array(item_type, item_count),
            ) = (argument_variable, argument)
            {
                if BrilligContext::has_nested_arrays(item_type) {
                    let deflattened_address =
                        self.deflatten_array(item_type, array.size, array.pointer);
                    self.mov_instruction(array.pointer, deflattened_address);
                    array.size = item_type.len() * item_count;
                    self.deallocate_register(deflattened_address);
                }
            }
        }
    }

    /// Computes the size of a parameter if it was flattened
    fn flattened_size(param: &BrilligParameter) -> usize {
        match param {
            BrilligParameter::Simple => 1,
            BrilligParameter::Array(item_types, item_count) => {
                let item_size: usize = item_types.iter().map(BrilligContext::flattened_size).sum();
                item_count * item_size
            }
            BrilligParameter::Slice(_) => {
                unreachable!("ICE: Slices cannot be passed as entry point arguments")
            }
        }
    }

    /// Computes the size of a parameter if it was flattened
    fn flattened_tuple_size(tuple: &[BrilligParameter]) -> usize {
        tuple.iter().map(BrilligContext::flattened_size).sum()
    }

    /// Computes the size of a parameter if it was flattened
    fn has_nested_arrays(tuple: &[BrilligParameter]) -> bool {
        tuple.iter().any(|param| !matches!(param, BrilligParameter::Simple))
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

            self.allocate_fixed_length_array(
                deflattened_array_pointer,
                item_count * target_item_size,
            );

            for item_index in 0..item_count {
                let source_item_base_index = item_index * source_item_size;
                let target_item_base_index = item_index * target_item_size;

                let mut source_offset = 0;

                for (subitem_index, subitem) in item_type.iter().enumerate() {
                    let source_index =
                        self.make_usize_constant((source_item_base_index + source_offset).into());

                    let target_index =
                        self.make_usize_constant((target_item_base_index + subitem_index).into());

                    match subitem {
                        BrilligParameter::Simple => {
                            self.array_get(
                                flattened_array_pointer,
                                source_index,
                                movement_register,
                            );
                            self.array_set(
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
                            self.memory_op(
                                nested_array_pointer,
                                source_index,
                                nested_array_pointer,
                                acvm::brillig_vm::brillig::BinaryIntOp::Add,
                            );
                            let deflattened_nested_array_pointer = self.deflatten_array(
                                nested_array_item_type,
                                *nested_array_item_count,
                                nested_array_pointer,
                            );
                            let reference = self.allocate_register();
                            let rc = self.allocate_register();
                            self.usize_const(rc, 1_usize.into());

                            self.allocate_array_reference_instruction(reference);
                            let array_variable = BrilligVariable::BrilligArray(BrilligArray {
                                pointer: deflattened_nested_array_pointer,
                                size: nested_array_item_type.len() * nested_array_item_count,
                                rc,
                            });
                            self.store_variable_instruction(reference, array_variable);

                            self.array_set(deflattened_array_pointer, target_index, reference);

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

                    self.deallocate_register(source_index);
                    self.deallocate_register(target_index);
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
    fn exit_point_instruction(
        &mut self,
        arguments: &[BrilligParameter],
        return_parameters: &[BrilligParameter],
    ) {
        // First, we allocate the registers that hold the returned variables from the function call.
        self.set_allocated_registers(vec![]);
        let returned_variables: Vec<_> = return_parameters
            .iter()
            .map(|return_parameter| match return_parameter {
                BrilligParameter::Simple => BrilligVariable::Simple(self.allocate_register()),
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
                BrilligParameter::Simple => {
                    self.mov_instruction(
                        MemoryAddress(return_data_index),
                        returned_variable.extract_register(),
                    );
                    return_data_index += 1;
                }
                BrilligParameter::Array(item_type, item_count) => {
                    let returned_pointer = returned_variable.extract_array().pointer;
                    let pointer_to_return_data = self.make_usize_constant(return_data_index.into());

                    self.flatten_array(
                        item_type,
                        *item_count,
                        pointer_to_return_data,
                        returned_pointer,
                    );

                    self.deallocate_register(pointer_to_return_data);
                    return_data_index += BrilligContext::flattened_size(return_param);
                }
                BrilligParameter::Slice(..) => {
                    unreachable!("ICE: Cannot return slices from brillig entrypoints")
                }
            }
        }

        self.push_opcode(BrilligOpcode::Stop { return_data_offset, return_data_size });
    }

    // Flattens an array by recursively copying nested arrays and regular items.
    fn flatten_array(
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
                    let source_index =
                        self.make_usize_constant((source_item_base_index + subitem_index).into());
                    let target_index =
                        self.make_usize_constant((target_item_base_index + target_offset).into());

                    match subitem {
                        BrilligParameter::Simple => {
                            self.array_get(
                                deflattened_array_pointer,
                                source_index,
                                movement_register,
                            );
                            self.array_set(
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
                            self.array_get(
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

                            self.load_variable_instruction(
                                nested_array_variable,
                                nested_array_reference,
                            );

                            let flattened_nested_array_pointer = self.allocate_register();

                            self.mov_instruction(
                                flattened_nested_array_pointer,
                                flattened_array_pointer,
                            );

                            self.memory_op(
                                flattened_nested_array_pointer,
                                target_index,
                                flattened_nested_array_pointer,
                                acvm::brillig_vm::brillig::BinaryIntOp::Add,
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

                    self.deallocate_register(source_index);
                    self.deallocate_register(target_index);
                }
            }

            self.deallocate_register(movement_register);
        } else {
            let item_count = self.make_usize_constant((item_count * item_type.len()).into());
            self.copy_array_instruction(
                deflattened_array_pointer,
                flattened_array_pointer,
                item_count,
            );
            self.deallocate_register(item_count);
        }
    }
}

#[cfg(test)]
mod tests {
    use acvm::brillig_vm::brillig::Value;

    use crate::brillig::brillig_ir::{
        artifact::BrilligParameter,
        brillig_variable::BrilligArray,
        tests::{create_and_run_vm, create_context, create_entry_point_bytecode},
    };

    #[test]
    fn entry_point_with_nested_array_parameter() {
        let calldata = vec![
            Value::from(1_usize),
            Value::from(2_usize),
            Value::from(3_usize),
            Value::from(4_usize),
            Value::from(5_usize),
            Value::from(6_usize),
        ];
        let arguments = vec![BrilligParameter::Array(
            vec![
                BrilligParameter::Array(vec![BrilligParameter::Simple], 2),
                BrilligParameter::Simple,
            ],
            2,
        )];
        let returns = vec![BrilligParameter::Simple];

        let mut context = create_context();

        // Allocate the parameter
        let array_pointer = context.allocate_register();
        let array_value = context.allocate_register();

        context.load_instruction(array_pointer, array_pointer);
        context.load_instruction(array_pointer, array_pointer);
        context.load_instruction(array_value, array_pointer);

        context.return_instruction(&[array_value]);

        let bytecode = create_entry_point_bytecode(context, arguments, returns).byte_code;
        let (vm, return_data_offset, return_data_size) =
            create_and_run_vm(calldata.clone(), &bytecode);
        assert_eq!(return_data_size, 1, "Return data size is incorrect");
        assert_eq!(vm.get_memory()[return_data_offset], Value::from(1_usize));
    }

    #[test]
    fn entry_point_with_nested_array_return() {
        let flattened_array = vec![
            Value::from(1_usize),
            Value::from(2_usize),
            Value::from(3_usize),
            Value::from(4_usize),
            Value::from(5_usize),
            Value::from(6_usize),
        ];
        let array_param = BrilligParameter::Array(
            vec![
                BrilligParameter::Array(vec![BrilligParameter::Simple], 2),
                BrilligParameter::Simple,
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

        context.return_instruction(&brillig_array.extract_registers());

        let bytecode = create_entry_point_bytecode(context, arguments, returns).byte_code;
        let (vm, return_data_pointer, return_data_size) =
            create_and_run_vm(flattened_array.clone(), &bytecode);
        let memory = vm.get_memory();

        assert_eq!(
            memory[return_data_pointer..(return_data_pointer + flattened_array.len())],
            flattened_array
        );
        assert_eq!(return_data_size, flattened_array.len());
    }
}
