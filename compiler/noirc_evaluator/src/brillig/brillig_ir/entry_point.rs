use crate::brillig::brillig_ir::ReservedRegisters;

use super::{
    artifact::{BrilligArtifact, BrilligParameter},
    brillig_variable::{BrilligArray, BrilligVariable},
    debug_show::DebugShow,
    registers::BrilligRegistersContext,
    BrilligContext,
};
use acvm::acir::brillig::{Opcode as BrilligOpcode, RegisterIndex};

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

        context.entry_point_instruction(arguments);

        context.add_external_call_instruction(target_function);

        context.exit_point_instruction(return_parameters);
        context.artifact()
    }

    /// Adds the instructions needed to handle entry point parameters
    /// The runtime will leave the parameters in the first `n` registers.
    /// Arrays will be passed as pointers to the first element, with all the nested arrays flattened.
    /// First, reserve the registers that contain the parameters.
    /// This function also sets the starting value of the reserved registers
    fn entry_point_instruction(&mut self, arguments: Vec<BrilligParameter>) {
        let preallocated_registers: Vec<_> =
            arguments.iter().enumerate().map(|(i, _)| RegisterIndex::from(i)).collect();
        self.set_allocated_registers(preallocated_registers.clone());

        // Then allocate and initialize the variables that will hold the parameters
        let argument_variables: Vec<_> = arguments
            .iter()
            .zip(preallocated_registers)
            .map(|(argument, param_register)| match argument {
                BrilligParameter::Simple => {
                    let variable_register = self.allocate_register();
                    self.mov_instruction(variable_register, param_register);
                    BrilligVariable::Simple(variable_register)
                }
                BrilligParameter::Array(item_types, item_count) => {
                    let pointer_register = self.allocate_register();
                    let rc_register = self.allocate_register();
                    self.mov_instruction(pointer_register, param_register);
                    self.const_instruction(rc_register, 1_usize.into());
                    BrilligVariable::BrilligArray(BrilligArray {
                        pointer: pointer_register,
                        size: item_types.len() * item_count,
                        rc: rc_register,
                    })
                }
                BrilligParameter::Slice(_) => unimplemented!("Unsupported slices as parameter"),
            })
            .collect();

        // Calculate the initial value for the stack pointer register
        let size_arguments_memory: usize = arguments
            .iter()
            .map(|arg| match arg {
                BrilligParameter::Simple => 0,
                _ => BrilligContext::flattened_size(arg),
            })
            .sum();

        // Set the initial value of the stack pointer register
        self.push_opcode(BrilligOpcode::Const {
            destination: ReservedRegisters::stack_pointer(),
            value: size_arguments_memory.into(),
        });
        // Set the initial value of the previous stack pointer register
        self.push_opcode(BrilligOpcode::Const {
            destination: ReservedRegisters::previous_stack_pointer(),
            value: 0_usize.into(),
        });

        // Deflatten the arrays
        for (parameter, assigned_variable) in arguments.iter().zip(&argument_variables) {
            if let BrilligParameter::Array(item_type, item_count) = parameter {
                if item_type.iter().any(|param| !matches!(param, BrilligParameter::Simple)) {
                    let pointer_register = assigned_variable.extract_array().pointer;
                    let deflattened_register =
                        self.deflatten_array(item_type, *item_count, pointer_register);
                    self.mov_instruction(pointer_register, deflattened_register);
                }
            }
        }

        // Move the parameters to the first user defined registers, to follow function call convention.
        for (i, register) in
            argument_variables.into_iter().flat_map(|arg| arg.extract_registers()).enumerate()
        {
            self.mov_instruction(ReservedRegisters::user_register_index(i), register);
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

    /// Deflatten an array by recursively allocating nested arrays and copying the plain values.
    /// Returns the pointer to the deflattened items.
    fn deflatten_array(
        &mut self,
        item_type: &[BrilligParameter],
        item_count: usize,
        flattened_array_pointer: RegisterIndex,
    ) -> RegisterIndex {
        let movement_register = self.allocate_register();
        let deflattened_array_pointer = self.allocate_register();

        let target_item_size = item_type.len();
        let source_item_size: usize = item_type.iter().map(BrilligContext::flattened_size).sum();

        self.allocate_fixed_length_array(deflattened_array_pointer, item_count * target_item_size);

        for item_index in 0..item_count {
            let source_item_base_index = item_index * source_item_size;
            let target_item_base_index = item_index * target_item_size;

            let mut source_offset = 0;

            for (subitem_index, subitem) in item_type.iter().enumerate() {
                let source_index =
                    self.make_constant((source_item_base_index + source_offset).into());

                let target_index =
                    self.make_constant((target_item_base_index + subitem_index).into());

                match subitem {
                    BrilligParameter::Simple => {
                        self.array_get(flattened_array_pointer, source_index, movement_register);
                        self.array_set(deflattened_array_pointer, target_index, movement_register);
                        source_offset += 1;
                    }
                    BrilligParameter::Array(nested_array_item_type, nested_array_item_count) => {
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
                        self.const_instruction(rc, 1_usize.into());

                        self.allocate_array_reference_instruction(reference);
                        self.store_variable_instruction(
                            reference,
                            BrilligVariable::BrilligArray(BrilligArray {
                                pointer: deflattened_nested_array_pointer,
                                size: nested_array_item_type.len() * nested_array_item_count,
                                rc,
                            }),
                        );

                        self.array_set(deflattened_array_pointer, target_index, reference);

                        self.deallocate_register(nested_array_pointer);
                        self.deallocate_register(reference);
                        self.deallocate_register(rc);

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
    }

    /// Adds the instructions needed to handle return parameters
    /// The runtime expects the results in the first `n` registers.
    /// Arrays are expected to be returned as pointers to the first element with all the nested arrays flattened.
    /// However, the function called returns variables (that have extra data) and the returned arrays are deflattened.
    fn exit_point_instruction(&mut self, return_parameters: Vec<BrilligParameter>) {
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
        // Now, we deflatten the returned arrays
        for (return_param, returned_variable) in return_parameters.iter().zip(&returned_variables) {
            if let BrilligParameter::Array(item_type, item_count) = return_param {
                if item_type.iter().any(|item| !matches!(item, BrilligParameter::Simple)) {
                    let returned_pointer = returned_variable.extract_array().pointer;
                    let flattened_array_pointer = self.allocate_register();

                    self.allocate_fixed_length_array(
                        flattened_array_pointer,
                        BrilligContext::flattened_size(return_param),
                    );

                    self.flatten_array(
                        item_type,
                        *item_count,
                        flattened_array_pointer,
                        returned_pointer,
                    );

                    self.mov_instruction(returned_pointer, flattened_array_pointer);
                }
            }
        }
        // The VM expects us to follow the calling convention of returning
        // their results in the first `n` registers. So we to move the return values
        // to the first `n` registers once completed.

        // Move the results to registers 0..n
        for (i, returned_variable) in returned_variables.into_iter().enumerate() {
            let register = match returned_variable {
                BrilligVariable::Simple(register) => register,
                BrilligVariable::BrilligArray(array) => array.pointer,
                BrilligVariable::BrilligVector(vector) => vector.pointer,
            };
            self.push_opcode(BrilligOpcode::Mov { destination: i.into(), source: register });
        }
        self.push_opcode(BrilligOpcode::Stop);
    }

    // Flattens an array by recursively copying nested arrays and regular items.
    fn flatten_array(
        &mut self,
        item_type: &[BrilligParameter],
        item_count: usize,
        flattened_array_pointer: RegisterIndex,
        deflattened_array_pointer: RegisterIndex,
    ) {
        let movement_register = self.allocate_register();

        let source_item_size = item_type.len();
        let target_item_size: usize = item_type.iter().map(BrilligContext::flattened_size).sum();

        for item_index in 0..item_count {
            let source_item_base_index = item_index * source_item_size;
            let target_item_base_index = item_index * target_item_size;

            let mut target_offset = 0;

            for (subitem_index, subitem) in item_type.iter().enumerate() {
                let source_index =
                    self.make_constant((source_item_base_index + subitem_index).into());
                let target_index =
                    self.make_constant((target_item_base_index + target_offset).into());

                match subitem {
                    BrilligParameter::Simple => {
                        self.array_get(deflattened_array_pointer, source_index, movement_register);
                        self.array_set(flattened_array_pointer, target_index, movement_register);
                        target_offset += 1;
                    }
                    BrilligParameter::Array(nested_array_item_type, nested_array_item_count) => {
                        let nested_array_reference = self.allocate_register();
                        self.array_get(
                            deflattened_array_pointer,
                            source_index,
                            nested_array_reference,
                        );

                        let nested_array_variable = BrilligVariable::BrilligArray(BrilligArray {
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
    }
}

#[cfg(test)]
mod tests {
    use acvm::brillig_vm::brillig::{RegisterIndex, Value};

    use crate::brillig::brillig_ir::{
        artifact::BrilligParameter,
        brillig_variable::BrilligArray,
        tests::{create_and_run_vm, create_context, create_entry_point_bytecode},
    };

    #[test]
    fn entry_point_with_nested_array_parameter() {
        let flattened_array = vec![
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

        context.return_instruction(&[array_pointer]);

        let bytecode = create_entry_point_bytecode(context, arguments, returns).byte_code;
        let vm = create_and_run_vm(flattened_array.clone(), vec![Value::from(0_usize)], &bytecode);
        let memory = vm.get_memory();

        assert_eq!(vm.get_registers().get(RegisterIndex(0)), Value::from(flattened_array.len()));
        assert_eq!(
            memory,
            &vec![
                // The original flattened values
                Value::from(1_usize),
                Value::from(2_usize),
                Value::from(3_usize),
                Value::from(4_usize),
                Value::from(5_usize),
                Value::from(6_usize),
                // The pointer to the nested reference of the first item
                Value::from(12_usize),
                Value::from(3_usize),
                // The pointer to the nested reference of the second item
                Value::from(16_usize),
                Value::from(6_usize),
                // The nested array of the first item
                Value::from(1_usize),
                Value::from(2_usize),
                // The nested reference of the first item
                Value::from(10_usize),
                Value::from(1_usize),
                // The nested array of the second item
                Value::from(4_usize),
                Value::from(5_usize),
                // The nested reference of the second item
                Value::from(14_usize),
                Value::from(1_usize),
            ]
        );
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
        let vm = create_and_run_vm(flattened_array.clone(), vec![Value::from(0_usize)], &bytecode);
        let memory = vm.get_memory();

        assert_eq!(
            memory,
            &vec![
                // The original flattened values
                Value::from(1_usize),
                Value::from(2_usize),
                Value::from(3_usize),
                Value::from(4_usize),
                Value::from(5_usize),
                Value::from(6_usize),
                // The pointer to the nested reference of the first item
                Value::from(12_usize),
                Value::from(3_usize),
                // The pointer to the nested reference of the second item
                Value::from(16_usize),
                Value::from(6_usize),
                // The nested array of the first item
                Value::from(1_usize),
                Value::from(2_usize),
                // The nested reference of the first item
                Value::from(10_usize),
                Value::from(1_usize),
                // The nested array of the second item
                Value::from(4_usize),
                Value::from(5_usize),
                // The nested reference of the second item
                Value::from(14_usize),
                Value::from(1_usize),
                // The original flattened again
                Value::from(1_usize),
                Value::from(2_usize),
                Value::from(3_usize),
                Value::from(4_usize),
                Value::from(5_usize),
                Value::from(6_usize),
            ]
        );
        assert_eq!(vm.get_registers().get(RegisterIndex(0)), 18_usize.into());
    }
}
